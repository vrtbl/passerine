use crate::common::{
    span::Spanned,
    data::Data,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Symbol,
    Data(Data),
    Label(Box<Spanned<Pattern>>),
}
