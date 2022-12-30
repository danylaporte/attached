use crate::{DropVar, NoopDrop};
use std::{
    cell::UnsafeCell,
    mem::replace,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

pub(super) struct Slot {
    drop: UnsafeCell<&'static dyn DropVar>,
    ptr: AtomicUsize,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            drop: UnsafeCell::new(&NOOP_DROP),
            ptr: AtomicUsize::new(0),
        }
    }
}

impl Slot {
    #[cfg(test)]
    pub fn new<T: Sized>(val: T, drop: &'static dyn DropVar) -> Self {
        Self {
            drop: UnsafeCell::new(drop),
            ptr: AtomicUsize::new(into_usize(val)),
        }
    }

    pub fn get<T: Sized>(&self) -> Option<&T> {
        let v = self.ptr.load(Relaxed);
        if v == 0 {
            None
        } else {
            Some(unsafe { &*(v as *mut T) })
        }
    }

    pub fn get_mut<T: Sized>(&mut self) -> Option<&mut T> {
        let v = *self.ptr.get_mut();
        if v == 0 {
            None
        } else {
            Some(unsafe { &mut *(v as *mut T) })
        }
    }

    pub fn replace<T: Sized>(&mut self, val: Option<T>, drop: &'static dyn DropVar) -> Option<T> {
        let v = val.map(into_usize).unwrap_or(0);
        let v = replace(self.ptr.get_mut(), v);

        *self.drop.get_mut() = drop;

        if v == 0 {
            None
        } else {
            Some(from_usize(v))
        }
    }

    pub fn set<T: Sized>(&self, val: T, drop: &'static dyn DropVar) -> Result<(), T> {
        let v = into_usize(val);

        match self.ptr.compare_exchange_weak(0, v, Relaxed, Relaxed) {
            Ok(_) => {
                // make sure the value is droppable.
                *unsafe { &mut *self.drop.get() } = drop;
                Ok(())
            }
            Err(_) => Err(from_usize(v)),
        }
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        let v = *self.ptr.get_mut();
        if v > 0 {
            self.drop.get_mut().drop_var(*self.ptr.get_mut());
        }
    }
}

fn from_usize<T: Sized>(v: usize) -> T {
    *unsafe { Box::from_raw(v as *mut T) }
}

fn into_usize<T: Sized>(v: T) -> usize {
    Box::into_raw(Box::new(v)) as usize
}

#[test]
fn slot_drop() {
    use crate::Dropper;
    use std::sync::Arc;

    static DROPPER: Dropper<Arc<()>> = Dropper::new();

    let v = Arc::new(());
    let p = Slot::new(Arc::clone(&v), &DROPPER);

    assert_eq!(Arc::strong_count(&v), 2);

    drop(p);

    assert_eq!(Arc::strong_count(&v), 1);
}

static NOOP_DROP: NoopDrop = NoopDrop;

#[test]
fn slot_get_mut() {
    use crate::Dropper;

    static DROPPER: Dropper<String> = Dropper::new();

    let mut p = Slot::new("Hello".to_string(), &DROPPER);

    *p.get_mut::<String>().unwrap() += " world";

    assert_eq!("Hello world", p.get_mut::<String>().unwrap());
}
