use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
    ops::{Index, IndexMut}
};
use core::sync::atomic::AtomicUsize;
use crate::memory::{allocate_pages, PAGE_SIZE};

pub mod mutex;

pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    pub const fn new() -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {}
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

/// An object that is meant to be initialized once at boot, then accessed in read-only.
///
/// The value **must** be initialized with `init` before calling `get`. Failure to do so results in
/// an undefined behavior.
pub struct OnceInit<T> {
    /// The inner value. If `None`, it has not been initialized yet.
    val: UnsafeCell<MaybeUninit<T>>,
}

impl<T> OnceInit<T> {
    /// Creates a new instance waiting to be initialized.
    ///
    /// # Safety
    ///
    /// The value **must** be initialized with before calling `get`.
    pub const unsafe fn new() -> Self {
        Self {
            val: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Initializes with the given value.
    ///
    /// If already initialized, the previous value is **not** dropped.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to enforce concurrency rules.
    pub unsafe fn init(&self, val: T) {
        unsafe { (*self.val.get()).write(val) };
    }

    /// Returns the inner value.
    pub fn get(&self) -> &T {
        unsafe { (*self.val.get()).assume_init_ref() }
    }

    /// Returns the inner value as mutable.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { (*self.val.get()).assume_init_mut() }
    }
}

unsafe impl<T> Sync for OnceInit<T> {}


pub struct CircularBuffer<T> {
    buffer: *mut T,
    buffer_size: usize,
    start: AtomicUsize,
    end: AtomicUsize,
    write_lock: SpinLock,
    read_lock: SpinLock,
}

impl<T> CircularBuffer<T> {
    pub fn allocate(pages: usize) -> CircularBuffer<T> {
        let buffer_size = pages * PAGE_SIZE / size_of::<T>();
        let buffer = allocate_pages(pages) as *mut T;
        CircularBuffer {
            buffer,
            buffer_size,
            end: AtomicUsize::new(0),
            start: AtomicUsize::new(0),
            write_lock: SpinLock::new(),
            read_lock: SpinLock::new(),
        }
    }
    fn start(&self) -> usize { self.start.load(Ordering::Acquire) }
    fn end(&self) -> usize { self.end.load(Ordering::Acquire) }
    pub fn size(&self) -> usize {
        let start = self.start();
        let end = self.end();
        if end < start {
            end - start + self.buffer_size
        } else {
            end - start
        }
    }

    pub fn writer(&self) -> CircularWriteLock<T> {
        self.write_lock.lock();
        CircularWriteLock {
            buffer: self
        }
    }
    pub fn reader(&self) -> CircularReadLock<T> {
        self.read_lock.lock();
        CircularReadLock {
            buffer: self
        }
    }
}

pub struct CircularWriteLock<'a, T> {
    buffer: &'a CircularBuffer<T>
}

impl<'a, T> CircularWriteLock<'a, T> {
    pub fn push_back(&mut self, value: T) {
        let end = self.buffer.end();
        let start = self.buffer.start();
        if (end+1)%self.buffer.buffer_size == start {
            panic!("CircularWriteLock: buffer overflow");
        }
        unsafe { self.buffer.buffer.add(self.buffer.end()).write(value); }
        self.buffer.end.store((end+1)%self.buffer.buffer_size, Ordering::Release);
    }
    pub fn size(&self) -> usize {
        self.buffer.size()
    }
}

impl<'a, T> Drop for CircularWriteLock<'a, T> {
    fn drop(&mut self) {
        self.buffer.write_lock.unlock();
    }
}

pub struct CircularReadLock<'a, T> {
    buffer: &'a CircularBuffer<T>
}

impl<'a, T> CircularReadLock<'a, T> {
    pub fn pop_front(&mut self) -> T {
        let end = self.buffer.end();
        let start = self.buffer.start();
        if start == end {
            panic!("CircularReadLock: empty buffer");
        }
        let value = unsafe { self.buffer.buffer.add(start).read() };
        self.buffer.start.store((start + 1) % self.buffer.buffer_size, Ordering::Release);
        value
    }
    pub fn size(&self) -> usize {
        self.buffer.size()
    }
}

impl<'a, T> Drop for CircularReadLock<'a, T> {
    fn drop(&mut self) {
        self.buffer.read_lock.unlock();
    }
}