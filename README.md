# attached

By using attached, it is now possible to define and add properties dynamically to a VarCtx. 

# Example

```rust
use attached::{Var, VarCtx};
use static_init::dynamic;

#[dynamic]
static MY_ATTACHED_PROP: Var<i32> = Var::new();

fn main() {
    let my_extensible_struct = MyExtensibleStruct::default();
    let v = MY_ATTACHED_PROP.get_or_init(&my_extensible_struct.ctx, || 20);

    assert_eq!(*v, 20);
}

#[derive(Default)]
struct MyExtensibleStruct {
    ctx: VarCtx,
}
```

