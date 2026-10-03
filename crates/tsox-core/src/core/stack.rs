#[derive(Debug, Clone, Default)]
pub struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self { crate::fntrace::enter("new"); 
        Self { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self { crate::fntrace::enter("with_capacity"); 
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, item: T) { crate::fntrace::enter("push"); 
        self.data.push(item);
    }

    pub fn pop(&mut self) -> T { crate::fntrace::enter("pop"); 
        self.data.pop().expect("stack is empty")
    }

    pub fn peek(&self) -> &T { crate::fntrace::enter("peek"); 
        self.data.last().expect("stack is empty")
    }

    pub fn peek_mut(&mut self) -> &mut T { crate::fntrace::enter("peek_mut"); 
        self.data.last_mut().expect("stack is empty")
    }

    pub fn len(&self) -> usize { crate::fntrace::enter("len"); 
        self.data.len()
    }

    pub fn is_empty(&self) -> bool { crate::fntrace::enter("is_empty"); 
        self.data.is_empty()
    }

    pub fn clear(&mut self) { crate::fntrace::enter("clear"); 
        self.data.clear();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> { crate::fntrace::enter("iter"); 
        self.data.iter()
    }
}
