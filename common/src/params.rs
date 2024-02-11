#[derive(Debug)]
pub struct Params {
    pub aspiration_window: i32,
    pub aspiration_scaling: i32,
    pub aspiration_attempts: u32,
    pub use_pvs: bool,
    pub tt_size: usize,

    pub dist_cap_offset: i32,

    pub flat_count: i32,
    pub stones_left: i32,
    pub caps_left: i32,
    pub total_dist: i32,
    pub smallest_dist: i32,
    pub side_to_move: i32,
}

pub const DEFAULT_PARAMS: Params = Params {
    aspiration_window: 20,
    aspiration_scaling: 4,
    aspiration_attempts: 0,
    use_pvs: true,
    tt_size: 1 << 24,

    dist_cap_offset: -1,

    flat_count: 10,
    stones_left: -7,
    caps_left: -15,
    total_dist: -1,
    smallest_dist: -2,
    side_to_move: 21,
};
