use std::{
    mem,
    convert::TryFrom,
    collections::HashMap,
};

use crate::common::{
    span::{Span, Spanned},
    data::Data,
};

use crate::compiler::{lower::Lower, syntax::Syntax};

use crate::construct::{
    token::{Token, Tokens, Delim, ResOp, ResIden},
    tree::{AST, Base, Sugar, Lambda, Pattern, ArgPattern},
    symbol::SharedSymbol,
    module::{ThinModule, Module},
};

/// TODO: Instead of calling .body, wrap everything in a Block,
/// Which should simplify code later on

impl Lower for ThinModule<Vec<Spanned<Token>>> {
    type Out = Module<Spanned<AST>, usize>;

    /// Simple function that parses a token stream into an AST.
    /// Exposes the functionality of the `Parser`.
    fn lower(self) -> Result<Self::Out, Syntax> {
        let mut parser = Parser::new(self.repr);
        let ast = parser.body()?;

        println!("{:#?}", ast);
        todo!("See above TODO");
        // parser.consume(Token::End)?;

        return Ok(Module::new(
            Spanned::new(ast, Span::empty()),
            parser.symbols.len()
        ));
    }
}


/// We're using a Pratt parser, so this little enum
/// defines different precedence levels.
/// Each successive level is higher, so, for example,
/// `* > +`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prec {
    None = 0,
    Assign,
    Pair,
    Is,
    Lambda,

    Logic,

    AddSub,
    MulDiv,
    Pow,

    Compose, // TODO: where should this be, precedence-wise?
    Call,
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
    pub fn left(&self) -> Prec {
        if let Prec::End = self { panic!("Can not associate further left") }
        return unsafe { mem::transmute(self.clone() as u8 + 1) };
    }
}

/// Constructs an `AST` from a token stream.
/// Note that this struct should not be controlled manually,
/// use the `parse` function instead.
#[derive(Debug)]
pub struct Parser {
    /// Stack of token streams because tokens can be grouped.
    /// The topmost token stream is the one being parsed.
    tokens:  Vec<Vec<Spanned<Token>>>,
    index:   usize,
    symbols: HashMap<String, SharedSymbol>,
}

impl Parser {
    /// Create a new `parser`.
    pub fn new(tokens: Vec<Spanned<Token>>) -> Parser {
        Parser { tokens: vec![tokens], index: 0, symbols: HashMap::new() }
    }

    // Cookie Monster's Helper Functions:

    pub fn tokens(&self) -> &Vec<Spanned<Token>> {
        return &self.tokens[self.tokens.len() - 1];
    }

    // NOTE: Maybe don't return bool?
    /// Consumes all seperator tokens, returning whether there were any.
    pub fn sep(&mut self) -> bool {
        if self.tokens()[self.index].item != Token::Sep { false } else {
            while self.tokens()[self.index].item == Token::Sep {
                self.index += 1;
            };
            true
        }
    }

    // TODO: merge with sep?
    /// Returns the next non-sep tokens,
    /// without advancing the parser.
    pub fn draw(&self) -> &Spanned<Token> {
        let mut offset = 0;

        while self.tokens()[self.index + offset].item == Token::Sep {
            offset += 1;
        }

        return &self.tokens()[self.index + offset];
    }

    /// Returns the current token then advances the parser.
    pub fn advance(&mut self) -> &Spanned<Token> {
        self.index += 1;
        &self.tokens()[self.index - 1]
    }

    /// Returns the first token.
    pub fn current(&self) -> &Spanned<Token> {
        &self.tokens()[self.index]
    }

    /// Returns the first non-Sep token.
    pub fn skip(&mut self) -> &Spanned<Token> {
        self.sep();
        self.current()
    }

    /// Throws an error if the next token is unexpected.
    /// I get one funny error message and this is it.
    /// The error message returned by this function will be changed frequently
    pub fn unexpected(&self) -> Syntax {
        let token = self.current();
        Syntax::error(
            &format!("Oopsie woopsie, what's {} doing here?", token.item),
            &token.span
        )
    }

