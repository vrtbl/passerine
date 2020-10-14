use std::{
    mem,
    f64,
    fmt::{Formatter, Debug, Error},
};

use crate::common::data::Data;

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
/// Where `S` is sign, `Q` is quiet flag, `I` is Intel’s "QNan Floating-Point Indefinite";
/// `P` is pointer flag, `D` is Data Tag (should always be 1), `T` is Tag.
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

const QNAN:   u64 = 0x7ffe_0000_0000_0000;
const P_FLAG: u64 = 0x8000_0000_0000_0000;
const P_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;
const S_FLAG: u64 = 0x0000_0000_0000_0000; // stack frame
const U_FLAG: u64 = 0x0000_0000_0000_0001; // unit
const F_FLAG: u64 = 0x0000_0000_0000_0002; // false
const T_FLAG: u64 = 0x0000_0000_0000_0004; // true

impl Tagged {
    /// Wraps `Data` to create a new tagged pointer.
    pub fn new(data: Data) -> Tagged {
        match data {
            // Real
            Data::Real(f) => Tagged(f.to_bits()),
            // Unit
            Data::Unit => Tagged(QNAN | U_FLAG),
            // True and false
            Data::Boolean(false) => Tagged(QNAN | F_FLAG),
            Data::Boolean(true)  => Tagged(QNAN | T_FLAG),
            // Stack frame
            Data::Frame => Tagged(QNAN | S_FLAG),

            // on the heap
            // TODO: layout to make sure pointer is the right size when boxing
            other => Tagged(P_FLAG | QNAN | (P_MASK & (Box::into_raw(Box::new(other))) as u64)),
        }
    }

    // TODO: encode frame in tag itself; a frame is not data
    /// Creates a new stack frame.
    pub fn frame() -> Tagged {
        Tagged::new(Data::Frame)
    }

    /// Returns the underlying `Data` (or a pointer to that `Data`).
    unsafe fn extract(&self) -> Result<Data, Box<Data>> {
        // println!("-- Extracting...");
        let Tagged(bits) = self;

        return match bits {
            n if (n & QNAN) != QNAN    => Ok(Data::Real(f64::from_bits(*n))),
            u if u == &(QNAN | U_FLAG) => Ok(Data::Unit),
            f if f == &(QNAN | F_FLAG) => Ok(Data::Boolean(false)),
            t if t == &(QNAN | T_FLAG) => Ok(Data::Boolean(true)),
            s if s == &(QNAN | S_FLAG) => Ok(Data::Frame),
            p if (p & P_FLAG) == P_FLAG => Err({
                // println!("{:#x}", p & P_MASK);
                // unsafe part
                Box::from_raw((bits & P_MASK) as *mut Data)
            }),
            _ => unreachable!("Corrupted tagged data"),
        }
    }

    // TODO: use deref trait
    // Can't for not because of E0515 caused by &Data result
    /// Unwrapps a tagged number into the appropriate datatype,
    /// consuming the tagged number.
    pub fn data(self) -> Data {
        // println!("-- Data...");

        let d = unsafe {
            match self.extract() {
                Ok(data) => data,
                Err(boxed) => {
                    *boxed
                }
            }
        };

        // println!("-- Forgetting...");
        mem::drop(self.0);
        mem::forget(self);
        return d;
    }

    /// Deeply copies some `Tagged` data.
    pub fn copy(&self) -> Data {
        // println!("-- Copy...");
        unsafe {
            match self.extract() {
                Ok(data) => data.to_owned(),
                Err(boxed) => {
                    let copy = boxed.clone();
                    // println!("-- Leaking...");
                    // we took ownership to clone the pointer,
                    // but we do not own the pointer,
                    // so we 'leak' it - &self still holds a reference
                    Box::leak(boxed);
                    *copy
                },
            }
        }
    }
}

impl Drop for Tagged {
    fn drop(&mut self) {
        // println!("-- Dropping...");
        // println!("{:#x}", self.0 & P_MASK);
        unsafe { mem::drop(self.extract()) };
    }
}

