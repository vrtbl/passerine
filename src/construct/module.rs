// this is a small file

/// Represents a module during the compilation process,
/// i.e. a syntax tree + some state needed to resolve it.
#[derive(Debug, PartialEq, Eq)]
pub struct Module<A, B> {
    pub repr:  A,
    pub assoc: B,
}

impl<A, B> Module<A, B> {
    pub fn new(repr: A, assoc: B) -> Module<A, B> {
        Module { repr, assoc }
    }
}

pub type ThinModule<A> = Module<A, ()>;

impl<A> ThinModule<A> {
    pub fn thin(repr: A) -> ThinModule<A> {
        Module::new(repr, ())
    }
}
