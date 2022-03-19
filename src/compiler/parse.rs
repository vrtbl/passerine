use std::{
    collections::HashMap,
    convert::TryFrom,
    mem,
    rc::Rc,
};

use crate::{
    common::{
        lit::Lit,
        span::{
            Span,
            Spanned,
        },
    },
    compiler::{
        read::Reader,
        syntax::Syntax,
    },
    construct::{
        symbol::SharedSymbol,
        token::{
            Delim,
            ResIden,
            ResOp,
            TokenTree,
            TokenTrees,
        },
        tree::{
            Base,
            Lambda,
            Pattern,
            Sugar,
            AST,
        },
    },
};

/// We're using a Pratt parser, so this little enum
/// defines different precedence levels.
/// Each successive level is higher, so, for example,
/// multiplication is higher than addition: `* > +`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prec {
    /// No precedence.
    None = 0,
    /// `=`
    Assign,
    /// `,`
    Pair,
    /// `|>`
    Compose,
    /// `->`
    Lambda,
    /// Boolean logic.
    Logic,
    /// `+`, `-`
    AddSub,
    /// `*`, `/`, etc.
    MulDiv,
    /// `**`
    Pow,
    /// `:`
    Is,
    /// Implicit function call operator.
    Call,
    /// `.`
    Field,
    /// Highest precedence.
    End,
}

impl Prec {
    /// Increments precedence level to cause the
    /// parser to associate infix operators to the left.
    /// For example, addition is left-associated:
    /// ```build
    /// Prec::Addition.left()
    /// ```
    /// `a + b + c` left-associated becomes `(a + b) + c`.
    /// By default, the parser accociates right.
    ///
    /// Panics if you try to associate left on `Prec::End`,
    /// as this is the highest precedence.
    pub fn left(&self) -> Prec {
        if let Prec::End = self {
            panic!("Can not associate further left")
        }
        return unsafe { mem::transmute(self.clone() as u8 + 1) };
    }
}

#[derive(Debug)]
pub struct Parser {
    /// Symbols with the same name are interned.
    /// We don't do this during lexing so that token-based
    /// macros can work with strings.
    symbols: HashMap<String, SharedSymbol>,
}

impl Parser {
    /// Parses a token tree into a syntax tree.
    /// This will produce a module as opposed to a block.
    /// Also returns the symbol interning table.
    pub fn parse(
        parse_tree: Spanned<TokenTree>,
    ) -> Result<(Spanned<AST>, HashMap<String, SharedSymbol>), Syntax> {
        // build base parser
        let mut parser = Parser {
            symbols: HashMap::new(),
        };

        let ast = parser.entry(&parse_tree)?;

        Ok((ast, parser.symbols))
    }

    // entry point to parse a token tree into an AST
    fn entry(
        &mut self,
        token_tree: &Spanned<TokenTree>,
    ) -> Result<Spanned<AST>, Syntax> {
        let result = match &token_tree.item {
            TokenTree::Lit(_) => self.literal(&token_tree)?,
            TokenTree::Op(op) => {
                return Err(Syntax::error(
                    &format!("Unexpected operator {}", op).to_string(),
                    &token_tree.span,
                ))
            },
            TokenTree::Label(_) => self.label(&token_tree)?,
            TokenTree::Iden(_) => self.symbol(&token_tree)?,
            TokenTree::Form(trees) => self.expr(&trees, &mut 0, Prec::None)?,
            TokenTree::Block(trees) => {
                let mut expressions = vec![];
                for tree in trees {
                    expressions.push(self.expr(
                        &tree.item,
                        &mut 0,
                        Prec::None,
                    )?);
                }
                let span = Spanned::build(&expressions);
                Spanned::new(AST::Base(Base::Block(expressions)), span)
            },
            TokenTree::List(_) => unimplemented!(),
        };
        Ok(result)
    }

