use std::{io::Result, path::PathBuf};

use chewing::{
    dictionary::{Dictionary, TrieDictionary},
    syl,
    zhuyin::Bopomofo,
};
use log::{debug, info};

pub fn main() -> Result<()> {
    env_logger::init();

    let flags = xflags::parse_or_exit! {
        /// Trie dictionary path
        required dict_path: PathBuf
    };

    info!("[*] try to load the dictionary...");
    let dict = TrieDictionary::open(flags.dict_path)?;

    info!("[*] try to read the metadata...");
    let info = dict.about();
    debug!("[+] {:?}", info);

    info!("[*] try to lookup a phrase...");
    let entries = dict.lookup_all_phrases(&[
        syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
        syl![Bopomofo::SH, Bopomofo::TONE4],
    ]);
    for phrase in entries {
        debug!("[+] found {:?}", phrase);
    }
    for phrase in dict.entries() {
        debug!("[+] found {:?}", phrase);
    }

    Ok(())
}