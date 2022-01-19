use std::{
    mem,
    rc::Rc,
    collections::HashMap,
    convert::TryFrom,
};

use crate::common::{
    span::{Span, Spanned},
    lit::Lit,
};

use crate::compiler::syntax::Syntax;

use crate::construct::{
    token::{Token, Tokens, Delim, ResOp, ResIden},
    tree::{AST, Base, Sugar, Lambda, Pattern},
    symbol::SharedSymbol,
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
        if let Prec::End = self { panic!("Can not associate further left") }
        return unsafe { mem::transmute(self.clone() as u8 + 1) };
    }
}

#[derive(Debug)]
pub struct Parser {
    /// Stack of token streams because tokens can be grouped.
    /// The topmost token stream is the one being parsed.
    tokens_stack: Vec<Rc<Tokens>>,
    /// Stack of locations in the parsing stream.
    /// The topmost token is the current token being looked at.
    indicies: Vec<usize>,
    /// Symbols with the same name are interned.
    /// We don't do this during lexing so that token-based macros
    /// can work with strings.
    symbols: HashMap<String, SharedSymbol>,
}

impl Parser {
    /// Parses some tokens into a syntax tree.
    /// This will produce a module as opposed to a block.
    /// Also returns the symbol interning table.
    pub fn parse(tokens: Tokens) -> Result<(Spanned<AST>, HashMap<String, SharedSymbol>), Syntax> {
        // build base parser
        let mut parser = Parser {
            tokens_stack: vec![Rc::new(tokens)],
            indicies: vec![0],
            symbols: HashMap::new(),
        };

        // parse and wrap it in a module
        let ast = parser.bare_module()?;

        // return itg
        Ok((ast, parser.symbols))
    }

    /// Gets the stream of tokens currently being parsed.
    fn tokens(&self) -> &Tokens {
        &self.tokens_stack.last().unwrap()
    }

    /// Gets the index of the current token being parsed.
    fn index(&self) -> usize {
        *self.indicies.last().unwrap()
    }

    /// Returns a mutable reference to the current index.
    /// used to advance the parser.
    fn index_mut(&mut self) -> &mut usize {
        self.indicies.last_mut().unwrap()
    }

    /// Peeks the current token, does not advance the parser.
    fn peek_token(&self) -> Result<&Spanned<Token>, Syntax> {
        self.tokens().get(self.index()).ok_or_else(|| self.end_of_source())
    }

    /// Peeks the current non-sep token,
    /// returning None if none exists (i.e. we hit the end of the file).
    /// Does not advance the parser.
    fn peek_non_sep(&self) -> Result<&Spanned<Token>, Syntax> {
        for i in 0.. {
            let token = self.tokens().get(self.index() + i);
            let token = if token.is_none() { break; } else { token.unwrap() };
            if token.item != Token::Sep { return Ok(token); }
        }
        Err(self.end_of_source())
    }

    /// Advances the parser by one token.
    fn advance_token(&mut self) -> Result<&Spanned<Token>, Syntax> {
        *self.index_mut() = self.index() + 1;
        self.tokens().get(self.index() - 1)
            .ok_or_else(|| self.end_of_source())
    }

    /// Advances the parser until the first non-sep token
    /// Stops advancing if it runs out of tokens
    fn advance_non_sep(&mut self) /* -> Result<&Spanned<Token>> */ {
        for i in 0.. {
            match self.tokens().get(self.index() + i) {
                Some(t) if t.item != Token::Sep => break,
                Some(_) => (),
                None => return /* None */,
            }
        }
        // doesn't need to be `peek_non_sep`
        // self.peek_token()
    }

    /// Returns whether the Parser is done parsing the current token stream.
    fn is_done(&self) -> bool {
        self.index() >= self.tokens().len()
    }

    /// Finds the corresponding [`ResOp`] for a string.
    /// Raises a syntax error if the operator string is invalid.
    fn to_op(name: &str, span: &Span) -> Result<ResOp, Syntax> {
        ResOp::try_new(&name)
            .ok_or_else(|| Syntax::error(
                &format!("Invalid operator `{}`", name),
                &span,
            ))
    }

    // TODO: delete AAH to_tokendjk

