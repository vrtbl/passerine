use std::mem;

use crate::common::{
    number::split_number,
    span::{Span, Spanned},
    lambda::Lambda,
    opcode::Opcode,
    captured::Captured,
    data::Data,
};

use crate::compiler::{
    ast::AST,
    syntax::Syntax,
};

/// Simple function that generates unoptimized bytecode from an `AST`.
/// Exposes the functionality of the `Compiler`.
pub fn gen(ast: Spanned<AST>) -> Result<Lambda, Syntax> {
    let mut compiler = Compiler::base();
    compiler.walk(&ast)?;
    return Ok(compiler.lambda);
}

// TODO: implement equality
// TODO: maybe just move into compiler?
/// Represents a local when compiling.
#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    pub span: Span,
    pub depth: usize
}

impl Local {
    // Creates a new `Local`.
    pub fn new(span: Span, depth: usize) -> Local {
        Local { span, depth }
    }
}

// TODO: annotations in bytecode

/// Compiler is a bytecode generator that walks an AST and produces (unoptimized) Bytecode.
/// There are plans to add a bytecode optimizer in the future.
/// Note that this struct should not be controlled manually,
/// use the `gen` function instead.
pub struct Compiler {
    /// The previous compiler (when compiling nested scopes).
    enclosing: Option<Box<Compiler>>,
    /// The current bytecode emission target.
    lambda: Lambda,
    /// The locals in the current scope.
    locals: Vec<Local>,
    /// The captured variables used in the current scope.
    captureds: Vec<Captured>,
    /// The nested depth of the current compiler.
    depth: usize,
}

impl Compiler {
    /// Construct a new `Compiler`.
    pub fn base() -> Compiler {
        Compiler {
            enclosing: None,
            lambda: Lambda::empty(),
            locals: vec![],
            captureds: vec![],
            depth: 0,
        }
    }

    /// Declare a local variable.
    pub fn declare(&mut self, span: Span) {
        self.locals.push(
            Local { span, depth: self.depth }
        )
    }

    /// Replace the current compiler with a fresh one,
    /// keeping a reference to the old one in `self.enclosing`.
    pub fn enter_scope(&mut self) {
        let depth = self.depth + 1;
        let mut nested = Compiler::base();
        nested.depth = depth;
        let enclosing = mem::replace(self, nested);
        self.enclosing = Some(Box::new(enclosing));
    }

    /// Restore the enclosing compiler,
    /// returning the nested one for data extraction.
    pub fn exit_scope(&mut self) -> Compiler {
        let enclosing = mem::replace(&mut self.enclosing, None);
        let nested = match enclosing {
            Some(compiler) => mem::replace(self, *compiler),
            None => panic!("Can not go back past base compiler"),
        };
        return nested;
    }

