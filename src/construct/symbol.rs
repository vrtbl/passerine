// use std::collections::HashMap;

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

// pub struct SymbolTable<T> {
//     symbols: Vec<T>,
//     lookup:  HashMap<T, usize>,
// }
//
// impl<T> SymbolTable<T> {
//     pub fn new() -> SymbolTable<T> {
//         SymbolTable { symbols: vec![], lookup: HashMap::new() }
//     }
//
//     pub fn insert(&mut self, value: T) {
//         if let Some(index) = self.lookup.get(&value) { return index; }
//
//         self.symbols.push(value);
//         self.lookup.insert(value, self.symbols.len() - 1);
//     }
// }
