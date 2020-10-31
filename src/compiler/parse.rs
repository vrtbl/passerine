use std::mem;
use crate::common::{
    span::{Span, Spanned},
    data::Data,
};

use crate::compiler::{
    syntax::Syntax,
    token::Token,
    ast::{AST, Pattern, ArgPat},
};

/// Simple function that parses a token stream into an AST.
/// Exposes the functionality of the `Parser`.
pub fn parse(tokens: Vec<Spanned<Token>>) -> Result<Spanned<AST>, Syntax> {
    let mut parser = Parser::new(tokens);
    let ast = parser.body(Token::End)?;
    parser.consume(Token::End)?;
    return Ok(Spanned::new(ast, Span::empty()));
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
    Lambda,
    Call,
    End,
}

impl Prec {
    /// Increments precedence level to cause the
    /// parser to associate infix operators to the left.
    /// For example, addition is left-associated:
    /// ```build
    /// Prec::Addition.associate_left()
    /// ```
    /// By default, the parser accociates right.
    pub fn associate_left(&self) -> Prec {
        if let Prec::End = self { panic!("Can not associate further left") }
        return unsafe { mem::transmute(self.clone() as u8 + 1) };
    }
}

/// Constructs an `AST` from a token stream.
/// Note that this struct should not be controlled manually,
/// use the `parse` function instead.
#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    index:  usize,
}

impl Parser {
    /// Create a new `parser`.
    pub fn new(tokens: Vec<Spanned<Token>>) -> Parser {
        Parser { tokens, index: 0 }
    }

    // Cookie Monster's Helper Functions:

    // NOTE: Maybe don't return bool?
    /// Consumes all seperator tokens, returning whether there were any.
    pub fn sep(&mut self) -> bool {
        if self.tokens[self.index].item != Token::Sep { false } else {
            while self.tokens[self.index].item == Token::Sep {
                self.index += 1;
            };
            true
        }
    }

    /// Returns the current token then advances the parser.
    pub fn advance(&mut self) -> &Spanned<Token> {
        self.index += 1;
        &self.tokens[self.index - 1]
    }

