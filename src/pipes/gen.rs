use crate::pipeline::ast::{AST, Operation};
use crate::pipeline::bytecode::{Bytecode, Constants, Opcode};
use crate::vm::data::Data;
use crate::utils::number::split_number;

// so, constanst table is made by walking the tree and sweeping for values
// then, a second pass walks the tree and builds the bytecode
// then, a third pass walks the tree and optimizes the bytecode
// TODO: consts and bytecode in single pass.
// TODO: annotations in bytecode

fn block(children: &Vec<AST>, constants: &mut Constants, bytecode: &mut Bytecode) {
    for child in children {
        walk(&child, constants, bytecode);
        bytecode.push(Opcode::Clear.to_byte());
    }

    // remove the last clear instruction
    bytecode.pop();
}

fn assign(children: &Vec<AST>, constants: &mut Constants, bytecode: &mut Bytecode) {
    if children.len() != 2 {
        panic!("Assignment expects 2 children")
    }

    // TODO: this if-let feels... cheap. avoid?
    let symbol = &children[0];
    let expr   = &children[1];

    // load the const symbol
    // TODO: is it redundant? check that left arm is symbol
    match symbol {
        AST::Leaf { data: d, ann: _ } => {
            // manually push to not load symbol value
            bytecode.push(Opcode::Con.to_byte());
            bytecode.append(&mut split_number(find(&d, constants)));
        },
        _ => panic!("Symbol expected, found something else")
    }

    // eval the expression
    walk(&expr, constants, bytecode);

    // save the binding
    bytecode.push(Opcode::Save.to_byte());
}

fn find(c: &Data, cs: &mut Constants) -> usize {
    match cs.iter().position(|d| d == c) {
        Some(d) => d,
        None    => { cs.push(c.clone()); cs.len() - 1 },
    }
}

fn walk(ast: &AST, constants: &mut Constants, bytecode: &mut Bytecode) {
    // push left, push right, push center
    match ast {
        AST::Leaf { data, ann: _ } => {
            bytecode.push(Opcode::Con.to_byte());
            bytecode.append(&mut split_number(find(&data, constants)));

            // variables should be loaded!
            if let Data::Symbol(_) = data {
                bytecode.push(Opcode::Load.to_byte());
            }
        },
        AST::Node { kind, ann: _, children } => match kind {
                Operation::Block  => block(&children, constants, bytecode),
                Operation::Assign => assign(&children, constants, bytecode),
        }
    }
}

pub fn gen(ast: AST) -> (Bytecode, Constants) {
    let mut constants = vec![];
    let mut bytecode = vec![];

    walk(&ast, &mut constants, &mut bytecode);
    return (bytecode, constants);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pipes::lex::lex;
    use crate::pipes::parse::parse;

    #[test]
    fn constants() {
        // TODO: flesh out as more datatypes are added
        let source = "heck = true; lol = heck; lmao = false";
        let ast    = parse(lex(source).unwrap()).unwrap();
        let result = vec![
            Data::Symbol("heck".to_string()), // NOTE: 'heck' appears twice in source but once here
            Data::Boolean(true),
            Data::Symbol("lol".to_string()),
            Data::Symbol("lmao".to_string()),
            Data::Boolean(false),
        ];

        let mut constants = vec![];
        walk(&ast, &mut constants, &mut vec![]);

        assert_eq!(constants, result);
    }

    #[test]
    fn bytecode() {
        let source = "heck = true; lol = heck; lmao = false";
        let ast    = parse(lex(source).unwrap()).unwrap();

        let (bytecode, constants) = gen(ast);
        let result = vec![0, 128, 0, 129, 1, 3, 0, 130, 0, 128, 2, 1, 3, 0, 131, 0, 132, 1];
        // con heck, con true, save, clear |                          |                  |
        // con lol, con heck, load heck, save, clear                  |                  |
        // load lmao, load false, save, clear                                            |

        assert_eq!(result, bytecode);
    }
}
