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

mod var;
mod var_cnt;
mod var_ctx;

pub use var::Var;
pub use var_cnt::{Cnt, VarCnt};
pub use var_ctx::VarCtx;
