/// A tagged copy-on-write pointer to some data in a managed heap.
#[derive(Debug, Clone, Copy)]
pub struct Pointer(u64);

const OWNED:   u64 = 0x8000000000000000;
const POINTER: u64 = 0x7fffffffffffffff;

impl Pointer {
    /// Create an owned pointer from an index.
    pub(super) fn new(idx: PointerIdx) -> Pointer {
        let idx = idx.to_u64();
        assert!(idx <= POINTER);
        Pointer(OWNED | (POINTER & idx))
    }

    /// Pointer arithmetic.
    /// Maintains ownership.
    pub(super) fn add(&self, slots: u64) -> Pointer {
        let new_index: u64 = self.to_idx().to_u64() + slots;
        assert!(new_index <= POINTER);
        Pointer((self.0 & OWNED) | new_index)
    }

    /// Return the internal index of the pointer.
    pub(super) fn to_idx(&self) -> PointerIdx {
        PointerIdx(self.0 & POINTER)
    }

    /// Check whether a reference is borrowing the data it points to.
    pub fn is_borrowed(&self) -> bool {
        self.0 & OWNED == 0
    }

    /// Check whether a reference owns the data it points to.
    pub fn is_owned(&self) -> bool {
        self.0 & OWNED == OWNED
    }

    /// Demote an owned pointer to a borrowed pointer.
    pub fn borrow(&self) -> Pointer {
        Pointer(self.0 & POINTER)
    }

    pub unsafe fn from_bits(bits: u64) -> Pointer {
        Pointer(bits)
    }

    pub unsafe fn to_bits(self) -> u64 {
        self.0
    }
}

/// Used as a key in the maps.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub(super) struct PointerIdx(u64);

impl PointerIdx {
    pub(super) fn new(idx: u64) -> PointerIdx {
        PointerIdx(idx)
    }

    pub(super) fn to_usize(self) -> usize {
        self.0 as usize
    }

    pub(super) fn to_u64(self) -> u64 {
        self.0
    }
}

impl From<Pointer> for PointerIdx {
    fn from(pointer: Pointer) -> PointerIdx {
        pointer.to_idx()
    }
}

impl std::ops::Add<u64> for PointerIdx {
    type Output = PointerIdx;

    fn add(self, other: u64) -> PointerIdx {
        PointerIdx(self.0 + other)
    }
}
