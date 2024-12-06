use crate::{Var, VarRegister};
use std::{cell::UnsafeCell, marker::PhantomData, mem::take, sync::Once};

pub struct Container<Ctx: VarRegister> {
    _ctx: PhantomData<Ctx>,
    onces: Box<[Once]>,
    values: Box<[UnsafeCell<usize>]>,
}

impl<Ctx: VarRegister> Container<Ctx> {
    pub fn new() -> Self {
        let count = Ctx::register().count();

        Self {
            _ctx: PhantomData,
            onces: (0..count).map(|_| Once::new()).collect(),
            values: (0..count).map(|_| UnsafeCell::new(0)).collect(),
        }
    }

    #[inline]
    pub fn clear<T>(&mut self, var: Var<T, Ctx>) {
        self.take::<T>(var);
    }

    #[inline]
    pub fn get<T>(&self, var: Var<T, Ctx>) -> Option<&T> {
        let value = unsafe { *self.values.get_unchecked(var.id).get() };

        if value == 0 {
            None
        } else {
            Some(unsafe { as_ref(value) })
        }
    }

    #[inline]
    pub fn get_mut<T>(&mut self, var: Var<T, Ctx>) -> Option<&mut T> {
        let value = unsafe { self.values.get_unchecked_mut(var.id).get_mut() };

        if *value == 0 {
            None
        } else {
            Some(unsafe { as_mut(value) })
        }
    }

    #[inline]
    pub fn get_or_init<F, T>(&self, var: Var<T, Ctx>, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        let value = unsafe { *self.values.get_unchecked(var.id).get() };

        if value == 0 {
            self.initialize_ref(var.id, init)
        } else {
            unsafe { as_ref(value) }
        }
    }

    #[inline]
    pub fn get_mut_or_init<F, T>(&mut self, var: Var<T, Ctx>, init: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        let value = unsafe { self.values.get_unchecked_mut(var.id) }.get_mut();

        if *value == 0 {
            self.initialize_mut(var.id, init);
        }

        unsafe { as_mut(self.values.get_unchecked_mut(var.id).get_mut()) }
    }

    fn initialize<F, T>(&self, index: usize, init: F)
    where
        F: FnOnce() -> T,
    {
        unsafe { self.onces.get_unchecked(index) }.call_once(|| {
            let v = Box::into_raw(Box::new(init())) as usize;
            unsafe { *self.values.get_unchecked(index).get() = v };
        });

        debug_assert!(unsafe { *self.values.get_unchecked(index).get() } != 0);
    }

    #[cold]
    fn initialize_mut<F, T>(&mut self, index: usize, init: F)
    where
        F: FnOnce() -> T,
    {
        self.initialize(index, init);
    }

    #[cold]
    fn initialize_ref<F, T>(&self, index: usize, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.initialize(index, init);

        unsafe { as_ref(*self.values.get_unchecked(index).get()) }
    }

    #[inline]
    pub fn take<T>(&mut self, var: Var<T, Ctx>) -> Option<T> {
        let value = take(unsafe { self.values.get_unchecked_mut(var.id) }.get_mut());

        if value == 0 {
            None
        } else {
            *unsafe { self.onces.get_unchecked_mut(var.id) } = Once::new();
            Some(from_usize(value))
        }
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

        for (index, ptr) in self.values.iter().enumerate() {
            let v = unsafe { *ptr.get() };

            if v != 0 {
                (unsafe { droppers.get_unchecked(index) })(v);
            }
        }
    }
}

unsafe impl<Ctx: VarRegister> Send for Container<Ctx> {}
unsafe impl<Ctx: VarRegister> Sync for Container<Ctx> {}

#[inline]
unsafe fn as_mut<T>(value: &mut usize) -> &mut T {
    &mut *(*value as *mut T)
}

#[inline]
unsafe fn as_ref<'a, T: 'a>(value: usize) -> &'a T {
    &*(value as *mut T)
}

#[doc(hidden)]
#[inline]
pub fn from_usize<T: Sized>(v: usize) -> T {
    *unsafe { Box::from_raw(v as *mut T) }
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