    // TODO: replace hacky trees/trees_idx
    fn expr(
        &mut self,
        trees: &TokenTrees,
        trees_idx: &mut usize,
        prec: Prec,
    ) -> Result<Spanned<AST>, Syntax> {
        if *trees_idx >= trees.len() {
            return Err(Syntax::error(
                "Expected an expression",
                &trees.last().unwrap().span,
            ));
        }

        let mut left = self.rule_prefix(&trees[*trees_idx])?;
        *trees_idx += 1;

        while *trees_idx < trees.len() {
            if self.prec(&trees[*trees_idx])? < prec {
                break;
            }
            left = self.rule_infix(left, trees, trees_idx)?;
        }

        Ok(left)
    }

    fn rule_prefix(
        &mut self,
        tree: &Spanned<TokenTree>,
    ) -> Result<Spanned<AST>, Syntax> {
        let result = match tree.item {
            TokenTree::Lit(_) => self.literal(&tree)?,
            TokenTree::Label(_) => self.label(&tree)?,
            TokenTree::Iden(ref name) => match ResIden::try_new(&name) {
                Some(iden) => match iden {
                    ResIden::Macro
                    | ResIden::Type
                    | ResIden::If
                    | ResIden::Match
                    | ResIden::Mod => {
                        return Err(Syntax::error(
                            "This feature is a work in progress",
                            &tree.span,
                        ))
                    },
                },
                None => self.symbol(&tree)?,
            },
            _ => {
                return Err(Syntax::error("Expected an expression", &tree.span))
            },
        };
        Ok(result)
    }

    /// Looks at the current token and parses an infix
    /// expression like an operator. Because an operator
    /// can be used to split an expression across multiple
    /// lines, this function ignores separator tokens
    /// around the operator.
    fn rule_infix(
        &mut self,
        left: Spanned<AST>,
        trees: &TokenTrees,
        trees_idx: &mut usize,
    ) -> Result<Spanned<AST>, Syntax> {
        use ResOp::*;
        let tree: &Spanned<TokenTree> = &trees[*trees_idx];
        match &tree.item {
            TokenTree::Op(name) => match Parser::to_op(&name, &tree.span)? {
                // Pattern-based
                Assign => self.assign(left, trees, trees_idx),
                Lambda => self.lambda(left, trees, trees_idx),

                // Simple binops
                Compose => {
                    self.binop(left, trees, trees_idx, true, Compose, |l, r| {
                        AST::Sugar(Sugar::comp(l, r))
                    })
                },
                Is => self.binop(left, trees, trees_idx, true, Is, |l, r| {
                    AST::Sugar(Sugar::is(l, r))
                }),
                Field => {
                    self.binop(left, trees, trees_idx, true, Field, |l, r| {
                        AST::Sugar(Sugar::field(l, r))
                    })
                },

                // Tuples
                Pair => {
                    self.binop(left, trees, trees_idx, true, Pair, |l, r| {
                        let mut tuple = match l.item {
                            AST::Base(Base::Tuple(t)) => t,
                            _ => vec![l],
                        };
                        tuple.push(r);
                        AST::Base(Base::Tuple(tuple))
                    })
                },

                // Builtins
                Add => todo!(),
                Sub => todo!(),
                Mul => todo!(),
                Div => todo!(),
                Rem => todo!(),
                Equal => todo!(),
                Pow => todo!(),
            },

            _ => self.call(left, trees, trees_idx),
        }
    }

    /// Finds the corresponding [`ResOp`] for a string.
    /// Raises a syntax error if the operator string is
    /// invalid.
    fn to_op(name: &str, span: &Span) -> Result<ResOp, Syntax> {
        ResOp::try_new(&name).ok_or_else(|| {
            Syntax::error(&format!("Invalid operator `{}`", name), &span)
        })
    }

    fn op_prec(op: ResOp) -> Prec {
        match op {
            ResOp::Assign => Prec::Assign,
            ResOp::Lambda => Prec::Lambda,
            ResOp::Pair => Prec::Pair,
            ResOp::Field => Prec::Field,
            ResOp::Compose => Prec::Compose,
            ResOp::Is => Prec::Is,

            ResOp::Add | ResOp::Sub => Prec::AddSub,

            ResOp::Mul | ResOp::Div | ResOp::Rem => Prec::MulDiv,

            ResOp::Equal => Prec::Logic,
            ResOp::Pow => Prec::Pow,
        }
    }

