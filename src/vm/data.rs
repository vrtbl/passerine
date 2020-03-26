// TODO: Box to make smaller?

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Data {
    // TODO: use uint for symbol, strings are slow and take up space.
    Symbol(String),
    Boolean(bool),
}
