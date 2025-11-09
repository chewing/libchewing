use anyhow::Result;
use chewing::{
    dictionary::{Dictionary, SingleDictionaryLoader, UserDictionaryLoader},
    path::{find_files_by_ext, sys_path_from_env_var},
};

use crate::flags;

pub(crate) fn run(args: flags::Info) -> Result<()> {
    if args.system {
        // FIXME: use find_files_by_ext and generic loader
        let loader = SingleDictionaryLoader::new();
        let search_path = sys_path_from_env_var();
        let files = find_files_by_ext(&search_path, &["dat", "sqlite3"]);
        let dictionaries: Vec<_> = files
            .iter()
            .filter(|file_name| !file_name.ends_with("chewing.dat"))
            .filter_map(|file_name| loader.guess_format_and_load(&file_name).ok())
            .collect();
        if args.json {
            print_json_info(&dictionaries, "system");
        } else {
            print_info(&dictionaries, "system");
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
