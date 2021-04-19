use crate::{VarCtx, VARIABLE_COUNTER};
use std::{marker::PhantomData, sync::atomic::Ordering::Relaxed};

pub struct Var<T> {
    id: usize,
    _t: PhantomData<T>,
}

impl<T> Var<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            id: VARIABLE_COUNTER.fetch_add(1, Relaxed),
            _t: PhantomData,
        }
    }

    #[inline]
    pub fn get<'a>(&'static self, ctx: &'a VarCtx) -> Option<&'a T> {
        ctx.get(self.id)
    }

    #[inline]
    pub fn get_mut<'a>(&'static self, ctx: &'a mut VarCtx) -> Option<&'a mut T> {
        ctx.get_mut(self.id)
    }

    #[inline]
    pub fn get_or_init<'a, F>(&'static self, ctx: &'a VarCtx, init: F) -> &'a T
    where
        F: FnOnce() -> T,
    {
        ctx.get(self.id)
            .unwrap_or_else(|| ctx.get_or_init(self.id, init))
    }

    #[inline]
    pub fn get_or_init_mut<'a, F>(&'static self, ctx: &'a mut VarCtx, init: F) -> &'a mut T
    where
        F: FnOnce() -> T,
    {
        match ctx.get::<T>(self.id).is_some() {
            true => ctx.get_mut(self.id).unwrap(),
            false => ctx.get_or_init_mut(self.id, init),
        }
    }

    #[inline]
    pub fn replace(&'static self, ctx: &mut VarCtx, val: Option<T>) -> Option<T> {
        ctx.replace(self.id, val)
    }

    #[inline]
    pub fn take(&'static self, ctx: &mut VarCtx) -> Option<T> {
        ctx.replace(self.id, None)
    }
}

unsafe impl<T> Sync for Var<T> {}
unsafe impl<T> Send for Var<T> {}
