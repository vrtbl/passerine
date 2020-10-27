use std::mem;

use crate::common::{
    number::split_number,
    span::{Span, Spanned},
    lambda::Lambda,
    opcode::Opcode,
    data::Data,
};

use crate::compiler::{
    cst::CST,
    syntax::Syntax,
};

/// Simple function that generates unoptimized bytecode from an `CST`.
/// Exposes the functionality of the `Compiler`.
pub fn gen(cst: Spanned<CST>) -> Result<Lambda, Syntax> {
    let mut compiler = Compiler::base();
    compiler.walk(&cst)?;
    return Ok(compiler.lambda);
}

// TODO: implement equality
/// Represents a local when compiling.
#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    pub name: String,
    pub depth: usize
}

impl Local {
    // Creates a new `Local`.
    pub fn new(name: String, depth: usize) -> Local {
        Local { name, depth }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    /// The index on the stack if the variable is local to the current scope
    Local(usize),
    /// The index of the upvalue in the enclosing scope
    Nonlocal(usize),
}

/// Compiler is a bytecode generator that walks an CST and produces (unoptimized) Bytecode.
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
    // /// The captured variables used in the current scope.
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
    pub fn declare(&mut self, name: String) {
        self.locals.push(
            Local { name, depth: self.depth }
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

    /// Walks an CST to generate bytecode.
    /// At this stage, the CST should've been verified, pruned, typechecked, etc.
    /// A malformed CST will cause a panic, as CSTs should be correct at this stage,
    /// and for them to be incorrect is an error in the compiler itself.
    pub fn walk(&mut self, cst: &Spanned<CST>) -> Result<(), Syntax> {
        // the entire span of the current node
        self.lambda.emit_span(&cst.span);

        // push left, push right, push center
        let result = match cst.item.clone() {
            CST::Data(data) => self.data(data),
            CST::Symbol(name) => self.symbol(&name, cst.span.clone()),
            CST::Block(block) => self.block(block),
            CST::Print(expression) => self.print(*expression),
            CST::Label(name, expression) => self.label(name, *expression),
            CST::Assign { pattern, expression } => self.assign(*pattern, *expression),
            CST::Lambda { pattern, expression } => self.lambda(*pattern, *expression),
            CST::Call   { fun,     arg        } => self.call(*fun, *arg),
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

    // TODO: nested too deep :(
    /// Returns the relative position on the stack of a declared local,
    /// if it exists in the current scope.
    pub fn local(&self, name: &str) -> Option<usize> {
        for (index, l) in self.locals.iter().enumerate() {
            if name == l.name {
                return Some(index)
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

        // is not yet captured
        match captured {
            Captured::Local(index) => {
                self.lambda.emit(Opcode::Capture);
                self.lambda.emit_bytes(&mut split_number(index));
            },
            Captured::Nonlocal(upvalue) => {
                self.lambda.captureds.push(upvalue);
            },
        }

        self.captureds.push(captured);
        return self.captureds.len() - 1;
    }

    // Tries to resolve a variable in enclosing scopes
    // if resolution it successful, it captures the variable in the original scope
    // then builds a chain of upvalues to hoist that upvalue where it's needed.
    // This can be made more efficient.
    pub fn captured(&mut self, name: &str) -> Option<usize> {
        // return index in compiler captureds - 1

        if let Some(index) = self.local(name) {
            // if the variable exists in this scope:
            // capture that variable (i.e. emit opcode)
            // add to compiler captureds
            return Some(self.capture(Captured::Local(index)));
        }

        if let Some(enclosing) = self.enclosing.as_mut() {
            if let Some(upvalue) = enclosing.captured(name) {
                // if the variable exists in an enclosing scope and has been captured:
                // add to compiler captureds
                // add to lambda captureds
                return Some(self.capture(Captured::Nonlocal(upvalue)))
            }
        }

        return None;
    }

    // TODO: rewrite according to new local rules
    /// Takes a symbol leaf, and produces some code to load the local
    pub fn symbol(&mut self, name: &str, span: Span) -> Result<(), Syntax> {
        if let Some(index) = self.local(name) {
            // if the variable is locally in scope
            self.lambda.emit(Opcode::Load);
            self.lambda.emit_bytes(&mut split_number(index))
        } else if let Some(index) = self.captured(name) {
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
    pub fn block(&mut self, children: Vec<Spanned<CST>>) -> Result<(), Syntax> {
        if children.is_empty() {
            self.data(Data::Unit)?;
            return Ok(());
        }

        for child in children {
            self.walk(&child)?;
            self.lambda.emit(Opcode::Del);
        }

        // remove the last delete instruction
        self.lambda.demit();
        Ok(())
    }

    pub fn print(&mut self, expression: Spanned<CST>) -> Result<(), Syntax> {
        self.walk(&expression)?;
        self.lambda.emit(Opcode::Print);
        Ok(())
    }

    pub fn label(&mut self, name: String, expression: Spanned<CST>) -> Result<(), Syntax> {
        self.walk(&expression)?;
        self.data(Data::Kind(name))?;
        self.lambda.emit(Opcode::Label);
        Ok(())
    }

    /// Assign a value to a variable.
    pub fn assign(&mut self, symbol: Spanned<CST>, expression: Spanned<CST>) -> Result<(), Syntax> {
        // eval the expression
        self.walk(&expression)?;

        // load the following symbol ...
        let name = match symbol.item {
            CST::Symbol(name) => name,
            _ => unreachable!(),
        };

        // the span of the variable to be assigned to
        self.lambda.emit_span(&symbol.span);

        // abstract out?
        let index = if let Some(i) = self.local(&name) {
            self.lambda.emit(Opcode::Save); i
        } else if let Some(i) = self.captured(&name) {
            self.lambda.emit(Opcode::SaveCap); i
        } else {
            self.lambda.emit(Opcode::Save);
            self.locals.push(Local::new(name, self.depth));
            self.locals.len() - 1
        };

        self.lambda.emit_bytes(&mut split_number(index));
        self.data(Data::Unit)?;

        Ok(())
    }

    // TODO: rewrite according to new symbol rules
    /// Recursively compiles a lambda declaration in a new scope.
    pub fn lambda(&mut self, symbol: Spanned<CST>, expression: Spanned<CST>) -> Result<(), Syntax> {
        // just so the parallel is visually apparent
        self.enter_scope();
        {
            // save the argument into the given variable
            if let CST::Symbol(name) = symbol.item {
                self.declare(name);
            } else {
                unreachable!()
            }

            self.lambda.emit(Opcode::Save);
            self.lambda.emit_bytes(&mut split_number(0)); // will always be zerost item on stack

            // enter a new scope and walk the function body
            self.walk(&expression)?;          // run the function
            self.lambda.emit(Opcode::Return); // return the result
            self.lambda.emit_bytes(&mut split_number(self.locals.len()));

            // TODO: lift locals off stack if captured
        }
        let lambda = self.exit_scope().lambda;

        // push the lambda object onto the callee's stack.
        let lambda_index = self.lambda.index_data(Data::Lambda(lambda));
        self.lambda.emit(Opcode::Closure);
        self.lambda.emit_bytes(&mut split_number(lambda_index));

        Ok(())
    }

    /// When a function is called, the top two items are taken off the stack,
    /// The topmost item is expected to be a function.
    pub fn call(&mut self, fun: Spanned<CST>, arg: Spanned<CST>) -> Result<(), Syntax> {
        self.walk(&arg)?;
        self.walk(&fun)?;

        self.lambda.emit_span(&Span::combine(&fun.span, &arg.span));
        self.lambda.emit(Opcode::Call);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::{
        lex::lex,
        parse::parse,
        desugar::desugar,
    };
    use crate::common::source::Source;

    #[test]
    fn constants() {
        let source = Source::source("heck = true; lol = 0.0; lmao = false; eyy = \"GOod MoRNiNg, SiR\"");
        let lambda = gen(desugar(parse(lex(source).unwrap()).unwrap()).unwrap()).unwrap();

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
        let lambda = gen(desugar(parse(lex(source).unwrap()).unwrap()).unwrap()).unwrap();

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

    // NOTE: instead of veryfying bytecode output,
    // write a test in vm::vm::test
    // and check behaviour that way
}
