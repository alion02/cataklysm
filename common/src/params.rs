#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchParams {
    pub aspiration_window: i32,
    pub aspiration_scaling: i32,
    pub aspiration_attempts: u32,
    pub use_pvs: bool,
    pub tt_size: usize,
}

pub static SEARCH_PARAMS: SearchParams = SearchParams {
    aspiration_window: 20,
    aspiration_scaling: 4,
    aspiration_attempts: 0,
    use_pvs: true,
    tt_size: 1 << 24,
};
