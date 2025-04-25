use std::{
    fs::File,
    io::{BufWriter, Write, stdout},
    path::PathBuf,
};

use anyhow::Result;
use chewing::dictionary::{Dictionary, SqliteDictionary, Trie};

use crate::flags;

pub(crate) fn run(args: flags::Dump) -> Result<()> {
    let ext = args
        .path
        .extension()
        .ok_or(anyhow::anyhow!("Unknown dictionary format."))?;
    let dict: Box<dyn Dictionary> = if ext.eq_ignore_ascii_case("sqlite3") {
        Box::new(SqliteDictionary::open_read_only(&args.path)?)
    } else {
        Box::new(Trie::open(&args.path)?)
    };
    let sink: Box<dyn Write> = if let Some(output) = args.output {
        if output == PathBuf::from("-") {
            Box::new(stdout())
        } else {
            Box::new(File::create(output)?)
        }
    } else {
        Box::new(stdout())
    };
    let sink = BufWriter::new(sink);
    if args.csv {
        dump_dict_csv(sink, dict.as_ref())?;
    } else {
        dump_dict_tsi_src(sink, dict.as_ref())?;
    }
    Ok(())
}

fn dump_dict_tsi_src(mut sink: BufWriter<Box<dyn Write>>, dict: &dyn Dictionary) -> Result<()> {
    for (syllables, phrase) in dict.entries() {
        writeln!(
            sink,
            "{} {} {}",
            phrase,
            phrase.freq(),
            syllables
                .iter()
                .map(|syl| syl.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )?;
    }
    Ok(())
}

fn dump_dict_csv(mut sink: BufWriter<Box<dyn Write>>, dict: &dyn Dictionary) -> Result<()> {
    writeln!(sink, "詞(phrase),詞頻(freq),注音(bopomofo)")?;
    for (syllables, phrase) in dict.entries() {
        writeln!(
            sink,
            "{},{},{}",
            phrase,
            phrase.freq(),
            syllables
                .iter()
                .map(|syl| syl.to_string())
                .collect::<Vec<_>>()
                .join("　")
        )?;
    }
    Ok(())
}
