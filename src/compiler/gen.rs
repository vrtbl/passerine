use crate::common::{
    number::split_number,
    span::Spanned,
    chunk::Chunk,
    opcode::Opcode,
    local::Local,
};

use crate::compiler::ast::AST;

pub struct Compiler {
    locals: Vec<Local>,
    depth: usize,
    chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            locals: vec![],
            depth: 0,
            chunk: Chunk::empty()
        }
    }

    pub fn begin_scope(&mut self) { self.depth += 1; }
    pub fn end_scope(&mut self) {
        self.depth -= 1;

        while let Some(_) = self.locals.pop() {
            self.chunk.emit(Opcode::Del)
        }
    }

    pub fn declare(&mut self) {
        self.locals.push(
            Local { symbol: todo!(), depth: self.depth }
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
            AST::Symbol(symbol) => self.symbol(symbol),
            AST::Block(block) => self.block(block),
            AST::Assign { pattern, expression } => self.assign(*pattern, *expression),
            AST::Lambda { pattern, expression } => self.lambda(*pattern, *expression),
            AST::Call   { fun,     arg        } => self.call(*fun, *arg),
        }
    }

    /// Takes a `Data` leaf and and produces some code to load the constant
    fn data(&mut self, data: Data) {
        self.code.push(Opcode::Con as u8);
        let mut split = split_number(self.index_data(data));
        self.code.append(&mut split);
    }

    /// Takes a symbol leaf, and produces some code to load the local.
    fn symbol(&mut self, symbol: Local) {
        self.code.push(Opcode::Load as u8);
        let index = self.index_symbol(symbol);
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
    /// ```plain
    /// --- Increments a variable by 1, returns new value.
    /// increment = var ~> { var = var + 1; var }
    ///
    /// counter = 7
    /// counter.increment ()
    /// -- desugars to
    /// increment counter
    /// -- desugars to
    /// { counter = counter + 1; counter }
    /// ```
    /// To demonstrate what I mean, let's annotate the vars.
    /// ```plain
    /// increment = var<`a> ~> {
    ///     var<`b> = var<`a> + 1
    ///     var<`b>
    /// }
    /// ```
    /// `<\`_>` means that the value held by var is the same.
    /// Because
    /// ```plain
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
    /// So for larger datastructures, some sort of persistent ARC implementation might be used.
    fn assign(&mut self, symbol: Spanned<AST>, expression: Spanned<AST>) {
        // eval the expression
        self.walk(&expression);
        // load the following symbol ...
        self.code.push(Opcode::Save as u8);
        // ... the symbol to be loaded
        let index = match symbol.item {
            AST::Symbol(l) => self.index_symbol(l),
            _              => unreachable!(),
        };
        self.code.append(&mut split_number(index));
        // TODO: load Unit
    }

    /// Walks a function, creates a chunk, then pushes the resulting chunk onto the stack.
    /// All functions take and return one value.
    /// This allows for parital application,
    /// but is slow if you just want to run a function,
    /// because a function of three arguments is essentially three function calls.
    /// In the future, repeated calls should be optimized out.
    /// TODO: closures
    /// The issue with closures is that they allow the data to escape
    /// which makes vaporization less useful as a result.
    /// There are a few potential solutions:
    /// - The easiest solution is to disallow closures. This is lame.
    /// - The second easiest solution is to simply copy the data
    ///   when creating a closure.
    ///   While easy to implement, captured values would not represent
    ///   the same object:
    ///   ```plain
    ///   counter = start -> {
    ///       value = start
    ///       increment = () -> { value = value + 1 }
    ///       decrement = () -> { value = value - 1 }
    ///       (increment, decrement)
    ///   }
    ///   ```
    ///   If a counter was constructed using the above code,
    ///   increment and decrement would refer to different values.
    /// - As closures are a poor man's object,
    ///   an alternative would be adding support for pseudo-objects via macros.
    ///   this wouldn't clash with naïeve closure implementations;
    ///   here's what I'm getting at:
    ///   ```plain
    ///   counter = start -> {
    ///       Counter start -- wrap value in Label, creating a type
    ///   }
    ///
    ///   increment = value: Counter _ ~> { value = value + 1 }
    ///   decrement = value: Counter _ ~> { value = value - 1 }
    ///   tally     = Counter value -> value
    ///
    ///   my_counter = counter 5
    ///   increment counter; increment counter
    ///   decrement counter
    ///   print (tally counter)
    ///   ```
    ///   I like this solution, but I think it should be its own thing
    ///   rather than boot actual closures from the language.
    ///   Passerine's an impure functional language,
    ///   so no closures would be a little silly.
    /// - Another solution would be to store the values on the heap, arc'd.
    ///   Nah.
    /// - Ok, I think I've got it.
    ///   At compile time, each function contains a list of variables
    ///   of the environment it's created in, ad infinitum.
    ///   When a function is defined within a function (read closure),
    ///   during definition, it marks all variables used by that function.
    ///   At the end of the original function, all unmarked (read uncaptured)
    ///   variables are removed from the environment,
    ///   And all functions return containing an ARC to the base function's environment
    /// - I noticed an issue. Take this example:
    ///   ```plain
    ///   escape = huge tiny -> {
    ///       forget   = () -> huge,
    ///       remember = () -> tiny
    ///       remember
    ///   }
    ///   ```
    ///   In this case, `huge` is captured by the `forget closure`
    ///   But only remember is returned.
    ///   However, everything is stored in the same struct
    ///   So huge isn't deallocated until remember is.
    /// - Here's my Final Solution. We introduce a new type of data,
    ///   `ReferenceCoutned`.
    ///   When data is captured, it's converted to reference-counted data
    ///   if it hasn't been already, and the reference count is increased.
    ///   The only downside is that this is a bit slower,
    ///   but it's a small price to pay for salvation.
    fn lambda(&mut self, symbol: Spanned<AST>, expression: Spanned<AST>) {
        let mut fun = Chunk::empty();

        // inside the function
        // save the argument into the given variable
        fun.code.push(Opcode::Save as u8);
        let index = fun.index_symbol(match symbol.item {
            AST::Symbol(l) => l,
            _               => unreachable!(),
        });
        fun.code.append(&mut split_number(index));

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
