use bumpalo::Bump;

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

    pub fn alloc(&self, value: T) -> &mut T {
        self.bump.alloc(value)
    }

    pub fn alloc_slice<F: Fn(usize) -> T>(&self, len: usize, init: F) -> &mut [T] {
        self.bump.alloc_slice_fill_with(len, init)
    }

    pub fn alloc_slice_default(&self, len: usize) -> &mut [T]
    where
        T: Default,
    {
        self.bump.alloc_slice_fill_default(len)
    }

    pub fn clone_slice(&self, source: &[T]) -> &mut [T]
    where
        T: Clone,
    {
        self.bump.alloc_slice_clone(source)
    }

    pub fn alloc_slice1(&self, value: T) -> &mut [T]
    where
        T: Clone,
    {
        self.bump.alloc_slice_fill_with(1, |_| value.clone())
    }
}

#[cfg(test)]
mod tests;
