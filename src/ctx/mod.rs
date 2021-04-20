mod cell_opt;
mod node;
mod untyped_ptr;

use crate::VarCnt;
use cell_opt::CellOpt;
use node::Node;
use parking_lot::Mutex;
use std::marker::PhantomData;
use untyped_ptr::UntypedPtr;

pub struct VarCtx<CTX> {
    _ctx: PhantomData<CTX>,
    mutex: Mutex<()>,
    node: Node<CTX>,
}

impl<CTX: VarCnt> VarCtx<CTX> {
    pub fn new() -> Self {
        Self {
            _ctx: PhantomData,
            mutex: Mutex::new(()),
            node: Node::with_offset(0),
        }
    }

    pub(crate) fn clear(&mut self, index: usize) {
        if let Some(v) = self.node.slot_mut(index) {
            *v.get_mut() = None;
        }
    }

    #[inline]
    pub(crate) fn get<T>(&self, index: usize) -> Option<&T> {
        self.node
            .slot(index)
            .and_then(|slot| slot.get().map(|ptr| ptr.get()))
    }

    #[inline]
    pub(crate) fn get_mut<T>(&mut self, index: usize) -> Option<&mut T> {
        self.node
            .slot_mut(index)
            .and_then(|slot| slot.get_mut().as_mut())
            .map(|ptr| ptr.get_mut())
    }

    /// call `get` method first; it's faster. If the value is not found then call `get_or_init`.
    pub(crate) fn get_or_init<F, T>(&self, index: usize, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        let _guard = self.mutex.lock();
        let slot = self.node.slot_or_init(index);

        slot.get()
            .unwrap_or_else(|| {
                slot.replace(Some(UntypedPtr::new(init())));
                slot.get().unwrap()
            })
            .get()
    }

    pub(crate) fn get_or_init_mut<F, T>(&mut self, index: usize, init: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        let slot = self.node.slot_or_init_mut(index);

        if slot.get().is_none() {
            slot.replace(Some(UntypedPtr::new(init())));
        }

        slot.get_mut().as_mut().map(|ptr| ptr.get_mut()).unwrap()
    }

    pub(crate) fn replace<T>(&mut self, index: usize, val: Option<T>) -> Option<T> {
        match val {
            Some(val) => self
                .node
                .slot_or_init_mut(index)
                .replace(Some(UntypedPtr::new(val))),
            None => self
                .node
                .slot_mut(index)
                .and_then(|slot| slot.replace(None)),
        }
        .map(|ptr| ptr.into_inner())
    }
}

impl<CTX: VarCnt> Default for VarCtx<CTX> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<CTX> Send for VarCtx<CTX> {}
unsafe impl<CTX> Sync for VarCtx<CTX> {}

#[test]
fn value_lifecycle() {
    use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

    static CREATE_COUNT: AtomicUsize = AtomicUsize::new(0);
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    var_ctx!(MY);

    let ctx = VarCtx::<MY>::new();

    // values are lazy created. Nothing should be created yet.
    assert_eq!(CREATE_COUNT.load(Relaxed), 0);

    // creates and keeps a reference to LifeCheck.
    let _ = ctx.get_or_init(0, LifeCheck::default);
    assert_eq!(CREATE_COUNT.load(Relaxed), 1);
    assert_eq!(DROP_COUNT.load(Relaxed), 0);

    // should get the same instance of LifeCheck
    let _ = ctx.get_or_init(0, LifeCheck::default);
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
