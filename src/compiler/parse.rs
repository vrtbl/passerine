use crate::utils::error::CompilerError;
use crate::utils::annotation::Ann;
use crate::pipeline::token::{Token, AnnToken};
use crate::pipeline::ast::{AST, Node};
use crate::vm::data::Data;
use crate::vm::local::Local;

// This is a recursive descent parser that builds the AST
// TODO: the 'vacuum' seems kind of cheap.

// some sort of recursive descent parser, I guess
type Tokens<'a>      = &'a [AnnToken<'a>];
type Branch<'a>      = Result<(AST<'a>, Tokens<'a>), (String, Ann<'a>)>;
type ParseResult<'a> = Result<AST<'a>, CompilerError<'a>>;
type Rule            = Box<dyn Fn(Tokens) -> Branch>;

pub fn parse<'a>(tokens: Vec<AnnToken<'a>>) -> ParseResult<'a> {
    // parse the file
    // slices are easier to work with
    match block(&tokens[..]) {
        // vaccum all extra seperators
        Ok((node, parsed)) => if vaccum(parsed, Token::Sep).is_empty()
            { Ok(node) } else { unreachable!() },
        // if there are still tokens left, something's gone wrong.
        // TODO: print the error with utils
        Err((m, a)) => Err(
            CompilerError::Syntax(&m, a),
        ),
    }
}

// cookie-monster's helper functions

// each function is responsible for vaccuming its input
fn vaccum(tokens: Tokens, token: Token) -> Tokens {
    // vaccums all leading tokens that match token
    let mut remaining = tokens;

    while !remaining.is_empty() {
        let t = &remaining[0].kind;
        if t != &token { break; }
        remaining = &remaining[1..];
    }

    return remaining;
}

fn consume(tokens: Tokens, token: Token) -> Result<Tokens, (String, Ann)> {
    let t = match tokens.iter().next() { Some(t) => t, None => return Err(("Unexpected EOF".to_string(), Ann::empty())) };
    if t.kind != token { return Err((format!("Expected {:?}, found {:?}", token, t.kind), t.ann)); }
    return Ok(&tokens[1..]);
}

fn first(tokens: Tokens, rules: Vec<Rule>) -> Branch {
    for rule in rules {
        if let Ok((ast, r)) = rule(tokens) {
            return Ok((ast, r))
        }
    }

    match tokens.iter().next() {
        Some(t) => Err(("Unexpected construct".to_string(), t.ann)),
        None    => Err(("Unexpected EOF while parsing".to_string(), Ann::empty())),
    }
}

// fn parse_op(tokens: Tokens, left: Rule, op: Token, right:Rule) -> Branch {
//     unimplemented!()
// }

// Tokens -> Branch

fn block(tokens: Tokens) -> Branch {
    let mut expressions = vec![];
    let mut annotations = vec![];
    let mut remaining   = vaccum(tokens, Token::Sep);

    while !remaining.is_empty() {
        match call(remaining) {
            Ok((e, r)) => {
                annotations.push(e.ann);
                expressions.push(e);
                remaining = r;
            },
            Err(_) => break,
        }

        // TODO: implement one-or-more, rename vaccum (which is really just a special case of zero or more)
        // expect at least one separator between statements
        // remaining = match consume(tokens, Token::Sep) { Ok(r) => r, Err(_) => break };
        remaining = vaccum(remaining, Token::Sep);
        // println!("{:?}", remaining);
    }

    // TODO: is this true? an empty program is should be valid
    // what does it make sense for an empty block to return?
    // empty blocks don't make any sense - use unit
    if annotations.is_empty() {
        return Err(("Block can't be empty, use Unit '()' instead".to_string(), Ann::empty()))
    }

    let ast = AST::new(
        Node::block(expressions),
        Ann::span(annotations), // a parent node spans all it's children's nodes
    );

    return Ok((ast, remaining));
}

fn call(tokens: Tokens) -> Branch {
    // try to eat an new expression
    // if it's successfull, nest like so:
    // previous = Call(previous, new)
    // empty    => error
    // single   => expression
    // multiple => call
    let (mut previous, mut remaining) = expr(vaccum(tokens, Token::Sep))?;

    while !remaining.is_empty() {
        match expr(remaining) {
            Ok((arg, r)) => {
                remaining = r;
                let ann = Ann::combine(&previous.ann, &arg.ann);
                previous = AST::new(Node::call(previous, arg), ann);
            },
            Err(_) => break,
        }
    }

    return Ok((previous, remaining));
}

fn expr(tokens: Tokens) -> Branch {
    let rules: Vec<Rule> = vec![
        Box::new(|s| expr_block(s)),
        Box::new(|s| expr_call(s)),
        Box::new(|s| op(s)),
        Box::new(|s| literal(s)),
    ];

    return first(tokens, rules);
}

