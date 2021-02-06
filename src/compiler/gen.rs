use std::mem;

use crate::common::{
    number::split_number,
    span::{Span, Spanned},
    lambda::{Captured, Lambda},
    opcode::Opcode,
    data::Data,
};

// TODO: do a pass where we hoist and resolve variables?
// may work well for types too.

use crate::compiler::{
    cst::{CST, CSTPattern},
    // TODO: pattern for where?
    syntax::Syntax,
};

use crate::core::{
    ffi_core,
    ffi::FFI,
};

/// Simple function that generates unoptimized bytecode from an `CST`.
/// Exposes the functionality of the `Compiler`.
pub fn gen(cst: Spanned<CST>) -> Result<Lambda, Syntax> {
    let ffi = ffi_core();
    let mut compiler = Compiler::base(ffi);
    compiler.walk(&cst)?;
    return Ok(compiler.lambda);
}

// TODO: gen with FFI
// TODO: methods to combine FFIs
// TODO: namespaces for FFIs?

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
    /// The indicies of captured locals in the current scope
    captures: Vec<usize>,
    /// The nested depth of the current compiler.
    depth: usize,
    /// The foreign functional interface used to bind values
    ffi: FFI,
    /// The FFI functions that have been bound in this scope.
    ffi_names: Vec<String>,
}

impl Compiler {
    /// Construct a new `Compiler`.
    pub fn base(ffi: FFI) -> Compiler {
        Compiler {
            enclosing: None,
            lambda:    Lambda::empty(),
            locals:    vec![],
            captures:  vec![],
            depth:     0,
            ffi,
            ffi_names: vec![]
        }
    }

    // TODO: delcs and locals a bit redundant...

    /// Declare a local variable.
    pub fn declare(&mut self, name: String) {
        self.locals.push(Local { name, depth: self.depth });
        self.lambda.decls = self.locals.len();
    }

    /// Replace the current compiler with a fresh one,
    /// keeping a reference to the old one in `self.enclosing`,
    /// and moving the FFI into the current compiler.
    pub fn enter_scope(&mut self) {
        let ffi        = mem::replace(&mut self.ffi, FFI::new());
        let mut nested = Compiler::base(ffi);
        nested.depth   = self.depth + 1;
        let enclosing  = mem::replace(self, nested);
        self.enclosing = Some(Box::new(enclosing));
    }

    /// Restore the enclosing compiler,
    /// returning the nested one for data (Lambda) extraction,
    /// and moving the FFI mappings back into the enclosing compiler.
    pub fn exit_scope(&mut self) -> Compiler {
        let ffi       = mem::replace(&mut self.ffi, FFI::new());
        let enclosing = mem::replace(&mut self.enclosing, None);
        let nested = match enclosing {
            Some(compiler) => mem::replace(self, *compiler),
            None => unreachable!("Can not go back past root copiler"),
        };
        self.ffi = ffi;
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
        return match cst.item.clone() {
            CST::Data(data) => Ok(self.data(data)),
            CST::Symbol(name) => self.symbol(&name, cst.span.clone()),
            CST::Block(block) => self.block(block),
            CST::Print(expression) => self.print(*expression),
            CST::Label(name, expression) => self.label(name, *expression),
            CST::Tuple(tuple) => self.tuple(tuple),
            CST::FFI    { name,    expression } => self.ffi(name, *expression, cst.span.clone()),
            CST::Assign { pattern, expression } => self.assign(*pattern, *expression),
            CST::Lambda { pattern, expression } => self.lambda(*pattern, *expression),
            CST::Call   { fun,     arg        } => self.call(*fun, *arg),
        };
    }

