use crate::{Var, VarRegister};
use std::{
    marker::PhantomData,
    mem::replace,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

pub struct Container<Ctx: VarRegister> {
    _ctx: PhantomData<Ctx>,
    ptrs: Box<[AtomicUsize]>,
}

impl<Ctx: VarRegister> Container<Ctx> {
    pub fn new() -> Self {
        Self {
            _ctx: PhantomData,
            ptrs: (0..Ctx::register().count())
                .map(|_| AtomicUsize::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub fn clear<T>(&mut self, var: Var<T, Ctx>) {
        let v = replace(unsafe { self.ptrs.get_unchecked_mut(var.id) }.get_mut(), 0);

        if v > 0 {
            // drop the value
            let _ = from_usize::<T>(v);
        }
    }

    pub fn get<T: Sized>(&self, var: Var<T, Ctx>) -> Option<&T> {
        let v = unsafe { self.ptrs.get_unchecked(var.id) }.load(Relaxed);

        if v == 0 {
            None
        } else {
            Some(unsafe { &*(v as *mut T) })
        }
    }

    pub fn get_mut<T: Sized>(&mut self, var: Var<T, Ctx>) -> Option<&mut T> {
        let v = *unsafe { self.ptrs.get_unchecked_mut(var.id) }.get_mut();

        if v == 0 {
            None
        } else {
            Some(unsafe { &mut *(v as *mut T) })
        }
    }

    pub fn get_or_init<F, T: Sized>(&self, var: Var<T, Ctx>, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        let ptr = unsafe { self.ptrs.get_unchecked(var.id) };
        let mut v = ptr.load(Relaxed);

        if v == 0 {
            v = into_usize(init());

            if let Err(old) = ptr.compare_exchange_weak(0, v, Relaxed, Relaxed) {
                // drop v
                let _ = from_usize::<T>(v);
                v = old
            }
        }

        unsafe { &*(v as *mut T) }
    }

    pub fn get_or_init_mut<F, T: Sized>(&mut self, var: Var<T, Ctx>, init: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        let v = unsafe { self.ptrs.get_unchecked_mut(var.id) }.get_mut();

        if *v == 0 {
            *v = into_usize(init());
        }

        unsafe { &mut *(*v as *mut T) }
    }

    pub fn get_or_init_val<T: Sized>(&self, var: Var<T, Ctx>, val: T) -> (&T, Option<T>) {
        let ptr = unsafe { self.ptrs.get_unchecked(var.id) };
        let mut v = ptr.load(Relaxed);

        if v == 0 {
            v = into_usize(val);

            if let Err(old) = ptr.compare_exchange_weak(0, v, Relaxed, Relaxed) {
                return (unsafe { &*(old as *mut T) }, Some(from_usize::<T>(v)));
            }
        }

        (unsafe { &*(v as *mut T) }, None)
    }

    pub fn replace<T: Sized>(&mut self, var: Var<T, Ctx>, val: Option<T>) -> Option<T> {
        let v = unsafe { self.ptrs.get_unchecked_mut(var.id) }.get_mut();

        let old = if *v == 0 { None } else { Some(from_usize(*v)) };

        match val {
            Some(val) => *v = into_usize(val),
            None => *v = 0,
        }

        old
    }
}

impl<Ctx: VarRegister> Default for Container<Ctx> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Ctx: VarRegister> Drop for Container<Ctx> {
    fn drop(&mut self) {
        let droppers = Ctx::register().vec();

        for (index, ptr) in self.ptrs.iter_mut().enumerate() {
            let v = *ptr.get_mut();
            if v > 0 {
                (unsafe { droppers.get_unchecked(index) })(v);
            }
        }
    }
}

unsafe impl<Ctx: VarRegister> Send for Container<Ctx> {}
unsafe impl<Ctx: VarRegister> Sync for Container<Ctx> {}

#[doc(hidden)]
pub fn from_usize<T: Sized>(v: usize) -> T {
    *unsafe { Box::from_raw(v as *mut T) }
}

fn into_usize<T: Sized>(v: T) -> usize {
    Box::into_raw(Box::new(v)) as usize
}

#[test]
fn value_lifecycle() {
    use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

    static CREATE_COUNT: AtomicUsize = AtomicUsize::new(0);
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    container!(MY);

    var!(V: LifeCheck, MY);

    let ctx = Container::<MY>::new();

    // values are lazy created. Nothing should be created yet.
    assert_eq!(CREATE_COUNT.load(Relaxed), 0);

    // creates and keeps a reference to LifeCheck.
    let _ = ctx.get_or_init(*V, LifeCheck::default);
    assert_eq!(CREATE_COUNT.load(Relaxed), 1);
    assert_eq!(DROP_COUNT.load(Relaxed), 0);

    // should get the same instance of LifeCheck
    let _ = ctx.get_or_init(*V, LifeCheck::default);
    assert_eq!(CREATE_COUNT.load(Relaxed), 1);
    assert_eq!(DROP_COUNT.load(Relaxed), 0);

    // values should be dropped.
    drop(ctx);
    assert_eq!(CREATE_COUNT.load(Relaxed), 1);
    assert_eq!(DROP_COUNT.load(Relaxed), 1);

    struct LifeCheck;

    impl Default for LifeCheck {
        fn default() -> Self {
            CREATE_COUNT.fetch_add(1, Relaxed);
            Self
        }
    }

    impl Drop for LifeCheck {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Relaxed);
        }
    }
}
