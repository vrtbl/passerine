use crate::compiler::syntax::Syntax;

/// A trait that represents a compilation step,
/// i.e. 'lowering' a program representation from one form to another.
pub trait Lower {
    type Out;
    fn lower(self) -> Result<Self::Out, Syntax>;
}
