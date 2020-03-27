use crate::utils::annotation::Ann;
use crate::pipeline::token::{Token, AnnotatedToken};
use crate::pipeline::ast::{AST, Construct};
use crate::vm::data::Data;

// some sort of recursive descent parser, I guess
type Tokens<'a> = &'a [AnnotatedToken];
type Branch<'a> = Option<(AST, Tokens<'a>)>;

pub fn parse(tokens: Vec<AnnotatedToken>) -> Option<AST> {
    // slices are easier to work with
    // consume all preceeding seperators
    let stripped = consume(&tokens[..], Token::Sep);

    // parse the file
    if let Some((node, parsed)) = block(stripped) {
        // consume all extra seperators
        if consume(parsed, Token::Sep).len() == 0 {
            return Some(node);
        }
    }

    // if there are still tokens left, something's gone wrong.
    return None;
}

// cookie-monster's helper functions

fn consume(tokens: Tokens, token: Token) -> Tokens {
    // vaccums all leading tokens that match token
    let mut remaining = tokens;

    while remaining.len() > 0 {
        let t = remaining[0].kind;

        if t != token {
            break;
        }

        remaining = &remaining[1..];
    }

    return remaining;
}

fn longest(tokens: Tokens, rules: Vec<Box<dyn Fn(Tokens) -> Branch>>) -> Branch {
    let mut best = None;

    // I've done this twice, might move to utils bc ro3.
    for rule in rules {
        if let Some((ast, r)) = rule(tokens) {
            match best {
                // r and g are remaining. less remaining = more parsed
                // items first should be preferred
                None                              => best = Some((ast, r)),
                Some((_, g)) if r.len() < g.len() => best = Some((ast, r)),
                Some(_)                           => (),
            }
        }
    }

    return best;
}

// Tokens -> Branch

fn block(tokens: Tokens) -> Branch {
    let mut expressions = vec![];
    let mut annotations = vec![];
    let mut remaining   = tokens;

    while remaining.len() > 0 {
        match expr(remaining) {
            Some((e, r)) => {
                annotations.push(e.ann());
                expressions.push(e);
                remaining = r;
            },
            None => break,
        }

        remaining = consume(remaining, Token::Sep);
    }

    let node = AST::node(
        Construct::Block,
        Ann::span(annotations), // a parent node spans all it's children's nodes
        expressions,
    );

    return Some((node, remaining));
}

fn expr(tokens: Tokens) -> Branch {
    let rules: Vec<Box<dyn Fn(Tokens) -> Branch>> = vec![
        Box::new(|s| op(s)),
        Box::new(|s| literal(s)),
    ];

    return longest(tokens, rules);
}

fn op(tokens: Tokens) -> Branch {
    // TODO: pattern matching support!
    // get symbol being assigned too
    let (s, mut remaining) = symbol(tokens)?;

    // eat the = sign
    // TODO: make like a consume_single function
    let t = remaining.iter().next()?.kind;
    if t != Token::Assign { return None; }
    remaining = &remaining[1..];

    let (e, remaining) = expr(remaining)?;
    return Some((
        AST::node(
            Construct::Assign,
            Ann::combine(&s.ann(), &e.ann()),
            vec![s, e],
        ),
        remaining,
    ));
}

fn literal(tokens: Tokens) -> Branch {
    let rules: Vec<Box<dyn Fn(Tokens) -> Branch>> = vec![
        Box::new(|s| symbol(s)),
        Box::new(|s| boolean(s)),
    ];

    return longest(tokens, rules);
}

fn symbol(tokens: Tokens) -> Branch {
    match tokens.iter().next() {
        Some(AnnotatedToken { kind: Token::Symbol, ann }) => Some((
            AST::node(Construct::Symbol, ann.clone(), vec![]),
            &tokens[1..],
        )),
        _ => None,
    }
}

fn boolean(tokens: Tokens) -> Branch {
    match tokens.iter().next() {
        Some(AnnotatedToken { kind: Token::Boolean, ann }) => Some((
            AST::leaf(
                match ann.contents() {
                    "true"  => Data::Boolean(true),
                    "false" => Data::Boolean(false),
                    _ => panic!("Lexer classified token as boolean, boolean not found!")
                },
                ann.clone(),
            ),
            &tokens[1..],
        )),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use crate::pipes::lex::lex;
    use super::*;

    #[test]
    fn assignment() {
        // who knew so little could mean so much?
        // forget verbose, we should all write ast
        let source = "heck = false; naw = heck";

        // oof, I wrote this out by hand
        let result = AST::node(
            Construct::Block,
            Ann::new(source, 0, 24),
            vec![
                AST::node(
                    Construct::Assign,
                    Ann::new(source, 0, 12),
                    vec![
                        AST::leaf(Data::Symbol("heck".to_string()),  Ann::new(source, 0, 4)),
                        AST::leaf(Data::Boolean(false), Ann::new(source, 7, 5)),
                    ],
                ),
                AST::node(
                    Construct::Assign,
                    Ann::new(source, 14, 10),
                    vec![
                        AST::leaf(Data::Symbol("naw".to_string()),  Ann::new(source, 14, 3)),
                        AST::leaf(Data::Symbol("heck".to_string()), Ann::new(source, 20, 4)),
                    ],
                ),
            ],
        );

        assert_eq!(parse(lex(source).unwrap()), Some(result));
    }

    #[test]
    fn failure() {
        let source = "\n hello9 \n \n = true; ";

        assert_eq!(parse(lex(source).unwrap()), None);
    }
}
