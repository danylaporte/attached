use std::ops::Deref;

use crate::GetVersion;

pub struct Version<T>(pub T, pub u64);

impl<T> Deref for Version<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> GetVersion for Version<T> {
    #[inline]
    fn get_version(&self) -> u64 {
        self.1
    }
}
