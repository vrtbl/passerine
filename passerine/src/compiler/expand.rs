use std::collections::HashMap;
use std::rc::Rc;

use crate::common::{lambda::Lambda, Spanned};
use crate::compiler::{compile_token_tree, Syntax};
use crate::construct::token::{Token, TokenTree};

pub struct Expander {
    rules: HashMap<String, Rc<Lambda>>,
}

impl Expander {
    pub fn expand(
        token_tree: Spanned<TokenTree>,
        mut rules: HashMap<String, Spanned<TokenTree>>,
    ) -> Result<Spanned<TokenTree>, Syntax> {
        let mut expander = Expander {
            rules: HashMap::new(),
        };

        for (name, rule) in rules.drain() {
            let bytecode = compile_token_tree(rule)?;
            expander.rules.insert(name, bytecode);
        }

        expander.walk(token_tree)
    }

    pub fn walk(
        &self,
        token_tree: Spanned<TokenTree>,
    ) -> Result<Spanned<TokenTree>, Syntax> {
        let Spanned {
            item: token_tree,
            span,
        } = token_tree;

        let result = match token_tree {
            TokenTree::Form(form) => self.expand_form(form)?,

            // trivial conversion
            TokenTree::Block(block) => {
                let mut new_block = Vec::new();
                for trees in block {
                    let mut new_trees = Vec::new();
                    let Spanned { item, span } = trees;
                    for tree in item.into_iter() {
                        new_trees.push(self.walk(tree)?)
                    }
                    new_block.push(Spanned::new(new_trees, span));
                }
                TokenTree::Block(new_block)
            },
            TokenTree::List(trees) => TokenTree::List(
                trees
                    .into_iter()
                    .map(|tree| self.walk(tree))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            // trivial conversion
            TokenTree::Iden(iden) => TokenTree::Iden(iden),
            TokenTree::Label(label) => TokenTree::Label(label),
            TokenTree::Op(op) => TokenTree::Op(op),
            TokenTree::Lit(lit) => TokenTree::Lit(lit),
        };

        Ok(Spanned::new(result, span))
    }

    pub fn walk_form(
        &self,
        form: Vec<Spanned<TokenTree>>,
    ) -> Result<TokenTree, Syntax> {
        Ok(TokenTree::Form(
            form.into_iter()
                .map(|tree| self.walk(tree))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    pub fn expand_form(
        &self,
        mut form: Vec<Spanned<TokenTree>>,
    ) -> Result<TokenTree, Syntax> {
        assert!(form.len() >= 2);
        let first = form.remove(0);

        if let TokenTree::Iden(iden) = first.item {
            if let Some(rule) = self.rules.get(&iden) {
                // TODO: call compiled function somehow
                let expanded = todo!();
                return expanded;
            }
        }

        self.walk_form(form)
    }
}
