use anyhow::{bail, Context, Result};
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
    source: anyhow::Error,
}

fn parse_error(line_num: usize) -> ParseError {
    ParseError {
        line_num,
        source: anyhow::anyhow!("Invalid format. Use the --csv flag to enable CSV parsing."),
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parsing failed at line {}", self.line_num)
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

trait IntoParseError<T> {
    fn parse_error(self, line_num: usize) -> std::result::Result<T, ParseError>;
}

impl<T> IntoParseError<T> for Result<T> {
    fn parse_error(self, line_num: usize) -> std::result::Result<T, ParseError> {
        self.map_err(|source| ParseError { line_num, source })
    }
}

pub fn run(args: flags::InitDatabase) -> Result<()> {
    let mut builder: Box<dyn DictionaryBuilder> = match args.db_type_or_default().as_str() {
        "sqlite" => Box::new(SqliteDictionaryBuilder::new()),
        "trie" => Box::new(TrieBuilder::new()),
        ty => bail!("Unknown database type {ty}"),
    };

    builder.set_info(DictionaryInfo {
        name: args.name_or_default(),
        copyright: args.copyright_or_default(),
        license: args.license_or_default(),
        version: args.version_or_default(),
        software: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
    })?;

    let tsi = File::open(args.tsi_src)?;
    let reader = BufReader::new(tsi);
    let delimiter = if args.csv { ',' } else { ' ' };

    for (line_num, line) in reader.lines().enumerate() {
        if args.csv && line_num == 0 {
            continue;
        }

        let mut syllables = vec![];
        let line = line?;
        let phrase = line.split(delimiter).next().ok_or(parse_error(line_num))?;

        let freq: u32 = match phrase.chars().count() {
            1 if !args.keep_word_freq => 0,
            _ => line
                .split(delimiter)
                .nth(1)
                .ok_or(parse_error(line_num))?
                .parse()
                .context("Unable to parse frequency")
                .parse_error(line_num)?,
        };

        for syllable_str in line.split(delimiter).skip(2) {
            let mut syllable_builder = Syllable::builder();
            if syllable_str.starts_with('#') {
                break;
            }
            for c in syllable_str.chars() {
                syllable_builder = syllable_builder
                    .insert(
                        Bopomofo::try_from(c)
                            .context("parsing bopomofo")
                            .parse_error(line_num)?,
                    )
                    .with_context(|| format!("Parsing syllables {}", syllable_str))
                    .parse_error(line_num)?;
            }
            syllables.push(syllable_builder.build());
        }
        builder.insert(&syllables, (phrase, freq).into())?;
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
