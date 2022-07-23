use std::collections::BTreeMap;
use crate::{Stack, Heap};

pub struct HandlerId(usize);

pub struct Handler {
    fiber: Fiber,
}

pub struct Fiber {
    handlers: BTreeMap<HandlerId, Handler>,
    stack:    Stack,
    heap:     Heap,
    parent:   FiberId,
}
