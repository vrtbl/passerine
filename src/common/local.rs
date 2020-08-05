use crate::common::span::Span;

// TODO: implement equality

#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    symbol: Span,
    depth: usize
}

impl Local {
    
}
