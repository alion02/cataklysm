pub union Out<T: Copy> {
    _uninit: (),
    pub val: T,
}
