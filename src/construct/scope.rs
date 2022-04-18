use std::{
    collections::HashMap,
    hash::Hash,
};

use crate::construct::symbol::UniqueSymbol;

/// Represents an ordered set of elements with O(1) membership checking.
/// Note that this is insert-only.
/// Should be treated like an allocation pool.
#[derive(Debug, Clone, PartialEq)]
pub struct VecSet<T: Eq + Hash + Clone> {
    order:   Vec<T>,
    members: HashMap<T, usize>,
}

impl<T: Eq + Hash + Clone> VecSet<T> {
    pub fn new() -> Self {
        VecSet {
            order:   vec![],
            members: HashMap::new(),
        }
    }

    /// Push a member onto the Vec.
    /// Does not check if the member already exists in the Vec.
    pub fn push(&mut self, item: T) {
        if !self.contains(&item) {
            self.members.insert(item.clone(), self.order.len());
            self.order.push(item)
        }
    }

    pub fn contains(&self, item: &T) -> bool { self.members.contains_key(item) }

    pub fn index_of(&self, item: &T) -> Option<usize> {
        self.members.get(item).map(|x| *x)
    }

    // Marks an item as removed. Does not actually remove the item to preserve
    // indexes in hash map. Returns none if the item existed and was removed.
    pub fn remove(&mut self, item: &T) -> bool {
        self.members.remove(item).is_some()
    }

    // TODO: this function needs to be DELETED:

    pub fn items(&self) -> Vec<T> {
        self.order
            .iter()
            .filter(|x| self.contains(x))
            .map(|x| x.clone())
            .collect()
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