    /// Consumes a specific token then advances the parser.
    /// Can be used to consume Sep tokens, which are normally skipped.
    pub fn consume_iden(&mut self, iden: ResIden) -> Result<&Spanned<Token>, Syntax> {
        let current = self.advance();

        if let Token::Iden(ref name) = current.item {
            if ResIden::try_new(&name)
                .ok_or(Syntax::error("Invalid keyword", &current.span))?
            == iden {
                return Ok(current);
            }
        }

        Err(Syntax::error(
            &format!("Encountered unexpected {}", current.item),
            &current.span
        ))
    }

    pub fn consume_op(&mut self, op: ResOp) -> Result<&Spanned<Token>, Syntax> {
        let current = self.advance();

        if let Token::Op(ref name) = current.item {
            if ResOp::try_new(&name)
                .ok_or(Syntax::error("Invalid operator", &current.span))?
            == op {
                return Ok(current);
            }
        }

        Err(Syntax::error(
            &format!("Encountered unexpected {}", current.item),
            &current.span
        ))
    }

    pub fn intern_symbol(&mut self, name: &str) -> SharedSymbol {
        if let Some(symbol) = self.symbols.get(name) {
            *symbol
        } else {
            let symbol = SharedSymbol(self.symbols.len());
            self.symbols.insert(name.to_string(), symbol);
            symbol
        }
    }

    // Core Pratt Parser:

    /// Looks at the current token and parses an infix expression
    pub fn rule_prefix(&mut self) -> Result<Spanned<AST>, Syntax> {
        match self.skip().item {
            Token::End => todo!("remove end from prefix rule"), // Ok(Spanned::new(AST::Base(Base::Block(vec![])), Span::empty())),
            Token::Group { delim, .. } => match delim {
                Delim::Curly => self.block(),
                Delim::Paren => self.group(),
                Delim::Square => todo!("Lists are not yet implemented"),
            },
            Token::Iden(ref name) => match ResIden::try_new(&name) {
                // keywords
                Some(ResIden::Type)  => todo!(),
                Some(ResIden::Magic) => self.magic(),
                None                 => self.symbol(),
            },
            Token::Label(_) => self.label(),
            Token::Data(_)  => self.literal(),
            Token::Sep      => unreachable!(),
            _               => Err(Syntax::error("Expected an expression", &self.current().span)),
        }
    }

    /// Looks at the current token and parses the right side of any infix expressions.
    pub fn rule_infix(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        match self.skip().item {
            Token::Op(ref name) => match ResOp::try_new(&name)
                .ok_or(Syntax::error("Invalid operator", &self.current().span))?
            {
                ResOp::Assign  => self.assign(left),
                ResOp::Lambda  => self.lambda(left),
                ResOp::Compose => self.compose(left),
                ResOp::Pair    => self.pair(left),
                ResOp::Add     => self.binop(Prec::AddSub.left(), "add",   left),
                ResOp::Sub     => self.binop(Prec::AddSub.left(), "sub",   left),
                ResOp::Mul     => self.binop(Prec::MulDiv.left(), "mul",   left),
                ResOp::Div     => self.binop(Prec::MulDiv.left(), "div",   left),
                ResOp::Rem     => self.binop(Prec::MulDiv.left(), "rem",   left),
                ResOp::Equal   => self.binop(Prec::Logic.left(),  "equal", left),
                ResOp::Pow     => self.binop(Prec::Pow,           "pow",   left),
            },

            Token::End => Err(self.unexpected()),
            Token::Sep => unreachable!(),
            _          => self.call(left),
        }
    }

