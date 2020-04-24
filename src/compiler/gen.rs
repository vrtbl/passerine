use crate::pipeline::ast::{AST, Node};
use crate::pipeline::bytecode::Opcode;
use crate::vm::data::Data;
use crate::vm::local::Local;
use crate::utils::annotation::Ann;
use crate::utils::number::split_number;

// The bytecode generator (emitter) walks the AST and produces (unoptimized) Bytecode
// There are plans to add a bytecode optimizer in the future.
// The bytecode generator
// TODO: annotations in bytecode

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Chunk {
    pub code:      Vec<u8>,    // each byte is an opcode or a number-stream
    pub offsets:   Vec<usize>, // each usize indexes the bytecode op that begins each line
    pub constants: Vec<Data>,  // number-stream indexed, used to load constants
    pub locals:    Vec<Local>, // ''                                  variables
}

impl Chunk {
    pub fn empty() -> Chunk {
        Chunk {
            code:      vec![],
            offsets:   vec![],
            constants: vec![],
            locals:    vec![],
        }
    }

    // TODO: bytecode chunk dissambler

    fn walk(&mut self, ast: &AST) {
        // push left, push right, push center
        // NOTE: borrowing here introduces some complexity and cloning...
        // AST should be immutable and not behind shared reference so does not make sense to clone
        match &ast.node {
            Node::Data(data) => self.data(data.clone()),
            Node::Symbol(symbol) => self.symbol(symbol.clone()),
            Node::Block(block) => self.block(&block),
            Node::Assign { pattern, expression } => self.assign(*pattern.clone(), *expression.clone()),
            Node::Lambda { pattern, expression } => self.lambda(*pattern.clone(), *expression.clone()),
            Node::Call   { fun,     arg        } => self.call(*fun.clone(), *arg.clone()),
        }
    }

    pub fn index_constant(&mut self, data: Data) -> usize {
        match self.constants.iter().position(|d| d == &data) {
            Some(d) => d,
            None    => {
                self.constants.push(data);
                self.constants.len() - 1
            },
        }
    }

    fn data(&mut self, data: Data) {
        self.code.push(Opcode::Con as u8);
        let mut split = split_number(self.index_constant(data));
        self.code.append(&mut split);
    }

    fn block(&mut self, children: &[AST]) {
        for child in children {
            self.walk(&child);
            self.code.push(Opcode::Clear as u8);
        }

        // remove the last clear instruction
        self.code.pop();
    }

    fn assign(&mut self, symbol: AST, expression: AST) {
        // eval the expression
        self.walk(&expression);
        // load the following symbol ...
        self.code.push(Opcode::Save as u8);
        // ... the symbol to be loaded
        match symbol.node {
            Node::Symbol(l) => self.index_symbol(l),
            _               => unreachable!(),
        };
        // TODO: load Unit
    }

    fn lambda(&mut self, symbol: AST, expression: AST) {
        // TODO: closures
        let mut fun = Chunk::empty();

        // save the argument into the given variable
        fun.code.push(Opcode::Save as u8);
        fun.index_symbol(match symbol.node {
            Node::Symbol(l) => l,
            _               => unreachable!(),
        });

        // clear the stack
        fun.code.push(Opcode::Clear as u8);

        // run the function
        fun.walk(&expression);

        // return the result
        fun.code.push(Opcode::Return as u8);
        self.data(Data::Lambda(fun));
    }

    fn call(&mut self, fun: AST, arg: AST) {
        // TODO: gaurantee that this is a fun
        self.walk(&fun);
        self.walk(&arg);
        self.code.push(Opcode::Call as u8);
    }

    fn index_symbol(&mut self, symbol: Local) {
        let index = match self.locals.iter().position(|l| l == &symbol) {
            Some(l) => l,
            None    => {
                self.locals.push(symbol);
                self.locals.len() - 1
            },
        };

        self.code.append(&mut split_number(index));
    }

    fn symbol(&mut self, symbol: Local) {
        self.code.push(Opcode::Load as u8);
        self.index_symbol(symbol);
    }
}

// Just a wrapper, really
pub fn gen(ast: AST) -> Chunk {
    let mut generator = Chunk::empty();
    generator.walk(&ast);
    generator
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::lex::lex;
    use crate::compiler::parse::parse;

    #[test]
    fn constants() {
        // TODO: flesh out as more datatypes are added
        let source = "heck = true; lol = 0.0; lmao = false; eyy = \"GOod MoRNiNg, SiR\"";
        let ast    = parse(
            lex(source).unwrap()
        ).unwrap();
        let chunk = gen(ast);

        let result = vec![
            Data::Boolean(true),
            Data::Real(0.0),
            Data::Boolean(false),
            Data::String("GOod MoRNiNg, SiR".to_string()),
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
