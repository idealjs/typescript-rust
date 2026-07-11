//! Arena allocator ported from `internal/core/arena.go`.
//!
//! The Go implementation uses a custom typed arena that grows geometrically.
//! In Rust we use `bumpalo` for the general arena and provide a thin typed
//! wrapper that mirrors the Go API (`New`, `NewSlice`, `Clone`).

use bumpalo::Bump;

/// A typed arena that allocates values of type `T`.
///
/// Mirrors `core.Arena[T]` in Go. Under the hood we use `bumpalo` so that
/// allocations from many typed arenas can share a single chunk-based
/// allocator.
pub struct Arena<T> {
    bump: Bump,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Allocate a single value, initialized via the closure, and return a
    /// reference to it with the same lifetime as the arena.
    ///
    /// Mirrors `Arena[T].New()` in Go (which returns a pointer to
    /// zero-initialized memory; here we require an initial value).
    pub fn alloc(&self, value: T) -> &mut T {
        self.bump.alloc(value)
    }

    /// Allocate a slice of length `len`, initialized via the closure, and
    /// return a mutable reference to it.
    ///
    /// Mirrors `Arena[T].NewSlice(size)` in Go.
    pub fn alloc_slice<F: Fn(usize) -> T>(&self, len: usize, init: F) -> &mut [T] {
        self.bump.alloc_slice_fill_with(len, init)
    }

    /// Allocate a slice initialized with the default value of `T`.
    pub fn alloc_slice_default(&self, len: usize) -> &mut [T]
    where
        T: Default,
    {
        self.bump.alloc_slice_fill_default(len)
    }

    /// Clone a slice into the arena.
    ///
    /// Mirrors `Arena[T].Clone(t)` in Go.
    pub fn clone_slice(&self, source: &[T]) -> &mut [T]
    where
        T: Clone,
    {
        self.bump.alloc_slice_clone(source)
    }

    /// Allocate a single-element slice.
    ///
    /// Mirrors `Arena[T].NewSlice1(t)` in Go.
    pub fn alloc_slice1(&self, value: T) -> &mut [T]
    where
        T: Clone,
    {
        self.bump.alloc_slice_fill_with(1, |_| value.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_single() {
        let arena: Arena<i32> = Arena::new();
        let v = arena.alloc(42);
        assert_eq!(*v, 42);
    }

    #[test]
    fn alloc_slice() {
        let arena: Arena<i32> = Arena::new();
        let s = arena.alloc_slice(3, |i| i as i32);
        assert_eq!(s, &[0, 1, 2]);
    }

    #[test]
    fn clone_slice() {
        let arena: Arena<i32> = Arena::new();
        let original = [1, 2, 3, 4];
        let s = arena.clone_slice(&original);
        assert_eq!(s, &original[..]);
    }
}