    /// Looks at the current operator token and determines the precedence
    pub fn prec(&mut self) -> Result<Prec, Syntax> {
        let next = self.draw().item.clone();
        let current = self.current().item.clone();
        let sep = next != current;

        let prec = match next {
            // infix
            Token::Op(name) => match ResOp::try_new(&name)
                .ok_or(Syntax::error("Invalid operator", &self.current().span))?
            {
                ResOp::Assign  => Prec::Assign,
                ResOp::Lambda  => Prec::Lambda,
                ResOp::Compose => Prec::Compose,
                ResOp::Pair    => Prec::Pair,
                ResOp::Add     => Prec::AddSub,
                ResOp::Sub     => Prec::AddSub,
                ResOp::Mul     => Prec::MulDiv,
                ResOp::Div     => Prec::MulDiv,
                ResOp::Rem     => Prec::MulDiv,
                ResOp::Equal   => Prec::Logic,
                ResOp::Pow     => Prec::Pow,
            },

            // postfix
            Token::End => Prec::End,

            // prefix
              Token::Group { .. }
            | Token::Iden(_)
            | Token::Label(_)
            | Token::Data(_) => Prec::Call,

            // unreachable (doh)
            Token::Sep => unreachable!(),
        };

        if sep && prec == Prec::Call {
            Ok(Prec::End)
        } else {
            Ok(prec)
        }
    }

    // TODO: factor out? only group uses the skip sep flag...
    /// Uses some pratt parser magic to parse an expression.
    /// It's essentially a fold-left over tokens
    /// based on the precedence and content.
    /// Cool stuff.
    pub fn expression(&mut self, prec: Prec, skip_sep: bool) -> Result<Spanned<AST>, Syntax> {
        let mut left = self.rule_prefix()?;

        // TODO: switch to this?
        // loop {
        //     if skip_sep { self.sep(); }
        //     let p = self.prec()?;
        //     if p < prec || p == Prec::End { break; }
        //     left = self.rule_infix(left)?;
        // }

        while {
            if skip_sep { self.sep(); }
            let p = self.prec()?;
            p >= prec && p != Prec::End
        } {
            left = self.rule_infix(left)?;
        }

        return Ok(left);
    }

    // Rule Definitions:

    // Prefix:

    fn super_ugly_println_hack_named_something_silly_so_I_will_remove_it(
        &mut self,
        span: Span,
    ) -> Spanned<AST> {
        // nothing is more permanant, but
        // temporary workaround until prelude
        let var = self.intern_symbol("#println");

        return Spanned::new(
            AST::Lambda(Lambda::new(
                Spanned::new(Pattern::Symbol(var), Span::empty()),
                Spanned::new(AST::Base(Base::ffi(
                    "println",
                    Spanned::new(AST::Base(Base::Symbol(var)), Span::empty()),
                )), Span::empty()),
            )),
            span,
        );
    }

    /// Constructs an AST for a symbol.
    pub fn symbol(&mut self) -> Result<Spanned<AST>, Syntax> {
        let token = self.advance();
        let span = token.span.clone();

        let index = if let Token::Iden(name) = token.item.clone() {
            if name == "println" {
                return Ok(
                    self.super_ugly_println_hack_named_something_silly_so_I_will_remove_it(span)
                );
            } else {
                self.intern_symbol(&name)
            }
        } else {
            todo!()
        };

        Ok(Spanned::new(AST::Base(Base::Symbol(index)), span))
    }

    /// Constructs the AST for a literal, such as a number or string.
    pub fn literal(&mut self) -> Result<Spanned<AST>, Syntax> {
        let Spanned { item: token, span } = self.advance();

        let leaf = if let Token::Data(d) = token {
            AST::Base(Base::Data(d.clone()))
        } else {
            return Err(Syntax::error(
                &format!("Expected a literal, found {}", token),
                &span
            ));
        };

        Ok(Spanned::new(leaf, span.clone()))
    }

    /// Expects the next token to be a group token, containing a sub token stream.
    /// Unwraps the group, and returns the spanned token stream.
    /// Appends Token::End to the expanded token stream.
    /// The returned Span includes the delimiters.
    pub fn ungroup(&mut self, expected_delim: Delim) -> Result<Spanned<Vec<Spanned<Token>>>, Syntax> {
        let group = self.advance();
        // self.index += 1;
        // let len = self.tokens.len() - 1;
        // let group = mem::replace(
        //     &mut self.tokens[len][self.index],
        //     Spanned::new(Token::End, Span::empty()),
        // );
        let span = group.span.clone();

        // TODO: remove clone, nondestructively
        // (not like example above)
        // will have to adjust types (slice over vec) and lifetimes
        if let Token::Group {
            delim,
            mut tokens,
        } = group.item.clone() {
            if delim == expected_delim {
                tokens.push(Spanned::new(Token::End, Span::empty()));
                return Ok(Spanned::new(tokens, span));
            }
        }

        // specified group delimiter was not matched
        return Err(self.unexpected());
    }

