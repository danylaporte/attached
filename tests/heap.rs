use attached::{container, var, Container};

container!(C);

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// declare a new ctx
container!(MyCtx);

type Ctx = Container<MyCtx>;

// declare a new variable in the ctx
var!(V: Vec<u8>, MyCtx);

#[test]
fn var_drop() {
    let _profiler = dhat::Profiler::builder().testing().build();

    // instanciate the ctx
    let c = Ctx::new();

    // instanciate the vec inside the ctx
    c.get_or_init(*V, || vec![0, 1, 2, 3, 4]);

    // drop the context.
    drop(c);

    let stats = dhat::HeapStats::get();

    assert_eq!(stats.curr_blocks, 0);
    assert_eq!(stats.curr_bytes, 0);

    check_no_allocate_after_multiple_get_or_init();
}

fn check_no_allocate_after_multiple_get_or_init() {
    let mut c = Ctx::new();

    let stats = dhat::HeapStats::get();

    assert_eq!(stats.curr_blocks, 1);
    assert_eq!(stats.curr_bytes, 8);

    c.get_or_init(*V, || vec![0, 1, 2, 3, 4]);

    let stats = dhat::HeapStats::get();

    assert_eq!(stats.curr_blocks, 3);
    assert_eq!(stats.curr_bytes, 37);

    c.get_or_init(*V, || vec![0, 1, 2, 3, 4]);

    let stats = dhat::HeapStats::get();

    assert_eq!(stats.curr_blocks, 3);
    assert_eq!(stats.curr_bytes, 37);

    c.replace(*V, Some(vec![]));

    let stats = dhat::HeapStats::get();

    assert_eq!(stats.curr_blocks, 2);
    assert_eq!(stats.curr_bytes, 32);

    c.replace(*V, Some(vec![0, 1, 2, 3, 4]));

    let stats = dhat::HeapStats::get();

    assert_eq!(stats.curr_blocks, 3);
    assert_eq!(stats.curr_bytes, 37);
}
