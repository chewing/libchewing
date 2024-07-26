use std::{env, io::Result};

use chewing::{
    dictionary::{Dictionary, LookupStrategy, Trie},
    syl,
    zhuyin::Bopomofo,
};
use log::{debug, info};

pub fn main() -> Result<()> {
    env_logger::init();

    let dict_path = env::args()
        .nth(1)
        .expect("The required argument dictionary path <PATH> is not provided.");

    info!("[*] try to load the dictionary...");
    let dict = Trie::open(dict_path)?;

    info!("[*] try to read the metadata...");
    let info = dict.about();
    debug!("[+] {:?}", info);

    info!("[*] try to lookup a phrase...");
    let entries = dict.lookup_all_phrases(
        &[
            syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
            syl![Bopomofo::SH, Bopomofo::TONE4],
        ],
        LookupStrategy::Standard,
    );
    for phrase in entries {
        debug!("[+] found {:?}", phrase);
    }
    for phrase in dict.entries() {
        debug!("[+] found {:?}", phrase);
    }

    Ok(())
}
