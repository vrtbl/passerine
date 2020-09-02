// TODO: is the is_local field really required?
// TODO: move to compiler?
/// Work-in-progress, not stable yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Captured {
    pub index: usize,
    pub is_local: bool,
}

impl Captured {
    pub fn local(index: usize) -> Captured {
        Captured { index, is_local: true }
    }

    pub fn nonlocal(index: usize) -> Captured {
        Captured { index, is_local: false }
    }
}
