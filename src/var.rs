use crate::{VarCnt, VarCtx};
use std::marker::PhantomData;

pub struct Var<T, CTX> {
    id: usize,
    _ctx: PhantomData<CTX>,
    _t: PhantomData<T>,
}

impl<CTX: VarCnt, T> Var<T, CTX> {
    #[inline]
    pub fn new() -> Self {
        Self {
            id: CTX::var_cnt().next(),
            _ctx: PhantomData,
            _t: PhantomData,
        }
    }

    pub fn clear(&'static self, ctx: &mut VarCtx<CTX>) {
        ctx.clear(self.id);
    }

    #[inline]
    pub fn get<'a>(&'static self, ctx: &'a VarCtx<CTX>) -> Option<&'a T> {
        ctx.get(self.id)
    }

    #[inline]
    pub fn get_mut<'a>(&'static self, ctx: &'a mut VarCtx<CTX>) -> Option<&'a mut T> {
        ctx.get_mut(self.id)
    }

    #[inline]
    pub fn get_or_init<'a, F>(&'static self, ctx: &'a VarCtx<CTX>, init: F) -> &'a T
    where
        F: FnOnce() -> T,
    {
        ctx.get(self.id)
            .unwrap_or_else(|| ctx.get_or_init(self.id, init))
    }

    #[inline]
    pub fn get_or_init_mut<'a, F>(&'static self, ctx: &'a mut VarCtx<CTX>, init: F) -> &'a mut T
    where
        F: FnOnce() -> T,
    {
        match ctx.get::<T>(self.id).is_some() {
            true => ctx.get_mut(self.id).unwrap(),
            false => ctx.get_or_init_mut(self.id, init),
        }
    }

    #[inline]
    pub fn replace(&'static self, ctx: &mut VarCtx<CTX>, val: Option<T>) -> Option<T> {
        ctx.replace(self.id, val)
    }

    #[inline]
    pub fn take(&'static self, ctx: &mut VarCtx<CTX>) -> Option<T> {
        ctx.replace(self.id, None)
    }
}

impl<CTX: VarCnt, T> Default for Var<T, CTX> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
