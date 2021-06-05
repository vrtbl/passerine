use crate::construct::symbol::UniqueSymbol;

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub locals:    Vec<UniqueSymbol>,
    pub nonlocals: Vec<UniqueSymbol>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            locals:    vec![],
            nonlocals: vec![],
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
        self.locals.iter().position(|l| l == &unique_symbol)
    }

    pub fn nonlocal_index(&self, unique_symbol: UniqueSymbol) -> Option<usize> {
        self.nonlocals.iter().position(|l| l == &unique_symbol)
    }
}
