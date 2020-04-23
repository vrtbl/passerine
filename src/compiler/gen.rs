use crate::pipeline::ast::{AST, Node};
use crate::pipeline::bytecode::{Opcode, Chunk};
use crate::vm::data::Data;
use crate::vm::local::Local;
use crate::utils::annotation::Ann;
use crate::utils::number::split_number;

// The bytecode generator (emitter) walks the AST and produces (unoptimized) Bytecode
// There are plans to add a bytecode optimizer in the future.
// The bytecode generator
// TODO: annotations in bytecode

// TODO: locals are no longer pre-allocated
// either rewrite pre-allocation code,
// remove depth
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
        // NOTE: borrowing here introduces some complexity and cloning...
        // AST should be immutable and not behind shared reference so does not make sense to clone
        match &ast.node {
            Node::Data(data) => self.data(data.clone()),
            Node::Symbol(symbol) => self.symbol(symbol.clone()),
            Node::Block(block) => self.block(&block),
            Node::Assign { pattern, expression } => self.assign(*pattern.clone(), *expression.clone()),
        }
    }

    fn data(&mut self, data: Data) {
        self.chunk.code.push(Opcode::Con as u8);
        let mut split = split_number(self.chunk.index_constant(data));
        self.chunk.code.append(&mut split);
    }

    fn block(&mut self, children: &[AST]) {
        self.depth += 1;
        for child in children {
            self.walk(&child);
            self.chunk.code.push(Opcode::Clear as u8);
        }

        // remove the last clear instruction
        self.chunk.code.pop();
        self.depth -= 1;
    }

    fn assign(&mut self, symbol: AST, expression: AST) {
        // eval the expression
        self.walk(&expression);
        // load the following symbol ...
        self.chunk.code.push(Opcode::Save as u8);
        // ... the symbol to be loaded
        match symbol.node {
            Node::Symbol(l) => self.index_symbol(l),
            _               => unreachable!(),
        };
    }

    fn index_symbol(&mut self, symbol: Local) {
        let index = self.chunk.index_local(symbol);
        self.chunk.code.append(&mut split_number(index));
    }

    fn symbol(&mut self, symbol: Local) {
        self.chunk.code.push(Opcode::Load as u8);
        self.index_symbol(symbol);
    }
}

pub fn gen(ast: AST) -> Chunk {
    let mut generator = Gen::new();
    generator.walk(&ast);
    return generator.chunk;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::lex::lex;
    use crate::compiler::parse::parse;

    #[test]
    fn constants() {
        // TODO: flesh out as more datatypes are added
        let source = "heck = true; lol = 0.0; lmao = false";
        let ast    = parse(
            lex(source).unwrap()
        ).unwrap();
        let chunk = gen(ast);

        let result = vec![
            Data::Boolean(true),
            Data::Real(0.0),
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