fn expr_block(tokens: Tokens) -> Branch {
    let start      = consume(tokens, Token::OpenBracket)?;
    let (ast, end) = block(start)?;
    let remaining  = consume(end, Token::CloseBracket)?;

    return Ok((ast, remaining));
}

fn expr_call(tokens: Tokens) -> Branch {
    let start      = consume(tokens, Token::OpenParen)?;
    let (ast, end) = call(start)?;
    let remaining  = consume(end, Token::CloseParen)?;

    return Ok((ast, remaining));
}

fn op(tokens: Tokens) -> Branch {
    assign(tokens)
}

fn assign(tokens: Tokens) -> Branch {
    let rules: Vec<Rule> = vec![
        Box::new(|s| assign_assign(s)),
        Box::new(|s| lambda(s)),
    ];

    return first(tokens, rules);
}

// TODO: implement parse_op and rewrite lambda / assign

fn assign_assign(tokens: Tokens) -> Branch {
    // TODO: pattern matching support!
    // get symbol being assigned too
    let (next, mut remaining) = literal(tokens)?;
    let s = match next {
        // Destructure restucture
        AST { node: Node::Symbol(l), ann} => AST::new(Node::Symbol(l), ann),
        other => return Err(("Expected symbol for assignment".to_string(), other.ann)),
    };

    // eat the = sign
    remaining = consume(remaining, Token::Assign)?;
    let (e, remaining) = call(remaining)?;
    let combined       = Ann::combine(&s.ann, &e.ann);
    Ok((AST::new(Node::assign(s, e), combined), remaining))
}

fn lambda(tokens: Tokens) -> Branch {
    // get symbol acting as arg to function
    let (next, mut remaining) = literal(tokens)?;
    let s = match next {
        AST { node: Node::Symbol(l), ann} => AST::new(Node::Symbol(l), ann),
        other => return Err(("Expected symbol for function paramater".to_string(), other.ann)),
    };

    // eat the '->'
    remaining = consume(remaining, Token::Lambda)?;
    let (e, remaining) = call(remaining)?;
    let combined       = Ann::combine(&s.ann, &e.ann);
    Ok((AST::new(Node::lambda(s, e), combined), remaining))
}

fn literal(tokens: Tokens) -> Branch {
    if let Some(AnnToken { kind, ann }) = tokens.iter().next() {
        Ok((AST::new(
            match kind {
                Token::Symbol(l)  => Node::Symbol(l.clone()),
                Token::Number(n)  => Node::Data(n.clone()),
                Token::String(s)  => Node::Data(s.clone()),
                Token::Boolean(b) => Node::Data(b.clone()),
                _ => return Err(("Unexpected token".to_string(), *ann)),
            },
            *ann
        ), &tokens[1..]))
    } else {
        Err(("Unexpected EOF while parsing".to_string(), Ann::empty()))
    }
}

// TODO: ASTs can get really big, really fast - have tests in external file?
#[cfg(test)]
mod test {
    use crate::pipeline::source::Source;
    use crate::compiler::lex::lex;
    use super::*;

    #[test]
    fn assignment() {
        // who knew so little could mean so much?
        // forget verbose, we should all write ~~lisp~~ ast
        let source = Source::source("heck = false; naw = heck");

        // oof, I wrote this out by hand
        let result = AST::new(
            Node::block(vec![
                AST::new(
                    Node::assign(
                        AST::new(Node::symbol(Local::new("heck".to_string())), Ann::new(&source, 0, 4)),
                        AST::new(Node::data(Data::Boolean(false)), Ann::new(&source, 7, 5)),
                    ),
                    Ann::new(&source, 0, 12),
                ),
                AST::new(
                    Node::assign(
                        AST::new(Node::Symbol(Local::new("naw".to_string())), Ann::new(&source, 14, 3)),
                        AST::new(Node::Symbol(Local::new("heck".to_string())), Ann::new(&source, 20, 4)),
                    ),
                    Ann::new(&source, 14, 10),
                ),
            ]),
            Ann::new(&source, 0, 24),
        );

        assert_eq!(parse(lex(source).unwrap()), Ok(result));
    }

    #[test]
    fn failure() {
        let source = Source::source("\n hello9 = {; ");

        assert_eq!(parse(lex(source).unwrap()), None);
    }