    /// Returns the precedence of the current non-sep token
    /// being parsed.
    fn prec(&mut self, tree: &Spanned<TokenTree>) -> Result<Prec, Syntax> {
        let result = match &tree.item {
            // Prefix
            TokenTree::Label(_) | TokenTree::Iden(_) | TokenTree::Lit(_) => {
                Prec::Call
            },

            // Infix ops
            TokenTree::Op(name) => {
                Parser::op_prec(Parser::to_op(&name, &tree.span)?)
            },

            // Unreachable because we skip all all non-sep tokens
            _ => panic!(),
        };

        Ok(result)
    }

    /// Constructs the AST for a literal, such as a number
    /// or string.
    fn literal(
        &mut self,
        tree: &Spanned<TokenTree>,
    ) -> Result<Spanned<AST>, Syntax> {
        let leaf = if let TokenTree::Lit(lit) = &tree.item {
            AST::Base(Base::Lit(lit.clone()))
        } else {
            return Err(Syntax::error(
                &format!("Expected a literal, found {}", &tree.item),
                &tree.span,
            ));
        };

        Ok(Spanned::new(leaf, tree.span.clone()))
    }

    /// Interns a symbol in the parser,
    /// so that future symbols with the same name can be
    /// replaced consistently.
    fn intern_symbol(&mut self, name: &str) -> SharedSymbol {
        if let Some(symbol) = self.symbols.get(name) {
            *symbol
        } else {
            let symbol = SharedSymbol(self.symbols.len());
            self.symbols.insert(name.to_string(), symbol);
            symbol
        }
    }

    /// Parses a Label.
    fn label(
        &mut self,
        tree: &Spanned<TokenTree>,
    ) -> Result<Spanned<AST>, Syntax> {
        // TODO: keep track of labels for typedefs?
        let symbol = if let TokenTree::Label(label) = &tree.item {
            self.intern_symbol(&label)
        } else {
            return Err(Syntax::error(
                &format!("Expected a label, found {}", &tree.item),
                &tree.span,
            ));
        };
        Ok(Spanned::new(
            AST::Base(Base::Symbol(symbol)),
            tree.span.clone(),
        ))
    }

    /// Constructs an AST for a symbol,
    /// interning symbols with same names in the parser.
    /// So, for instance, in the following snippet:
    /// ```ignore
    /// x = 0
    /// x -> x + 1
    /// ```
    /// All `x`s would be interned to the same number,
    /// even though they represent semantically different
    /// things. Semantic names are resoled in a later
    /// pass.
    fn symbol(
        &mut self,
        tree: &Spanned<TokenTree>,
    ) -> Result<Spanned<AST>, Syntax> {
        let symbol = if let TokenTree::Iden(label) = &tree.item {
            self.intern_symbol(&label)
        } else {
            return Err(Syntax::error(
                &format!("Expected an identifier, found {}", &tree.item),
                &tree.span,
            ));
        };

        Ok(Spanned::new(
            AST::Base(Base::Symbol(symbol)),
            tree.span.clone(),
        ))
    }

    /// Parses a function call.
    /// Function calls are a bit magical,
    /// because they're just a series of expressions.
    /// There's a bit of magic involved -
    /// we interpret anything that isn't an operator as a
    /// function call operator. Then pull a fast one and
    /// not parse it like an operator at all.
    fn call(
        &mut self,
        left: Spanned<AST>,
        trees: &TokenTrees,
        trees_idx: &mut usize,
    ) -> Result<Spanned<AST>, Syntax> {
        let argument = self.expr(trees, trees_idx, Prec::Call.left())?;
        let combined = Span::combine(&left.span, &argument.span);

        let mut form = match left.item {
            AST::Sugar(Sugar::Form(f)) => f,
            _ => vec![left],
        };
        form.push(argument);
        return Ok(Spanned::new(AST::Sugar(Sugar::Form(form)), combined));
    }

