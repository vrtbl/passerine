use crate::pipeline::ast::{AST, Construct};
use crate::pipeline::bytecode::{Opcode, Chunk};
use crate::vm::data::Data;
use crate::vm::local::Local;
use crate::utils::annotation::Ann;
use crate::utils::number::split_number;

// so, constanst table is made by walking the tree and sweeping for values
// then, a second pass walks the tree and builds the bytecode
// then, a third pass walks the tree and optimizes the bytecode
// TODO: annotations in bytecode

struct Gen {
    chunk: Chunk,
    depth: usize,
}

impl Gen {
    pub fn new() -> Gen {
        Gen {
            chunk: Chunk::empty(),
            depth: 0,
        }
    }

    fn walk(&mut self, ast: &AST) {
        // push left, push right, push center
        match ast {
            AST::Leaf { data, ann } => {
                self.chunk.code.push(Opcode::Con as u8);
                let mut split = split_number(self.chunk.index_constant(data.clone()));
                self.chunk.code.append(&mut split);
            },
            AST::Node { kind, ann, children } => match kind {
                    Construct::Block  => self.block(&children),
                    Construct::Assign => self.assign(&children),
                    Construct::Symbol => self.symbol(&ann),
            },
        }
    }

    fn block(&mut self, children: &Vec<AST>) {
        self.depth += 1;
        for child in children {
            self.walk(&child);
            self.chunk.code.push(Opcode::Clear as u8);
        }

        // remove the last clear instruction
        self.chunk.code.pop();
        self.depth -= 1;
    }

    fn assign(&mut self, children: &Vec<AST>) {
        if children.len() != 2 {
            panic!("Assignment expects 2 children")
        }

        let symbol = &children[0];
        let expr   = &children[1];

        // eval the expression
        self.walk(&expr);

        // load the following symbol..
        self.chunk.code.push(Opcode::Save as u8);

        // ... the symbol to be loaded
        match symbol {
            AST::Node { kind: Construct::Symbol, ann, children: _ } =>
                self.index_symbol(ann),
            _ => panic!("Assignment expects symbol"),
        }
    }

    fn index_symbol(&mut self, ann: &Ann) {
        let index = self.chunk.index_local(Local::new(ann.contents().to_string(), self.depth));
        self.chunk.code.append(&mut split_number(index));
    }

    fn symbol(&mut self, ann: &Ann) {
        self.chunk.code.push(Opcode::Load as u8);
        self.index_symbol(ann);
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
        let ast    = parse(
            lex(source).unwrap()
        ).unwrap();
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
        let result = vec![
            // con true, save to heck, clear
            (Opcode::Con as u8), 128, (Opcode::Save as u8), 128, (Opcode::Clear as u8),
            // load heck, save to lol, clear
            (Opcode::Load as u8), 128, (Opcode::Save as u8), 129, (Opcode::Clear as u8),
            // con false, save to lmao
            (Opcode::Con as u8), 129, (Opcode::Save as u8), 130,
        ];

        assert_eq!(result, chunk.code);
    }
}
