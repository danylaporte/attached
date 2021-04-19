use std::cmp::max;

pub trait GetVersion {
    fn get_version(&self) -> u64;

    fn max(self, version: &mut u64) -> Self
    where
        Self: Sized,
    {
        *version = max(self.get_version(), *version);
        self
    }
}

impl<T: GetVersion> GetVersion for &T {
    #[inline]
    fn get_version(&self) -> u64 {
        (**self).get_version()
    }
}
