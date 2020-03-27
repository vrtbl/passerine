#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Local {
    name:  String, // TODO: better type
    depth: usize,
}
