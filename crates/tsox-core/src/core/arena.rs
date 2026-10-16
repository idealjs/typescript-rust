pub(crate) use bumpalo::Bump;

pub struct Arena<T> {
    bump: Bump,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self { crate::fntrace::enter("default"); 
        Self::new()
    }
}

impl<T> Arena<T> {
    pub fn new() -> Self { crate::fntrace::enter("new"); 
        Self {
            bump: Bump::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn alloc(&self, value: T) -> &mut T { crate::fntrace::enter("alloc"); 
        self.bump.alloc(value)
    }

    pub fn alloc_slice<F: Fn(usize) -> T>(&self, len: usize, init: F) -> &mut [T] { crate::fntrace::enter("alloc_slice"); 
        self.bump.alloc_slice_fill_with(len, init)
    }

    pub fn alloc_slice_default(&self, len: usize) -> &mut [T]
    where
        T: Default,
    { crate::fntrace::enter("alloc_slice_default"); 
        self.bump.alloc_slice_fill_default(len)
    }

    pub fn clone_slice(&self, source: &[T]) -> &mut [T]
    where
        T: Clone,
    { crate::fntrace::enter("clone_slice"); 
        self.bump.alloc_slice_clone(source)
    }

    pub fn alloc_slice1(&self, value: T) -> &mut [T]
    where
        T: Clone,
    { crate::fntrace::enter("alloc_slice1"); 
        self.bump.alloc_slice_fill_with(1, |_| value.clone())
    }
}
