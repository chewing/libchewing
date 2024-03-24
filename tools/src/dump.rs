use anyhow::Result;
use chewing::dictionary::{CdbDictionary, Dictionary, SqliteDictionary, TrieDictionary};

use crate::flags;

pub fn run(args: flags::Dump) -> Result<()> {
    let ext = args
        .path
        .extension()
        .ok_or(anyhow::anyhow!("Unknown dictionary format."))?;
    let dict: Box<dyn Dictionary> = if ext.eq_ignore_ascii_case("sqlite3") {
        Box::new(SqliteDictionary::open_read_only(&args.path)?)
    } else if ext.eq_ignore_ascii_case("cdb") {
        Box::new(CdbDictionary::open(&args.path)?)
    } else {
        Box::new(TrieDictionary::open(&args.path)?)
    };
    if args.csv {
        dump_dict_csv(&dict);
    } else {
        dump_dict_tsi_src(&dict);
    }
    Ok(())
}

fn dump_dict_tsi_src(dict: &Box<dyn Dictionary>) {
    for (syllables, phrase) in dict.entries() {
        println!(
            "{} {} {}",
            phrase,
            phrase.freq(),
            syllables
                .iter()
                .map(|syl| syl.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

fn dump_dict_csv(dict: &Box<dyn Dictionary>) {
    println!("phrase,user_freq,bopomofo");
    for (syllables, phrase) in dict.entries() {
        println!(
            "{},{},{}",
            phrase,
            phrase.freq(),
            syllables
                .iter()
                .map(|syl| syl.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
