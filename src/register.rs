use std::sync::{Mutex, MutexGuard};

pub trait VarRegister {
    fn register() -> &'static Register;
}

pub struct Register(Mutex<Vec<&'static (dyn Fn(usize) + Sync)>>);

impl Register {
    pub const fn new() -> Self {
        Self(Mutex::new(Vec::new()))
    }

    pub(crate) fn count(&self) -> usize {
        self.vec().len()
    }

    pub(crate) fn register(&self, dropper: &'static (dyn Fn(usize) + Sync)) -> usize {
        let mut lock = self.vec();
        let index = lock.len();

        lock.push(dropper);

        index
    }

    pub(crate) fn vec(&self) -> MutexGuard<Vec<&'static (dyn Fn(usize) + Sync)>> {
        self.0.lock().expect("lock")
    }
}
