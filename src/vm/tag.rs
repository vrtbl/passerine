use std::{
    mem,
    f64,
    fmt::{Formatter, Debug, Error},
};

// TODO: implement stack Frame

use crate::common::data::Data;

/// Tagged implements Nan-tagging around the `Data` enum.
/// In essence, it's possible to exploit the representation of f64 NaNs
/// to store pointers to other datatypes.
/// When layed out, this is what the bit-level representation of a
/// double-precision floating-point number looks like.
/// Below is the bit-level layout of a tagged NaN.
/// ```plain
/// SExponent---QIMantissa------------------------------------------
/// PNaN--------11D-Payload---------------------------------------TT
/// ```
/// Where `S` is sign, `Q` is quiet flag, `I` is Intel’s “QNan Floating-Point Indefinite”,
/// `P` is pointer flag, `D` is Data Tag (should always be 1), `T` is Tag.
/// We have 2 tag bits, 4 values: 00 is unit '()', 10 is false, 11 is true,
/// but this might change if I figure out what to do with them
/// NOTE: maybe add tag bit for 'unit'
/// NOTE: implementation modeled after:
/// https://github.com/rpjohnst/dejavu/blob/master/gml/src/vm/value.rs
/// and the Optimization chapter from Crafting Interpreters
/// Thank you!
pub struct Tagged(u64);

const QNAN:   u64 = 0x7ffe_0000_0000_0000;
const P_FLAG: u64 = 0x8000_0000_0000_0000;
const P_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;
const U_FLAG: u64 = 0x0000_0000_0000_0000;
const F_FLAG: u64 = 0x0000_0000_0000_0010;
const T_FLAG: u64 = 0x0000_0000_0000_0011;

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

            // on the heap
            // TODO: layout to make sure pointer is the right size when boxing
            other => Tagged(P_FLAG | QNAN | (P_MASK & (Box::into_raw(Box::new(other))) as u64)),
        }
    }

    // TODO: encode frame in tag itself; a frame is not data
    pub fn frame() -> Tagged {
        Tagged::new(Data::Frame)
    }

    // TODO: use deref trait
    // Can't for not because of E0515 caused by &Data result
    /// Unwrapps a tagged number into the appropriate datatype.
    pub fn data(self) -> Data {
        // This function drops the data upon unpack, resulting in a double-free
        let Tagged(bits) = self;

        match bits {
            n if (n & QNAN) != QNAN   => Data::Real(f64::from_bits(n)),
            u if u == (QNAN | U_FLAG) => Data::Unit,
            f if f == (QNAN | F_FLAG) => Data::Boolean(false),
            t if t == (QNAN | T_FLAG) => Data::Boolean(true),
            p if (p & P_FLAG) == P_FLAG => {
                // TODO: Not sure if this will work correctly...
                // Might need to have someone else look over it
                unsafe {
                    let ptr = (bits & P_MASK) as *mut Data;
                    let raw = Box::from_raw(ptr);
                    let data = raw.clone();
                    Box::into_raw(raw);
                    *data
                }
            },
            _ => unreachable!("Corrupted tagged data"),
        }
    }
}

// TODO: verify this works as intended
impl Drop for Tagged {
    fn drop(&mut self) {
        println!("Dropping!");
        let Tagged(bits) = &self;
        let pointer = P_FLAG | QNAN;

        match *bits {
            p if (pointer & p) == pointer => unsafe {
                // this should drop the data the tag points to as well
                println!("yeet");
                mem::drop(*Box::from_raw((bits & P_MASK) as *mut Data));
                println!("yote");
            },
            _ => (),
        }
        println!("done Dropping!");
    }
}

impl Debug for Tagged {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        Debug::fmt("Tagged(<Hidden>)", f)
        // TODO: causes double-free?
        // let Data
        // Debug::fmt(&self.data(), formatter)
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
                other           => {
                    println!("{:#?}", other);
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
            println!("{:?}", test);
            println!("starting pack");
            let tagged = Tagged::new(test.clone());
            println!("starting unpack");
            let untagged = tagged.data();
            println!("finished unpack");

            if let Data::Real(f) = untagged {
                if let Data::Real(n) = test {
                    if n.is_nan() {
                        assert!(f.is_nan())
                    } else {
                        assert_eq!(f, n);
                    }
                }
            } else {
                assert_eq!(test, untagged);
            }
        }
    }
}
