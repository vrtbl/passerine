// TODO: move to vm?

/// Splits a number into a vector of bytes.
/// Each byte in the vector is layed out as follows:
/// ```plain
/// CDDDDDDD
/// ```
/// Where C is the continue bit-flag, and d are data bits.
/// These sequences are designed to be insertable into byte streams.
/// For example, `0b10001000_00100101_00100100` would become
/// `[0b10100100, 0b11001010, 0b10100000, 0b00001000]`.
pub fn split_number(n: usize) -> Vec<u8> {
    // high bit = last byte in chain?
    // low bits = binary representation
    let mut bytes = vec![];
    let mut i     = n;
    let chunk     = 0b1000_0000;

    loop {
        let low = i % chunk; // get next 7 bits in usize
        i /= chunk; // shift right by 7

        // set high bit & append to chain
        bytes.push(if bytes.is_empty() { chunk + low } else { low } as u8);

        // like a do-while
        // makes sure a number is always pushed
        if i == 0 { break; }
    }

    // reverse chain so high bit byte is last
    bytes.reverse();
    return bytes;
}

/// This takes a stream of bytes, and builds the next number in it.
/// Note that this function tries to build a number no matter what,
/// even if the byte stream does not have a number, is empty, or ends after a continue bit is set.
pub fn build_number(bytes: Vec<u8>) -> (usize, usize) /* (index, eaten) */ {
    let mut i: usize = 0;
    let mut e        = 0;
    let chunk        = 0b1000_0000;

    for byte in bytes {
        // shift left by 7
        e += 1;
        i *= chunk as usize;

        // check if this byte is the last byte in the sequence
        // you pass remaining bytecode, so early breaking is important
        if byte >= chunk {
            i += (byte - chunk) as usize;
            break;
         } else {
             i += byte as usize;
         }
    }

    return (i, e);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_decode() {
        // big number
        let x = 7_289_529_732_981_739_357;
        assert_eq!(build_number(split_number(x)), (x, 9));
    }

    #[test]
    fn rollover() {
        let x = 256;
        assert_eq!(split_number(x), vec![0b0000_0010, 0b1000_0000])
    }

    #[test]
    fn extra_bytes() {
        // encode number
        let x     = 42069;
        let bytes = split_number(x);
        let eat   = bytes.len();

        // add junk data to end
        let mut extra = bytes.clone();
        extra.append(&mut vec![0xBA, 0xDA, 0x55]);

        assert_eq!((x, eat), build_number(bytes));
        assert_eq!((x, eat), build_number(extra));
    }

    #[test]
    fn zero() {
        let mut zero = split_number(0);
        zero.push(2); // will most likely be 2 if split/build_number doesn't work
        assert_eq!(build_number(zero), (0, 1));
    }
}