    /// Constructs the ast for a group,
    /// i.e. an expression between parenthesis.
    pub fn group(&mut self) -> Result<Spanned<AST>, Syntax> {
        let ungrouped = self.ungroup(Delim::Paren)?;
        self.tokens.push(ungrouped.item);
        // TODO verify that error doesn't mess up parsing by not popping
        let ast = self.expression(Prec::None.left(), true)?;
        todo!("add finalize method that checks for End");
        self.tokens.pop();
        Ok(Spanned::new(AST::Sugar(Sugar::group(ast)), ungrouped.span))
    }

    /// Parses the body of a block.
    /// A block is one or more expressions, separated by separators.
    /// This is more of a helper function, as it serves as both the
    /// parser entrypoint while still being recursively nestable.
    pub fn body(&mut self) -> Result<AST, Syntax> {
        let mut expressions = vec![];

        while self.skip().item != Token::End {
            let ast = self.expression(Prec::None, false)?;
            expressions.push(ast);
            match self.advance().item {
                Token::Sep => (),
                _          => { break; },
            }
        }

        return Ok(AST::Base(Base::Block(expressions)));
    }

    /// Parse a block as an expression,
    /// Building the appropriate `AST`.
    /// Just a body between curlies.
    pub fn block(&mut self) -> Result<Spanned<AST>, Syntax> {
        let ungrouped = self.ungroup(Delim::Paren)?;
        self.tokens.push(ungrouped.item);
        let mut ast = self.body()?;
        self.tokens.pop();

        // construct a record if applicable
        // match ast {
        //     AST::Block(ref b) if b.len() == 0 => match b[0].item {
        //         AST::Tuple(ref t) => {
        //             ast = AST::Record(t.clone())
        //         },
        //         _ => (),
        //     },
        //     _ => (),
        // }

        return Ok(Spanned::new(ast, ungrouped.span));
    }

    // pub fn type_(&mut self) -> Result<Spanned<AST>, Syntax> {
    //     let start = self.consume(Token::Type)?.span.clone();
    //     let label = self.consume(Token::Label)?.span.clone();

    //     return Ok(Spanned::new(AST::type_(label), Span::combine(start, label)))
    // }

    /// Parse an `extern` statement.
    /// used for compiler magic and other glue.
    /// takes the form:
    /// ```ignore
    /// magic "FFI String Name" data_to_pass_out
    /// ```
    /// and evaluates to the value returned by the ffi function.
    pub fn magic(&mut self) -> Result<Spanned<AST>, Syntax> {
        let start = self.consume_iden(ResIden::Magic)?.span.clone();

        let Spanned { item: token, span } = self.advance();
        let name = match token {
            Token::Data(Data::String(s))  => s.clone(),
            unexpected => return Err(Syntax::error(
                &format!("Expected a string, found {}", unexpected),
                &span
            )),
        };

        let ast = self.expression(Prec::End, false)?;
        let end = ast.span.clone();

        return Ok(Spanned::new(
            AST::Base(Base::ffi(&name, ast)),
            Span::combine(&start, &end),
        ));
    }

    /// Parse a label.
    /// A label must be the first element of a form
    pub fn label(&mut self) -> Result<Spanned<AST>, Syntax> {
        let token = self.advance();
        let span = token.span.clone();

        // we have to clone here to avoid mut reference conflicts
        if let Token::Label(name) = token.item.clone() {
            let ast = AST::Base(Base::Label(self.intern_symbol(&name)));
            Ok(Spanned::new(ast, span))
        } else {
            unreachable!("")
        }
    }

    pub fn neg(&mut self) -> Result<Spanned<AST>, Syntax> {
        let start = self.consume_op(ResOp::Sub)?.span.clone();
        let ast = self.expression(Prec::End, false)?;
        let end = ast.span.clone();

        return Ok(
            Spanned::new(AST::Base(Base::ffi("neg", ast)),
            Span::combine(&start, &end))
        );
    }

