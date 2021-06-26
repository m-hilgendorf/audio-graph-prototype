//! # resource-stack: a simple algorithm to reuse resources
//! ## Usage:
//! ```rust
//! use resource_stack::ResourceStack
//! let mut counter = (0..).into_iter();
//! let mut stack = Resource::new(move || counter.next().unwrap());
//! let a = stack.acquire();
//! stack.release(a);
//! let b = stack.acquire();
//! assert_eq!(a, b);
//! ```
pub struct ResourceStack<T, F> {
    alloc: F,
    stack: Vec<T>,
}

impl<T, F> ResourceStack<T, F>
where
    F: FnMut() -> T,
{
    /// Create a new resource stack with an allocator.
    pub fn new(alloc: F) -> Self {
        Self {
            alloc,
            stack: vec![],
        }
    }

    /// Create a new resource stack with an allocator and max capacity.
    pub fn with_capacity(alloc: F, cap: usize) -> Self {
        Self {
            alloc,
            stack: Vec::with_capacity(cap),
        }
    }

    /// Create or reuse a resource
    pub fn acquire(&mut self) -> T {
        self.stack.pop().unwrap_or((self.alloc)())
    }

    /// Place the resource back on the stack
    pub fn release(&mut self, resource: T) {
        self.stack.push(resource);
    }

    /// Empty the stack
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sanity_check() {
        let mut counter = (0..).into_iter();
        let mut stack = ResourceStack::new(move || counter.next().unwrap());
        let a = stack.acquire();
        let b = stack.acquire();
        stack.release(a);
        assert_eq!(a, stack.acquire());
        stack.release(b);
        assert_eq!(b, stack.acquire());
    }
}
