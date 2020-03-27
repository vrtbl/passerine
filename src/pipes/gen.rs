use crate::pipeline::ast::{AST, Construct};
use crate::pipeline::bytecode::{Opcode, Chunk};
use crate::vm::data::Data;
use crate::utils::number::split_number;

// so, constanst table is made by walking the tree and sweeping for values
// then, a second pass walks the tree and builds the bytecode
// then, a third pass walks the tree and optimizes the bytecode
// TODO: consts and bytecode in single pass.
// TODO: annotations in bytecode

struct Gen {
    chunk: Chunk,
}

impl Gen {
    pub fn new() -> Gen {
        Gen {
            chunk: Chunk::empty(),
        }
    }

    fn walk(&mut self, ast: &AST) {
        // push left, push right, push center
        match ast {
            AST::Leaf { data, ann: _ } => {
                self.chunk.code.push(Opcode::Con as u8);
                self.chunk.code.append(&mut split_number(self.chunk.index_constant(&data)));
            },
            AST::Node { kind, ann: _, children } => match kind {
                    Construct::Block  => self.block(&children),
                    Construct::Assign => self.assign(&children),
            },
        }
    }

    fn block(&mut self, children: &Vec<AST>) {
        for child in children {
            self.walk(&child);
            self.chunk.code.push(Opcode::Clear as u8);
        }

        // remove the last clear instruction
        self.chunk.code.pop();
    }

    fn assign(&mut self, children: &Vec<AST>) {
        if children.len() != 2 {
            panic!("Assignment expects 2 children")
        }

        let symbol = &children[0];
        let expr   = &children[1];

        // load the const symbol
        // TODO: is it redundant? check that left arm is symbol
        if let AST::Leaf { data: _, ann: _ } = symbol {
            self.walk(symbol);
        } else {
            panic!("Symbol expected, found something else")
        }

        // eval the expression
        self.walk(&expr);

        // save the binding
        self.chunk.code.push(Opcode::Save as u8);
    }
}

pub fn gen(ast: AST) -> Chunk {
    let mut generator = Gen::new();
    generator.walk(&ast);
    return generator.chunk;
}

// TODO: rewrite tests

#[cfg(test)]
mod test {
    use super::*;
    use crate::pipes::lex::lex;
    use crate::pipes::parse::parse;

    #[test]
    fn constants() {
        // TODO: flesh out as more datatypes are added
        let source = "heck = true; lol = heck; lmao = false";
        let ast    = parse(lex(source).unwrap()).unwrap();
        let chunk = gen(ast);

        let result = vec![
            Data::Boolean(true),
            Data::Boolean(false),
        ];

        assert_eq!(chunk.constants, result);
    }

    #[test]
    fn bytecode() {
        let source = "heck = true; lol = heck; lmao = false";
        let ast    = parse(lex(source).unwrap()).unwrap();

        let chunk = gen(ast);
        let result = vec![0, 128, 0, 129, 1, 3, 0, 130, 0, 128, 2, 1, 3, 0, 131, 0, 132, 1];
        // con heck, con true, save, clear |                          |                  |
        // con lol, con heck, load heck, save, clear                  |                  |
        // load lmao, load false, save, clear                                            |

        assert_eq!(result, chunk.code);
    }
}
