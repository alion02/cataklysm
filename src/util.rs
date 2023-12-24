use std::{convert::Infallible, ops::ControlFlow};

pub trait ControlFlowIntoContinue<T> {
    fn into_continue(self) -> T;
}

impl<T> ControlFlowIntoContinue<T> for ControlFlow<Infallible, T> {
    fn into_continue(self) -> T {
        match self {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(v) => match v {},
        }
    }
}

pub trait ControlFlowIntoInner<T> {
    fn into_inner(self) -> T;
}

impl<T> ControlFlowIntoInner<T> for ControlFlow<T, T> {
    fn into_inner(self) -> T {
        match self {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(v) => v,
        }
    }
}
