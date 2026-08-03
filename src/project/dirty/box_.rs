//! Copy-on-write single-value container.
//! Port of Go's `internal/project/dirty/box.go`.

/// Copy-on-write box for a single value.
/// Matches Go's `Box[T Cloneable[T]]`.
pub struct DirtyBox<T: Clone> {
    original: T,
    value: T,
    dirty: bool,
    delete: bool,
}

impl<T: Clone> DirtyBox<T> {
    pub fn new(original: T) -> Self {
        DirtyBox {
            original: original.clone(),
            value: original,
            dirty: false,
            delete: false,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn original(&self) -> &T {
        &self.original
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.delete = false;
        self.dirty = true;
    }

    pub fn change<F>(&mut self, apply: F)
    where
        F: FnOnce(&mut T),
    {
        if !self.dirty {
            self.value = self.value.clone();
            self.dirty = true;
        }
        apply(&mut self.value);
    }

    pub fn change_if<C, A>(&mut self, cond: C, apply: A) -> bool
    where
        C: FnOnce(&T) -> bool,
        A: FnOnce(&mut T),
    {
        if cond(&self.value) {
            self.change(apply);
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self) {
        self.delete = true;
    }

    pub fn finalize(&self) -> (&T, bool) {
        (&self.value, self.dirty || self.delete)
    }
}
