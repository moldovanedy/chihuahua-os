use core::cell::UnsafeCell;

/// An unsafe container for static data (although it can be used for non-static data as well, there are better
/// alternatives, such as Mutex, UnsafeCell, Cell etc.). Only use this in single-threaded environments.
pub struct StaticCell<T> {
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for StaticCell<T> {}

impl<T> StaticCell<T> {
    /// Create a new cell with an initial value.
    pub const fn new(initial_value: T) -> Self {
        StaticCell {
            data: UnsafeCell::new(initial_value),
        }
    }

    /// Gets a reference to the value. Note that this is dangerous and leads to UB when there are multiple alive
    /// references to the data.
    pub fn get_value_unsafe(&self) -> &T {
        unsafe { self.data.as_ref_unchecked() }
    }

    /// Directly sets the value. Note that this can be dangerous.
    pub fn set_value_unsafe(&self, value: T) {
        unsafe {
            (*self.data.get()) = value;
        }
    }
}
