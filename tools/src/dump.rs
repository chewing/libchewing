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
    if args.csv {
        dump_dict_csv(dict.as_ref());
    } else {
        dump_dict_tsi_src(dict.as_ref());
    }
    Ok(())
}

fn dump_dict_tsi_src(dict: &dyn Dictionary) {
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

fn dump_dict_csv(dict: &dyn Dictionary) {
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
