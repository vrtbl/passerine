#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Local {
    name:  String, // TODO: better type
    depth: usize,
}

// For when Pattern matching is implemented
// enum Pattern {
//     Local(Local),
//     // Tuple(Vec<Pattern>),
//     // Union(Pattern),
//     // Struct(Vec<(Local, Pattern)>),
//     // Map(Vec<Data, Pattern>),
// }

impl Local {
    pub fn new(name: String, depth: usize) -> Local {
        Local { name, depth }
    }
}