    // fn to_token<'a, 'b>(&'a self, option: Option<&'b Spanned<Token>>) -> Result<&'b Spanned<Token>, Syntax> {
    //     option.ok_or_else(|| self.end_of_source())
    // }

    fn end_of_source(&self) -> Syntax {
        let last_span = self.tokens()
            .last()
            .expect("Can't figure out the cause of this error, aah!")
            .span.clone();

        Syntax::error (
            "Unexpected end of source while parsing",
            &last_span,
        )
    }

    fn op_prec(op: ResOp) -> Prec {
        match op {
            ResOp::Assign  => Prec::Assign,
            ResOp::Lambda  => Prec::Lambda,
            ResOp::Pair    => Prec::Pair,
            ResOp::Field   => Prec::Field,
            ResOp::Compose => Prec::Compose,
            ResOp::Is      => Prec::Is,

            | ResOp::Add
            | ResOp::Sub => Prec::AddSub,

            | ResOp::Mul
            | ResOp::Div
            | ResOp::Rem => Prec::MulDiv,

            ResOp::Equal => Prec::Logic,
            ResOp::Pow   => Prec::Pow,
        }
    }

    /// Returns the precedence of the current non-sep token being parsed.
    /// Note that when parsing a form, a separator token has a precedence of `Prec::End`.
    /// ```ignore
    /// these are two
    /// different forms
    /// ```
    /// Is parsed as: `{(these are two); (different forms)}`
    fn prec(&mut self) -> Result<Prec, Syntax> {
        let is_sep = self.peek_token()?.item == Token::Sep;
        let token = self.peek_non_sep()?;

        let result = match &token.item {
            // Prefix
            | Token::Delim(_, _)
            | Token::Label(_)
            | Token::Iden(_)
            | Token::Lit(_) => if is_sep { Prec::End } else { Prec::Call },

            // Infix ops
            Token::Op(name) => Parser::op_prec(Parser::to_op(&name, &token.span)?),

            // Unreachable because we skip all all non-sep tokens
            Token::Sep => unreachable!(),
        };

        Ok(result)
    }

    /// Looks at the current token and parses a prefix expression, like keywords.
    /// This function will strip preceeding separator tokens.
    fn rule_prefix(&mut self) -> Result<Spanned<AST>, Syntax> {
        self.advance_non_sep();
        let token = self.peek_token()?;

        match &token.item {
            Token::Delim(delim, _) => match delim {
                Delim::Curly  => self.block(),
                Delim::Paren  => self.group(),
                Delim::Square => Err(Syntax::error("Lists are not yet implemented", &token.span)),
            },
            Token::Iden(ref name) => match ResIden::try_new(&name) {
                Some(iden) => match iden {
                    | ResIden::Macro
                    | ResIden::Type
                    | ResIden::If
                    | ResIden::Match
                    | ResIden::Mod => Err(Syntax::error(
                        "This feature is a work in progress",
                        &token.span
                    )),
                },
                None => self.symbol(),
            },
            Token::Label(_) => self.label(),
            Token::Lit(_)   => self.literal(),
            _               => Err(Syntax::error("Expected an expression", &token.span)),
        }
    }

    /// Constructs the AST for a literal, such as a number or string.
    fn literal(&mut self) -> Result<Spanned<AST>, Syntax> {
        let token = self.advance_token()?;

        let leaf = if let Token::Lit(l) = &token.item {
            AST::Base(Base::Lit(l.clone()))
        } else {
            return Err(Syntax::error(
                &format!("Expected a literal, found {}", token.item),
                &token.span
            ));
        };

        Ok(Spanned::new(leaf, token.span.clone()))
    }

    /// Interns a symbol in the parser,
    /// so that future symbols with the same name can be replaced consistently.
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
    fn label(&mut self) -> Result<Spanned<AST>, Syntax> {
        let token = self.advance_token()?;
        let span = token.span.clone();

        // TODO: keep track of labels for typedefs?
        let index = match &token.item {
            Token::Label(name) => {
                let name = name.clone();
                self.intern_symbol(&name)
            },
            _ => unreachable!(),
        };

        Ok(Spanned::new(AST::Base(Base::Symbol(index)), span))
    }

