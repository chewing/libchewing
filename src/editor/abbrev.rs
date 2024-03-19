use std::{
    collections::BTreeMap,
    io::{self, BufRead},
};

#[derive(Debug)]
pub(crate) struct AbbrevTable {
    table: BTreeMap<char, String>,
}

impl AbbrevTable {
    pub(crate) fn new() -> io::Result<AbbrevTable> {
        // FIXME load from data
        let reader = io::Cursor::new(include_str!("../../data/swkb.dat"));
        let mut table = BTreeMap::new();
        for line in reader.lines() {
            let line = line?;
            let (abbr, expended) = line
                .split_once(' ')
                .expect("each line should have at last one separator");
            table.insert(abbr.chars().nth(0).unwrap(), expended.to_owned());
        }
        Ok(AbbrevTable { table })
    }

    pub(crate) fn find_abbrev(&self, ch: char) -> Option<&String> {
        self.table.get(&ch)
    }
}
