mod get_version;
mod var_ctx;
mod variables;
mod version;

pub use get_version::GetVersion;
use std::sync::atomic::AtomicUsize;
pub use var_ctx::VarCtx;
pub use variables::*;
pub use version::Version;

static VARIABLE_COUNTER: AtomicUsize = AtomicUsize::new(0);
