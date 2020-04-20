use std::hash::{Hash, Hasher};
use std::fmt::{Debug, Error, Formatter};
use std::ops::Deref;
use std::mem;
use std::f64;

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Real(f64),
    Boolean(bool),
    String(String),
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
// PNaN--------11---Payload---------------------------------------T
// 1111111111111100011111111100111011000110011100000100000010100000
// S is sign, P is pointer flag, Q is quiet flag, I is Intel-whatever, T is Tag
// We have 1 tag bits, 2 values: 0 is false, 1 is true
// but this might change if I figure out what to do with them
const QNAN:   u64 = 0x7ffc_0000_0000_0000;
const P_FLAG: u64 = 0x8000_0000_0000_0000;
const P_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;
const F_FLAG: u64 = 0x0000_0000_0000_0000;
const T_FLAG: u64 = 0x0000_0000_0000_0001;

impl Tagged {
    pub fn from(data: Data) -> Tagged {
        match data {
            // Real
            Data::Real(f) => Tagged(unsafe { f.to_bits() }),
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
        let positive = 478329 as f64;
        let negative = -231   as f64;
        let neg_inf  = f64::NEG_INFINITY;

        for n in &[positive, negative, neg_inf] {
            let data    = Data::Real(*n);
            let wrapped = Tagged::from(data);
            match wrapped.deref() {
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
}
