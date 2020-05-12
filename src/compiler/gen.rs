use crate::pipeline::ast::AST;
use crate::pipeline::bytecode::Opcode;
use crate::vm::data::Data;
use crate::vm::local::Local;
use crate::utils::span::{ Span, Spanned };
use crate::utils::number::split_number;

// The bytecode generator (emitter) walks the AST and produces (unoptimized) Bytecode
// There are plans to add a bytecode optimizer in the future.
// The bytecode generator
// TODO: annotations in bytecode

pub fn gen(ast: Spanned<AST>) -> Chunk {
    let mut generator = Chunk::empty();
    generator.walk(&ast);
    generator
}

/// Represents a single interpretable chunk of bytecode,
/// Think a function.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Chunk {
    pub code:      Vec<u8>,    // each byte is an opcode or a number-stream
    pub offsets:   Vec<usize>, // each usize indexes the bytecode op that begins each line
    pub constants: Vec<Data>,  // number-stream indexed, used to load constants
    pub locals:    Vec<Local>, // ''                                  variables
}

impl Chunk {
    /// Creates a new empty chunk to be filled.
    pub fn empty() -> Chunk {
        Chunk {
            code:      vec![],
            offsets:   vec![],
            constants: vec![],
            locals:    vec![],
        }
    }

    // TODO: bytecode chunk dissambler

    /// Walks an AST to generate bytecode.
    /// At this stage, the AST should've been verified, pruned, typechecked, etc.
    /// A malformed AST will cause a panic, as ASTs should be correct at this stage,
    /// and for them to be incorrect is an error in the compiler itself.
    fn walk(&mut self, ast: &Spanned<AST>) {
        // push left, push right, push center
        // NOTE: borrowing here introduces some complexity and cloning...
        // AST should be immutable and not behind shared reference so does not make sense to clone
        match &ast.item {
            AST::Data(data) => self.data(data.clone()),
            AST::Symbol(symbol) => self.symbol(symbol.clone()),
            AST::Block(block) => self.block(*block),

            // TODO: code smell, I'm almost a 3-star programmer.
            AST::Assign { pattern, expression } => self.assign(**pattern, **expression),
            AST::Lambda { pattern, expression } => self.lambda(**pattern, **expression),
            AST::Call   { fun,     arg        } => self.call(**fun, **arg),
        }
    }

    /// Given some data, this function adds it to the constants table,
    /// and returns the data's index.
    /// The constants table is push only, so constants are identified by their index.
    /// The resulting usize can be split up into a number byte stream,
    /// and be inserted into the bytecode.
    pub fn index_data(&mut self, data: Data) -> usize {
        match self.constants.iter().position(|d| d == &data) {
            Some(d) => d,
            None    => {
                self.constants.push(data);
                self.constants.len() - 1
            },
        }
    }

    /// Takes a `Data` leaf and and produces some code to load the constant
    fn data(&mut self, data: Data) {
        self.code.push(Opcode::Con as u8);
        let mut split = split_number(self.index_data(data));
        self.code.append(&mut split);
    }

    /// Similar to index constant, but indexes variables instead.
    fn index_symbol(&mut self, symbol: Local) -> usize {
        match self.locals.iter().position(|l| l == &symbol) {
            Some(l) => l,
            None    => {
                self.locals.push(symbol);
                self.locals.len() - 1
            },
        }
    }

    /// Takes a symbol leaf, and produces some code to load the local.
    fn symbol(&mut self, symbol: Local) {
        self.code.push(Opcode::Load as u8);
        let mut index = self.index_symbol(symbol);
        self.code.append(&mut split_number(index));
    }

    /// A block is a series of expressions where the last is returned.
    /// Each sup-expression is walked, the last value is left on the stack.
    /// (In the future, block should create a new scope.)
    fn block(&mut self, children: Vec<Spanned<AST>>) {
        for child in children {
            self.walk(&child);
            // TODO: Should `Opcode::Clear` not be a thing?
            self.code.push(Opcode::Clear as u8);
        }

        // remove the last clear instruction
        self.code.pop();
    }

    /// Binds a variable to a value in the current scope.
    /// Note that values are immutable, but variables aren't.
    /// Passerine uses a special form of reference counting,
    /// Where each object can only have one reference.
    /// This allows for lifetime optimizations later on.
    ///
    /// When a variable is reassigned, the value it holds is dropped.
    /// When a variable is loaded, the value it points to is copied,
    /// Unless it's the last occurance of that variable in it's lifetime.
    /// This makes passerine strictly pass by value.
    /// Though mutable objects can be simulated with macros.
    /// For example:
    /// ```
    /// --- Increments a variable by 1, returns new value.
    /// increment = var ~> { var = var + 1; var }
    ///
    /// counter = 7
    /// counter.increment ()
    /// -- desugars to
    /// increment counter
    /// -- desugars to
    /// counter = { counter + 1; counter }
    /// ```
    /// To demonstrate what I mean, let's annotate the vars.
    /// ```
    /// increment = var<`a> ~> {
    ///     var<`b> = var<`a> + 1
    ///     var<`b>
    /// }
    /// ```
    /// `<\`_>` means that the value held by var is the same.
    /// Because
    /// ```
    ///     var<`b> = var<`a> + 1
    ///                  ^^^^
    /// ```
    /// is the last use of the value of var<`a>, instead of copying the value,
    /// the value var points to is used, and var<`a`> is removed from the scope.
    ///
    /// There are still many optimizations that can be made,
    /// but needless to say, Passerine uses dynamically inferred lifetimes
    /// in lieu of garbage collecting.
    /// One issue with this strategy is having multiple copies of the same data,
    /// So for larger datastructures, some sort of persistent arc implementation might be used.
    fn assign(&mut self, symbol: Spanned<AST>, expression: Spanned<AST>) {
        // eval the expression
        self.walk(&expression);
        // load the following symbol ...
        self.code.push(Opcode::Save as u8);
        // ... the symbol to be loaded
        match symbol.item {
            AST::Symbol(l) => self.index_symbol(l),
            _               => unreachable!(),
        };
        // TODO: load Unit
    }

    /// Walks a function, creates a chunk, then pushes the resulting chunk onto the stack.
    /// All functions take and return one value.
    /// This allows for parital application,
    /// but is slow if you just want to run a function,
    /// because a function of three arguments is essentially three function calls.
    /// In the future, repeated calls should be optimized out.
    fn lambda(&mut self, symbol: Spanned<AST>, expression: Spanned<AST>) {
        // TODO: closures
        let mut fun = Chunk::empty();

        // inside the function
        // save the argument into the given variable
        fun.code.push(Opcode::Save as u8);
        fun.index_symbol(match symbol.item {
            AST::Symbol(l) => l,
            _               => unreachable!(),
        });

        fun.code.push(Opcode::Clear as u8);  // clear the stack
        fun.walk(&expression);               // run the function
        fun.code.push(Opcode::Return as u8); // return the result

        // push the lambda object onto the callee's stack.
        self.data(Data::Lambda(fun));
    }

    /// When a function is called, the top two items are taken off the stack,
    /// The topmost item is expected to be a function.
    fn call(&mut self, fun: Spanned<AST>, arg: Spanned<AST>) {
        self.walk(&arg);
        self.walk(&fun);
        self.code.push(Opcode::Call as u8);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::lex::lex;
    use crate::compiler::parse::parse;
    use crate::pipeline::source::Source;

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
