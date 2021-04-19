use std::ffi::c_void;

pub(super) struct UntypedPtr {
    cell: *mut c_void,
    drop: Box<dyn Fn()>,
}

impl UntypedPtr {
    pub fn new<T>(val: T) -> Self {
        let cell = Box::into_raw(Box::new(val)) as *mut c_void;

        let drop = Box::new(move || unsafe {
            Box::from_raw(cell as *mut T);
        });

        Self { cell, drop }
    }

    pub fn get<T>(&self) -> &T {
        unsafe { &*(self.cell as *mut T) }
    }

    pub fn get_mut<T>(&mut self) -> &mut T {
        unsafe { &mut *(self.cell as *mut T) }
    }

    pub fn into_inner<T>(self) -> T {
        *unsafe { Box::from_raw(self.cell as *mut T) }
    }
}

impl Drop for UntypedPtr {
    fn drop(&mut self) {
        (*self.drop)();
    }
}

#[test]
fn value_inside_ptr_is_dropped() {
    use std::sync::Arc;

    let v = Arc::new(());
    let p = UntypedPtr::new(Arc::clone(&v));

    assert_eq!(Arc::strong_count(&v), 2);

    drop(p);

    assert_eq!(Arc::strong_count(&v), 1);
}