    /// Constructs an AST for a symbol,
    /// interning symbols with same names in the parser.
    /// So, for instance, in the following snippet:
    /// ```ignore
    /// x = 0
    /// x -> x + 1
    /// ```
    /// All `x`s would be interned to the same number,
    /// even though they represent semantically different things.
    /// Semantic names are resoled in a later pass.
    fn symbol(&mut self) -> Result<Spanned<AST>, Syntax> {
        let token = self.advance_token()?;
        let span = token.span.clone();

        // TODO: println
        // hook into effect system for operators.
        let index = match &token.item {
            Token::Iden(name) => {
                let name = name.clone();
                self.intern_symbol(&name)
            },
            _ => unreachable!(),
        };

        Ok(Spanned::new(AST::Base(Base::Symbol(index)), span))
    }

    /// Constructs the ast for a group,
    /// i.e. an expression between parenthesis.
    fn group(&mut self) -> Result<Spanned<AST>, Syntax> {
        let ungrouped = self.delim_inner(Delim::Paren)?;
        self.enter_delim(ungrouped.item);
        let ast = self.expr(Prec::None.left(), true)?;
        self.exit_delim();
        Ok(Spanned::new(AST::Sugar(Sugar::group(ast)), ungrouped.span))
    }

    /// Enters a new group for parsing.
    /// Note that this call must be balanced with a call to [`exit_group`]
    fn enter_delim(&mut self, tokens: Rc<Tokens>) {
        self.indicies.push(0);
        self.tokens_stack.push(tokens);
    }

    /// Exits a group once done parsing that group.
    fn exit_delim(&mut self) {
        self.indicies.pop();
        self.tokens_stack.pop();
    }

    /// Throws an error if the next token is unexpected.
    /// I get one funny error message and this is it.
    /// The error message returned by this function will be changed frequently.
    /// I will merge any PR that changes this error message to something funny (within reason).
    fn unexpected(&self) -> Syntax {
        let token = match self.peek_token() {
            Ok(t) => t,
            Err(s) => return s,
        };

        Syntax::error(
            &format!("Zoinks Scoob! What's {} doing here!?", token.item),
            &token.span,
        )
    }

    /// Expects the next token to be a group token, containing a sub token stream.
    /// Unwraps the group, and returns the spanned token stream.
    /// Appends Token::End to the expanded token stream.
    /// The returned Span includes the delimiters.
    fn delim_inner(&mut self, expected_delim: Delim) -> Result<Spanned<Rc<Tokens>>, Syntax> {
        let group = self.advance_token()?;
        if let Token::Delim(delim, tokens) = &group.item {
            if delim == &expected_delim {
                return Ok(Spanned::new(tokens.clone(), group.span.clone()));
            }
        };

        Err(self.unexpected())
    }

    /// Parses the body of a block.
    /// A block is one or more expressions, separated by separators.
    /// This is more of a helper function, as it serves as both the
    /// parser entrypoint while still being recursively nestable.
    fn body(&mut self) -> Result<Spanned<AST>, Syntax> {
        let mut expressions = vec![];

        while !self.is_done() {
            let ast = self.expr(Prec::None, false)?;
            expressions.push(ast);
            match self.advance_token()?.item {
                Token::Sep => (),
                _          => break,
            }
        }

        dbg!(self);

        let span = Spanned::build(&expressions);
        return Ok(Spanned::new(AST::Base(Base::Block(expressions)), span));
    }

    fn bare_module(&mut self) -> Result<Spanned<AST>, Syntax> {
        let ast = self.body()?;
        let span = ast.span.clone();
        Ok(Spanned::new(AST::Base(Base::Module(Box::new(ast))), span))
    }

    // TODO: maybe just stop finangling and reference count `Tokens` already
    /// Parses a block, i.e. a list of expressions executed one after another between curlies.
    fn block(&mut self) -> Result<Spanned<AST>, Syntax> {
        let tokens = self.delim_inner(Delim::Paren)?;
        self.enter_delim(tokens.item);
        let mut ast = self.body()?;
        self.exit_delim();

        // TODO: construct a record if applicable
        return Ok(Spanned::new(ast.item, tokens.span));
    }

