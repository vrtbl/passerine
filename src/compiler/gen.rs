use crate::common::{
    number::split_number,
    span::{Span, Spanned},
    lambda::Lambda,
    opcode::Opcode,
    local::Local,
    captured::Captured,
    data::Data,
};

use crate::compiler::ast::AST;

pub struct Compiler {
    enclosing: Option<Box<Compiler>>,
    lambda: Lambda,
    locals: Vec<Local>,
    captured: Vec<Captured>,
    depth: usize,
}

impl Compiler {
    pub fn base() -> Compiler {
        Compiler {
            enclosing: None,
            lambda: Lambda::empty(),
            locals: vec![],
            captured: vec![],
            depth: 0,
        }
    }

    pub fn over(compiler: Compiler) -> Compiler {
        Compiler {
            enclosing: Some(Box::new(compiler)),
            lambda: Lambda::empty(),
            locals: vec![],
            captured: vec![],
            depth: 0,
        }
    }


    pub fn begin_scope(&mut self) { self.depth += 1; }
    pub fn end_scope(&mut self) {
        self.depth -= 1;

        while let Some(_) = self.locals.pop() {
            self.lambda.emit(Opcode::Del)
        }
    }

    pub fn declare(&mut self, span: Span) {
        self.locals.push(
            Local { span, depth: self.depth }
        )
    }

    // TODO: bytecode chunk dissambler

    /// Walks an AST to generate bytecode.
    /// At this stage, the AST should've been verified, pruned, typechecked, etc.
    /// A malformed AST will cause a panic, as ASTs should be correct at this stage,
    /// and for them to be incorrect is an error in the compiler itself.
    fn walk(&mut self, ast: &Spanned<AST>) {
        // push left, push right, push center
        match ast.item.clone() {
            AST::Data(data) => self.data(data),
            AST::Symbol => self.symbol(ast.span),
            AST::Block(block) => self.block(block),
            AST::Assign { pattern, expression } => self.assign(*pattern, *expression),
            AST::Lambda { pattern, expression } => self.lambda(*pattern, *expression),
            AST::Call   { fun,     arg        } => self.call(*fun, *arg),
        }
    }

    /// Takes a `Data` leaf and and produces some code to load the constant
    fn data(&mut self, data: Data) {
        self.lambda.emit(Opcode::Con);
        let mut split = split_number(self.lambda.index_data(data));
        self.lambda.emit_bytes(&mut split);
    }

    fn local(&self, span: Span) -> Option<usize> {
        for (index, l) in self.locals.iter().enumerate() {
            if span.contents() == l.span.contents() {
                return Some(index);
            }
        }

        return None;
    }

    fn captured(&self, span: Span) -> Option<usize> {
        match self.enclosing {
            Some(enclosing) => {
                match Compiler::local(&enclosing, span) {
                    Some(index) =>
                }
            }
            None => None,
        }
    }

    // TODO: rewrite according to new local rules
    /// Takes a symbol leaf, and produces some code to load the local
    fn symbol(&mut self, span: Span) {
        if let Some(index) = self.local(span) {
            self.lambda.emit(Opcode::Load);
            self.lambda.emit_bytes(&mut split_number(index))
        } else if let Some(w) = self.captured(w) {

        }
    }

    // TODO: require all ops to require exactly one item be left on stack
    /// A block is a series of expressions where the last is returned.
    /// Each sup-expression is walked, the last value is left on the stack.
    /// (In the future, block should create a new scope.)
    fn block(&mut self, children: Vec<Spanned<AST>>) {
        for child in children {
            self.walk(&child);
            self.lambda.emit(Opcode::Del);
        }

        // remove the last clear instruction
        self.lambda.demit();
    }

    // TODO: rewrite according to new symbol rules
    // fn assign(&mut self, symbol: Spanned<AST>, expression: Spanned<AST>) {
    //     // eval the expression
    //     self.walk(&expression);
    //     // load the following symbol ...
    //     self.chunk.emit(Opcode::Save);
    //     // ... the symbol to be loaded
    //     let index = match symbol.item {
    //         AST::Symbol(l) => self.index_symbol(l),
    //         _              => unreachable!(),
    //     };
    //     self.code.append(&mut split_number(index));
    //     // TODO: load Unit
    // }

    // TODO: rewrite according to new symbol rules
    // fn lambda(&mut self, symbol: Spanned<AST>, expression: Spanned<AST>) {
    //     let mut fun = Chunk::empty();
    //
    //     // inside the function
    //     // save the argument into the given variable
    //     fun.code.push(Opcode::Save as u8);
    //     let index = fun.index_symbol(match symbol.item {
    //         AST::Symbol(l) => l,
    //         _               => unreachable!(),
    //     });
    //     fun.code.append(&mut split_number(index));
    //
    //     fun.code.push(Opcode::Clear as u8);  // clear the stack
    //     fun.walk(&expression);               // run the function
    //     fun.code.push(Opcode::Return as u8); // return the result
    //
    //     // push the lambda object onto the callee's stack.
    //     self.data(Data::Lambda(fun));
    // }

    /// When a function is called, the top two items are taken off the stack,
    /// The topmost item is expected to be a function.
    fn call(&mut self, fun: Spanned<AST>, arg: Spanned<AST>) {
        self.walk(&arg);
        self.walk(&fun);
        self.lambda.emit(Opcode::Call);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::lex::lex;
    use crate::compiler::parse::parse;
    use crate::common::source::Source;

    #[test]
    fn constants() {
        // TODO: flesh out as more datatypes are added
        let source = Source::source("heck = true; lol = 0.0; lmao = false; eyy = \"GOod MoRNiNg, SiR\"");
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
        let source = Source::source("heck = true; lol = heck; lmao = false");
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