    /// Walks an AST to generate bytecode.
    /// At this stage, the AST should've been verified, pruned, typechecked, etc.
    /// A malformed AST will cause a panic, as ASTs should be correct at this stage,
    /// and for them to be incorrect is an error in the compiler itself.
    pub fn walk(&mut self, ast: &Spanned<AST>) -> Result<(), Syntax> {
        // the entire span of the current node
        self.lambda.emit_span(&ast.span);

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
    pub fn data(&mut self, data: Data) -> Result<(), Syntax> {
        self.lambda.emit(Opcode::Con);
        let mut split = split_number(self.lambda.index_data(data));
        self.lambda.emit_bytes(&mut split);
        Ok(())
    }

    /// Returns the relative position on the stack of a declared local, if it exists.
    pub fn local(&self, span: Span) -> Option<usize> {
        for (index, l) in self.locals.iter().enumerate() {
            if span.contents() == l.span.contents() {
                return Some(index);
            }
        }

        return None;
    }

    /// Marks a local as captured in a closure,
    /// which essentially tells the VM to move it to the heap.
    /// Returns the index of the captured variable.
    pub fn capture(&mut self, captured: Captured) -> usize {
        // is already captured
        for (i, c) in self.captureds.iter().enumerate() {
            if &captured == c {
                return i;
            }
        }

        // the index of the local on the stack
        // relative to the current stack frame
        let index = captured.index;

        // is not captured
        self.captureds.push(captured);
        self.lambda.emit(Opcode::Capture);
        self.lambda.emit_bytes(&mut split_number(index));

        return self.captureds.len() - 1;
    }

    // TODO: rewrite
    /// NOTE: Function is not yet stable.
    pub fn captured(&mut self, span: Span) -> Option<usize> {
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
    pub fn symbol(&mut self, span: Span) -> Result<(), Syntax> {
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
    pub fn block(&mut self, children: Vec<Spanned<AST>>) -> Result<(), Syntax> {
        for child in children {
            self.walk(&child)?;
            self.lambda.emit(Opcode::Del);
        }

        // remove the last delete instruction
        self.lambda.demit();
        Ok(())
    }

    /// Assign a value to a variable.
    pub fn assign(&mut self, symbol: Spanned<AST>, expression: Spanned<AST>) -> Result<(), Syntax> {
        // eval the expression
        self.walk(&expression)?;

        // load the following symbol ...
        let span = match symbol.item {
            AST::Symbol => symbol.span,
            _ => unreachable!(),
        };

        // the span of the variable to be assigned to
        self.lambda.emit_span(&span);

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
        self.data(Data::Unit)?;

        Ok(())
    }

    // TODO: rewrite according to new symbol rules
    /// Recursively compiles a lambda declaration in a new scope.
    pub fn lambda(&mut self, symbol: Spanned<AST>, expression: Spanned<AST>) -> Result<(), Syntax> {
        // just so the parallel is visually apparent
        self.enter_scope();
        {
            // save the argument into the given variable
            if let AST::Symbol = symbol.item {} else { unreachable!() }
            self.lambda.emit(Opcode::Save);
            self.locals.push(Local::new(symbol.span, self.depth));
            self.lambda.emit_bytes(&mut split_number(0)); // will always be zerost item on stack

            // enter a new scope and walk the function body
            // let mut nested = Compiler::over(&mut);
            self.walk(&expression)?;    // run the function
            self.lambda.emit(Opcode::Return); // return the result
            self.lambda.emit_bytes(&mut split_number(self.locals.len()));

            // TODO: lift locals off stack if captured
        }
        let lambda = self.exit_scope().lambda;

        // push the lambda object onto the callee's stack.
        self.data(Data::Lambda(lambda))?;
        Ok(())
    }

    /// When a function is called, the top two items are taken off the stack,
    /// The topmost item is expected to be a function.
    pub fn call(&mut self, fun: Spanned<AST>, arg: Spanned<AST>) -> Result<(), Syntax> {
        self.walk(&arg)?;
        self.walk(&fun)?;

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

    #[test]
    fn constants() {
        let source = Source::source("heck = true; lol = 0.0; lmao = false; eyy = \"GOod MoRNiNg, SiR\"");
        let lambda = gen(parse(lex(source).unwrap()).unwrap()).unwrap();

        println!("{:?}", lambda);

        let result = vec![
            Data::Boolean(true),
            Data::Unit, // from assignment
            Data::Real(0.0),
            Data::Boolean(false),
            Data::String("GOod MoRNiNg, SiR".to_string()),
        ];

        assert_eq!(lambda.constants, result);
    }

    #[test]
    fn bytecode() {
        let source = Source::source("heck = true; lol = heck; lmao = false");
        let lambda = gen(parse(lex(source).unwrap()).unwrap()).unwrap();

        let result = vec![
            (Opcode::Con as u8), 128, (Opcode::Save as u8), 128,  // con true, save to heck,
                (Opcode::Con as u8), 129, (Opcode::Del as u8),    // load unit, delete
            (Opcode::Load as u8), 128, (Opcode::Save as u8), 129, // load heck, save to lol,
                (Opcode::Con as u8), 129, (Opcode::Del as u8),    // load unit, delete
            (Opcode::Con as u8), 130, (Opcode::Save as u8), 130,  // con false, save to lmao
                (Opcode::Con as u8), 129,                         // load unit
        ];

        assert_eq!(result, lambda.code);
    }

    #[test]
    fn closure() {
        // TODO: replace add1 with actual addition
        let source = Source::source("q = 1.0; x = (); w = y -> { x }; w ()");
        let _lambda = gen(parse(lex(source).unwrap()).unwrap()).unwrap();

        // I verified the output by hand
        // TODO: verify manually
        // println!("{:?}", lambda);
    }
}