    /// Looks at the current token and parses an infix expression like an operator.
    /// Because an operator can be used to split an expression across multiple lines,
    /// this function ignores separator tokens around the operator.
    fn rule_infix(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        self.advance_non_sep();
        let token = self.peek_token()?;

        use ResOp::*;
        match &token.item {
            Token::Op(name) => match Parser::to_op(&name, &token.span)? {
                // Pattern-based
                Assign  => self.assign(left),
                Lambda  => self.lambda(left),

                // Simple binops
                Compose => self.binop(left, true, Compose, |l, r| AST::Sugar(Sugar::comp(l, r))),
                Is      => self.binop(left, true, Is,      |l, r| AST::Sugar(Sugar::is(l, r))),
                Field   => self.binop(left, true, Field,   |l, r| AST::Sugar(Sugar::field(l, r))),

                // Tuples
                Pair => self.binop(left, true, Pair, |l, r| {
                    let mut tuple = match l.item {
                        AST::Base(Base::Tuple(t)) => t,
                        _ => vec![l],
                    };
                    tuple.push(r);
                    AST::Base(Base::Tuple(tuple))
                }),

                // Builtins
                Add   => todo!(),
                Sub   => todo!(),
                Mul   => todo!(),
                Div   => todo!(),
                Rem   => todo!(),
                Equal => todo!(),
                Pow   => todo!(),
            },

            Token::Sep => unreachable!(),
            _ => self.call(left),
        }
    }

    /// Parses a function call.
    /// Function calls are a bit magical,
    /// because they're just a series of expressions.
    /// There's a bit of magic involved -
    /// we interpret anything that isn't an operator as a function call operator.
    /// Then pull a fast one and not parse it like an operator at all.
    fn call(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let argument = self.expr(Prec::Call.left(), false)?;
        let combined = Span::combine(&left.span, &argument.span);

        let mut form = match left.item {
            AST::Sugar(Sugar::Form(f)) => f,
            _ => vec![left],
        };
        form.push(argument);
        return Ok(Spanned::new(AST::Sugar(Sugar::Form(form)), combined));
    }

    /// Consumes an expected operator, produces an error otherwise.
    fn consume_op(&mut self, op: ResOp) -> Result<&Spanned<Token>, Syntax> {
        let err = Err(self.unexpected());
        let token = self.advance_token()?;

        if let Token::Op(ref name) = token.item {
            if Parser::to_op(name, &token.span)? == op {
                return Ok(token);
            }
        }

        return err;
    }

    /// Parses a binary operation.
    fn binop<T>(
        &mut self,
        left: Spanned<T>,
        is_left: bool,
        op: ResOp,
        make_ast: impl Fn(Spanned<T>, Spanned<AST>) -> AST,
    ) -> Result<Spanned<AST>, Syntax> {
        self.consume_op(op)?;

        let prec  = Parser::op_prec(op);
        let prec  = if is_left { prec.left() } else { prec };
        let right = self.expr(prec, false)?;

        let combined = Span::combine(&left.span, &right.span);
        Ok(Spanned::new(make_ast(left, right), combined))
    }

    /// Parses a lambda definition, associates right.
    fn lambda(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let left_span = left.span.clone();
        let pattern = left.map(Pattern::try_from)
            .map_err(|e| Syntax::error(&e, &left_span))?;
        self.binop(pattern, false, ResOp::Lambda, |l, r| AST::Lambda(Lambda::new(l, r)))
    }

    /// Parses an assignment, associates right.
    fn assign(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let left_span = left.span.clone();
        let pattern = left.map(Pattern::<SharedSymbol>::try_from)
            .map_err(|e| Syntax::error(&e, &left_span))?;
        self.binop(pattern, false, ResOp::Assign, |l, r| AST::Base(Base::assign(l, r)))
    }

    /// Parses an expression within a given precedence,
    /// which produces an AST node.
    /// If the expression is empty, returns an empty AST block.
    fn expr(&mut self, prec: Prec, is_form: bool) -> Result<Spanned<AST>, Syntax> {
        let mut left = self.rule_prefix()?;

        while !self.is_done() {
            if is_form { self.advance_non_sep() }
            if self.prec()? < prec { break; }
            left = self.rule_infix(left)?;
        }

        Ok(left)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::source::Source;
    use crate::compiler::lex::Lexer;
    use super::*;

    fn tokens(source: &str) -> Tokens {
        Lexer::lex(Source::source("")).unwrap()
    }

    #[test]
    fn empty() {
        Parser::parse(tokens("y = x -> x + 1")).unwrap();
    }
}