    /// Takes a `Data` leaf and and produces some code to load the constant
    pub fn data(&mut self, data: Data) {
        self.lambda.emit(Opcode::Con);
        let mut split = split_number(self.lambda.index_data(data));
        self.lambda.emit_bytes(&mut split);
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

    /// Tries to resolve a variable in enclosing scopes
    /// if resolution it successful, it captures the variable in the original scope
    /// then builds a chain of upvalues to hoist that upvalue where it's needed.
    pub fn captured(&mut self, name: &str) -> Option<Captured> {
        if let Some(index) = self.local(name) {
            let already = self.captures.contains(&index);
            if !already {
                self.captures.push(index);
                self.lambda.emit(Opcode::Capture);
                self.lambda.emit_bytes(&mut split_number(index));
            }
            return Some(Captured::Local(index));
        }

        if let Some(enclosing) = self.enclosing.as_mut() {
            if let Some(captured) = enclosing.captured(name) {
                let included = self.lambda.captures.contains(&captured);
                let upvalue = if !included {
                    self.lambda.captures.push(captured);
                    self.lambda.captures.len() - 1
                } else {
                    self.lambda.captures.iter().position(|c| c == &captured).unwrap()
                };
                return Some(Captured::Nonlocal(upvalue));
            }
        }

        return None
    }

    /// returns the index of a captured non-local.
    pub fn captured_upvalue(&mut self, name: &str) -> Option<usize> {
        match self.captured(name) {
            Some(Captured::Nonlocal(upvalue)) => Some(upvalue),
            _ => None,
        }
    }

    // TODO: rewrite according to new local rules
    /// Takes a symbol leaf, and produces some code to load the local
    pub fn symbol(&mut self, name: &str, span: Span) -> Result<(), Syntax> {
        if let Some(index) = self.local(name) {
            // if the variable is locally in scope
            self.lambda.emit(Opcode::Load);
            self.lambda.emit_bytes(&mut split_number(index))
        } else if let Some(upvalue) = self.captured_upvalue(name) {
            // if the variable is captured in a closure
            self.lambda.emit(Opcode::LoadCap);
            self.lambda.emit_bytes(&mut split_number(upvalue))
        } else {
            // TODO: hoist?
            return Err(Syntax::error(
                "Variable referenced before assignment; it is undefined", &span
            ));
        }
        Ok(())
    }

    /// A block is a series of expressions where the last is returned.
    /// Each sup-expression is walked, the last value is left on the stack.
    pub fn block(&mut self, children: Vec<Spanned<CST>>) -> Result<(), Syntax> {
        if children.is_empty() {
            self.data(Data::Unit);
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

    /// Generates a print expression
    /// Note that currently printing is a baked-in language feature,
    /// but the second the FFI becomes a thing
    /// it'll no longer be one.
    pub fn print(&mut self, expression: Spanned<CST>) -> Result<(), Syntax> {
        self.walk(&expression)?;
        self.lambda.emit(Opcode::Print);
        Ok(())
    }

    /// Generates a Label construction
    /// that loads the variant, then wraps some data
    pub fn label(&mut self, name: String, expression: Spanned<CST>) -> Result<(), Syntax> {
        self.walk(&expression)?;
        self.data(Data::Kind(name));
        self.lambda.emit(Opcode::Label);
        Ok(())
    }

    /// Generates a Tuple construction
    /// that loads all fields in the tuple
    /// then rips them off the stack into a vec.
    pub fn tuple(&mut self, tuple: Vec<Spanned<CST>>) -> Result<(), Syntax> {
        let length = tuple.len();

        for item in tuple.into_iter().rev() {
            self.walk(&item)?;
        }

        self.lambda.emit(Opcode::Tuple);
        self.lambda.emit_bytes(&mut split_number(length));
        Ok(())
    }

    // Makes a rust function callable from passerine
    // TODO: make a macro to map Passerine's data model to Rust's
    pub fn ffi(&mut self, name: String, expression: Spanned<CST>, span: Span) -> Result<(), Syntax> {
        self.walk(&expression)?;

        let function = self.ffi.get(&name)
            .map_err(|s| Syntax::error(&s, &span))?;

        let index = match self.ffi_names.iter().position(|n| n == &name) {
            Some(p) => p,
            None => {
                // TODO: keeping track of state
                // in two different places is a code smell imo
                // Reason: don't want to include strings in lambda
                // optimal solutions:
                // have an earlier step that normalizes AST,
                // determines scope of all names/symbols,
                // and replaces all names/symbols with indicies
                // before codgen.
                self.ffi_names.push(name);
                self.lambda.add_ffi(function)
            },
        };

        self.lambda.emit(Opcode::FFICall);
        self.lambda.emit_bytes(&mut split_number(index));
        Ok(())
    }

    // resolves the assignment of a variable
    // returns true if the variable was declared.
    pub fn resolve_assign(&mut self, name: &str) -> bool {
        let mut declared = false;

        let index = if let Some(i) = self.local(name) {
            self.lambda.emit(Opcode::Save); i
        } else if let Some(i) = self.captured_upvalue(name) {
            self.lambda.emit(Opcode::SaveCap); i
        } else {
            self.declare(name.to_string());
            declared = true;
            self.lambda.emit(Opcode::Save);
            self.locals.len() - 1
        };

        self.lambda.emit_bytes(&mut split_number(index));
        return declared;
    }

    // TODO: simplify destructure.
    // because declarations can only happen with Symbol,
    // there can be at most one declaration

    /// Destructures a pattern into
    /// a series of unpack and assign instructions.
    /// Instructions match against the topmost stack item.
    /// Does delete the data that is matched against.
    pub fn destructure(&mut self, pattern: Spanned<CSTPattern>, redeclare: bool) {
        self.lambda.emit_span(&pattern.span);

        match pattern.item {
            CSTPattern::Symbol(name) => {
                if redeclare { self.declare(name.to_string()) }
                self.resolve_assign(&name);
            },
            CSTPattern::Data(expected) => {
                self.data(expected);
                self.lambda.emit(Opcode::UnData);
            }
            CSTPattern::Label(name, pattern) => {
                self.data(Data::Kind(name));
                self.lambda.emit(Opcode::UnLabel);
                self.destructure(*pattern, redeclare);
            }
            CSTPattern::Tuple(tuple) => {
                for (index, sub_pattern) in tuple.into_iter().enumerate() {
                    self.lambda.emit(Opcode::UnTuple);
                    self.lambda.emit_bytes(&mut split_number(index));
                    self.destructure(sub_pattern, redeclare);
                }
                // Delete the tuple moved to the top of the stack.
                self.lambda.emit(Opcode::Del);
            },
        }
    }

    /// Assign a value to a variable.
    pub fn assign(
        &mut self,
        pattern: Spanned<CSTPattern>,
        expression: Spanned<CST>
    ) -> Result<(), Syntax> {
        // eval the expression
        self.walk(&expression)?;
        self.destructure(pattern, false);
        self.data(Data::Unit);
        Ok(())
    }

    // TODO: rewrite according to new symbol rules
    /// Recursively compiles a lambda declaration in a new scope.
    pub fn lambda(
        &mut self,
        pattern: Spanned<CSTPattern>,
        expression: Spanned<CST>
    ) -> Result<(), Syntax> {
        // just so the parallel is visually apparent
        self.enter_scope();
        {
            // match the argument against the pattern, binding variables
            self.destructure(pattern, true);

            // enter a new scope and walk the function body
            self.walk(&expression)?;

            // return the result
            self.lambda.emit(Opcode::Return);
            self.lambda.emit_bytes(&mut split_number(self.locals.len()));
        }
        let lambda = self.exit_scope().lambda;

        // push the lambda object onto the callee's stack.
        let lambda_index = self.lambda.index_data(Data::Lambda(Box::new(lambda)));
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
