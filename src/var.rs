use crate::VarRegister;
use std::marker::PhantomData;

pub struct Var<T: Sized, CTX> {
    pub(crate) id: usize,
    _ctx: PhantomData<CTX>,
    _t: PhantomData<T>,
}

impl<CTX: VarRegister, T: Sized> Clone for Var<T, CTX> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<CTX: VarRegister, T: Sized> Copy for Var<T, CTX> {}

impl<CTX: VarRegister, T: Sized> Var<T, CTX> {
    #[doc(hidden)]
    pub fn __new(dropper: &'static (dyn Fn(usize) + Sync)) -> Self {
        Self {
            id: CTX::register().register(dropper),
            _ctx: PhantomData,
            _t: PhantomData,
        }
    }
}
