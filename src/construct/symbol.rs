use std::{
    // collections::HashMap,
    hash::Hash,
};

// TODO: should SharedSymbol be hash of name or something similar?

/// Represents a symbol that corresponds to a name.
/// In other words, if two variables have the same name,
/// even if they exist in different scopes,
/// They will have the same [`SharedSymbol`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SharedSymbol(pub usize);

/// Represents a unique symbol that corresponds to a single variable.
/// In other words, if two variables with the same name exist in different scopes,
/// They will have different [`UniqueSymbol`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniqueSymbol(pub usize);

/// Represents a set of symbols, whether they be unique by name
/// Or unique by some other measure.
pub struct SymbolTable {
    // Ordered list of symbols.
    // A symbol is in the symbol table if it's inner number is less than lowest
    interns: Vec<SharedSymbol>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { interns: vec![] }
    }

    pub fn name(&self, unique: &UniqueSymbol) -> SharedSymbol {
        return self.interns[unique.0];
    }

    pub fn push(&mut self, shared: SharedSymbol) -> UniqueSymbol {
        let index = self.interns.len();
        self.interns.push(shared);
        return UniqueSymbol(index);
    }
}