    /// Returns the first token.
    pub fn current(&self) -> &Spanned<Token> {
        &self.tokens[self.index]
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
            token.span.clone()
        )
    }

    /// Consumes a specific token then advances the parser.
    /// Can be used to consume Sep tokens, which are normally skipped.
    pub fn consume(&mut self, token: Token) -> Result<&Spanned<Token>, Syntax> {
        self.index += 1;
        let current = &self.tokens[self.index - 1];
        if current.item != token {
            self.index -= 1;
            Err(Syntax::error(&format!("Expected {}, found {}", token, current.item), current.span.clone()))
        } else {
            Ok(current)
        }
    }

    // Core Pratt Parser:

    /// Looks at the current token and parses an infix expression
    pub fn rule_prefix(&mut self) -> Result<Spanned<AST>, Syntax> {
        match self.current().item {
            Token::End         => Ok(Spanned::new(AST::Block(vec![]), Span::empty())),

            Token::Syntax      => self.syntax(),
            Token::OpenParen   => self.group(),
            Token::OpenBracket => self.block(),
            Token::Symbol      => self.symbol(),
            Token::Print       => self.print(),
            Token::Label       => self.label(),
            Token::Keyword(_)  => self.keyword(),

            Token::Unit
            | Token::Number(_)
            | Token::String(_)
            | Token::Boolean(_) => self.literal(),

            Token::Sep => Err(self.unexpected()),
             _ => Err(Syntax::error("Expected an expression", self.current().span.clone())),
        }
    }

    /// Looks at the current token and parses the right side of any infix expressions.
    pub fn rule_infix(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        match self.current().item {
            Token::Assign => self.assign(left),
            Token::Lambda => self.lambda(left),

            Token::End => Err(self.unexpected()),
            Token::Sep => Err(self.unexpected()),

            _ => self.call(left),
        }
    }

    /// Looks at the current operator token and determines the precedence
    pub fn prec(&mut self) -> Result<Prec, Syntax> {
        let prec = match self.current().item {
            Token::Assign => Prec::Assign,
            Token::Lambda => Prec::Lambda,

              Token::End
            | Token::Sep
            | Token::CloseParen
            | Token::CloseBracket => Prec::End,

            // prefix rules
              Token::OpenParen
            | Token::OpenBracket
            | Token::Unit
            | Token::Syntax
            | Token::Print
            | Token::Symbol
            | Token::Keyword(_)
            | Token::Label
            | Token::Number(_)
            | Token::String(_)
            | Token::Boolean(_) => Prec::Call,
        };
        Ok(prec)
    }

    /// Uses some pratt parser magic to parse an expression.
    /// It's essentially a fold-left over tokens
    /// based on the precedence and content.
    /// Cool stuff.
    pub fn expression(&mut self, prec: Prec) -> Result<Spanned<AST>, Syntax> {
        let mut left = self.rule_prefix()?;

        while self.prec()? >= prec
           && self.prec()? != Prec::End
        {
            left = self.rule_infix(left)?;
        }

        return Ok(left);
    }

    // Rule Definitions:

    // Prefix:

    /// Constructs an AST for a symbol.
    pub fn symbol(&mut self) -> Result<Spanned<AST>, Syntax> {
        let symbol = self.consume(Token::Symbol)?;
        Ok(Spanned::new(AST::Symbol(symbol.span.contents()), symbol.span.clone()))
    }

    /// Parses a keyword.
    /// Note that this is wrapped in a Pattern node.
    pub fn keyword(&mut self) -> Result<Spanned<AST>, Syntax> {
        if let Spanned { item: Token::Keyword(name), span } = self.advance() {
            let wrapped = AST::ArgPat(ArgPat::Keyword(name.clone()));
            Ok(Spanned::new(wrapped, span.clone()))
        } else {
            unreachable!("Expected a keyword");
        }
    }

    /// Constructs the AST for a literal, such as a number or string.
    pub fn literal(&mut self) -> Result<Spanned<AST>, Syntax> {
        let Spanned { item: token, span } = self.advance();

        let leaf = match token {
            Token::Unit       => AST::Data(Data::Unit),
            Token::Number(n)  => AST::Data(n.clone()),
            Token::String(s)  => AST::Data(s.clone()),
            Token::Boolean(b) => AST::Data(b.clone()),
            unexpected => return Err(Syntax::error(
                &format!("Expected a literal, found {}", unexpected),
                span.clone()
            )),
        };

        Result::Ok(Spanned::new(leaf, span.clone()))
    }

    /// Constructs the ast for a group,
    /// i.e. an expression between parenthesis.
    pub fn group(&mut self) -> Result<Spanned<AST>, Syntax> {
        let start = self.consume(Token::OpenParen)?.span.clone();
        let ast   = self.expression(Prec::None.associate_left())?.item;
        let end   = self.consume(Token::CloseParen)?.span.clone();
        Ok(Spanned::new(ast, Span::combine(&start, &end)))
    }

    /// Parses the body of a block.
    /// A block is one or more expressions, separated by separators.
    /// This is more of a helper function, as it serves as both the
    /// parser entrypoint while still being recursively nestable.
    pub fn body(&mut self, end: Token) -> Result<AST, Syntax> {
        let mut expressions = vec![];

        while self.skip().item != end {
            let ast = self.expression(Prec::None)?;
            expressions.push(ast);
            if let Err(_) = self.consume(Token::Sep) {
                break;
            }
        }

        return Ok(AST::Block(expressions));
    }

    /// Parse a block as an expression,
    /// Building the appropriate `AST`.
    /// Just a body between curlies.
    pub fn block(&mut self) -> Result<Spanned<AST>, Syntax> {
        let start = self.consume(Token::OpenBracket)?.span.clone();
        let ast = self.body(Token::CloseBracket)?;
        let end = self.consume(Token::CloseBracket)?.span.clone();
        return Ok(Spanned::new(ast, Span::combine(&start, &end)));
    }

    // TODO: unwrap from outside in to prevent nesting
    /// Parse a macro definition.
    /// `syntax`, followed by a pattern, followed by a `block`
    pub fn syntax(&mut self) -> Result<Spanned<AST>, Syntax> {
        let start = self.consume(Token::Syntax)?.span.clone();
        let mut after = self.expression(Prec::Call)?;

        let mut form = match after.item {
            AST::Form(p) => p,
            _ => return Err(Syntax::error(
                "Expected a pattern and a block after 'syntax'",
                after.span,
            )),
        };

        let last = form.pop().unwrap();
        after.item = AST::Form(form);
        let block = match (last.item, last.span) {
            (b @ AST::Block(_), s) => Spanned::new(b, s),
            _ => return Err(Syntax::error(
                "Expected a block after the syntax pattern",
                after.span,
            )),
        };

        let mut arg_pat = Parser::arg_pat(after)?;

        let span = Span::join(vec![
            start,
            arg_pat.span.clone(),
            block.span.clone()
        ]);

        return Ok(Spanned::new(AST::syntax(arg_pat, block), span));
    }

    /// Parse a print statement.
    /// A print statement takes the form `print <expression>`
    /// Where expression is exactly one expression
    /// Note that this is just a temporaty workaround;
    /// Once the FFI is solidified, printing will be a function like any other.
    pub fn print(&mut self) -> Result<Spanned<AST>, Syntax> {
        let start = self.consume(Token::Print)?.span.clone();
        let ast = self.expression(Prec::Call)?;
        let end = ast.span.clone();
        return Ok(Spanned::new(
            AST::Print(Box::new(ast)),
            Span::combine(&start, &end),
        ))
    }

    /// Parse a label.
    /// A label takes the form of `<Label> <expression>`
    pub fn label(&mut self) -> Result<Spanned<AST>, Syntax> {
        let start = self.consume(Token::Label)?.span.clone();
        let ast = self.expression(Prec::End)?;
        let end = ast.span.clone();
        return Ok(Spanned::new(
            AST::Label(start.contents(), Box::new(ast)),
            Span::combine(&start, &end),
        ));
    }

    // Infix:

    pub fn pattern(ast: Spanned<AST>) -> Result<Spanned<Pattern>, Syntax> {
        let item = match ast.item {
            AST::Symbol(s) => Pattern::Symbol(s),
            AST::Data(d) => Pattern::Data(d),
            AST::Label(k, a) => Pattern::Label(k, Box::new(Parser::pattern(*a)?)),
            AST::Pattern(p) => p,
            AST::Form(f) if f.len() == 1 => { return Parser::pattern(f[0].clone()) },
            _ => Err(Syntax::error("Unexpected construct inside pattern", ast.span.clone()))?,
        };

        return Ok(Spanned::new(item, ast.span));
    }

    pub fn arg_pat(ast: Spanned<AST>) -> Result<Spanned<ArgPat>, Syntax> {
        let item = match ast.item {
            AST::Symbol(s) => ArgPat::Symbol(s),
            AST::ArgPat(p) => p,
            AST::Form(f) => {
                let mut mapped = vec![];
                for a in f { mapped.push(Parser::arg_pat(a)?); }
                ArgPat::Group(mapped)
            }
            _ => Err(Syntax::error(
                "Unexpected construct inside argument pattern",
                ast.span.clone()
            ))?,
        };

        return Ok(Spanned::new(item, ast.span));
    }

    // TODO: assign and lambda are similar... combine?

    /// Parses an assignment, associates right.
    pub fn assign(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let pattern = Parser::pattern(left)?;

        self.consume(Token::Assign)?;
        let expression = self.expression(Prec::Assign)?;
        let combined   = Span::combine(&pattern.span, &expression.span);
        Result::Ok(Spanned::new(AST::assign(pattern, expression), combined))
    }

    /// Parses a lambda definition, associates right.
    pub fn lambda(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let pattern = Parser::pattern(left)?;

        self.consume(Token::Lambda)?;
        let expression = self.expression(Prec::Lambda)?;
        let combined   = Span::combine(&pattern.span, &expression.span);
        Result::Ok(Spanned::new(AST::lambda(pattern, expression), combined))
    }

    /// Parses a function call.
    /// Function calls are a bit magical,
    /// because they're just a series of expressions.
    /// There's a bit of magic involved -
    /// we interpret anything that isn't an operator as a function call operator.
    /// Then pull a fast one and not parse it like an operator at all.
    pub fn call(&mut self, left: Spanned<AST>) -> Result<Spanned<AST>, Syntax> {
        let argument = self.expression(Prec::Call.associate_left())?;
        let combined = Span::combine(&left.span, &argument.span);

        let mut form = match left.item {
            AST::Form(f) => f,
            _ => vec![left],
        };

        form.push(argument);
        return Ok(Spanned::new(AST::Form(form), combined));
    }
}