    // Infix:

    /// Parses an assignment, associates right.
    pub fn assign(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let left_span = left.span.clone();
        let pattern = left.map(Pattern::try_from)
            .map_err(|e| Syntax::error(&e, &left_span))?;

        self.consume_op(ResOp::Assign)?;
        let expression = self.expression(Prec::Assign, false)?;
        let combined   = Span::combine(&pattern.span, &expression.span);
        Ok(Spanned::new(AST::Base(Base::assign(pattern, expression)), combined))
    }

    /// Parses a lambda definition, associates right.
    pub fn lambda(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let left_span = left.span.clone();
        let pattern = left.map(Pattern::try_from)
            .map_err(|e| Syntax::error(&e, &left_span))?;

        self.consume_op(ResOp::Lambda)?;
        let expression = self.expression(Prec::Lambda, false)?;
        let combined   = Span::combine(&pattern.span, &expression.span);
        Ok(Spanned::new(AST::Lambda(Lambda::new(pattern, expression)), combined))
    }

    // TODO: trailing comma must be grouped
    /// Parses a pair operator, i.e. the comma used to build tuples: `a, b, c`.
    pub fn pair(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let left_span = left.span.clone();
        self.consume_op(ResOp::Pair)?;

        let mut tuple = match left.item {
            AST::Base(Base::Tuple(t)) => t,
            _ => vec![left],
        };

        let index = self.index;
        let span = if let Ok(item) = self.expression(Prec::Pair.left(), false) {
            let combined = Span::combine(&left_span, &item.span);
            tuple.push(item);
            combined
        } else {
            // restore parser to location right after trailing comma
            self.index = index;
            left_span
        };

        return Ok(Spanned::new(AST::Base(Base::Tuple(tuple)), span));
    }

    /// Parses a function comp, i.e. `a . b`
    pub fn compose(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        self.consume_op(ResOp::Compose)?;
        let right = self.expression(Prec::Compose.left(), false)?;
        let combined = Span::combine(&left.span, &right.span);
        return Ok(Spanned::new(AST::Sugar(Sugar::comp(left, right)), combined));
    }

    // TODO: names must be full qualified paths.

    /// Parses a left-associative binary operator.
    /// Note that this method does not automatically associate left,
    /// So when parsing a binop that is left-associative, like addition,
    /// make sure to associate left.
    fn binop(
        &mut self,
        // op: Token,
        prec: Prec,
        name: &str,
        left: Spanned<AST>
    ) -> Result<Spanned<AST>, Syntax> {
        // self.consume(op)?;
        self.advance();
        let right = self.expression(prec, false)?;
        let combined = Span::combine(&left.span, &right.span);

        let arguments = Spanned::new(AST::Base(Base::Tuple(vec![left, right])), combined.clone());
        return Ok(Spanned::new(AST::Base(Base::ffi(name, arguments)), combined));
    }

    /// Parses a function call.
    /// Function calls are a bit magical,
    /// because they're just a series of expressions.
    /// There's a bit of magic involved -
    /// we interpret anything that isn't an operator as a function call operator.
    /// Then pull a fast one and not parse it like an operator at all.
    pub fn call(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let argument = self.expression(Prec::Call.left(), false)?;
        let combined = Span::combine(&left.span, &argument.span);

        let mut form = match left.item {
            AST::Sugar(Sugar::Form(f)) => f,
            _ => vec![left],
        };

        form.push(argument);
        return Ok(Spanned::new(AST::Sugar(Sugar::Form(form)), combined));
    }
}

#[cfg(test)]
mod test {
    use crate::common::source::Source;
    use super::*;

    #[test]
    pub fn empty() {
        let source = Source::source("");
        let ast = ThinModule::thin(source).lower().unwrap().lower();
        let result = Module::new(Spanned::new(AST::Base(Base::Block(vec![])), Span::empty()), 0);
        assert_eq!(ast, Ok(result));
    }

    // TODO: fuzzing
}
