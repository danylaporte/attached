#[macro_export]
macro_rules! container {
    ($vis:vis $name:ident) => {
        $vis struct $name;

        impl $crate::VarRegister for $name {
            fn register() -> &'static $crate::Register {
                static REGISTER: $crate::Register = $crate::Register::new();
                &REGISTER
            }
        }
    };
}

#[macro_export]
macro_rules! var {
    ($var_name:ident: $var_ty:ty, $ctx:ty) => {
        #[$crate::static_init::dynamic]
        static $var_name: $crate::Var<$var_ty, $ctx> = {
            fn dropper(ptr: usize) {
                $crate::from_usize::<$var_ty>(ptr);
            }

            $crate::Var::__new(&dropper)
        };
    };
}

mod container;
mod register;
mod var;

pub use container::{from_usize, Container};
pub use register::{Register, VarRegister};
pub use static_init;
pub use var::Var;
