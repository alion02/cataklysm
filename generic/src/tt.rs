use crate::*;

#[derive(Debug, Clone, Copy, Default)]
#[repr(align(32))]
pub struct TtBucket([TtEntry; 2]);

impl TtBucket {
    pub fn entry(&mut self, sig: u64) -> Option<&mut TtEntry> {
        self.0.iter_mut().find(|e| e.sig == sig)
    }

    pub fn worst_entry(&mut self, curr_gen: u32) -> &mut TtEntry {
        self.0
            .iter_mut()
            .min_by_key(|e| rate_entry(e.depth, e.packed.generation(), curr_gen))
            .unwrap()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TtEntry {
    pub sig: u64,
    pub score: Eval,
    pub action: Action,
    pub depth: u8,
    pub packed: Packed,
}

// TODO: Cleanup
pub fn rate_entry(depth: u8, entry_gen: u32, curr_gen: u32) -> i32 {
    depth as i32 - (curr_gen - entry_gen & 0x3F) as i32
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Packed(u8);

impl Packed {
    pub fn is_upper(self) -> bool {
        self.0 & 0x40 == 0
    }

    pub fn is_lower(self) -> bool {
        self.0 & 0x80 == 0
    }

    pub fn is_exact(self) -> bool {
        self.0 & 0xC0 == 0
    }

    pub fn generation(self) -> u32 {
        self.0 as u32 & 0x3F
    }

    pub fn set_upper(&mut self) {
        self.0 |= 0x80;
    }

    pub fn set_lower(&mut self) {
        self.0 |= 0x40;
    }

    pub fn set_generation(&mut self, generation: u32) {
        self.0 = self.0 & !0x3F | generation as u8 & 0x3F;
    }
}
