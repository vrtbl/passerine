use std::{
    f64,
    fmt::{
        Debug,
        Error,
        Formatter,
    },
    mem,
};

use crate::vm::{
    data::Data,
    slot::Slot,
};

// TODO: add fallback for 32-bit systems and so on.
/// `Tagged` implements Nan-tagging around the `Data` enum.
/// In essence, it's possible to exploit the representation of `f64` NaNs
/// to store pointers to other datatypes.
///
/// When laid out, this is what the bit-level representation of a
/// double-precision floating-point number looks like:
/// ```plain
/// SExponent---QIMantissa------------------------------------------
/// PNaN--------11D-Payload-------------------------------------...T
/// ```
/// Where `S` is sign, `Q` is quiet flag, `I` is Intel’s "QNan Floating-Point
/// Indefinite"; `P` is pointer flag, `D` is Data Tag (should always be 1), `T`
/// is Tag.
///
/// By exploiting this fact, assuming a 64-bit system,
/// each item on the stack only takes up a machine word.
/// This differs from having a stack of `Box`'d `Data`,
/// because small items, like booleans, stack frames, etc.
/// can be encoded directly into the tag
/// rather than having to follow a pointer.
/// It also keeps math fast for f64s, as a simple check and transmutation
/// is all that's needed to reinterpret the bits as a valid number.
///
/// > NOTE: implementation modeled after:
/// >
/// > - [rpjohnst/dejavu](https://github.com/rpjohnst/dejavu/blob/master/gml/src/vm/value.rs),
/// > - and the Optimization chapter from Crafting Interpreters.
/// >
/// > Thank you!
pub struct Tagged(u64);

const QNAN: u64 = 0x7ffe_0000_0000_0000;
const P_FLAG: u64 = 0x8000_0000_0000_0000;
const P_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;
const S_FLAG: u64 = 0x0000_0000_0000_0000; // stack frame
const U_FLAG: u64 = 0x0000_0000_0000_0001; // unit
const F_FLAG: u64 = 0x0000_0000_0000_0002; // false
const T_FLAG: u64 = 0x0000_0000_0000_0003; // true
const N_FLAG: u64 = 0x0000_0000_0000_0004; // not initialized

impl Tagged {
    /// Wraps `Data` to create a new tagged pointer.
    pub fn new(slot: Slot) -> Tagged {
        match slot {
            // Float
            Slot::Data(Data::Float(f)) => Tagged(f.to_bits()),
            // Unit
            Slot::Data(Data::Unit) => Tagged(QNAN | U_FLAG),
            // True and false
            Slot::Data(Data::Boolean(false)) => Tagged(QNAN | F_FLAG),
            Slot::Data(Data::Boolean(true)) => Tagged(QNAN | T_FLAG),
            // Stack frame
            Slot::Frame => Tagged(QNAN | S_FLAG),
            // Not Initialized
            Slot::Data(Data::NotInit) => Tagged(QNAN | N_FLAG),

            // on the heap
            // TODO: layout to make sure pointer is the right size when boxing
            other @ Slot::Data(_) | other @ Slot::Suspend { .. } => Tagged(
                P_FLAG
                    | QNAN
                    | (P_MASK & (Box::into_raw(Box::new(other))) as u64),
            ),
        }
    }

    // TODO: encode frame in tag itself; a frame is not data
    /// Creates a new stack frame.
    #[inline]
    pub fn frame() -> Tagged { Tagged::new(Slot::Frame) }

    /// Shortcut for creating a new `Tagged(Slot::NotInit)`.
    #[inline]
    pub fn not_init() -> Tagged { Tagged::new(Slot::Data(Data::NotInit)) }

    /// Returns the underlying `Data` (or a pointer to that `Data`).
    /// Unpacks the encoding used by [`Tagged`].
    ///
    /// `dereference` will be called with a valid raw pointer to a [`Box<Slot>`]
    ///
    /// The aliasing requirements on the source `Tagged` must be followed on
    /// this pointer.
    ///
    /// If the caller moves out of this pointer, the original [`Tagged`] must
    /// not be dropped
    fn extract(&self, dereference: impl FnOnce(*mut Slot) -> Slot) -> Slot {
        match self.0 {
            n if (n & QNAN) != QNAN => {
                Slot::Data(Data::Float(f64::from_bits(n)))
            },
            u if u == (QNAN | U_FLAG) => Slot::Data(Data::Unit),
            f if f == (QNAN | F_FLAG) => Slot::Data(Data::Boolean(false)),
            t if t == (QNAN | T_FLAG) => Slot::Data(Data::Boolean(true)),
            s if s == (QNAN | S_FLAG) => Slot::Frame,
            n if n == (QNAN | N_FLAG) => Slot::Data(Data::NotInit),
            p if (p & P_FLAG) == P_FLAG => {
                dereference((p & P_MASK) as *mut Slot)
            },
            _ => unreachable!("Corrupted tagged data"),
        }
    }

    /// Unwraps a tagged number into the appropriate datatype,
    /// consuming the tagged number.
    pub fn slot(self) -> Slot {
        let this = mem::ManuallyDrop::new(self);
        this.extract(|p| *unsafe {
            // Safety: We own `self` and the `Tagged` has been forgotten
            Box::from_raw(p)
        })
    }

