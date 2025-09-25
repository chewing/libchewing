use std::{
    fs::File,
    io::{BufWriter, Write, stdout},
    path::PathBuf,
};

use anyhow::{Result, bail};
use chewing::dictionary::{Dictionary, Trie};

#[cfg(feature = "sqlite")]
use chewing::dictionary::SqliteDictionary;

use crate::flags;

pub(crate) fn run(args: flags::Dump) -> Result<()> {
    let ext = args
        .path
        .extension()
        .ok_or(anyhow::anyhow!("Unknown dictionary format."))?;
    let dict: Box<dyn Dictionary> = if ext.eq_ignore_ascii_case("sqlite3") {
        if cfg!(feature = "sqlite") {
            #[cfg(feature = "sqlite")]
            {
                Box::new(SqliteDictionary::open(&args.path)?)
            }
            #[cfg(not(feature = "sqlite"))]
            unreachable!();
        } else {
            bail!("sqlite3 dictionary format support was not enabled.");
        }
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
    let info = dict.about();
    writeln!(sink, "# dc:title {}", info.name)?;
    writeln!(sink, "# dc:rights {}", info.copyright)?;
    writeln!(sink, "# dc:license {}", info.license)?;
    writeln!(sink, "# dc:identifier {}", info.version)?;
    writeln!(sink, "# 詞(phrase) 詞頻(freq) 注音(bopomofo)")?;
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
    let info = dict.about();
    writeln!(sink, "# dc:title,{},", info.name)?;
    writeln!(sink, "# dc:rights,{},", info.copyright)?;
    writeln!(sink, "# dc:license,{},", info.license)?;
    writeln!(sink, "# dc:identifier,{},", info.version)?;
    writeln!(sink, "# 詞(phrase),詞頻(freq),注音(bopomofo)")?;
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
