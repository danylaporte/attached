use attached::{container, var, Container};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

container!(C);

type Ctx = Container<C>;

var!(V: usize, C);

#[test]
fn var_get() {
    let c = Ctx::new();

    assert!(c.get(*V).is_none());
    c.get_or_init(*V, || 92);

    assert_eq!(c.get(*V), Some(&92));

    c.get_or_init(*V, || panic!("Kabom!"));
    assert_eq!(c.get(*V), Some(&92));
}

#[test]
fn var_get_mut() {
    let mut c = Ctx::new();

    assert!(c.get_mut(*V).is_none());
    c.get_or_init(*V, || 90);

    *c.get_mut(*V).unwrap() += 2;
    assert_eq!(c.get_mut(*V), Some(&mut 92));
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

    container!(L);

    type Ctx = Container<L>;

    var!(V: Dropper, L);

    let c = Ctx::new();

    c.get_or_init(*V, || Dropper);
    assert_eq!(DROP_CNT.load(SeqCst), 0);
    drop(c);
    assert_eq!(DROP_CNT.load(SeqCst), 1);
}

// #[test]
// fn reentrant_init() {
//     container!(L);

//     type Ctx = Container<L>;

//     var!(V: Box<i32>, L);

//     let c = Ctx::new();

//     let v = c.get_or_init(*V, || {
//         c.get_or_init(*V, || Box::new(92));
//         Box::new(62)
//     });

//     assert_eq!(**v, 92);
// }