    /// Deeply copies some `Tagged` data.
    pub fn copy(&self) -> Slot {
        self.extract(|p| {
            unsafe {
                // Safety: We have a shared borrow of `self`
                &*p
            }
            .clone()
        })
    }
}

impl Drop for Tagged {
    fn drop(&mut self) {
        self.extract(|p| *unsafe {
            // Safety: self will not be used again, so the contents can be
            // consumed
            Box::from_raw(p)
        });
    }
}

impl Debug for Tagged {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Tagged({:?})", self.copy())
    }
}

impl From<Tagged> for u64 {
    /// Unwraps a tagged pointer into the literal representation for debugging.
    fn from(tagged: Tagged) -> Self { tagged.0 }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn floats_eq() {
        let positive = 478_329.0;
        let negative = -231.0;
        let nan = f64::NAN;
        let neg_inf = f64::NEG_INFINITY;

        for n in &[positive, negative, nan, neg_inf] {
            let data = Data::Float(*n);
            let wrapped = Tagged::new(Slot::Data(data));
            match wrapped.copy().data() {
                Data::Float(f) if f.is_nan() => assert!(n.is_nan()),
                Data::Float(f) => assert_eq!(*n, f),
                _ => panic!("Didn't unwrap to a real"),
            }
        }
    }

    #[test]
    fn bool_and_back() {
        assert_eq!(
            Data::Boolean(true),
            Tagged::new(Slot::Data(Data::Boolean(true))).copy().data()
        );
        assert_eq!(
            Data::Boolean(false),
            Tagged::new(Slot::Data(Data::Boolean(false))).copy().data()
        );
    }

    #[test]
    fn unit() {
        assert_eq!(
            Data::Unit,
            Tagged::new(Slot::Data(Data::Unit)).copy().data()
        );
    }

    #[test]
    fn size() {
        let data_size = mem::size_of::<Data>();
        let tag_size = mem::size_of::<Tagged>();

        println!("Data size: {} bytes", data_size);
        println!("Tagged size: {} bytes", tag_size);

        // Tag == u64 == f64 == 64
        // If the tag is larger than the data, we're doing something wrong
        assert_eq!(tag_size, mem::size_of::<f64>());
        assert!(tag_size < data_size);
    }

    #[test]
    fn string_pointer() {
        let s = "I just lost the game".to_string();
        let three = "Elongated Muskrat".to_string();
        let x = "It's kind of a dead giveaway, isn't it?".to_string();

        for item in &[s, three, x] {
            let data = Data::String(item.clone());
            let wrapped = Tagged::new(Slot::Data(data));
            // println!("{:#b}", u64::from(wrapped));
            match wrapped.copy().data() {
                Data::String(s) => {
                    assert_eq!(item, &s)
                },
                _ => {
                    // println!("{:#b}", u64::from(wrapped));
                    panic!("Didn't unwrap to a string");
                },
            }
        }
    }

    #[test]
    fn other_tests_eq() {
        let tests = vec![
            Data::Float(f64::consts::PI),
            Data::Float(-2.12),
            Data::Float(2.5E10),
            Data::Float(2.5e10),
            Data::Float(2.5E-10),
            Data::Float(0.5),
            Data::Float(f64::MAX),
            Data::Float(f64::MIN),
            Data::Float(f64::INFINITY),
            Data::Float(f64::NEG_INFINITY),
            Data::Float(f64::NAN),
            Data::Boolean(true),
            Data::Boolean(false),
            Data::Unit,
            Data::String("Hello, World!".to_string()),
            Data::String("".to_string()),
            Data::String("Whoop 😋".to_string()),
        ];

        for test in tests {
            // println!("test: {:?}", test);
            let tagged = Tagged::new(Slot::Data(test.clone()));
            // println!("tagged: {:?}", tagged);
            let untagged = tagged.copy().data();
            // println!("untagged: {:?}", untagged);
            // println!("---");

            if let Data::Float(f) = untagged {
                if let Data::Float(n) = test {
                    if n.is_nan() {
                        assert!(f.is_nan())
                    } else {
                        assert_eq!(test, Data::Float(n));
                    }
                }
            } else {
                assert_eq!(test, untagged);
            }
        }
    }

    #[test]
    fn no_leak_round() {
        // TODO: check memory was freed properly
        let location = "This is a string".to_string();

        // drop dereferenced data
        let tagged = Tagged::new(Slot::Data(Data::String(location.clone())));
        let pointer = tagged.0 & P_MASK;
        let untagged = tagged.copy().data();
        // println!("-- Casting...");
        let data = unsafe { Box::from_raw(pointer as *mut Data) };
        // println!("before drop: {:?}", data);
        mem::forget(data);
        mem::drop(untagged);
        // println!("after drop: {:?}", data);
    }

    #[test]
    fn no_leak_tagged() {
        let location = "This is a string".to_string();

        // drop tagged data
        let tagged = Tagged::new(Slot::Data(Data::String(location.clone())));
        let pointer = tagged.0 & P_MASK;
        let data = unsafe { Box::from_raw(pointer as *mut Data) };
        // println!("-- Dropping...");
        // println!("before drop: {:?}", data);
        mem::forget(data);
        mem::drop(tagged);
        // println!("after drop: {:?}", data);
    }
}