#[cfg(test)]
mod test {
    use crate::common::{
        data::Data,
        source::Source
    };

    use crate::compiler::lex::lex;
    use super::*;

    #[test]
    pub fn empty() {
        let source = Source::source("");
        let ast = parse(lex(source.clone()).unwrap()).unwrap();
        assert_eq!(ast, Spanned::new(AST::Block(vec![]), Span::empty()));
    }

    #[test]
    pub fn literal() {
        let source = Source::source("x = 55.0");
        let ast = parse(lex(source.clone()).unwrap()).unwrap();
        assert_eq!(
            ast,
            Spanned::new(
                AST::Block(
                    vec![
                        Spanned::new(
                            AST::assign(
                                Spanned::new(Pattern::Symbol("x".to_string()), Span::new(&source, 0, 1)),
                                Spanned::new(
                                    AST::Data(Data::Real(55.0)),
                                    Span::new(&source, 4, 4),
                                ),
                            ),
                            Span::new(&source, 0, 8),
                        )
                    ]
                ),
                Span::empty(),
            )
        );
    }

    #[test]
    pub fn lambda() {
        let source = Source::source("x = y -> 3.141592");
        let ast = parse(lex(source.clone()).unwrap()).unwrap();
        // println!("{:#?}", ast);
        assert_eq!(
            ast,
            Spanned::new(
                AST::Block(
                    vec![
                        Spanned::new(
                            AST::assign(
                                Spanned::new(Pattern::Symbol("x".to_string()), Span::new(&source, 0, 1)),
                                Spanned::new(
                                    AST::lambda(
                                        Spanned::new(Pattern::Symbol("y".to_string()), Span::new(&source, 4, 1)),
                                        Spanned::new(
                                            AST::Data(Data::Real(3.141592)),
                                            Span::new(&source, 9, 8),
                                        ),
                                    ),
                                    Span::new(&source, 4, 13),
                                ),
                            ),
                            Span::new(&source, 0, 17),
                        ),
                    ],
                ),
                Span::empty(),
            )
        );
    }
}
