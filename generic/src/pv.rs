use crate::*;

pub struct Pv<'a>(RefCell<&'a mut State>);

impl<'a> Pv<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self(RefCell::new(state))
    }
}

impl<'a> fmt::Display for Pv<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn write_recursively(s: &mut State, f: &mut fmt::Formatter, first: bool) -> fmt::Result {
            let (idx, sig) = s.hash().split(s.tt.len());
            if let Some(&mut TtEntry { action, .. }) = s.tt[idx].entry(sig) {
                if s.is_legal(action) {
                    if !first {
                        f.write_str(" ")?;
                    }

                    write!(f, "{action}")?;

                    return s.with(true, action, |s| write_recursively(s, f, false));
                }
            }

            Ok(())
        }

        let mut s = self.0.borrow_mut();

        write_recursively(&mut s, f, true)
    }
}
