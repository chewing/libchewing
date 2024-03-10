use anyhow::{bail, Context, Result};
use chewing::{
    dictionary::{
        CdbDictionaryBuilder, DictionaryBuilder, DictionaryInfo, SqliteDictionaryBuilder,
        TrieDictionaryBuilder,
    },
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
    column: usize,
    source: anyhow::Error,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "parsing failed at line:column {}:{}",
            self.line_num, self.column
        )
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
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

pub fn run(args: flags::InitDatabase) -> Result<()> {
    let mut builder: Box<dyn DictionaryBuilder> = match args.db_type_or_default().as_str() {
        "sqlite" => Box::new(SqliteDictionaryBuilder::new()),
        "trie" => Box::new(TrieDictionaryBuilder::new()),
        "cdb" => Box::new(CdbDictionaryBuilder::new()),
        ty @ _ => bail!("Unknown database type {ty}"),
    };

    builder.set_info(DictionaryInfo {
        name: args.name_or_default().into(),
        copyright: args.copyright_or_default().into(),
        license: args.license_or_default().into(),
        version: args.version_or_default().into(),
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

    if let Some(trie_builder) = builder.as_any().downcast_ref::<TrieDictionaryBuilder>() {
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
