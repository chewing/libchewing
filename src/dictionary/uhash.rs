//! Reads old text or binary formatted user dictionary.
//!
//! The original chewing stores user dictionary in a text file named
//! `uhash.dat`. The file starts with decimal integer which stores the lifetime
//! record. Each line after the lifetime record is a user phrase record
//! delimited by spaces.
//!
//! Each user phrase record starts with a UTF-8 encoded phrase string, followed
//! by N decimal integers where N is the number of characters in the phrase,
//! followed by 4 decimal integers which are userFreq, recentTime, maxFreq,
//! origFreq respectively.

use std::{
    ffi::{c_int, c_ushort},
    io::{self, BufRead, BufReader, Read},
    mem::size_of,
    str::{self, FromStr},
};

use crate::zhuyin::Syllable;

use super::Phrase;

const BIN_FIELD_SIZE: usize = 125;
const BIN_HASH_SIG: &str = "CBiH";

fn invalid_data() -> io::Error {
    io::ErrorKind::InvalidData.into()
}

fn try_parse<T: FromStr>(input: Option<&str>) -> io::Result<T> {
    input
        .ok_or(invalid_data())?
        .parse::<T>()
        .or(Err(invalid_data()))
}

pub(crate) fn try_load_text<R: Read>(input: R) -> io::Result<Vec<(Vec<Syllable>, Phrase)>> {
    let reader = BufReader::new(input);
    let mut lines = reader.lines();

    let first_line = lines.next().ok_or(invalid_data())?;
    let _lifetime: c_ushort = first_line?.parse().or(Err(invalid_data()))?;

    let mut result = Vec::new();
    for line in lines {
        let line = line?;
        let mut columns = line.split_ascii_whitespace();
        let phrase_str = columns.next().ok_or(invalid_data())?;
        let n_chars = phrase_str.chars().count();
        let mut syllables: Vec<Syllable> = Vec::new();
        for _ in 0..n_chars {
            let syl_u16: u16 = try_parse(columns.next())?;
            syllables.push(syl_u16.try_into().or(Err(invalid_data()))?);
        }
        let user_freq = try_parse(columns.next())?;
        let recent_time = try_parse(columns.next())?;
        let _max_freq: u32 = try_parse(columns.next())?;
        let _orig_freq: u32 = try_parse(columns.next())?;
        result.push((
            syllables,
            Phrase::new(phrase_str, user_freq).with_time(recent_time),
        ));
    }

    Ok(result)
}

pub(crate) fn try_load_bin<R: Read>(mut input: R) -> io::Result<Vec<(Vec<Syllable>, Phrase)>> {
    let mut buf = [0_u8; BIN_FIELD_SIZE];

    input.read_exact(&mut buf[0..BIN_HASH_SIG.len()])?;
    if !buf.starts_with(BIN_HASH_SIG.as_bytes()) {
        return Err(invalid_data());
    }
    // NB: lifetime size is platform dependent
    input.read_exact(&mut buf[0..size_of::<c_int>()])?;

    let mut result = Vec::new();
    loop {
        if input.read_exact(&mut buf).is_err() {
            break;
        }

        // NB: other integers are also platform dependent
        let user_freq: i32 = i32::from_ne_bytes(buf[0..4].try_into().unwrap());
        let recent_time: i32 = i32::from_ne_bytes(buf[4..8].try_into().unwrap());
        let _max_freq: i32 = i32::from_ne_bytes(buf[8..12].try_into().unwrap());
        let _orig_freq: i32 = i32::from_ne_bytes(buf[12..16].try_into().unwrap());

        // Due to a bug in 0.3.5, some userphrase has negative frequency value.
        // In this case, we just skip this record.
        //
        // See https://github.com/chewing/libchewing/issues/75
        if user_freq < 0 || recent_time < 0 || _max_freq < 0 || _orig_freq < 0 {
            continue;
        }

        let len = buf[16] as usize;
        // addressing the start of phrase_str
        if buf[17 + (2 * len) + 1] == 0 {
            // This record is removed
            continue;
        }
        let mut syllables: Vec<Syllable> = Vec::new();
        let mut base = 17;
        for _ in 0..len {
            let syl_u16 = u16::from_ne_bytes(buf[base..base + 2].try_into().unwrap());
            syllables.push(syl_u16.try_into().or(Err(invalid_data()))?);
            base += 2;
        }
        let bytes = buf[base] as usize;
        let phrase_str = str::from_utf8(&buf[base + 1..base + bytes + 1]);
        if phrase_str.is_err() {
            continue;
        }

        result.push((
            syllables,
            Phrase::new(phrase_str.unwrap(), user_freq as u32).with_time(recent_time as u64),
        ));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::{ffi::c_int, iter, mem::size_of};

    use crate::zhuyin::Syllable;

    use super::{BIN_FIELD_SIZE, Phrase, try_load_bin, try_load_text};

    #[test]
    fn load_valid_text() {
        let input = b"42\nP 1 1 2 3 4\n";
        let phrases = try_load_text(&input[..]).unwrap();
        assert_eq!(
            vec![(
                vec![Syllable::try_from(1).unwrap()],
                Phrase::from(("P", 1, 2))
            )],
            phrases
        );
    }

    #[test]
    fn load_truncated_text() {
        let input = b"42\nPhrase 1 2 3 4 5 6\n";
        let phrases = try_load_text(&input[..]);
        assert!(phrases.is_err());
    }

    #[test]
    fn load_malformed_text() {
        let input = br#"<?xml version="1.0" encoding="UTF-8"?>\n"#;
        let phrases = try_load_text(&input[..]);
        assert!(phrases.is_err());
    }

    #[test]
    fn load_binary_as_text() {
        let input = b"CBiH\0\0\0\0";
        let phrases = try_load_text(&input[..]);
        assert!(phrases.is_err());
    }

    #[test]
    fn load_valid_bin() {
        let mut input = vec![b'C', b'B', b'i', b'H'];
        input.extend_from_slice(&(0 as c_int).to_ne_bytes());
        input.extend_from_slice(&1_i32.to_ne_bytes());
        input.extend_from_slice(&2_i32.to_ne_bytes());
        input.extend_from_slice(&3_i32.to_ne_bytes());
        input.extend_from_slice(&4_i32.to_ne_bytes());
        input.push(1);
        input.extend_from_slice(&1_u16.to_ne_bytes());
        input.push(1);
        input.extend_from_slice(b"P");
        input.extend(iter::repeat(0).take(BIN_FIELD_SIZE - input.len() + 4 + size_of::<c_int>()));
        let phrases = try_load_bin(&input[..]).unwrap();
        assert_eq!(
            vec![(
                vec![Syllable::try_from(1).unwrap()],
                Phrase::from(("P", 1, 2))
            )],
            phrases
        );
    }
}
