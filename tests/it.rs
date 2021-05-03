use attached::{var_ctx, Var, Vars};
use static_init::dynamic;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

var_ctx!(C);

type Ctx = Vars<C>;

#[dynamic]
static V: Var<usize, C> = Default::default();

#[test]
fn var_get() {
    let c = Ctx::new();

    assert!(c.get(&V).is_none());
    c.get_or_init(&V, || 92);

    assert_eq!(c.get(&V), Some(&92));

    c.get_or_init(&V, || panic!("Kabom!"));
    assert_eq!(c.get(&V), Some(&92));
}

#[test]
fn var_get_mut() {
    let mut c = Ctx::new();

    assert!(c.get_mut(&V).is_none());
    assert!(c.replace(&V, Some(90)).is_none());

    *c.get_mut(&V).unwrap() += 2;
    assert_eq!(c.get_mut(&V), Some(&mut 92));
}

#[test]
fn var_drop() {
    static DROP_CNT: AtomicUsize = AtomicUsize::new(0);

    struct Dropper;

    impl Drop for Dropper {
        fn drop(&mut self) {
            DROP_CNT.fetch_add(1, SeqCst);
        }
    }

    var_ctx!(L);

    type Ctx = Vars<L>;

    #[dynamic]
    static V: Var<Dropper, L> = Default::default();

    let c = Ctx::new();

    c.get_or_init(&V, || Dropper);
    assert_eq!(DROP_CNT.load(SeqCst), 0);
    drop(c);
    assert_eq!(DROP_CNT.load(SeqCst), 1);
}

#[test]
fn reentrant_init() {
    var_ctx!(L);

    type Ctx = Vars<L>;

    #[dynamic]
    static V: Var<Box<i32>, L> = Var::new();

    let c = Ctx::new();

    let v = c.get_or_init(&V, || {
        c.get_or_init(&V, || Box::new(92));
        Box::new(62)
    });

    assert_eq!(**v, 92);
}
