use anyhow::Result;
use chewing::dictionary::{
    CdbDictionary, Dictionary, DictionaryInfo, SqliteDictionary, TrieDictionary,
};

use crate::flags;

pub fn run(args: flags::Info) -> Result<()> {
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
    let info = dict.about();
    if args.json {
        print_json_info(&info);
    } else {
        print_info(&info);
    }
    Ok(())
}

fn print_json_info(info: &DictionaryInfo) {
    let empty = &String::new();
    println!("{{");
    println!(r#"  "name": "{}","#, info.name.as_ref().unwrap_or(empty));
    println!(
        r#"  "version": "{}","#,
        info.version.as_ref().unwrap_or(empty)
    );
    println!(
        r#"  "copyright": "{}","#,
        info.copyright.as_ref().unwrap_or(empty)
    );
    println!(
        r#"  "license": "{}","#,
        info.license.as_ref().unwrap_or(empty)
    );
    println!(
        r#"  "software": "{}""#,
        info.software.as_ref().unwrap_or(empty)
    );
    println!("}}");
}

fn print_info(info: &DictionaryInfo) {
    let empty = &String::new();
    println!("Name      : {}", info.name.as_ref().unwrap_or(empty));
    println!("Version   : {}", info.version.as_ref().unwrap_or(empty));
    println!("Copyright : {}", info.copyright.as_ref().unwrap_or(empty));
    println!("License   : {}", info.license.as_ref().unwrap_or(empty));
    println!("Software  : {}", info.software.as_ref().unwrap_or(empty));
}
