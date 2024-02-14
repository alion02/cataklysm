#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchParams {
    pub aspiration_window: i32,
    pub aspiration_scaling: i32,
    pub aspiration_attempts: u32,
    pub use_pvs: bool,
    pub nmp_factor: u32,
    pub nmp_fudge: i32,
    pub nmp_eval_margin: i32,
    pub tt_size: usize,
}

pub static SEARCH_PARAMS: SearchParams = SearchParams {
    aspiration_window: 20,
    aspiration_scaling: 4,
    aspiration_attempts: 0,
    use_pvs: true,
    nmp_factor: 2,
    nmp_fudge: 10,
    nmp_eval_margin: 5,
    tt_size: 1 << 24,
};
