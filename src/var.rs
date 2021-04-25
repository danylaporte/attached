use crate::{
    dropper::{DropVar, Dropper},
    VarCnt,
};
use std::marker::PhantomData;

pub struct Var<T: Sized, CTX> {
    pub(crate) id: usize,
    _ctx: PhantomData<CTX>,
    dropper: Dropper<T>,
}

impl<CTX: VarCnt, T: Sized> Var<T, CTX> {
    #[inline]
    pub fn new() -> Self {
        Self {
            id: CTX::var_cnt().next(),
            _ctx: PhantomData,
            dropper: Dropper::new(),
        }
    }

    pub(crate) fn dropper(&'static self) -> &'static dyn DropVar {
        &self.dropper
    }
}

impl<CTX: VarCnt, T> Default for Var<T, CTX> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
