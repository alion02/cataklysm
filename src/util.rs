use std::{convert::Infallible, ops::ControlFlow};

pub trait ControlFlowExt<T> {
    fn into_continue(self) -> T;
}

impl<T> ControlFlowExt<T> for ControlFlow<Infallible, T> {
    fn into_continue(self) -> T {
        match self {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(v) => match v {},
        }
    }
}
