use std::mem::transmute;
use crate::Pointer;

#[derive(Debug)]
pub struct Slot(u64);

impl Slot {
    pub unsafe fn zero() -> Slot {
        Slot(0)
    }

    pub unsafe fn from_bits(bits: u64) -> Slot {
        Slot(bits)
    }

    pub unsafe fn to_borrowed_pointer(&self) -> Pointer {
        Pointer::from_bits(self.0).borrow()
    }

    pub unsafe fn to_pointer(&self) -> Pointer {
        Pointer::from_bits(self.0)
    }

    pub unsafe fn to_u64(&self) -> u64 {
        self.0
    }

    pub unsafe fn to_i64(&self) -> i64 {
        transmute(self.0)
    }

    pub unsafe fn to_f64(&self) -> f64 {
        transmute(self.0)
    }

    /// Interpreting the slot as a bitfield,
    /// returns the bit at a given index.
    /// If we have the bifield `0b10`, which is 5 decimal:
    /// bit at position `0` is `0b0` is `false`,
    /// bit at position `1` is `0b1` is `true`,
    /// bit at positions `2..=64` are zeroed, so the rest are `false`.
    ///
    /// # Safety
    /// Caller must ensure that the slot in question is a bitfield.
    pub unsafe fn to_bool(&self, position: usize) -> bool {
        let mask = 1 << position;
        mask & self.to_u64() == mask
    }

    /// Interpreting the slot as a byte array,
    /// returns the byte at a given index.
    /// Say we have the usize `0xAAFF`, which is `43775` decimal.
    /// Byte at position `0` is `0xFF`, `1` is `0xAA`;
    /// the rest of the bits are zeroed, so positions `2..=8` would be `0x00`.
    /// # Safety
    /// Caller must ensure that the slot in question is a byte.
    pub unsafe fn to_byte(&self, position: usize) -> u8 {
        let shifted = self.to_u64() >> (position * 8);
        (shifted & 0xFF) as u8
    }
}
