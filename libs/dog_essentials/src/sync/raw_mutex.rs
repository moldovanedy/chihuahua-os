use core::{
    hint,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct RawMutex<T> {
    is_locked: AtomicBool,
    data: T,
}

impl<T> RawMutex<T> {
    pub const fn new(initial_data: T) -> Self {
        RawMutex {
            is_locked: AtomicBool::new(false),
            data: initial_data,
        }
    }

    pub fn lock(&'_ self) {
        //spinlock
        if self.is_locked() {
            while self.is_locked() {
                hint::spin_loop();
            }
        }

        self.is_locked.store(true, Ordering::Release);
    }

    pub fn unlock(&self) {
        self.is_locked.store(false, Ordering::Release);
    }

    pub fn get_value(&self) -> Option<&T> {
        if !self.is_locked() {
            return None;
        }

        return Some(&self.data);
    }

    pub fn set_value(&mut self, new_value: T) {
        if !self.is_locked() {
            return;
        }

        self.data = new_value;
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked.load(Ordering::Acquire)
    }
}
