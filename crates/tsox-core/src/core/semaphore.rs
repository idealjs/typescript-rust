pub(crate) use std::sync::{Condvar, Mutex};

pub trait Semaphore: Send + Sync {
    fn acquire(&self) -> SemaphoreGuard<'_>;
}

pub struct SemaphoreGuard<'a> {
    release: Option<Box<dyn FnOnce() + 'a>>,
}

impl<'a> SemaphoreGuard<'a> {
    pub(crate) fn new(release: impl FnOnce() + 'a) -> Self { crate::fntrace::enter("new"); 
        Self {
            release: Some(Box::new(release)),
        }
    }
}

impl<'a> Drop for SemaphoreGuard<'a> {
    fn drop(&mut self) { crate::fntrace::enter("drop"); 
        if let Some(release) = self.release.take() {
            release();
        }
    }
}

pub struct UnlimitedSemaphore;

impl Semaphore for UnlimitedSemaphore {
    fn acquire(&self) -> SemaphoreGuard<'_> { crate::fntrace::enter("acquire"); 
        SemaphoreGuard::new(|| {})
    }
}

pub(crate) struct Inner {
    available: usize,
}

pub struct LimitedSemaphore {
    pub(crate) inner: Mutex<Inner>,
    pub(crate) cvar: Condvar,
}

impl LimitedSemaphore {
    pub fn new(max_concurrency: usize) -> Self { crate::fntrace::enter("new"); 
        assert!(max_concurrency > 0, "max_concurrency must be positive");
        Self {
            inner: Mutex::new(Inner {
                available: max_concurrency,
            }),
            cvar: Condvar::new(),
        }
    }
}

impl Semaphore for LimitedSemaphore {
    fn acquire(&self) -> SemaphoreGuard<'_> { crate::fntrace::enter("acquire"); 
        let mut guard = self.inner.lock().unwrap();
        while guard.available == 0 {
            guard = self.cvar.wait(guard).unwrap();
        }
        guard.available -= 1;
        SemaphoreGuard::new(move || {
            let this = unsafe { &*(self as *const Self) };
            let mut guard = this.inner.lock().unwrap();
            guard.available += 1;
            this.cvar.notify_one();
        })
    }
}

impl UnlimitedSemaphore {
    pub fn try_acquire(&self) -> Option<SemaphoreGuard<'_>> { crate::fntrace::enter("try_acquire"); 
        Some(SemaphoreGuard::new(|| {}))
    }
}

impl LimitedSemaphore {
    pub fn try_acquire(&self) -> Option<SemaphoreGuard<'_>> { crate::fntrace::enter("try_acquire"); 
        let mut guard = self.inner.lock().unwrap();
        if guard.available == 0 {
            return None;
        }
        guard.available -= 1;
        Some(SemaphoreGuard::new(move || {
            let this = unsafe { &*(self as *const Self) };
            let mut guard = self.inner.lock().unwrap();
            guard.available += 1;
            self.cvar.notify_one();
        }))
    }
}