impl Debug for Tagged {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // println!("-- Fmt...");
        // let Tagged(bits) = &self;
        // let pointer = P_FLAG | QNAN;
        //
        // if (pointer & bits) == pointer {
        //     let item = unsafe { Box::from_raw((bits & P_MASK) as *mut Data) };
        //     write!(f, "Data {:?}", item)?;
        //     mem::forget(item);
        // } else {
        //     write!(f, "Real {}", f64::from_bits(bits.clone()))?;
        // }
        //
        // Ok(())
        // // write an exact copy of the data

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
    fn reals_eq() {
        let positive = 478_329.0;
        let negative = -231.0;
        let nan      = f64::NAN;
        let neg_inf  = f64::NEG_INFINITY;

        for n in &[positive, negative, nan, neg_inf] {
            let data    = Data::Real(*n);
            let wrapped = Tagged::new(data);
            match wrapped.data() {
                Data::Real(f) if f.is_nan() => assert!(n.is_nan()),
                Data::Real(f) => assert_eq!(*n, f),
                _             => panic!("Didn't unwrap to a real"),
            }
        }
    }

    #[test]
    fn bool_and_back() {
        assert_eq!(Data::Boolean(true),  Tagged::new(Data::Boolean(true) ).data());
        assert_eq!(Data::Boolean(false), Tagged::new(Data::Boolean(false)).data());
    }

    #[test]
    fn unit() {
        assert_eq!(Data::Unit, Tagged::new(Data::Unit).data());
    }

    #[test]
    fn size() {
        let data_size = mem::size_of::<Data>();
        let tag_size  = mem::size_of::<Tagged>();

        // Tag == u64 == f64 == 64
        // If the tag is larger than the data, we're doing something wrong
        assert_eq!(tag_size, mem::size_of::<f64>());
        assert!(tag_size < data_size);
    }

    #[test]
    fn string_pointer() {
        let s =     "I just lost the game".to_string();
        let three = "Elongated Muskrat".to_string();
        let x =     "It's kind of a dead giveaway, isn't it?".to_string();

        for item in &[s, three, x] {
            let data    = Data::String(item.clone());
            let wrapped = Tagged::new(data);
            // println!("{:#b}", u64::from(wrapped));
            match wrapped.data() {
                Data::String(s) => { assert_eq!(item, &s) },
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
            Data::Real(f64::consts::PI),
            Data::Real(-2.12),
            Data::Real(2.5E10),
            Data::Real(2.5e10),
            Data::Real(2.5E-10),
            Data::Real(0.5),
            Data::Real(f64::MAX),
            Data::Real(f64::MIN),
            Data::Real(f64::INFINITY),
            Data::Real(f64::NEG_INFINITY),
            Data::Real(f64::NAN),
            Data::Boolean(true),
            Data::Boolean(false),
            Data::Unit,
            Data::String("Hello, World!".to_string()),
            Data::String("".to_string()),
            Data::String("Whoop 😋".to_string()),
        ];

        for test in tests {
            // println!("test: {:?}", test);
            let tagged = Tagged::new(test.clone());
            // println!("tagged: {:?}", tagged);
            let untagged = tagged.data();
            // println!("untagged: {:?}", untagged);
            // println!("---");

            if let Data::Real(f) = untagged {
                if let Data::Real(n) = test {
                    if n.is_nan() {
                        assert!(f.is_nan())
                    } else {
                        assert_eq!(test, Data::Real(n));
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
        let tagged = Tagged::new(Data::String(location.clone()));
        let pointer = tagged.0 & P_MASK;
        let untagged = tagged.data();
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
        let tagged = Tagged::new(Data::String(location.clone()));
        let pointer = tagged.0 & P_MASK;
        let data = unsafe { Box::from_raw(pointer as *mut Data) };
        // println!("-- Dropping...");
        // println!("before drop: {:?}", data);
        mem::forget(data);
        mem::drop(tagged);
        // println!("after drop: {:?}", data);
    }
}
