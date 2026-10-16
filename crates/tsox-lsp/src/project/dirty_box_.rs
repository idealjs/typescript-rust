pub struct DirtyBox<T: Clone> {
    original: T,
    value: T,
    dirty: bool,
    delete: bool,
}

impl<T: Clone> DirtyBox<T> {
    pub fn new(original: T) -> Self { ::tsox_core::fntrace::enter("new"); 
        DirtyBox {
            original: original.clone(),
            value: original,
            dirty: false,
            delete: false,
        }
    }

    pub fn value(&self) -> &T { ::tsox_core::fntrace::enter("value"); 
        &self.value
    }

    pub fn original(&self) -> &T { ::tsox_core::fntrace::enter("original"); 
        &self.original
    }

    pub fn dirty(&self) -> bool { ::tsox_core::fntrace::enter("dirty"); 
        self.dirty
    }

    pub fn set(&mut self, value: T) { ::tsox_core::fntrace::enter("set"); 
        self.value = value;
        self.delete = false;
        self.dirty = true;
    }

    pub fn change<F>(&mut self, apply: F)
    where
        F: FnOnce(&mut T),
    { ::tsox_core::fntrace::enter("change"); 
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
    { ::tsox_core::fntrace::enter("change_if"); 
        if cond(&self.value) {
            self.change(apply);
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self) { ::tsox_core::fntrace::enter("delete"); 
        self.delete = true;
    }

    pub fn deleted(&self) -> bool { ::tsox_core::fntrace::enter("deleted"); 
        self.delete
    }

    pub fn finalize(&self) -> (T, bool)
    where
        T: Default,
    { ::tsox_core::fntrace::enter("finalize"); 
        if self.delete {
            (T::default(), true)
        } else {
            (self.value.clone(), self.dirty)
        }
    }
}
