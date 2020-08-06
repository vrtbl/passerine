use crate::common::span::Span;

// TODO: implement equality

#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    pub span: Span,
    pub depth: usize
}

impl Local {
    pub fn new(span: Span, depth: usize) -> Local {
        Local { span, depth }
    }
}
