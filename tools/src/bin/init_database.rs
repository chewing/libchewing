use anyhow::{bail, Context, Result};
use argh::FromArgs;
use chewing::{
    dictionary::{
        DictionaryBuilder, DictionaryInfo, SqliteDictionaryBuilder, TrieDictionaryBuilder,
    },
    zhuyin::{Bopomofo, Syllable},
};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use thiserror::Error;

#[derive(Error, Debug)]
#[error("parsing failed at line {line_num}")]
struct ParseError {
    line_num: usize,
    column: usize,
    #[source]
    source: anyhow::Error,
}

trait IntoParseError<T> {
    fn parse_error(self, line_num: usize, column: usize) -> std::result::Result<T, ParseError>;
}

impl<T> IntoParseError<T> for Result<T> {
    fn parse_error(self, line_num: usize, column: usize) -> std::result::Result<T, ParseError> {
        self.map_err(|source| ParseError {
            line_num,
            column,
            source,
        })
    }
}

#[derive(FromArgs)]
/// This program creates a new chewing phrase dictionary file.
pub struct Args {
    /// choose the underlying database implementation, must be either "trie" or "sqlite"
    #[argh(option, short = 't', default = "String::from(\"trie\")")]
    pub db_type: String,

    /// name of the phrase dictionary
    #[argh(option, short = 'n', default = "String::from(\"我的詞庫\")")]
    pub name: String,

    /// copyright information of the dictionary
    #[argh(option, short = 'c', default = "String::from(\"Unknown\")")]
    pub copyright: String,

    /// license information about the dictionary
    #[argh(option, short = 'l', default = "String::from(\"Unknown\")")]
    pub license: String,

    /// version information
    #[argh(option, short = 'r', default = "String::from(\"1.0.0\")")]
    pub version: String,

    /// keep word frequency
    #[argh(switch, short = 'k')]
    pub keep_word_freq: bool,

    /// path to the input tsi file
    #[argh(positional)]
    pub tsi_src: String,

    /// path to the output file
    #[argh(positional)]
    pub output: String,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let mut builder: Box<dyn DictionaryBuilder> = match args.db_type.as_str() {
        "sqlite" => Box::new(SqliteDictionaryBuilder::new()),
        "trie" => Box::new(TrieDictionaryBuilder::new()),
        _ => bail!("Unknown database type {}", args.db_type),
    };

    builder.set_info(DictionaryInfo {
        name: args.name.to_owned().into(),
        copyright: args.copyright.to_owned().into(),
        license: args.license.to_owned().into(),
        version: args.version.to_owned().into(),
        software: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).into(),
    })?;

    let tsi = File::open(args.tsi_src)?;
    let reader = BufReader::new(tsi);

    for (line_num, line) in reader.lines().enumerate() {
        let mut syllables = vec![];
        let line = line?;
        let phrase = line.split_ascii_whitespace().next().unwrap();

        let freq: u32 = match phrase.chars().count() {
            1 if !args.keep_word_freq => 0,
            _ => line
                .split_ascii_whitespace()
                .nth(1)
                .unwrap()
                .parse()
                .context("unable to parse frequency")
                .parse_error(line_num, 0)?,
        };

        for syllable_str in line.split_ascii_whitespace().skip(2) {
            let mut syllable_builder = Syllable::builder();
            if syllable_str.starts_with('#') {
                break;
            }
            for c in syllable_str.chars() {
                syllable_builder = syllable_builder.insert(Bopomofo::try_from(c)?)?;
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
    Ok(())
}
