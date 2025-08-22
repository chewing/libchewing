#[cfg(not(feature = "sqlite"))]
use anyhow::bail;
use anyhow::{Context, Result, anyhow};
#[cfg(feature = "sqlite")]
use chewing::dictionary::SqliteDictionaryBuilder;
use chewing::{
    dictionary::{DictionaryBuilder, DictionaryInfo, TrieBuilder},
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
        flags::DbType::Sqlite => {
            #[cfg(feature = "sqlite")]
            {
                Box::new(SqliteDictionaryBuilder::new())
            }
            #[cfg(not(feature = "sqlite"))]
            bail!("sqlite3 dictionary format support was not enabled.");
        }
        flags::DbType::Trie => Box::new(TrieBuilder::new()),
    };

    let mut name = args.name;
    let mut copyright = args.copyright;
    let mut license = args.license;
    let mut version = args.version;
    let software = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let tsi = File::open(args.tsi_src)?;
    let reader = BufReader::new(tsi);
    let delimiter = if args.csv { ',' } else { ' ' };
    let mut read_front_matter = true;
    let mut errors = vec![];

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Read front matter until first non-comment line
        if read_front_matter && !line.starts_with('#') {
            read_front_matter = false;
        } else if read_front_matter {
            let Some((key, value)) = line.trim_start_matches('#').trim().split_once(delimiter)
            else {
                errors.push(parse_error(line_num, "invalid metadata").into());
                continue;
            };
            let value = value.trim_end_matches(delimiter).to_string();
            match key.trim() {
                "dc:title" => name = value,
                "dc:rights" => copyright = value,
                "dc:license" => license = value,
                "dc:identifier" => version = value,
                _ => (),
            }
            continue;
        } else if line.starts_with('#') {
            continue;
        }
        match parse_line(line_num, delimiter, &line, args.keep_word_freq, args.fix) {
            Ok((syllables, phrase, freq)) => {
                if syllables.len() != phrase.chars().count() {
                    errors.push(
                        anyhow!("word count doesn't match").context(parse_error(line_num, line)),
                    );
                    continue;
                }
                builder.insert(&syllables, (phrase, freq).into())?;
            }
            Err(error) => errors.push(error),
        };
    }
    if !errors.is_empty() {
        for err in errors {
            eprintln!("{:#}", err);
        }
        if !args.fix {
            eprintln!("Hint: Use --fix to automatically fix common errors");
        }
        if !args.skip_invalid {
            std::process::exit(1)
        }
    }
    let path: &Path = args.output.as_ref();
    if path.exists() {
        fs::remove_file(path).context("unable to overwrite output")?;
    }

    let info = DictionaryInfo {
        name,
        copyright,
        license,
        version,
        software,
    };
    builder.set_info(info.clone())?;

    builder.build(path)?;

    if let Some(trie_builder) = builder.as_any().downcast_ref::<TrieBuilder>() {
        let stats = trie_builder.statistics();
        eprintln!("== Trie Dictionary Statistics ==");
        eprintln!("Name                 : {}", info.name);
        eprintln!("Copyright            : {}", info.copyright);
        eprintln!("License              : {}", info.license);
        eprintln!("Version              : {}", info.version);
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
    fix: bool,
) -> Result<(Vec<Syllable>, &str, u32)> {
    let phrase = line
        .split(delimiter)
        .find(|s| !s.is_empty())
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
        if syllable_str.is_empty() {
            continue;
        }
        let mut syllable_builder = Syllable::builder();
        if syllable_str.starts_with('#') {
            break;
        }
        for c in syllable_str.chars() {
            let c = if fix {
                fix_common_syllable_errors(c)
            } else {
                c
            };
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

fn fix_common_syllable_errors(c: char) -> char {
    match c {
        '一' => 'ㄧ',
        '丫' => 'ㄚ',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use chewing::syl;
    use chewing::zhuyin::Bopomofo::*;

    use super::parse_line;

    #[test]
    fn parse_ssv() {
        let line = "鑰匙 668 ㄧㄠˋ ㄔˊ # not official";
        if let Ok((syllables, phrase, freq)) = parse_line(0, ' ', &line, false, false) {
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
        if let Ok((syllables, phrase, freq)) = parse_line(0, ' ', &line, false, false) {
            assert_eq!(syllables, vec![syl![I, AU, TONE4], syl![CH, TONE2]]);
            assert_eq!("鑰匙", phrase);
            assert_eq!(668, freq);
        } else {
            panic!()
        }
    }

    #[test]
    fn parse_ssv_syllable_errors() {
        let line = "地永天長 50 ㄉ一ˋ ㄩㄥˇ ㄊ一ㄢ ㄔ丫ˊ";
        if let Ok((syllables, phrase, freq)) = parse_line(0, ' ', &line, false, true) {
            assert_eq!(
                syllables,
                vec![
                    syl![D, I, TONE4],
                    syl![IU, ENG, TONE3],
                    syl![T, I, AN],
                    syl![CH, A, TONE2]
                ]
            );
            assert_eq!("地永天長", phrase);
            assert_eq!(50, freq);
        } else {
            panic!()
        }
    }

    #[test]
    fn parse_csv() {
        let line = "鑰匙,668,ㄧㄠˋ ㄔˊ # not official";
        if let Ok((syllables, phrase, freq)) = parse_line(0, ',', &line, false, false) {
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
        if let Ok((syllables, phrase, freq)) = parse_line(0, ',', &line, false, false) {
            assert_eq!(syllables, vec![syl![I, AU, TONE4], syl![CH, TONE2]]);
            assert_eq!("鑰匙", phrase);
            assert_eq!(668, freq);
        } else {
            panic!()
        }
    }
}