    #[test]
    fn block() {
        // TODO: Put this bad-boy somewhere else.
        // maybe just have one test file and a huge hand-verified ast
        let source = Source::source("x = true\n{\n\ty = {x; true; false}\n\tz = false\n}");
        let parsed = parse(lex(source).unwrap());
        let result = Some(
            AST::new(
                Node::block(vec![
                    AST::new(
                        Node::assign(
                            AST::new(Node::symbol(Local::new("x".to_string())), Ann::new(&source, 0, 1)),
                            AST::new(Node::data(Data::Boolean(true)),           Ann::new(&source, 4, 4)),
                        ),
                        Ann::new(&source, 0, 8)
                    ),
                    AST::new(Node::block(
                        vec![
                            AST::new(
                                Node::assign(
                                    AST::new(Node::symbol(Local::new("y".to_string())), Ann::new(&source, 12, 1)),
                                    AST::new(
                                        Node::block(vec![
                                            AST::new(Node::symbol(Local::new("x".to_string())), Ann::new(&source, 17, 1)),
                                            AST::new(Node::data(Data::Boolean(true)),           Ann::new(&source, 20, 4)),
                                            AST::new(Node::data(Data::Boolean(false)),          Ann::new(&source, 26, 5)),
                                        ]),
                                        Ann::new(&source, 17, 14),
                                    )
                                ),
                                Ann::new(&source, 12, 19),
                            ),
                            AST::new(
                                Node::assign(
                                    AST::new(Node::symbol(Local::new("z".to_string())),Ann::new(&source, 34, 1)),
                                    AST::new(Node::data(Data::Boolean(false)), Ann::new(&source, 38, 5)),
                                ),
                                Ann::new(&source, 34, 9),
                            ),
                        ]),
                        Ann::new(&source, 12, 31),
                    ),
                ]),
                Ann::new(&source, 0, 43),
            ),
        );
        assert_eq!(parsed, result);
    }

    #[test]
    fn number() {
        let source = Source::source("number = { true; 0.0 }");
        let parsed = parse(lex(source).unwrap());
        let result = Some(
            AST::new(
                Node::block(vec![
                    AST::new(
                        Node::assign(
                            AST::new(Node::symbol(Local::new("number".to_string())), Ann::new(&source, 0, 6)),
                            AST::new(
                                Node::block(vec![
                                    AST::new(Node::data(Data::Boolean(true)), Ann::new(&source, 11, 4)),
                                    AST::new(Node::data(Data::Real(0.0)), Ann::new(&source, 17, 3)),
                                ]),
                                Ann::new(&source, 11, 9),
                            ),
                        ),
                        Ann::new(&source, 0, 20),
                    )
                ]),
                Ann::new(&source, 0, 20),
            ),
        );

        assert_eq!(parsed, result);
    }

    #[test]
    fn functions() {
        let source = Source::source("applyzero = fun -> arg -> fun arg 0.0");
        let parsed = parse(lex(source).unwrap());
        let result = Some(
            AST::new(
                Node::block(vec![
                    AST::new(
                        Node::assign(
                            AST::new(Node::symbol(Local::new("applyzero".to_string())), Ann::new(&source, 0, 9)),
                            AST::new(
                                Node::lambda(
                                    AST::new(Node::symbol(Local::new("fun".to_string())), Ann::new(&source, 12, 3)),
                                    AST::new(Node::lambda(
                                        AST::new(Node::symbol(Local::new("arg".to_string())),  Ann::new(&source, 19, 3)),
                                        AST::new(
                                            Node::call(
                                                AST::new(
                                                    Node::call(
                                                        AST::new(Node::symbol(Local::new("fun".to_string())), Ann::new(&source, 26, 3)),
                                                        AST::new(Node::symbol(Local::new("arg".to_string())), Ann::new(&source, 30, 3)),
                                                    ),
                                                    Ann::new(&source, 26, 7),
                                                ),
                                                AST::new(Node::data(Data::Real(0.0)), Ann::new(&source, 34, 3)),
                                            ),
                                            Ann::new(&source, 26, 11)
                                        )
                                    ),
                                    Ann::new(&source, 19, 18),
                                ),
                            ),
                            Ann::new(&source, 12, 25),
                        ),
                    ),
                    Ann::new(&source, 0, 37),
                )]),
                Ann::new(&source, 0, 37),
            ),
        );

        assert_eq!(parsed, result);
    }

    #[test]
    fn calling() {
        let source = Source::source("bink (bonk 0.0)");
        let parsed = parse(lex(source).unwrap());

        let result = Some(
            AST::new(
                Node::block(vec![
                    AST::new(
                        Node::call (
                            AST::new(Node::symbol(Local::new("bink".to_string())), Ann::new(&source, 0, 4)),
                            AST::new(
                                Node::call(
                                    AST::new(Node::symbol(Local::new("bonk".to_string())), Ann::new(&source, 6, 4)),
                                    AST::new(Node::data(Data::Real(0.0)), Ann::new(&source, 11, 3)),
                                ),
                                Ann::new(&source, 6, 8),
                            ),
                        ),
                        Ann::new(&source, 0, 14)
                    ),
                ]),
                Ann::new(&source, 0, 14),
            ),
        );
        assert_eq!(parsed, result);
    }
}
