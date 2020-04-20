use std::ops::Deref;
use std::mem;
use std::f64;

// No Nan-tagging for now.
// HAHA: it's nan-tagging time
// NOTE: implementation modeled after:
// https://github.com/rpjohnst/dejavu/blob/master/gml/src/vm/value.rs
// Thank you!
pub struct Tagged(u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Data {
    Real(f64),
    Boolean(bool),
    // String(String),
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

// Double-precision floating-point format & tagged equiv.
// SExponent---QIMantissa------------------------------------------
// PNaN--------11Payload------------------------------------------T
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
            Data::Real(f) => Tagged(unsafe {mem::transmute::<_, u64>(f)}),
            // True and false
            Data::Boolean(false) => Tagged(QNAN | F_FLAG),
            Data::Boolean(true)  => Tagged(QNAN | T_FLAG),
            // on the heap
            // TODO: layout to make sure pointer is the right size
            other => Tagged(P_FLAG | QNAN | (P_MASK & (Box::into_raw(Box::new(other))) as u64)),
        }
    }
}

impl Deref for Tagged {
    type Target = Data;

    fn deref(&self) -> &Data {
        let Tagged(bits) = self;

        match bits {
            n if (bits & QNAN)   != QNAN   => &Data::Real(f64::from_bits(*bits)),
            f if (bits & F_FLAG) == F_FLAG => &Data::Boolean(false),
            t if (bits & T_FLAG) == T_FLAG => &Data::Boolean(true),
            p if (bits & P_FLAG) == P_FLAG => {
                // TODO: Not sure if this will work correctly...
                // Might need to have someone else look over it
                let pointer = (bits & P_MASK) as *mut Data;
                &*Box::from_raw(pointer)
            },
            _ => unreachable!("Corrupted tagged data"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn relas() {
        let positive = 478329 as f64;
        let negative = -231   as f64;
        let nan      = f64::NAN;
        let neg_inf  = f64::NEG_INFINITY;

        for n in &[positive, negative, nan, neg_inf] {
            let data    = Data::Real(*n);
            let wrapped = Tagged::from(data);
            match *wrapped {
                Data::Real(f) => assert_eq!(*n, f),
                _          => panic!("Didn't unwrap to a real")
            }
        }
    }
}
