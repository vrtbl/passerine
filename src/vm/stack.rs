use std::{
    mem,
    rc::Rc,
    cell::RefCell
};

use crate::common::data::Data;

use crate::vm::{
    tag::Tagged,
    linked::Linked,
};

/// A stack of `Tagged` `Data`.
/// Note that in general the stack is expected to follow the following pattern:
/// ```plain
/// FV...V...F V...T... ...
/// ```
/// Or in other words, a frame followed by a block of *n* values that are locals
/// followed by *n* temporaries, ad infinitum.
#[derive(Debug)]
pub struct Stack {
    // TODO: just use a Vec<usize>?
    pub frames: Linked,
    pub stack: Vec<Tagged>
}

impl Stack {
    /// Create a new `Stack` with a single frame.
    pub fn init() -> Stack {
        Stack {
            frames: Linked::new(0),
            stack: vec![Tagged::frame()],
        }
    }

    /// Pushes some `Data` onto the `Stack`, tagging it along the way
    #[inline]
    pub fn push_data(&mut self, data: Data) {
        self.stack.push(Tagged::new(data))
    }

    /// Pushes some `Tagged` `Data` onto the `Stack` without unwrapping it.
    #[inline]
    pub fn push_tagged(&mut self, tagged: Tagged) {
        self.stack.push(tagged)
    }

    /// Pops some `Data` of the `Stack`, panicking if what it pops is not `Data`.
    /// Note that this will never return a `Heaped` value, rather cloning the value inside.
    #[inline]
    pub fn pop_data(&mut self) -> Data {
        let value = self.stack.pop()
            .expect("VM tried to pop empty stack, stack should never be empty");

        match value.data() {
            Data::Frame     => unreachable!("tried to pop data, Frame is not data"),
            Data::Heaped(r) => r.borrow().clone(),
            data            => data,
        }
    }

    /// Pops a stack frame from the `Stack`, restoring the previous frame.
    /// Panics if there are no frames left on the stack.
    #[inline]
    pub fn pop_frame(&mut self) {
        let index = self.frames.prepop();
        if self.stack.len() - 1 == index {
            self.stack.pop();
        } else {
            unreachable!("Expected frame on top of stack, found data")
        }
    }

    /// Pushes a new stack frame onto the `Stack`.
    #[inline]
    pub fn push_frame(&mut self) {
        self.frames.prepend(self.stack.len());
        self.stack.push(Tagged::frame());
    }

    // TODO: Change behaviour? Make if heapify a specified local?
    /// Wraps the top data value on the stack in `Data::Heaped`,
    /// if it is not already on the heap.
    #[inline]
    pub fn heapify(&mut self, index: usize) -> Rc<RefCell<Data>> {
        let local_index = self.frames.peek() + index + 1;

        let data = mem::replace(&mut self.stack[local_index], Tagged::frame()).data();
        let reference = Rc::new(RefCell::new(data));
        let heaped = Data::Heaped(reference.clone());
        mem::drop(mem::replace(&mut self.stack[local_index], Tagged::new(heaped)));

        return reference;
    }

    /// Gets a local and pushes it onto the top of the `Stack`
    pub fn get_local(&mut self, index: usize) {
        let local_index = self.frames.peek() + index + 1;

        // a little bit of shuffling involved
        // I know that something better than this can be done
        let data = mem::replace(&mut self.stack[local_index], Tagged::frame()).data();
        let copy = data.clone();
        mem::drop(mem::replace(&mut self.stack[local_index], Tagged::new(data)));

        self.push_data(copy);
    }

    /// Sets a local - note that this function doesn't do much.
    /// It's a simple swap-and-drop.
    /// If a new local is being declared,
    /// it's literally a bounds-check and no-op.
    pub fn set_local(&mut self, index: usize) {
        let local_index = self.frames.peek() + index + 1;

        if self.stack.len() - 1 == local_index {
            // local is already in the correct spot; we declare it
            return;
        } else if self.stack.len() <= local_index {
            unreachable!("Can not set local that is not yet on stack");
        } else {
            // get the old local
            let data = mem::replace(&mut self.stack[local_index], Tagged::frame()).data();

            // replace the old value with the new one if on the heap
            let tagged = match data {
                Data::Frame => unreachable!(),
                // if it is on the heap, we replace in the old value
                Data::Heaped(ref cell) => {
                    mem::drop(cell.replace(self.pop_data()));
                    Tagged::new(data)
                }
                // if it's not on the heap, we assume it's data,
                // and do a quick swap-and-drop
                _ => self.stack.pop().unwrap(),
            };

            mem::drop(mem::replace(&mut self.stack[local_index], tagged))
        }
    }
}
