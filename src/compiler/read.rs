use crate::compiler::syntax::Syntax;
use crate::construct::token::{TokenTree, Tokens};

pub struct Reader {
    tokens: Tokens,
    index: usize,
}

impl Reader {
    pub fn read(tokens: Tokens) -> Result<TokenTree, Syntax> {
        dbg!(tokens);
        todo!();
    }
}
