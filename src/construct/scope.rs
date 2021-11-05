use std::{
    collections::HashMap,
    hash::Hash,
};
use crate::construct::symbol::UniqueSymbol;

#[derive(Debug, Clone, PartialEq)]
pub struct VecSet<T: Eq + Hash + Clone> {
    order: Vec<T>,
    members: HashMap<T, usize>,
}

impl<T: Eq + Hash + Clone> VecSet<T> {
    pub fn new() -> Self {
        VecSet {
            order: vec![],
            members: HashMap::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.members.insert(item.clone(), self.order.len());
        self.order.push(item)
    }

    pub fn contains(&self, item: &T) -> bool {
        self.members.contains_key(item)
    }

    pub fn index_of(&self, item: &T) -> Option<usize> {
        self.members.get(item).map(|x| *x)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub locals:    VecSet<UniqueSymbol>,
    pub nonlocals: VecSet<UniqueSymbol>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            locals:    VecSet::new(),
            nonlocals: VecSet::new(),
        }
    }

    pub fn is_local(&self, unique_symbol: UniqueSymbol) -> bool {
        self.locals.contains(&unique_symbol)
    }

    pub fn is_nonlocal(&self, unique_symbol: UniqueSymbol) -> bool {
        self.nonlocals.contains(&unique_symbol)
    }

    // TODO: these are linear, should be constant

    pub fn local_index(&self, unique_symbol: UniqueSymbol) -> Option<usize> {
        self.locals.index_of(&unique_symbol)
    }

    pub fn nonlocal_index(&self, unique_symbol: UniqueSymbol) -> Option<usize> {
        self.nonlocals.index_of(&unique_symbol)
    }
}
