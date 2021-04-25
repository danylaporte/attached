use attached::{var_ctx, Var, VarCtx};
use static_init::dynamic;

#[dynamic]
static MY_ATTACHED_PROP: Var<i32, MyCtx> = Var::new();

fn main() {
    let my_extensible_struct = MyExtensibleStruct::default();

    let v = my_extensible_struct
        .ctx
        .get_or_init(&MY_ATTACHED_PROP, || 20);

    assert_eq!(*v, 20);
}

#[derive(Default)]
struct MyExtensibleStruct {
    ctx: VarCtx<MyCtx>,
}

var_ctx!(MyCtx);
