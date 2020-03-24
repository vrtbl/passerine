use crate::pipeline::token::Token;
use crate::utils::annotation::Annotation;
use crate::vm::data::Data;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operation {
    Block,
    Assign,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AST {
    Node {
        kind:     Operation,
        ann:      Annotation,
        children: Vec<AST>,
    },
    Leaf {
        data: Data,
        ann:  Annotation,
    },
}

impl AST {
    pub fn node(kind: Operation, ann: Annotation, children: Vec<AST>) -> AST {
        AST::Node {
            kind:     kind,
            ann:      ann,
            children: children,
        }
    }

    pub fn leaf(data: Data, ann: Annotation) -> AST {
        AST::Leaf {
            data: data,
            ann:  ann,
        }
    }

    pub fn ann(&self) -> Annotation {
        // get the annotation for both nodes and leafs.
        match self {
            AST::Node { kind: _, ann: a, children: _ } => a.clone(),
            AST::Leaf { data: _, ann: a }              => a.clone(),
        }
    }
}
