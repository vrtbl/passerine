use std::{
    mem,
    rc::Rc,
};

use crate::common::{
    number::split_number,
    span::{Span, Spanned},
    lambda::{Captured, Lambda},
    opcode::Opcode,
    data::Data,
};

// TODO: hoist and resolve types

use crate::construct::{
    sst::{Scope, SST, SSTPattern},
    symbol::UniqueSymbol,
    module::Module,
};

use crate::compiler::{lower::Lower, syntax::Syntax};

use crate::core::{
    ffi_core,
    ffi::FFI,
};

// TODO: namespaces for FFIs?

impl Lower for Module<Spanned<SST>, Scope> {
    type Out = Rc<Lambda>;

    /// Simple function that generates unoptimized bytecode from an `SST`.
    /// Exposes the functionality of the `Compiler`.
    fn lower(self) -> Result<Self::Out, Syntax> {
        let module = Module::new(self.repr, (self.assoc, ffi_core()));
        return module.lower();
    }
}

impl Lower for Module<Spanned<SST>, (Scope, FFI)> {
    type Out = Rc<Lambda>;

    /// Generates unoptimized bytecode from a `SST`,
    /// Given a specific FFI. Note that this doesn't even assume the core ffi,
    /// So it's required you generate a core ffi with `core::ffi_core()`,
    /// Then merge it with your ffi with `FFI::combine(...)`.
    fn lower(self) -> Result<Self::Out, Syntax> {
        let mut compiler = Compiler::base(self.assoc.1, self.assoc.0);
        compiler.walk(&self.repr)?;
        return Ok(Rc::new(compiler.lambda));
    }
}

/// Compiler is a bytecode generator that walks an SST and produces (unoptimized) Bytecode.
/// There are plans to add a bytecode optimizer in the future.
/// Note that this struct should not be controlled manually,
/// use the `gen` function instead.
pub struct Compiler {
    /// The previous compiler (when compiling nested scopes).
    enclosing: Option<Box<Compiler>>,
    /// The current bytecode emission target.
    lambda: Lambda,
    /// Names of symbols,
    // symbol_table: Vec<String>,
    /// The foreign functional interface used to bind values
    ffi: FFI,
    /// The FFI functions that have been bound in this scope.
    ffi_names: Vec<String>,
    // determined in hoisting
    scope: Scope,
}

impl Compiler {
    /// Construct a new `Compiler`.
    pub fn base(ffi: FFI, scope: Scope) -> Compiler {
        Compiler {
            enclosing: None,
            lambda:    Lambda::empty(),
            ffi,
            ffi_names: vec![],
            scope,
        }
    }

