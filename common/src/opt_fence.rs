#[macro_export]
macro_rules! opt_fence {
    ($value:expr) => {
        {
            let mut v = $value;
            unsafe {
                core::arch::asm!(
                    "/*{v}*/",
                    v = inout(reg) v,
                    options(pure, nomem, nostack, preserves_flags),
                );
            }
            v
        }
    };
}
