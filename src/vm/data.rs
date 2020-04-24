use std::hash::{Hash, Hasher};
use std::fmt::{Debug, Error, Formatter};
use std::ops::Deref;
use std::mem;
use std::f64;

use crate::compiler::gen::Chunk;

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Real(f64),
    Boolean(bool),
    String(String),
    Lambda(Chunk),
    Unit, // an empty typle
    // Tuple(Vec<Data>),
    // // TODO: Hashmap?
    // // I mean, it's overkill for small things
    // // yet if people have very big records, yk.
    // Record(Vec<(Local, Data)>),
    // ArbInt(ArbInt),
}

// TODO: manually implement the equality trait
// NOTE: might have to implement partial equality as well
// NOTE: equality represents passerine equality, not rust equality

impl Eq for Data {}

// No Nan-tagging for now...
// HAHA: it's nan-tagging time
// NOTE: implementation modeled after:
// https://github.com/rpjohnst/dejavu/blob/master/gml/src/vm/value.rs
// and the Optimization chapter from Crafting Interpreters
// Thank you!
pub struct Tagged(u64);

// Double-precision floating-point format & tagged equiv.
// SExponent---QIMantissa------------------------------------------
// PNaN--------11D-Payload---------------------------------------TT
// S is sign, Q is quiet flag, I is Intel’s “QNan Floating-Point Indefinite”,
// P is pointer flag, D is Data Tag (should always be 1), T is Tag.
// We have 2 tag bits, 4 values: 00 is unit '()', 10 is false, 11 is true,
// but this might change if I figure out what to do with them
// NOTE: maybe add tag bit for 'unit'
const QNAN:   u64 = 0x7ffe_0000_0000_0000;
const P_FLAG: u64 = 0x8000_0000_0000_0000;
const P_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;
const U_FLAG: u64 = 0x0000_0000_0000_0000;
const F_FLAG: u64 = 0x0000_0000_0000_0010;
const T_FLAG: u64 = 0x0000_0000_0000_0011;

impl Tagged {
    pub fn from(data: Data) -> Tagged {
        match data {
            // Real
            Data::Real(f) => Tagged(f.to_bits()),
            // Unit
            Data::Unit => Tagged(QNAN | U_FLAG),
            // True and false
            Data::Boolean(false) => Tagged(QNAN | F_FLAG),
            Data::Boolean(true)  => Tagged(QNAN | T_FLAG),

            // on the heap
            // TODO: layout to make sure pointer is the right size when boxing
            other => Tagged(P_FLAG | QNAN | (P_MASK & (Box::into_raw(Box::new(other))) as u64)),
        }
    }

    // TODO: use deref trait
    // Can't for not because of E0515 caused by &Data result
    pub fn deref(&self) -> Data {
        let Tagged(bits) = self;

        match *bits {
            n if (n & QNAN) != QNAN   => Data::Real(f64::from_bits(n)),
            u if u == (QNAN | U_FLAG) => Data::Unit,
            f if f == (QNAN | F_FLAG) => Data::Boolean(false),
            t if t == (QNAN | T_FLAG) => Data::Boolean(true),
            p if (p & P_FLAG) == P_FLAG => {
                // TODO: Not sure if this will work correctly...
                // Might need to have someone else look over it
                let pointer = (bits & P_MASK) as *mut Data;
                unsafe { *Box::from_raw(pointer) }
            },
            _ => unreachable!("Corrupted tagged data"),
        }
    }
}

impl Drop for Tagged {
    fn drop(&mut self) {
        // this should drop the data the tag points to as well
        self.deref();
    }
}

impl Debug for Tagged {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        let data: Data = (*self).deref();
        return Debug::fmt(&data, formatter);
    }
}

impl From<Tagged> for u64 {
    fn from(tagged: Tagged) -> Self { tagged.0 }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reals_eq() {
        let positive = 478329.0;
        let negative = -231.0;
        let nan      = f64::NAN;
        let neg_inf  = f64::NEG_INFINITY;

        for n in &[positive, negative, nan, neg_inf] {
            let data    = Data::Real(*n);
            let wrapped = Tagged::from(data);
            match wrapped.deref() {
                Data::Real(f) if f.is_nan() => assert!(n.is_nan()),
                Data::Real(f) => assert_eq!(*n, f),
                _             => panic!("Didn't unwrap to a real"),
            }
        }
    }

    #[test]
    fn bool_and_back() {
        assert_eq!(Data::Boolean(true),  Tagged::from(Data::Boolean(true) ).deref());
        assert_eq!(Data::Boolean(false), Tagged::from(Data::Boolean(false)).deref());
    }

    #[test]
    fn unit() {
        assert_eq!(Data::Unit, Tagged::from(Data::Unit).deref());
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
            let wrapped = Tagged::from(data);
            // println!("{:#b}", u64::from(wrapped));
            match wrapped.deref() {
                Data::String(s) => { assert_eq!(item, &s) },
                other           => {
                    println!("{:#?}", other);
                    println!("{:#b}", u64::from(wrapped));
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
            Data::Real(f64::INFINITY),
            Data::Real(f64::NEG_INFINITY),
            Data::Real(f64::NAN),
            Data::String("Hello, World!".to_string()),
            Data::String("".to_string()),
            Data::String("Whoop 😋".to_string()),
            Data::Boolean(true),
            Data::Boolean(false),
        ];

        for test in tests {
            let data     = test;
            let tagged   = Tagged::from(data.clone());
            let untagged = tagged.deref();

            if let Data::Real(f) = untagged {
                if let Data::Real(n) = data {
                    if n.is_nan() {
                        assert!(f.is_nan())
                    } else {
                        assert_eq!(f, n);
                    }
                }
            } else {
                assert_eq!(data, untagged);
            }
        }
    }
}
