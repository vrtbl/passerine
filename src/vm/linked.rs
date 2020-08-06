use std::mem;

/// A linked list of usizes that functions as a stack.
/// Used to keep track of the current stack frame while preserving
/// the indicies of past frames.
#[derive(Debug)]
pub struct Linked(usize, Option<Box<Linked>>);

impl Linked {
    pub fn new(index: usize) -> Linked {
        Linked(0, None)
    }

    pub fn prepend(&mut self, new_index: usize) {
        let old_tail = mem::replace(&mut self.1, None);
        let old = Linked(self.0, old_tail);
        *self = Linked(new_index, Some(Box::new(old)));
    }

    pub fn prepop(&mut self) -> usize {
        let index = self.0;
        mem::replace(self, *self.1.expect("can not prepop past base item"));
        return index;
    }

    pub fn peek(&self) -> usize { self.0 }
}
