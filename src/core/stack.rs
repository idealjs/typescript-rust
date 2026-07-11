//! A simple stack, ported from `internal/core/stack.go`.

/// A growable stack.
///
/// Mirrors `core.Stack[T]` in Go.
#[derive(Debug, Clone, Default)]
pub struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item);
    }

    /// Pop the top element. Panics if the stack is empty.
    pub fn pop(&mut self) -> T {
        self.data.pop().expect("stack is empty")
    }

    /// Peek at the top element. Panics if the stack is empty.
    pub fn peek(&self) -> &T {
        self.data.last().expect("stack is empty")
    }

    /// Peek at the top element mutably. Panics if the stack is empty.
    pub fn peek_mut(&mut self) -> &mut T {
        self.data.last_mut().expect("stack is empty")
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut s = Stack::new();
        s.push(1);
        s.push(2);
        s.push(3);
        assert_eq!(s.len(), 3);
        assert_eq!(s.pop(), 3);
        assert_eq!(s.pop(), 2);
        assert_eq!(s.pop(), 1);
        assert!(s.is_empty());
    }

    #[test]
    fn peek() {
        let mut s = Stack::new();
        s.push("a");
        s.push("b");
        assert_eq!(*s.peek(), "b");
    }
}
