pub enum Binop {
    Add,
    Sub,
    Mul,
    Div,
}

pub struct Binop {
    left: AST,
    right: AST,
}
