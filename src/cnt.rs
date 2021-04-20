use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

pub struct Cnt(AtomicUsize);

impl Cnt {
    pub const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }
}

impl Cnt {
    pub(super) fn cur(&self) -> usize {
        self.0.load(Relaxed)
    }

    pub(super) fn next(&self) -> usize {
        self.0.fetch_add(1, Relaxed)
    }
}

pub trait VarCnt {
    fn var_cnt() -> &'static Cnt;
}
