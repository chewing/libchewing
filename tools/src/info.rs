use anyhow::Result;
use chewing::dictionary::{Dictionary, SystemDictionaryLoader, UserDictionaryLoader};

use crate::flags;

pub(crate) fn run(args: flags::Info) -> Result<()> {
    if args.system {
        let dictionaries = SystemDictionaryLoader::new().load()?;
        if args.json {
            print_json_info(&dictionaries, "base");
        } else {
            print_info(&dictionaries, "base");
        }
        let drop_in = SystemDictionaryLoader::new().load_drop_in()?;
        if args.json {
            print_json_info(&drop_in, "drop_in");
        } else {
            print_info(&drop_in, "drop_in");
        }
    }
    if args.user {
        let dict = UserDictionaryLoader::new().load()?;
        if args.json {
            print_json_info(&[dict], "user");
        } else {
            print_info(&[dict], "user");
        }
    }
    if let Some(path) = args.path {
        let dict = UserDictionaryLoader::new().userphrase_path(path).load()?;
        if args.json {
            print_json_info(&[dict], "input");
        } else {
            print_info(&[dict], "input");
        }
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

fn print_json_info(dictionaries: &[Box<dyn Dictionary>], from: &str) {
    let mut iter = dictionaries.iter().peekable();
    println!("[");
    while let Some(dict) = iter.next() {
        let path = dict
            .path()
            .map(|p| p.display().to_string())
            .unwrap_or(String::new());
        let info = dict.about();
        println!("  {{");
        println!(r#"    "from": "{from}","#);
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

fn print_info(dictionaries: &[Box<dyn Dictionary>], from: &str) {
    for dict in dictionaries {
        let path = dict
            .path()
            .map(|p| p.display().to_string())
            .unwrap_or(String::new());
        let info = dict.about();
        println!("---");
        println!("From      : {from}");
        println!("Path      : {}", path);
        println!("Name      : {}", info.name);
        println!("Version   : {}", info.version);
        println!("Copyright : {}", info.copyright);
        println!("License   : {}", info.license);
        println!("Software  : {}", info.software);
    }
}
