#[macro_export]
macro_rules! var_ctx {
    ($vis:vis $name:ident) => {
        $vis struct $name;

        impl $crate::VarCnt for $name {
            fn var_cnt() -> &'static $crate::Cnt {
                static CNT: $crate::Cnt = $crate::Cnt::new();
                &CNT
            }
        }
    };
}

mod cnt;
mod ctx;
mod var;

pub use cnt::{Cnt, VarCnt};
pub use ctx::VarCtx;
pub use var::Var;
