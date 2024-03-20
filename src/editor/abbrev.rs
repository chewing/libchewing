use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

#[derive(Debug)]
pub struct AbbrevTable {
    table: BTreeMap<char, String>,
}

impl AbbrevTable {
    pub fn new() -> AbbrevTable {
        AbbrevTable {
            table: BTreeMap::default(),
        }
    }
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<AbbrevTable> {
        let reader = BufReader::new(File::open(path.as_ref())?);
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

    pub fn find_abbrev(&self, ch: char) -> Option<&String> {
        self.table.get(&ch)
    }
}
