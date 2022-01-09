//! Copied from [JoNil/ascii85](https://github.com/JoNil/ascii85).

use std::borrow::Cow;

use thiserror::Error;

const TABLE: [u32; 5] = [85 * 85 * 85 * 85, 85 * 85 * 85, 85 * 85, 85, 1];

pub fn encode(input: &[u8]) -> String {
    let mut result = String::with_capacity(5 * (input.len() / 4 + 16));

    for chunk in input.chunks(4) {
        let (chunk, count) = if chunk.len() == 4 {
            (Cow::from(chunk), 5)
        } else {
            let mut new_chunk = Vec::new();
            new_chunk.resize_with(4, || 0);
            new_chunk[..chunk.len()].copy_from_slice(chunk);
            (Cow::from(new_chunk), 5 - (4 - chunk.len()))
        };

        let number = u32::from_be_bytes(chunk.as_ref().try_into().unwrap());

        for i in 0..count {
            let digit = (((number / TABLE[i]) % 85) + 33) as u8;
            result.push(digit as char);
        }
    }

    result
}

fn decode_digit(digit: u8, counter: &mut usize, chunk: &mut u32, result: &mut Vec<u8>) {
    let byte = digit - 33;

    *chunk += byte as u32 * TABLE[*counter];

    if *counter == 4 {
        result.extend_from_slice(&chunk.to_be_bytes());
        *chunk = 0;
        *counter = 0;
    } else {
        *counter += 1;
    }
}

#[derive(Debug, Error)]
pub enum Ascii85Error {
    #[error("misaligned z in input")]
    MisalignedZ,

    #[error("character out of range")]
    OutOfRange,
}

pub fn decode(input: &str) -> Result<Vec<u8>, Ascii85Error> {
    let mut result = Vec::with_capacity(4 * (input.len() / 5 + 16));

    let mut counter = 0;
    let mut chunk = 0;

    for digit in input.bytes() {
        if digit == b'z' {
            if counter == 0 {
                result.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                return Err(Ascii85Error::MisalignedZ);
            }
        }

        if !(33..=117).contains(&digit) {
            return Err(Ascii85Error::OutOfRange);
        }

        decode_digit(digit, &mut counter, &mut chunk, &mut result);
    }

    let mut to_remove = 0;

    while counter != 0 {
        decode_digit(b'u', &mut counter, &mut chunk, &mut result);
        to_remove += 1;
    }

    result.drain((result.len() - to_remove)..result.len());

    Ok(result)
}
