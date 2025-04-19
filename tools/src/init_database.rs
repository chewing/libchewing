use anyhow::{Context, Result};
use chewing::{
    dictionary::{DictionaryBuilder, DictionaryInfo, SqliteDictionaryBuilder, TrieBuilder},
    zhuyin::{Bopomofo, Syllable},
};
use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use crate::flags;

#[derive(Debug)]
struct ParseError {
    line_num: usize,
    line: String,
    source: anyhow::Error,
}

fn parse_error(line_num: usize, line: &str) -> ParseError {
    ParseError {
        line_num,
        line: line.to_string(),
        source: anyhow::anyhow!("Invalid format. Use the --csv flag to enable CSV parsing."),
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parsing failed at line {}: {}",
            self.line_num + 1,
            self.line
        )
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

trait IntoParseError<T> {
    fn parse_error(self, line_num: usize, line: &str) -> std::result::Result<T, ParseError>;
}

impl<T> IntoParseError<T> for Result<T> {
    fn parse_error(self, line_num: usize, line: &str) -> std::result::Result<T, ParseError> {
        self.map_err(|source| ParseError {
            line_num,
            line: line.to_string(),
            source,
        })
    }
}

pub(crate) fn run(args: flags::InitDatabase) -> Result<()> {
    let mut builder: Box<dyn DictionaryBuilder> = match args.db_type {
        flags::DbType::Sqlite => Box::new(SqliteDictionaryBuilder::new()),
        flags::DbType::Trie => Box::new(TrieBuilder::new()),
    };

    builder.set_info(DictionaryInfo {
        name: args.name,
        copyright: args.copyright,
        license: args.license,
        version: args.version,
        software: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
    })?;

    let tsi = File::open(args.tsi_src)?;
    let reader = BufReader::new(tsi);
    let delimiter = if args.csv { ',' } else { ' ' };
    let mut errors = vec![];

    for (line_num, line) in reader.lines().enumerate() {
        if args.csv && line_num == 0 {
            continue;
        }
        let line = line?;
        match parse_line(line_num, delimiter, &line, args.keep_word_freq) {
            Ok((syllables, phrase, freq)) => builder.insert(&syllables, (phrase, freq).into())?,
            Err(error) => errors.push(error),
        };
    }
    if !errors.is_empty() {
        for err in errors {
            eprintln!("{}", err);
        }
        if !args.skip_invalid {
            std::process::exit(1)
        }
    }
    let path: &Path = args.output.as_ref();
    if path.exists() {
        fs::remove_file(path).context("unable to overwrite output")?;
    }
    builder.build(path)?;

    if let Some(trie_builder) = builder.as_any().downcast_ref::<TrieBuilder>() {
        let stats = trie_builder.statistics();
        eprintln!("== Trie Dictionary Statistics ==");
        eprintln!("Node count           : {}", stats.node_count);
        eprintln!("Leaf count           : {}", stats.leaf_count);
        eprintln!("Phrase count         : {}", stats.phrase_count);
        eprintln!("Max height           : {}", stats.max_height);
        eprintln!("Average height       : {}", stats.avg_height);
        eprintln!("Root branch count    : {}", stats.root_branch_count);
        eprintln!("Max branch count     : {}", stats.max_branch_count);
        eprintln!("Average branch count : {}", stats.avg_branch_count);
    }
    Ok(())
}

fn parse_line(
    line_num: usize,
    delimiter: char,
    line: &str,
    keep_word_freq: bool,
) -> Result<(Vec<Syllable>, &str, u32)> {
    let phrase = line
        .split(delimiter)
        .filter(|s| !s.is_empty())
        .next()
        .ok_or(parse_error(line_num, line))?
        .trim_matches('"');

    let freq: u32 = match phrase.chars().count() {
        1 if !keep_word_freq => 0,
        _ => line
            .split(delimiter)
            .filter(|s| !s.is_empty())
            .nth(1)
            .ok_or(parse_error(line_num, line))?
            .trim_matches('"')
            .parse()
            .context("Unable to parse frequency")
            .parse_error(line_num, line)?,
    };

    let mut syllables = vec![];

    for syllable_str in line
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .skip(2)
    // skip phrase and freq
    {
        let syllable_str = syllable_str.trim_matches('"');
        println!("syllable {:?}", syllable_str);
        if syllable_str.is_empty() {
            continue;
        }
        let mut syllable_builder = Syllable::builder();
        if syllable_str.starts_with('#') {
            break;
        }
        for c in syllable_str.chars() {
            syllable_builder = syllable_builder
                .insert(
                    Bopomofo::try_from(c)
                        .context("parsing bopomofo")
                        .parse_error(line_num, line)?,
                )
                .with_context(|| format!("Parsing syllables {}", syllable_str))
                .parse_error(line_num, line)?;
        }
        syllables.push(syllable_builder.build());
    }

    Ok((syllables, phrase, freq))
}

#[cfg(test)]
mod tests {
    use chewing::syl;
    use chewing::zhuyin::Bopomofo::*;

    use super::parse_line;

    #[test]
    fn parse_ssv() {
        let line = "鑰匙 668 ㄧㄠˋ ㄔˊ # not official";
        if let Ok((syllables, phrase, freq)) = parse_line(0, ' ', &line, false) {
            assert_eq!(syllables, vec![syl![I, AU, TONE4], syl![CH, TONE2]]);
            assert_eq!("鑰匙", phrase);
            assert_eq!(668, freq);
        } else {
            panic!()
        }
    }

    #[test]
    fn parse_ssv_multiple_whitespace() {
        let line = "鑰匙     668 ㄧㄠˋ ㄔˊ # not official";
        if let Ok((syllables, phrase, freq)) = parse_line(0, ' ', &line, false) {
            assert_eq!(syllables, vec![syl![I, AU, TONE4], syl![CH, TONE2]]);
            assert_eq!("鑰匙", phrase);
            assert_eq!(668, freq);
        } else {
            panic!()
        }
    }

    #[test]
    fn parse_csv() {
        let line = "鑰匙,668,ㄧㄠˋ ㄔˊ # not official";
        if let Ok((syllables, phrase, freq)) = parse_line(0, ',', &line, false) {
            assert_eq!(syllables, vec![syl![I, AU, TONE4], syl![CH, TONE2]]);
            assert_eq!("鑰匙", phrase);
            assert_eq!(668, freq);
        } else {
            panic!()
        }
    }

    #[test]
    fn parse_csv_quoted() {
        let line = "\"鑰匙\",668,\"ㄧㄠˋ ㄔˊ # not official\"";
        if let Ok((syllables, phrase, freq)) = parse_line(0, ',', &line, false) {
            assert_eq!(syllables, vec![syl![I, AU, TONE4], syl![CH, TONE2]]);
            assert_eq!("鑰匙", phrase);
            assert_eq!(668, freq);
        } else {
            panic!()
        }
    }
}
