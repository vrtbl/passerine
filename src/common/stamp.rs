use std::time::{SystemTime, UNIX_EPOCH};

/// Generates a pseudorandom byte, which is them formatted
/// as a two-character hexadecimal string.
pub fn shuffle(seed: u128) -> String {
    let now = SystemTime::now();
    let time = now.duration_since(UNIX_EPOCH)
        .expect("Could not determine time since epoch");
    let mash_up = time.as_millis() ^ time.as_nanos() ^ time.as_micros() ^ seed;
    let folded = mash_up.to_be_bytes().iter()
        .fold(0, |a, b| a ^ b);
    return format!("{:02x?}", folded);
}

/// Returns a pseudorandom 8 character hexadecimal string.
pub fn stamp(seed: u128) -> String {
    let mut combined = "".to_string();
    for i in 0..4 {
        combined += &shuffle(i + seed);
    }
    return combined;
}
