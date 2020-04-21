use crate::utils::annotation::Ann;
use crate::pipeline::token::{Token, AnnToken};
use crate::pipeline::ast::{AST, Node};
use crate::vm::data::Data;
use crate::vm::local::Local;

// some sort of recursive descent parser, I guess
type Tokens<'a> = &'a [AnnToken];
type Branch<'a> = Result<(AST, Tokens<'a>), (String, Ann)>;

// TODO: better error reporting

pub fn parse(tokens: Vec<AnnToken>) -> Option<AST> {
    // parse the file
    // slices are easier to work with
    return match block(&tokens[..]) {
        // vaccum all extra seperators
        Ok((node, parsed)) => if vaccum(parsed, Token::Sep).is_empty() {
            Some(node)
        } else {
            None
        },
        // if there are still tokens left, something's gone wrong.
        // TODO: print the error with utils
        Err((s, a)) => panic!(format!("{}: {:?}", s, a)),
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

fn longest(tokens: Tokens, rules: Vec<Box<dyn Fn(Tokens) -> Branch>>) -> Branch {
    // Need to figure out how to annotate right part of string
    let mut best = Err(("S".to_string(), Ann::empty()));

    // I've done this twice, might move to utils bc ro3.
    for rule in rules {
        if let Ok((ast, r)) = rule(tokens) {
            match best {
                // r and g are remaining. less remaining = more parsed
                // items first should be preferred
                Err(_)                          => best = Ok((ast, r)),
                Ok((_, g)) if r.len() < g.len() => best = Ok((ast, r)),
                Ok(_)                           => (),
            }
        }
    }

    return best;
}

// Tokens -> Branch

fn block(tokens: Tokens) -> Branch {
    let mut expressions = vec![];
    let mut annotations = vec![];
    let mut remaining   = vaccum(tokens, Token::Sep);

    while !remaining.is_empty() {
        match expr(remaining) {
            Ok((e, r)) => {
                annotations.push(e.ann);
                expressions.push(e);
                remaining = r;
            },
            Err(_) => break,
        }

        remaining = vaccum(remaining, Token::Sep);
    }

    let ast = AST::new(
        Node::block(expressions),
        Ann::span(annotations), // a parent node spans all it's children's nodes
    );

    return Ok((ast, remaining));
}

fn expr(tokens: Tokens) -> Branch {
    let rules: Vec<Box<dyn Fn(Tokens) -> Branch>> = vec![
        Box::new(|s| scope(s)),
        Box::new(|s| op(s)),
        Box::new(|s| literal(s)),
    ];

    return longest(tokens, rules);
}

fn scope(tokens: Tokens) -> Branch {
    // TODO: bug here, panics on parsing block.

    let start      = consume(tokens, Token::OpenBracket)?;
    let (ast, end) = block(start)?;
    let remaining  = consume(end, Token::CloseBracket)?;

    return Ok((ast, remaining));
}

fn op(tokens: Tokens) -> Branch {
    // TODO: pattern matching support!
    // get symbol being assigned too
    let (s, mut remaining) = symbol(tokens)?;

    // eat the = sign
    remaining = consume(remaining, Token::Assign)?;

    let (e, remaining) = expr(remaining)?;
    let combined       = Ann::combine(&s.ann, &e.ann);

    Ok((
        AST::new(
            Node::assign(s, e),
            combined,
        ),
        remaining,
    ))
}

fn literal(tokens: Tokens) -> Branch {
    let rules: Vec<Box<dyn Fn(Tokens) -> Branch>> = vec![
        Box::new(|s| symbol(s)),
        Box::new(|s| number(s)),
        Box::new(|s| boolean(s)),
    ];

    return longest(tokens, rules);
}

fn symbol(tokens: Tokens) -> Branch {
    match tokens.iter().next() {
        Some(AnnToken { kind: Token::Symbol(l), ann }) => Ok((
            AST::new(Node::Symbol(l.clone()), *ann),
            &tokens[1..],
        )),
        Some(_) => Err(("Expected a variable".to_string(), tokens.iter().next().unwrap().ann)),
        // TODO: make Ann:new(0, 0) an Ann::empty() constructor
        None    => Err(("Unexpected EOF while parsing".to_string(), Ann::empty()))
    }
}

// TODO: for number and boolean, should compiler check for unreachable (i.e. lexer) errors?
// TODO: this pattern for literals is similar, abstractify?

fn number(tokens: Tokens) -> Branch {
    if let Some(AnnToken { kind: Token::Number(n), ann }) = tokens.iter().next() {
        Ok((AST::new(Node::Data(n.clone()), *ann), &tokens[1..]))
    } else {
        Err(("Unexpected EOF while parsing".to_string(), Ann::empty()))
    }
}

fn boolean(tokens: Tokens) -> Branch {
    if let Some(AnnToken { kind: Token::Boolean(b), ann }) = tokens.iter().next() {
        Ok((AST::new(Node::data(b.clone()), *ann), &tokens[1..]))
    } else {
        Err(("Unexpected EOF while parsing".to_string(), Ann::empty()))
    }
}

// TODO: ASTs can get really big, really fast
// have tests in external file?
#[cfg(test)]
mod test {
    use crate::compiler::lex::lex;
    use super::*;

    #[test]
    fn assignment() {
        // who knew so little could mean so much?
        // forget verbose, we should all write ~~lisp~~ ast
        let source = "heck = false; naw = heck";

        // oof, I wrote this out by hand
        let result = AST::new(
            Node::block(vec![
                AST::new(
                    Node::assign(
                        AST::new(Node::symbol(Local::new("heck".to_string())), Ann::new(0, 4)),
                        AST::new(Node::data(Data::Boolean(false)), Ann::new(7, 5)),
                    ),
                    Ann::new(0, 12),
                ),
                AST::new(
                    Node::assign(
                        AST::new(Node::Symbol(Local::new("naw".to_string())), Ann::new(14, 3)),
                        AST::new(Node::Symbol(Local::new("heck".to_string())), Ann::new(20, 4)),
                    ),
                    Ann::new(14, 10),
                ),
            ]),
            Ann::new(0, 24),
        );

        assert_eq!(parse(lex(source).unwrap()), Some(result));
    }

    #[test]
    fn failure() {
        let source = "\n hello9 \n \n = true; ";

        assert_eq!(parse(lex(source).unwrap()), None);
    }

    #[test]
    fn block() {
        // TODO: Put this bad-boy somewhere else.
        // maybe just have one test file and a huge hand-verified ast
        let source = "x = true\n{\n\ty = {x; true; false}\n\tz = false\n}";
        let parsed = parse(lex(source).unwrap());
        let result = Some(
            AST::new(
                Node::block(vec![
                    AST::new(
                        Node::assign(
                            AST::new(Node::symbol(Local::new("x".to_string())), Ann::new(0, 1)),
                            AST::new(Node::data(Data::Boolean(true)),           Ann::new(4, 4)),
                        ),
                        Ann::new(0, 8)
                    ),
                    AST::new(Node::block(
                        vec![
                            AST::new(
                                Node::assign(
                                    AST::new(Node::symbol(Local::new("y".to_string())), Ann::new(12, 1)),
                                    AST::new(
                                        Node::block(vec![
                                            AST::new(Node::symbol(Local::new("x".to_string())), Ann::new(17, 1)),
                                            AST::new(Node::data(Data::Boolean(true)),           Ann::new(20, 4)),
                                            AST::new(Node::data(Data::Boolean(false)),          Ann::new(26, 5)),
                                        ]),
                                        Ann::new(17, 14),
                                    )
                                ),
                                Ann::new(12, 19),
                            ),
                            AST::new(
                                Node::assign(
                                    AST::new(Node::symbol(Local::new("z".to_string())),Ann::new(34, 1)),
                                    AST::new(Node::data(Data::Boolean(false)), Ann::new(38, 5)),
                                ),
                                Ann::new(34, 9),
                            ),
                        ]),
                        Ann::new(12, 31),
                    ),
                ]),
                Ann::new(0, 43),
            ),
        );
        assert_eq!(parsed, result);
    }

    #[test]
    fn number() {
        let source = "number = { true; 0.0 }";
        let parsed = parse(lex(source).unwrap());
        let result = Some(
            AST::new(
                Node::block(vec![
                    AST::new(
                        Node::assign(
                            AST::new(Node::symbol(Local::new("number".to_string())), Ann::new(0, 6)),
                            AST::new(
                                Node::block(vec![
                                    AST::new(Node::data(Data::Boolean(true)), Ann::new(11, 4)),
                                    AST::new(Node::data(Data::Real(0.0)), Ann::new(17, 3)),
                                ]),
                                Ann::new(11, 9),
                            ),
                        ),
                        Ann::new(0, 20),
                    )
                ]),
                Ann::new(0, 20),
            ),
        );

        assert_eq!(parsed, result);
    }
}
