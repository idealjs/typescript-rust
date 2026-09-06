use std::sync::{Condvar, Mutex};

pub trait Semaphore: Send + Sync {

    fn acquire(&self) -> SemaphoreGuard<'_>;
}

pub struct SemaphoreGuard<'a> {
    release: Option<Box<dyn FnOnce() + 'a>>,
}

impl<'a> SemaphoreGuard<'a> {
    fn new(release: impl FnOnce() + 'a) -> Self {
        Self {
            release: Some(Box::new(release)),
        }
    }
}

impl<'a> Drop for SemaphoreGuard<'a> {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            release();
        }
    }
}

pub struct UnlimitedSemaphore;

impl Semaphore for UnlimitedSemaphore {
    fn acquire(&self) -> SemaphoreGuard<'_> {
        SemaphoreGuard::new(|| {})
    }
}

struct Inner {
    available: usize,
}

pub struct LimitedSemaphore {
    inner: Mutex<Inner>,
    cvar: Condvar,
}

impl LimitedSemaphore {
    pub fn new(max_concurrency: usize) -> Self {
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
    fn acquire(&self) -> SemaphoreGuard<'_> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlimited() {
        let s = UnlimitedSemaphore;
        let _g = s.acquire();
    }

    #[test]
    fn limited() {
        let s = LimitedSemaphore::new(2);
        let g1 = s.acquire();
        let g2 = s.acquire();
        drop(g1);
        drop(g2);
        let _g3 = s.acquire();
    }
}
