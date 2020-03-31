use crate::utils::annotation::Ann;
use crate::vm::data::Data;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Construct {
    Block,
    Assign,
    Symbol, // Variables are weird - are they values, or lanugage constructs?
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AST {
    Node {
        kind:     Construct,
        ann:      Ann,
        children: Vec<AST>,
    },
    Leaf {
        data: Data,
        ann:  Ann,
    },
}

impl AST {
    pub fn node(kind: Construct, ann: Ann, children: Vec<AST>) -> AST {
        AST::Node { kind, ann, children }
    }

    pub fn leaf(data: Data, ann: Ann) -> AST {
        AST::Leaf { data, ann }
    }

    pub fn ann(&self) -> Ann {
        // get the annotation for both nodes and leafs.
        match self {
            AST::Node { ann: a, ..} => *a,
            AST::Leaf { ann: a, ..} => *a,
        }
    }
}
