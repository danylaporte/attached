mod node;
mod slot;

use crate::{Var, VarCnt};
use node::Node;
use slot::Slot;

pub struct Vars<CTX> {
    node: Node<CTX>,
}

impl<CTX: VarCnt> Vars<CTX> {
    pub fn new() -> Self {
        Self {
            node: Node::with_offset(0),
        }
    }

    pub fn clear<T: Sized>(&mut self, var: &'static Var<T, CTX>) {
        self.node
            .slot_mut(var.id)
            .map(|v| v.replace::<T>(None, var.dropper()));
    }

    pub fn get<T: Sized>(&self, var: &'static Var<T, CTX>) -> Option<&T> {
        self.node.slot(var.id).and_then(|slot| slot.get())
    }

    pub fn get_mut<T: Sized>(&mut self, var: &'static Var<T, CTX>) -> Option<&mut T> {
        self.node.slot_mut(var.id).and_then(|slot| slot.get_mut())
    }

    /// call `get` method first; it's faster. If the value is not found then call `get_or_init`.
    pub fn get_or_init<F, T: Sized>(&self, var: &'static Var<T, CTX>, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        let slot = self.node.slot_or_init(var.id);

        slot.get().unwrap_or_else(|| {
            let _ = slot.set(init(), var.dropper());
            slot.get().unwrap()
        })
    }

    pub fn get_or_init_mut<F, T: Sized>(&mut self, var: &'static Var<T, CTX>, init: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        let slot = self.node.slot_or_init_mut(var.id);

        if slot.get::<T>().is_none() {
            slot.replace(Some(init()), var.dropper());
        }

        slot.get_mut().unwrap()
    }

    pub fn replace<T: Sized>(&mut self, var: &'static Var<T, CTX>, val: Option<T>) -> Option<T> {
        self.node
            .slot_or_init_mut(var.id)
            .replace(val, var.dropper())
    }
}

impl<CTX: VarCnt> Default for Vars<CTX> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<CTX> Send for Vars<CTX> {}
unsafe impl<CTX> Sync for Vars<CTX> {}

#[test]
fn value_lifecycle() {
    use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

    static CREATE_COUNT: AtomicUsize = AtomicUsize::new(0);
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    var_ctx!(MY);

    #[static_init::dynamic]
    static V: Var<LifeCheck, MY> = Var::new();

    let ctx = Vars::<MY>::new();

    // values are lazy created. Nothing should be created yet.
    assert_eq!(CREATE_COUNT.load(Relaxed), 0);

    // creates and keeps a reference to LifeCheck.
    let _ = ctx.get_or_init(&V, LifeCheck::default);
    assert_eq!(CREATE_COUNT.load(Relaxed), 1);
    assert_eq!(DROP_COUNT.load(Relaxed), 0);

    // should get the same instance of LifeCheck
    let _ = ctx.get_or_init(&V, LifeCheck::default);
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
