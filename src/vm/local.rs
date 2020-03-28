#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Local {
    name:  String, // TODO: better type
    depth: usize,
}

impl Local {
    pub fn new(name: String, depth: usize) -> Local {
        Local {
            name:  name,
            depth: depth,
        }
    }
}
