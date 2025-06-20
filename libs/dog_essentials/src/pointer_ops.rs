use core::ptr;

pub struct PointerTraverser {
    ptr: *const u8,
}

impl PointerTraverser {
    pub fn new(ptr: *const u8) -> Self {
        Self { ptr: ptr }
    }

    /// Advances in memory the specified number of bytes. This cannot check bounds, so it's an unsafe function.
    pub unsafe fn advance_raw(&mut self, num_bytes: u32) {
        unsafe {
            self.ptr = self.ptr.byte_add(num_bytes as usize);
        }
    }

    /// Retreats (goes back) in memory the specified number of bytes. This cannot check bounds, so it's an
    /// unsafe function. It's the opposite of advance_raw.
    pub unsafe fn retreat_raw(&mut self, num_bytes: u32) {
        unsafe {
            self.ptr = self.ptr.byte_sub(num_bytes as usize);
        }
    }

    /// Converts the pointer to one of type T, reads it and clones it (this is necessary), then advances the pointer
    /// in memory with size_of(T) bytes. This cannot check bounds, so it's an unsafe function.
    pub unsafe fn read_and_advance<T: Clone>(&mut self) -> T {
        unsafe {
            let type_ptr: *const T = self.ptr as *const T;
            let val: T = ptr::read_unaligned(type_ptr);
            self.ptr = self.ptr.byte_add(core::mem::size_of::<T>());

            return val;
        }
    }
}

pub unsafe fn read_c_struct<T>(raw_bytes: *const u8) -> T {
    unsafe {
        let type_ptr: *const T = raw_bytes as *const T;
        let val: T = ptr::read_unaligned(type_ptr);
        return val;
    }
}