    /// Replace the current compiler with a fresh one,
    /// keeping a reference to the old one in `self.enclosing`,
    /// and moving the FFI into the current compiler.
    pub fn enter_scope(&mut self, scope: Scope) {
        let ffi        = mem::replace(&mut self.ffi, FFI::new());
        let nested     = Compiler::base(ffi, scope);
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

    /// Walks an SST to generate bytecode.
    /// At this stage, the SST should've been verified, pruned, typechecked, etc.
    /// A malformed SST will cause a panic, as SSTs should be correct at this stage,
    /// and for them to be incorrect is an error in the compiler itself.
    pub fn walk(&mut self, sst: &Spanned<SST>) -> Result<(), Syntax> {
        // TODO: move this to a better spot
        self.lambda.decls = self.scope.locals.len();

        // the entire span of the current node
        self.lambda.emit_span(&sst.span);

        // push left, push right, push center
        return match sst.item.clone() {
            SST::Data(data) => Ok(self.data(data)),
            SST::Symbol(unique) => Ok(self.symbol(unique)),
            SST::Block(block) => self.block(block),
            SST::Label(name, expression) => self.label(name, *expression),
            SST::Tuple(tuple) => self.tuple(tuple),
            SST::FFI    { name,    expression } => self.ffi(name, *expression, sst.span.clone()),
            SST::Assign { pattern, expression } => self.assign(*pattern, *expression),
            SST::Lambda { pattern, expression, scope } => self.lambda(*pattern, *expression, scope),
            SST::Call   { fun,     arg        } => self.call(*fun, *arg),
        };
    }

    // TODO: closures are just lambdas + records
    // refactor as such?

    /// Resovles a symbol lookup, e.g. something like `x`.
    pub fn symbol(&mut self, unique_symbol: UniqueSymbol) {
        let index = if let Some(i) = self.scope.local_index(unique_symbol) {
            self.lambda.emit(Opcode::Load); i
        } else if let Some(i) = self.scope.nonlocal_index(unique_symbol) {
            self.lambda.emit(Opcode::LoadCap); i
        } else {
            // unreachable?
            todo!();
        };

        self.lambda.emit_bytes(&mut split_number(index));
    }

    /// Takes a `Data` leaf and and produces some code to load the constant
    pub fn data(&mut self, data: Data) {
        self.lambda.emit(Opcode::Con);
        let mut split = split_number(self.lambda.index_data(data));
        self.lambda.emit_bytes(&mut split);
    }

    /// A block is a series of expressions where the last is returned.
    /// Each sup-expression is walked, the last value is left on the stack.
    pub fn block(&mut self, children: Vec<Spanned<SST>>) -> Result<(), Syntax> {
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
    pub fn print(&mut self, expression: Spanned<SST>) -> Result<(), Syntax> {
        self.walk(&expression)?;
        self.lambda.emit(Opcode::Print);
        Ok(())
    }

    /// Generates a Label construction
    /// that loads the variant, then wraps some data
    pub fn label(&mut self, name: UniqueSymbol, expression: Spanned<SST>) -> Result<(), Syntax> {
        self.walk(&expression)?;
        self.data(Data::Kind(name.0));
        self.lambda.emit(Opcode::Label);
        Ok(())
    }

    /// Generates a Tuple construction
    /// that loads all fields in the tuple
    /// then rips them off the stack into a vec.
    pub fn tuple(&mut self, tuple: Vec<Spanned<SST>>) -> Result<(), Syntax> {
        let length = tuple.len();

        for item in tuple.into_iter() {
            self.walk(&item)?;
        }

        self.lambda.emit(Opcode::Tuple);
        self.lambda.emit_bytes(&mut split_number(length));
        Ok(())
    }

    // TODO: make a macro to map Passerine's data model to Rust's
    /// Makes a Rust function callable from Passerine,
    /// by keeping a reference to that function.
    pub fn ffi(&mut self, name: String, expression: Spanned<SST>, span: Span) -> Result<(), Syntax> {
        self.walk(&expression)?;

        let function = self.ffi.get(&name)
            .map_err(|s| Syntax::error(&s, &span))?;

        let index = match self.ffi_names.iter().position(|n| n == &name) {
            Some(p) => p,
            None => {
                // TODO: switch ffi to symbol, just use unique symbol?
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

        self.lambda.emit_span(&span);
        self.lambda.emit(Opcode::FFICall);
        self.lambda.emit_bytes(&mut split_number(index));
        Ok(())
    }

    /// Resolves the assignment of a variable
    /// returns true if the variable was declared.
    pub fn resolve_assign(&mut self, unique_symbol: UniqueSymbol) {
        let index = if let Some(i) = self.scope.local_index(unique_symbol) {
            self.lambda.emit(Opcode::Save); i
        } else if let Some(i) = self.scope.nonlocal_index(unique_symbol) {
            self.lambda.emit(Opcode::SaveCap); i
        } else {
            // unreachable?
            todo!()
        };

        self.lambda.emit_bytes(&mut split_number(index));
    }

    /// Destructures a pattern into
    /// a series of unpack and assign instructions.
    /// Instructions match against the topmost stack item.
    /// Does delete the data that is matched against.
    pub fn destructure(&mut self, pattern: Spanned<SSTPattern>, redeclare: bool) {
        self.lambda.emit_span(&pattern.span);

        match pattern.item {
            SSTPattern::Symbol(unique_symbol) => {
                self.resolve_assign(unique_symbol);
            },
            SSTPattern::Data(expected) => {
                self.data(expected);
                self.lambda.emit(Opcode::UnData);
            }
            SSTPattern::Label(name, pattern) => {
                self.data(Data::Kind(name.0));
                self.lambda.emit(Opcode::UnLabel);
                self.destructure(*pattern, redeclare);
            }
            SSTPattern::Tuple(tuple) => {
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
        pattern: Spanned<SSTPattern>,
        expression: Spanned<SST>
    ) -> Result<(), Syntax> {
        // eval the expression
        self.walk(&expression)?;
        self.destructure(pattern, false);
        self.data(Data::Unit);
        Ok(())
    }

    /// Recursively compiles a lambda declaration in a new scope.
    pub fn lambda(
        &mut self,
        pattern: Spanned<SSTPattern>,
        expression: Spanned<SST>,
        scope: Scope,
    ) -> Result<(), Syntax> {
        // build a list of captures at the boundary
        let mut captures = vec![];
        for nonlocal in scope.nonlocals.iter() {
            let captured = if self.scope.is_local(*nonlocal) {
                let index = self.scope.local_index(*nonlocal).unwrap();
                self.lambda.emit(Opcode::Capture);
                self.lambda.emit_bytes(&mut split_number(index));
                Captured::Local(index)
            } else {
                Captured::Nonlocal(self.scope.nonlocal_index(*nonlocal).unwrap())
            };
            captures.push(captured);
        }

        // just so the parallel is visually apparent
        self.enter_scope(scope);
        {
            // push locals and captures into lambda
            self.lambda.captures = captures;

            // match the argument against the pattern, binding variables
            self.destructure(pattern, true);

            // enter a new scope and walk the function body
            self.walk(&expression)?;

            // return the result
            self.lambda.emit(Opcode::Return);
            self.lambda.emit_bytes(&mut split_number(self.scope.locals.len()));
        }
        let lambda = self.exit_scope().lambda;

        // push the lambda object onto the callee's stack.
        let lambda_index = self.lambda.index_data(Data::Lambda(Rc::new(lambda)));
        self.lambda.emit(Opcode::Closure);
        self.lambda.emit_bytes(&mut split_number(lambda_index));

        Ok(())
    }

    /// When a function is called, the top two items are taken off the stack,
    /// The topmost item is expected to be a function.
    pub fn call(&mut self, fun: Spanned<SST>, arg: Spanned<SST>) -> Result<(), Syntax> {
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
    use crate::construct::module::ThinModule;
    use crate::common::source::Source;

    #[test]
    fn constants() {
        let source = Source::source("heck = true; lol = 0.0; lmao = false; eyy = \"GOod MoRNiNg, SiR\"");
        let lambda = ThinModule::thin(source)
            .lower().unwrap()
            .lower().unwrap()
            .lower().unwrap()
            .lower().unwrap()
            .lower().unwrap();

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
        let lambda = ThinModule::thin(source)
            .lower().unwrap()
            .lower().unwrap()
            .lower().unwrap()
            .lower().unwrap()
            .lower().unwrap();

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
