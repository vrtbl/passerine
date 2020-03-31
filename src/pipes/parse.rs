use crate::utils::annotation::Ann;
use crate::pipeline::token::{Token, AnnToken};
use crate::pipeline::ast::{AST, Construct};
use crate::vm::data::Data;

// some sort of recursive descent parser, I guess
type Tokens<'a> = &'a [AnnToken];
type Branch<'a> = Result<(AST, Tokens<'a>), (String, Ann)>;

// TODO: better error reporting

pub fn parse(tokens: Vec<AnnToken>) -> Option<AST> {
    // slices are easier to work with
    // vaccum all preceeding seperators
    let stripped = vaccum(&tokens[..], Token::Sep);

    // parse the file
    return match block(stripped) {
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
    let t = match tokens.iter().next() { Some(t) => t, None => return Err(("Unexpected EOF".to_string(), Ann::new("", 0, 0))) };
    if t.kind != token { return Err((format!("Expected {:?}, found {:?}", token, t.kind), t.ann)); }
    return Ok(&tokens[1..]);
}

fn longest(tokens: Tokens, rules: Vec<Box<dyn Fn(Tokens) -> Branch>>) -> Branch {
    let mut best = Err(("S".to_string(), Ann::new("", 0, 0)));

    // I've done this twice, might move to utils bc ro3.
    for rule in rules {
        if let Ok((ast, r)) = rule(tokens) {
            match best {
                // r and g are remaining. less remaining = more parsed
                // items first should be preferred
                Err(_)                     => best = Ok((ast, r)),
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
    let mut remaining   = tokens;

    while !remaining.is_empty() {
        match expr(remaining) {
            Ok((e, r)) => {
                annotations.push(e.ann());
                expressions.push(e);
                remaining = r;
            },
            Err(_) => break,
        }

        remaining = vaccum(remaining, Token::Sep);
    }

    let node = AST::node(
        Construct::Block,
        Ann::span(annotations), // a parent node spans all it's children's nodes
        expressions,
    );

    return Ok((node, remaining));
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
    remaining = consume(remaining, Token::Assign)?;

    let (e, remaining) = expr(remaining)?;
    return Ok((
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
        Some(AnnToken { kind: Token::Symbol, ann }) => Ok((
            AST::node(Construct::Symbol, *ann, vec![]),
            &tokens[1..],
        )),
        Some(_) => Err(("Expected a variable".to_string(), tokens.iter().next().unwrap().ann)),
        None    => Err(("Unexpected EOF while parsing".to_string(), Ann::new("", 0, 0)))
    }
}

fn boolean(tokens: Tokens) -> Branch {
    match tokens.iter().next() {
        Some(AnnToken { kind: Token::Boolean, ann }) => Ok((
            AST::leaf(
                match ann.contents() {
                    "true"  => Data::Boolean(true),
                    "false" => Data::Boolean(false),
                    _ => panic!("Lexer classified token as boolean, boolean not found!")
                },
                *ann,
            ),
            &tokens[1..],
        )),
        Some(_) => Err(("Expected a boolean value".to_string(), tokens.iter().next().unwrap().ann)),
        None    => Err(("Unexpected EOF while parsing".to_string(), Ann::new("", 0, 0)))
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
                        AST::node(Construct::Symbol, Ann::new(source, 0, 4), vec![]),
                        AST::leaf(Data::Boolean(false), Ann::new(source, 7, 5)),
                    ],
                ),
                AST::node(
                    Construct::Assign,
                    Ann::new(source, 14, 10),
                    vec![
                        AST::node(Construct::Symbol,  Ann::new(source, 14, 3), vec![]),
                        AST::node(Construct::Symbol,  Ann::new(source, 20, 4), vec![]),
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
