use std::convert::TryInto;

fn shl(word: u64, bits: u64) -> u64 {
    assert!(bits <= 64);
    if bits == 64 {
        return 0;
    }
    return word << bits;
}

fn shr(word: u64, bits: u64) -> u64 {
    assert!(bits <= 64);
    if bits == 64 {
        return 0;
    }
    return word >> bits;
}

// shift right arithmetic
fn sra(word: u64, mut bits: u64) -> i64 {
    assert!(bits <= 64);
    if bits == 64 {
        bits = 63
    }
    return (word >> bits) as i64;
}

pub fn fitss(n: i64, width: u64) -> bool {
    if width >= 64 {
        return true;
    }
    let narrow = sra(shl(n as u64, 64 - width), 64 - width);
    return narrow == n;
}

pub fn fitsu(n: u64, width: u64) -> bool {
    if width >= 64 {
        return true;
    }
    return shr(n, width) == 0;
}

pub fn gets(word: u64, width: u64, lsb: u64) -> i64 {
    if width == 0 {
        return 0;
    }
    let hi = lsb + width;
    assert!(hi <= 64);
    return sra(shl(word, 64 - hi), ((64 - width) as i64).try_into().unwrap())
        .try_into()
        .unwrap();
}

pub fn getu(word: u64, width: u64, lsb: u64) -> u64 {
    let hi = lsb + width;
    assert!(hi <= 64);
    return shr(shl(word, 64 - hi), 64 - width);
}

pub fn newu(word: u64, width: u64, lsb: u64, value: u64) -> Option<u64> {
    let hi = lsb + width;
    assert!(hi <= 64);
    if !fitsu(value, width) {
        return None;
    }
    return Some(
        shl(shr(word, hi), hi) // high part
         | shr(shl(word, 64 - lsb), 64 - lsb) // low part
         | (value << lsb),
    ); // new value
}

pub fn news(word: u64, width: u64, lsb: u64, value: i64) -> Option<u64> {
    if !fitss(value, width) {
        return None;
    }
    return newu(word, width, lsb, getu(value as u64, width, 0));
}
