use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::{
    hint,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Debug)]
pub struct Mutex<T: ?Sized> {
    is_locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(initial_data: T) -> Self {
        Mutex {
            is_locked: AtomicBool::new(false),
            data: UnsafeCell::new(initial_data),
        }
    }

    /// Returns a raw pointer to the data.
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.data.get()
    }

    /// Acquires mutex lock if it's unlocked, otherwise blocks the thread until it's unlocked.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        loop {
            if let Some(guard) = self.try_lock_weak() {
                break guard;
            }

            //spinlock
            while self.is_locked() {
                hint::spin_loop();
            }
        }
    }

    /// This should be safe, as Rust guarantees that mutable references are exclusive.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    /// Very unsafe.
    pub unsafe fn force_unlock(&self) {
        self.is_locked.store(false, Ordering::Release);
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked.load(Ordering::Acquire)
    }

    /// Tries to acquire a lock and returns a guard if successful.
    #[inline(always)]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self
            .is_locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(MutexGuard {
                lock: &self.is_locked,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            None
        }
    }

    /// Tries to acquire a lock and returns a guard if successful. It's "weak" because it might
    /// fail even when the mutex is unlocked, but is faster than [`Mutex::try_lock`].
    #[inline(always)]
    pub fn try_lock_weak(&self) -> Option<MutexGuard<'_, T>> {
        if self
            .is_locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(MutexGuard {
                lock: &self.is_locked,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            None
        }
    }
}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: *mut T,
}

unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Send for MutexGuard<'_, T> {}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // We know statically that only we are referencing data
        unsafe { &*self.data }
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // We know statically that only we are referencing data
        unsafe { &mut *self.data }
    }
}
