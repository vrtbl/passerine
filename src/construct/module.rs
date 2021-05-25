// this is a small file

/// Represents a module during the compilation process,
/// i.e. a syntax tree + some state needed to resolve it.
pub struct Module<A, B> {
    pub syntax_tree: A,
    pub associated:  B,
}

impl<A, B> Module<A, B> {
    pub fn new(st: A, associated: B) -> Module<A, B> {
        Module { syntax_tree: st, associated }
    }
}

pub type ThinModule<A> = Module<A, ()>;
