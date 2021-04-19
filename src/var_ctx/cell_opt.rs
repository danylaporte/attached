use std::{cell::UnsafeCell, ptr::replace};

/// This cell allow to modify a value even if we don't have the right.
/// This is unsafe. Must be used under a lock.
pub(super) struct CellOpt<T>(UnsafeCell<Option<T>>);

impl<T> CellOpt<T> {
    pub fn new(val: Option<T>) -> Self {
        Self(UnsafeCell::new(val))
    }

    pub fn get(&self) -> Option<&T> {
        unsafe { &*self.0.get() }.as_ref()
    }

    pub fn get_mut(&mut self) -> &mut Option<T> {
        self.0.get_mut()
    }

    pub fn replace(&self, val: Option<T>) -> Option<T> {
        unsafe { replace(self.0.get(), val) }
    }
}