    /// TODO: just specify precedence directly?
    /// Parses a binary operation.
    /// Takes the left side of the operation,
    /// whether or not the operation is left-associative,
    /// the operator precedence,
    /// and a function that creates the AST node.
    fn binop<T>(
        &mut self,
        left: Spanned<T>,
        trees: &TokenTrees,
        trees_idx: &mut usize,
        is_left: bool,
        op: ResOp,
        make_ast: impl Fn(Spanned<T>, Spanned<AST>) -> AST,
    ) -> Result<Spanned<AST>, Syntax> {
        let prec = Parser::op_prec(op);
        let prec = if is_left { prec.left() } else { prec };
        *trees_idx += 1; // move on from operator
        let right = self.expr(trees, trees_idx, prec)?;

        let combined = Span::combine(&left.span, &right.span);
        Ok(Spanned::new(make_ast(left, right), combined))
    }

    /// Parses a lambda definition, associates right.
    fn lambda(
        &mut self,
        left: Spanned<AST>,
        trees: &TokenTrees,
        trees_idx: &mut usize,
    ) -> Result<Spanned<AST>, Syntax> {
        let left_span = left.span.clone();
        let pattern = left
            .map(Pattern::try_from)
            .map_err(|e| Syntax::error(&e, &left_span))?;
        self.binop(pattern, trees, trees_idx, false, ResOp::Lambda, |l, r| {
            AST::Lambda(Lambda::new(l, r))
        })
    }

    /// Parses an assignment, associates right.
    fn assign(
        &mut self,
        left: Spanned<AST>,
        trees: &TokenTrees,
        trees_idx: &mut usize,
    ) -> Result<Spanned<AST>, Syntax> {
        let left_span = left.span.clone();
        let pattern = left
            .map(Pattern::<SharedSymbol>::try_from)
            .map_err(|e| Syntax::error(&e, &left_span))?;
        self.binop(pattern, trees, trees_idx, false, ResOp::Assign, |l, r| {
            AST::Base(Base::assign(l, r))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::source::Source,
        compiler::lex::Lexer,
    };

    #[test]
    fn literal() {
        let source = "2";
        let tokens = Lexer::lex(Source::source(source)).unwrap();
        let token_tree = Reader::read(tokens).unwrap();
        let result = Parser::parse(token_tree);
        dbg!("{:?}", &result);
        let (ast, _symbols) = result.unwrap();
    }

    #[test]
    fn symbol() {
        let source = "x";
        let tokens = Lexer::lex(Source::source(source)).unwrap();
        let token_tree = Reader::read(tokens).unwrap();
        let result = Parser::parse(token_tree);
        dbg!("{:?}", &result);
        let (ast, _symbols) = result.unwrap();
    }

    #[test]
    fn assign() {
        let source = "x = 2\ny = 4";
        let tokens = Lexer::lex(Source::source(source)).unwrap();
        let token_tree = Reader::read(tokens).unwrap();
        let result = Parser::parse(token_tree);
        dbg!("{:?}", &result);
        let (ast, _symbols) = result.unwrap();
    }

    #[test]
    fn field() {
        let source = "x = hello.world";
        let tokens = Lexer::lex(Source::source(source)).unwrap();
        let token_tree = Reader::read(tokens).unwrap();
        let result = Parser::parse(token_tree);
        dbg!("{:?}", &result);
        let (ast, _symbols) = result.unwrap();
    }

    #[test]
    fn is() {
        // TODO: enforce labels to begin with a capital letter?
        let source = "y: asdf";
        let tokens = Lexer::lex(Source::source(source)).unwrap();
        let token_tree = Reader::read(tokens).unwrap();
        let result = Parser::parse(token_tree);
        dbg!("{:?}", &result);
        let (ast, _symbols) = result.unwrap();
    }

    #[test]
    fn lambda() {
        let source = "x = a -> f a";
        let tokens = Lexer::lex(Source::source(source)).unwrap();
        let token_tree = Reader::read(tokens).unwrap();
        let result = Parser::parse(token_tree);
        dbg!("{:?}", &result);
        let (ast, _symbols) = result.unwrap();
    }
}
