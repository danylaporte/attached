use std::marker::PhantomData;

pub(crate) struct Dropper<T: Sized>(PhantomData<T>);

impl<T: Sized> Dropper<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Sized> DropVar for Dropper<T> {
    fn drop_var(&self, ptr: usize) {
        let _ = unsafe { Box::from_raw(ptr as *mut T) };
    }
}

pub(crate) trait DropVar {
    fn drop_var(&self, ptr: usize);
}

pub(crate) struct NoopDrop;

impl DropVar for NoopDrop {
    fn drop_var(&self, _ptr: usize) {
        // do nothing...
    }
}
