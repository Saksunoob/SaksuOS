/*
 * Copyright 2024 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

//! Mutually exclusive access primitive implementation.
//!
//! A [`Mutex`] protects its wrapped data from being accessed concurrently, avoid data races.
//!
//! One particularity with kernel development is that multi-threading is not the
//! only way to get concurrency issues. Another factor to take into account is
//! that fact that an interruption may be triggered at any moment, unless disabled.
//!
//! For this reason, mutexes in the kernel are equipped with an option allowing to disable
//! interrupts while being locked.
//!
//! If an exception is raised while a mutex that disables interruptions is
//! acquired, the behaviour is undefined.

use crate::{
    sync::SpinLock,
};
use core::{
    cell::UnsafeCell,
    fmt::{self, Formatter},
    ops::{Deref, DerefMut},
    ptr,
};

/// Type used to declare a guard meant to unlock the associated `Mutex` at the
/// moment the execution gets out of the scope of its declaration.
pub struct MutexGuard<'m, T: ?Sized> {
    /// The locked mutex.
    mutex: &'m Mutex<T>,
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.mutex.inner.get()).data }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.mutex.inner.get()).data }
    }
}

impl<T: ?Sized> MutexGuard<'_, T> {
    pub fn inner_ptr(&mut self) -> *const T {
        let inner = unsafe { self.mutex.inner.get().as_ref().unwrap() };
        let ptr = ptr::from_ref(&inner.data);
        return ptr;
    }
}

impl<T: ?Sized> !Send for MutexGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

impl<T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            self.mutex.unlock();
        }
    }
}

/// The inner structure of [`Mutex`].
struct MutexIn<T: ?Sized> {
    /// The spinlock for the underlying data.
    spin: SpinLock,
    /// ThSpinLockssociated to the mutex.
    data: T,
}

/// The object wrapped in a `Mutex` can be accessed by only one thread at a time.
///
/// The `INT` generic parameter tells whether interrupts are allowed while
/// the mutex is locked. The default value is `true`.
pub struct Mutex<T: ?Sized> {
    /// An unsafe cell to the inner structure of the Mutex.
    inner: UnsafeCell<MutexIn<T>>,
}

impl<T> Mutex<T> {
    /// Creates a new Mutex with the given data to be owned.
    pub const fn new(data: T) -> Self {
        Self {
            inner: UnsafeCell::new(MutexIn {
                spin: SpinLock::new(),
                data,
            }),
        }
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Locks the mutex.
    ///
    /// If the mutex is already locked, the thread shall wait until it becomes available.
    ///
    /// The function returns a [`MutexGuard`] associated with `self`. When dropped, the mutex is
    /// unlocked.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // Safe because using the spinlock
        let inner = unsafe { &mut *self.inner.get() };
        inner.spin.lock();
        MutexGuard {
            mutex: self
        }
    }

    /// Unlocks the mutex. This function should not be used directly since it is called when the
    /// mutex guard is dropped.
    ///
    /// `int_state` is the state of interruptions before locking.
    ///
    /// # Safety
    ///
    /// If the mutex is not locked, the behaviour is undefined.
    ///
    /// Unlocking the mutex while the resource is being used may result in concurrent accesses.
    pub unsafe fn unlock(&self) {
        let inner = unsafe { &mut (*self.inner.get()) };
        inner.spin.unlock();
    }

    /// Returns the pointer to the inner value. Use is heavily discouraged as this defets the point
    /// of a Mutex
    pub unsafe fn inner_ptr(&self) -> *const T {
        let inner = unsafe { &mut (*self.inner.get()) };
        inner.spin.lock();
        let ptr = ptr::from_ref(&inner.data);
        inner.spin.unlock();
        return ptr;
    }
}

impl<T> Mutex<T> {
    /// Locks the mutex, consumes it and returns the inner value.
    ///
    /// If the mutex disables interruptions, it is the caller's responsibility to handle it
    /// afterward.
    pub fn into_inner(self) -> T {
        // Make sure no one is using the resource
        let inner = unsafe { &mut *self.inner.get() };
        inner.spin.lock();
        self.inner.into_inner().data
    }
}

unsafe impl<T> Sync for Mutex<T> {}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let guard = self.lock();
        fmt::Debug::fmt(&*guard, f)
    }
}