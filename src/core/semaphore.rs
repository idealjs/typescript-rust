//! Semaphores for concurrency limiting, ported from
//! `internal/core/semaphore.go`.

use std::sync::{Condvar, Mutex};

/// A semaphore that can be acquired (potentially blocking) and released.
///
/// Mirrors `core.Semaphore` in Go. The Go API returns a `release func()`;
/// in Rust we use a `SemaphoreGuard` that releases on drop.
pub trait Semaphore: Send + Sync {
    /// Acquire a permit, blocking until one is available.
    fn acquire(&self) -> SemaphoreGuard<'_>;
}

/// A guard that releases the semaphore permit when dropped.
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

/// An unlimited semaphore that never blocks.
///
/// Mirrors `core.UnlimitedSemaphore` in Go.
pub struct UnlimitedSemaphore;

impl Semaphore for UnlimitedSemaphore {
    fn acquire(&self) -> SemaphoreGuard<'_> {
        SemaphoreGuard::new(|| {})
    }
}

struct Inner {
    available: usize,
}

/// A counting semaphore with a fixed maximum concurrency.
///
/// Mirrors `core.LimitedSemaphore` in Go. Uses `Mutex + Condvar` for
/// blocking acquisition.
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
            // We can't capture `self` here because the guard's lifetime is
            // tied to `&self`. Instead, we use a raw pointer to self, which
            // is safe because the guard cannot outlive the semaphore (the
            // lifetime 'a ensures this).
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
