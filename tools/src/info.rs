use anyhow::Result;
use chewing::dictionary::{Dictionary, SystemDictionaryLoader, UserDictionaryLoader};

use crate::flags;

pub(crate) fn run(args: flags::Info) -> Result<()> {
    let mut dictionaries = vec![];
    if args.user {
        dictionaries.push(UserDictionaryLoader::new().load()?);
    }
    if args.system {
        dictionaries.extend(SystemDictionaryLoader::new().load()?);
    }
    if let Some(path) = args.path {
        dictionaries.push(UserDictionaryLoader::new().userphrase_path(path).load()?);
    }
    if args.json {
        print_json_info(&dictionaries);
    } else {
        print_info(&dictionaries);
    }
    Ok(())
}

fn escape_json(str: String) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    str.chars().for_each(|ch| {
        if ch.is_control() {
            write!(out, "\\u{:04x}", ch as u32).unwrap();
        } else if ch == '\\' {
            out.push_str("\\\\");
        } else if ch == '\"' {
            out.push_str("\\\"");
        } else {
            out.push(ch);
        }
    });
    out
}

fn print_json_info(dictionaries: &[Box<dyn Dictionary>]) {
    let mut iter = dictionaries.iter().peekable();
    println!("[");
    while let Some(dict) = iter.next() {
        let path = dict
            .path()
            .map(|p| p.display().to_string())
            .unwrap_or(String::new());
        let info = dict.about();
        println!("  {{");
        println!(r#"    "path": "{}","#, escape_json(path));
        println!(r#"    "name": "{}","#, escape_json(info.name));
        println!(r#"    "version": "{}","#, escape_json(info.version));
        println!(r#"    "copyright": "{}","#, escape_json(info.copyright));
        println!(r#"    "license": "{}","#, escape_json(info.license));
        println!(r#"    "software": "{}""#, escape_json(info.software));
        println!("  }}{}", if iter.peek().is_some() { "," } else { "" });
    }
    println!("]");
}

fn print_info(dictionaries: &[Box<dyn Dictionary>]) {
    for dict in dictionaries {
        let path = dict
            .path()
            .map(|p| p.display().to_string())
            .unwrap_or(String::new());
        let info = dict.about();
        println!("---");
        println!("Path      : {}", path);
        println!("Name      : {}", info.name);
        println!("Version   : {}", info.version);
        println!("Copyright : {}", info.copyright);
        println!("License   : {}", info.license);
        println!("Software  : {}", info.software);
    }
}
