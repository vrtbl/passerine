use crate::utils::annotation::Ann;
use crate::vm::data::Data;
use crate::vm::local::Local;

// TODO: it might make sense to have the AST enum extend the Construct one
// NOTE: above TODO is in progress

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Node {
    Data(Data),
    Symbol(Local),
    Block(Vec<AST>),
    Assign {
        pattern:    Box<AST>, // Note - should be pattern
        expression: Box<AST>,
    },
    Lambda {
        pattern: Box<AST>,
        expression: Box<AST>,
    },
    Call {
        fun: Box<AST>,
        arg: Box<AST>,
    }
    // TODO: support following constructs as they are implemented
    // Lambda {
    //     pattern:    Box<Node>, // Note - should be pattern
    //     expression: Box<Node>,
    // },
    // Macro {
    //     pattern:    Box<Node>,
    //     expression: Box<Node>,
    // }
    // Form(Vec<Node>) // function call -> (fun a1 a2 .. an)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AST {
    pub node: Node,
    pub ann:  Ann,
}

impl Node {
    // Leafs; terminals
    pub fn data(data: Data)      -> Node { Node::Data(data)     }
    pub fn symbol(symbol: Local) -> Node { Node::Symbol(symbol) }

    // Recursive
    pub fn block(exprs: Vec<AST>)                -> Node { Node::Block(exprs) }
    pub fn assign(pattern: AST, expression: AST) -> Node {
        Node::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }
}

impl AST {
    pub fn new(node: Node, ann: Ann) -> AST {
        AST { node, ann }
    }
}
