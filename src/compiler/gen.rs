use std::mem;

use crate::common::{
    number::split_number,
    span::{Span, Spanned},
    lambda::Lambda,
    opcode::Opcode,
    local::Local,
    captured::Captured,
    data::Data,
};

use crate::compiler::{
    ast::AST,
    syntax::Syntax,
};

pub fn gen(ast: Spanned<AST>) -> Result<Lambda, Syntax> {
    let mut compiler = Compiler::base();
    compiler.walk(&ast)?;
    return Ok(compiler.lambda);
}

pub struct Compiler {
    enclosing: Option<Box<Compiler>>,
    lambda: Lambda,
    locals: Vec<Local>,
    captureds: Vec<Captured>,
    depth: usize,
}

impl Compiler {
    pub fn base() -> Compiler {
        Compiler {
            enclosing: None,
            lambda: Lambda::empty(),
            locals: vec![],
            captureds: vec![],
            depth: 0,
        }
    }

    // pub fn over(/* compiler: Box<Compiler> */) -> Compiler {
    //     Compiler {
    //         enclosing: Some(compiler),
    //         lambda: Lambda::empty(),
    //         locals: vec![],
    //         captureds: vec![],
    //         depth: compiler.depth + 1, // at top to prevent used-after move error
    //     }
    // }

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
    fn walk(&mut self, ast: &Spanned<AST>) -> Result<(), Syntax> {
        // push left, push right, push center
        let result = match ast.item.clone() {
            AST::Data(data) => self.data(data),
            AST::Symbol => self.symbol(ast.span.clone()),
            AST::Block(block) => self.block(block),
            AST::Assign { pattern, expression } => self.assign(*pattern, *expression),
            AST::Lambda { pattern, expression } => self.lambda(*pattern, *expression),
            AST::Call   { fun,     arg        } => self.call(*fun, *arg),
        };

        return result;
    }

    /// Takes a `Data` leaf and and produces some code to load the constant
    fn data(&mut self, data: Data) -> Result<(), Syntax> {
        self.lambda.emit(Opcode::Con);
        let mut split = split_number(self.lambda.index_data(data));
        self.lambda.emit_bytes(&mut split);
        Ok(())
    }

    fn local(&self, span: Span) -> Option<usize> {
        for (index, l) in self.locals.iter().enumerate() {
            if span.contents() == l.span.contents() {
                return Some(index);
            }
        }

        return None;
    }

    fn capture(&mut self, captured: Captured) -> usize {
        // is already captured
        for (i, c) in self.captureds.iter().enumerate() {
            if &captured == c {
                return i;
            }
        }

        // is not captured
        self.captureds.push(captured);
        self.lambda.emit(Opcode::Capture);
        return self.captureds.len() - 1;
    }

    fn captured(&mut self, span: Span) -> Option<usize> {
        if let Some(enclosing) = self.enclosing.as_mut() {
            if let Some(index) = Compiler::local(&enclosing, span.clone()) {
                return Some(self.capture(Captured::local(index)));
            }

            if let Some(index) = Compiler::captured(enclosing.as_mut(), span) {
                return Some(self.capture(Captured::nonlocal(index)));
            }
        }

        return None;
    }

    // TODO: rewrite according to new local rules
    /// Takes a symbol leaf, and produces some code to load the local
    fn symbol(&mut self, span: Span) -> Result<(), Syntax> {
        if let Some(index) = self.local(span.clone()) {
            // if the variable is locally in scope
            self.lambda.emit(Opcode::Load);
            self.lambda.emit_bytes(&mut split_number(index))
        } else if let Some(index) = self.captured(span.clone()) {
            // if the variable is captured in a closure
            self.lambda.emit(Opcode::LoadCap);
            self.lambda.emit_bytes(&mut split_number(index))
        } else {
            // TODO: hoist?
            return Err(Syntax::error(
                "Variable referenced before assignment; it is undefined", span
            ));
        }
        Ok(())
    }

