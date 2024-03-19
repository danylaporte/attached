use attached::{container, var, Container};

fn main() {
    let my_extensible_struct = MyExtensibleStruct::default();

    let v = my_extensible_struct
        .ctx
        .get_or_init(*MY_ATTACHED_PROP, || 20);

    assert_eq!(*v, 20);
}

#[derive(Default)]
struct MyExtensibleStruct {
    ctx: Container<MyCtx>,
}

container!(MyCtx);
var!(MY_ATTACHED_PROP: i32, MyCtx);
