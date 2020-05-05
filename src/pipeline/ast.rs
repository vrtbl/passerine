use crate::utils::annotation::Ann;
use crate::vm::data::Data;
use crate::vm::local::Local;

// TODO: it might make sense to have the AST enum extend the Construct one
// NOTE: above TODO is in progress

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Node<'a> {
    Data(Data),
    Symbol(Local),
    Block(Vec<AST<'a>>),
    Assign {
        pattern:    Box<AST<'a>>, // Note - should be pattern
        expression: Box<AST<'a>>,
    },
    Lambda {
        pattern:    Box<AST<'a>>,
        expression: Box<AST<'a>>,
    },
    Call {
        fun: Box<AST<'a>>,
        arg: Box<AST<'a>>,
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

// TODO: Do annotations and nodes need separate lifetimes?
// anns live past the generator, nodes shouldn't
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AST<'a> {
    pub node: Node<'a>,
    pub ann:  Ann<'a>,
}

impl<'a> Node<'a> {
    // Leafs; terminals
    pub fn data(data: Data)      -> Node<'a> { Node::Data(data)     }
    pub fn symbol(symbol: Local) -> Node<'a> { Node::Symbol(symbol) }

    // Recursive
    pub fn block(exprs: Vec<AST>) -> Node { Node::Block(exprs) }

    pub fn assign(pattern: AST<'a>, expression: AST<'a>) -> Node<'a> {
        Node::Assign {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    pub fn lambda(pattern: AST<'a>, expression: AST<'a>) -> Node<'a> {
        Node::Lambda {
            pattern:    Box::new(pattern),
            expression: Box::new(expression)
        }
    }

    pub fn call(fun: AST<'a>, arg: AST<'a>) -> Node<'a> {
        Node::Call {
            fun: Box::new(fun),
            arg: Box::new(arg)
        }
    }
}

impl<'a> AST<'a> {
    pub fn new(node: Node<'a>, ann: Ann<'a>) -> AST<'a> {
        AST { node, ann }
    }
}
