use std::mem;

/// A linked list of usizes that functions as a stack.
/// Used to keep track of the current stack frame while preserving
/// the indicies of past frames.
#[derive(Debug)]
pub struct Linked(usize, Option<Box<Linked>>);

impl Linked {
    /// Create a new linked stack provided the first item.
    /// A linked stack can not be empty.
    pub fn new(index: usize) -> Linked {
        Linked(index, None)
    }

    /// Add a new entry to the top of the linked stack.
    pub fn prepend(&mut self, new_index: usize) {
        let old_tail = mem::replace(&mut self.1, None);
        let old = Linked(self.0, old_tail);
        *self = Linked(new_index, Some(Box::new(old)));
    }

    /// Remove the top entry of the linked stack, returning the top value.
    pub fn prepop(&mut self) -> usize {
        let index = self.0;
        *self = *mem::replace(&mut self.1, None)
            .expect("Can not pop back past root link");
        return index;
    }

    /// Peek at the current item on top of the stack.
    pub fn peek(&self) -> usize { self.0 }
}
