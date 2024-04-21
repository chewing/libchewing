use anyhow::Result;
use chewing::dictionary::{Dictionary, DictionaryInfo, SqliteDictionary, Trie};

use crate::flags;

pub fn run(args: flags::Info) -> Result<()> {
    let ext = args
        .path
        .extension()
        .ok_or(anyhow::anyhow!("Unknown dictionary format."))?;
    let dict: Box<dyn Dictionary> = if ext.eq_ignore_ascii_case("sqlite3") {
        Box::new(SqliteDictionary::open_read_only(&args.path)?)
    } else {
        Box::new(Trie::open(&args.path)?)
    };
    let info = dict.about();
    if args.json {
        print_json_info(&info);
    } else {
        print_info(&info);
    }
    Ok(())
}

fn print_json_info(info: &DictionaryInfo) {
    println!("{{");
    println!(r#"  "name": "{}","#, info.name);
    println!(r#"  "version": "{}","#, info.version);
    println!(r#"  "copyright": "{}","#, info.copyright);
    println!(r#"  "license": "{}","#, info.license);
    println!(r#"  "software": "{}""#, info.software);
    println!("}}");
}

fn print_info(info: &DictionaryInfo) {
    println!("Name      : {}", info.name);
    println!("Version   : {}", info.version);
    println!("Copyright : {}", info.copyright);
    println!("License   : {}", info.license);
    println!("Software  : {}", info.software);
}
