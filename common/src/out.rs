pub union Out<T: Copy> {
    uninit: (),
    pub val: T,
}
