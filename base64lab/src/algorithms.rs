use crate::error::Base64Error;
use std::collections::HashMap;

pub const ALPHABET: &str = r"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+\";

pub fn encode(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    for i in (0..data.len()).step_by(3) {
        let mut triplet = 0u64;

        triplet += (*data.get(i).unwrap_or(&0) as u64) << 16;
        triplet += (*data.get(i + 1).unwrap_or(&0) as u64) << 8;
        triplet += *data.get(i + 2).unwrap_or(&0) as u64;

        let c0 = ((triplet >> 18) & 0x3F) as usize;
        let c1 = ((triplet >> 12) & 0x3F) as usize;
        let c2 = ((triplet >> 6) & 0x3F) as usize;
        let c3 = (triplet & 0x3F) as usize;

        output += &ALPHABET[c0..c0 + 1];
        output += &ALPHABET[c1..c1 + 1];

        if i + 1 < data.len() {
            output += &ALPHABET[c2..c2 + 1];
        } else {
            output += "=";
        }

        if i + 2 < data.len() {
            output += &ALPHABET[c3..c3 + 1];
        } else {
            output += "=";
        }
    }

    output
}

pub fn decode(data: &str) -> Result<Vec<u8>, Base64Error> {
    if data.len() % 4 != 0 {
        return Err(Base64Error::IncorrectStringLength {
            line: 0,
            len: data.len() as u64,
        });
    }

    if data.len() == 0 {
        return Ok(vec![]);
    }

    let mut bytes = Vec::<u8>::new();

    // helper conversion table
    let mut table = HashMap::new();
    for (i, c) in ALPHABET.chars().enumerate() {
        table.insert(c, i as u64);
    }

    // decode without last quartet
    for i in (0..data.len() - 4).step_by(4) {
        let mut triplet = 0u64;

        for j in 0..4 {
            match data.as_bytes()[i + j] as char {
                '=' => {
                    return Err(Base64Error::IncorrectUseOfPadding {
                        line: 0,
                        pos: (i + j) as u64,
                    });
                }
                c if ALPHABET.contains(c) => {
                    triplet += table[&c] << (3 - j) * 6;
                }
                _ => {
                    return Err(Base64Error::InvalidInputCharacter {
                        line: 0,
                        pos: (i + j) as u64,
                    });
                }
            }
        }

        bytes.push(((triplet >> 16) & 0xFF) as u8);
        bytes.push(((triplet >> 8) & 0xFF) as u8);
        bytes.push((triplet & 0xFF) as u8);
    }

    // decode last quartet
    let mut triplet = 0u64;
    let mut eq_count = 0;
    for i in data.len() - 4..data.len() {
        match data.as_bytes()[i] as char {
            '=' => {
                if i % 4 < 2 {
                    return Err(Base64Error::IncorrectUseOfPadding {
                        line: 0,
                        pos: i as u64,
                    });
                } else {
                    eq_count += 1;
                }
            }
            c if ALPHABET.contains(c) => {
                if eq_count != 0 {
                    return Err(Base64Error::IncorrectUseOfPadding {
                        line: 0,
                        pos: i as u64,
                    });
                }
                triplet += table[&c] << (3 - i % 4) * 6;
            }
            _ => {
                return Err(Base64Error::InvalidInputCharacter {
                    line: 0,
                    pos: i as u64,
                });
            }
        }
    }

    bytes.push(((triplet >> 16) & 0xFF) as u8);
    if eq_count < 2 {
        bytes.push(((triplet >> 8) & 0xFF) as u8);
    }
    if eq_count < 1 {
        bytes.push((triplet & 0xFF) as u8);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use crate::algorithms::{decode, encode};
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;

    #[test]
    fn test_encode() {
        let data = "hello world".as_bytes();

        for i in 0..data.len() {
            assert_eq!(&data[0..i], decode(&encode(&data[0..i])).unwrap());
            assert_eq!(encode(&data[0..i]), BASE64_STANDARD.encode(&data[0..i]));

            let e = encode(&data[0..i]);
            assert_eq!(decode(&e).unwrap(), BASE64_STANDARD.decode(&e).unwrap());
        }
    }
}
