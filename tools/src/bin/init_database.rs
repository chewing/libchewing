use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{bail, Context, Result};
use chewing::{
    dictionary::{
        DictionaryBuilder, DictionaryInfo, SqliteDictionaryBuilder, TrieDictionaryBuilder,
    },
    zhuyin::{Bopomofo, Syllable},
};
use clap::{Arg, ArgAction, Command};
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

fn main() -> Result<()> {
    let m = Command::new("init_database")
        .about("This program creates a new chewing phrase dictionary file.")
        .arg(
            Arg::new("type")
                .short('t')
                .value_parser(["sqlite", "trie"])
                .default_value("trie"),
        )
        .arg(Arg::new("name").short('n').default_value("我的詞庫"))
        .arg(Arg::new("copyright").short('c').default_value("Unknown"))
        .arg(Arg::new("license").short('l').default_value("Unknown"))
        .arg(Arg::new("version").short('r').default_value("1.0.0"))
        .arg(
            Arg::new("keep-word-freq")
                .short('k')
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("tsi.src").required(true))
        .arg(Arg::new("output").required(true))
        .arg_required_else_help(true)
        .get_matches();

    let tsi_src: &String = m.get_one("tsi.src").unwrap();
    let output: &String = m.get_one("output").unwrap();
    let db_type: &String = m.get_one("type").unwrap();
    let name: &String = m.get_one("name").unwrap();
    let copyright: &String = m.get_one("copyright").unwrap();
    let license: &String = m.get_one("license").unwrap();
    let version: &String = m.get_one("version").unwrap();
    let keep_word_freq: bool = m.get_flag("keep-word-freq");

    let mut builder: Box<dyn DictionaryBuilder> = match db_type.as_str() {
        "sqlite" => Box::new(SqliteDictionaryBuilder::new()),
        "trie" => Box::new(TrieDictionaryBuilder::new()),
        _ => bail!("Unknown database type {}", db_type),
    };

    builder.set_info(DictionaryInfo {
        name: name.to_owned().into(),
        copyright: copyright.to_owned().into(),
        license: license.to_owned().into(),
        version: version.to_owned().into(),
        software: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).into(),
    })?;

    let tsi = File::open(tsi_src)?;
    let reader = BufReader::new(tsi);
    for (line_num, line) in reader.lines().enumerate() {
        let mut syllables = vec![];
        let line = line?;
        let phrase = line.split_ascii_whitespace().next().unwrap();
        let freq: u32 = match phrase.chars().count() {
            1 if !keep_word_freq => 0,
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
    let path: &Path = output.as_ref();
    if path.exists() {
        fs::remove_file(path).context("unable to overwrite output")?;
    }
    builder.build(path)?;

    Ok(())
}