    /// A block is a series of expressions where the last is returned.
    /// Each sup-expression is walked, the last value is left on the stack.
    fn block(&mut self, children: Vec<Spanned<AST>>) -> Result<(), Syntax> {
        for child in children {
            self.walk(&child)?;
            self.lambda.emit(Opcode::Del);
        }

        // remove the last delete instruction
        self.lambda.demit();
        Ok(())
    }

    // TODO: rewrite according to new symbol rules
    fn assign(&mut self, symbol: Spanned<AST>, expression: Spanned<AST>) -> Result<(), Syntax> {
        // eval the expression
        self.walk(&expression)?;

        // load the following symbol ...
        let span = match symbol.item {
            AST::Symbol => symbol.span,
            _ => unreachable!(),
        };

        // abstract out?
        let index = if let Some(i) = self.local(span.clone()) {
            self.lambda.emit(Opcode::Save); i
        } else if let Some(i) = self.captured(span.clone()) {
            self.lambda.emit(Opcode::SaveCap); i
        } else {
            self.lambda.emit(Opcode::Save);
            self.locals.push(Local::new(span, self.depth));
            self.locals.len() - 1
        };

        self.lambda.emit_bytes(&mut split_number(index));
        Ok(())
    }

    // TODO: rewrite according to new symbol rules
    fn lambda(&mut self, symbol: Spanned<AST>, expression: Spanned<AST>) -> Result<(), Syntax> {
        let mut lambda = Lambda::empty();

        // save the argument into the given variable
        lambda.emit(Opcode::Save);
        lambda.emit_bytes(&mut split_number(0)); // NOTE: should this always be 0?

        let mut nested = Compiler::base();
        nested.depth = self.depth + 1;
        let enclosing = mem::replace(self, nested);
        self.enclosing = Some(Box::new(enclosing));

        // enter a new scope and walk the function body
        // let mut nested = Compiler::over(&mut );
        self.walk(&expression);    // run the function
        lambda.emit(Opcode::Return); // return the result
        lambda.emit_bytes(&mut split_number(self.locals.len()));

        // TODO: lift locals off stack if captured

        // push the lambda object onto the callee's stack.
        self.lambda.index_data(Data::Lambda(lambda));
        Ok(())
    }

    /// When a function is called, the top two items are taken off the stack,
    /// The topmost item is expected to be a function.
    fn call(&mut self, fun: Spanned<AST>, arg: Spanned<AST>) -> Result<(), Syntax> {
        self.walk(&arg);
        self.walk(&fun);
        self.lambda.emit(Opcode::Call);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::lex::lex;
    use crate::compiler::parse::parse;
    use crate::common::source::Source;

    // #[test]
    // fn constants() {
    //     // TODO: flesh out as more datatypes are added
    //     let source = Source::source("heck = true; lol = 0.0; lmao = false; eyy = \"GOod MoRNiNg, SiR\"");
    //     let ast    = parse(
    //         lex(source).unwrap()
    //     ).unwrap();
    //     let chunk = gen(ast);
    //
    //     let result = vec![
    //         Data::Boolean(true),
    //         Data::Real(0.0),
    //         Data::Boolean(false),
    //         Data::String("GOod MoRNiNg, SiR".to_string()),
    //     ];
    //
    //     assert_eq!(chunk.constants, result);
    // }

    // #[test]
    // fn bytecode() {
    //     let source = Source::source("heck = true; lol = heck; lmao = false");
    //     let ast    = parse(lex(source).unwrap()).unwrap();
    //
    //     let chunk = gen(ast);
    //     let result = vec![
    //         // con true, save to heck, clear
    //         (Opcode::Con as u8), 128, (Opcode::Save as u8), 128, (Opcode::Clear as u8),
    //         // load heck, save to lol, clear
    //         (Opcode::Load as u8), 128, (Opcode::Save as u8), 129, (Opcode::Clear as u8),
    //         // con false, save to lmao
    //         (Opcode::Con as u8), 129, (Opcode::Save as u8), 130,
    //     ];
    //
    //     assert_eq!(result, chunk.code);
    // }
}
