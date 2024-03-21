use std::{
    io::{Error, ErrorKind, Result},
    path::PathBuf,
};

use chewing::{
    dictionary::{CdbDictionary, Dictionary},
    syl,
    zhuyin::Bopomofo,
};
use log::{debug, info};

pub fn main() -> Result<()> {
    env_logger::init();

    let flags = xflags::parse_or_exit! {
        /// CDB dictionary path
        required dict_path: PathBuf
    };

    info!("[*] try to load the dictionary...");
    let dict =
        CdbDictionary::open(flags.dict_path).map_err(|_| Error::from(ErrorKind::InvalidData))?;

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
