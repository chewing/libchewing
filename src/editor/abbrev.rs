use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

#[derive(Debug, Default)]
pub struct AbbrevTable {
    table: BTreeMap<char, String>,
}

impl AbbrevTable {
    pub fn new() -> AbbrevTable {
        AbbrevTable::default()
    }
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<AbbrevTable> {
        let reader = BufReader::new(File::open(path.as_ref())?);
        let mut table = BTreeMap::new();
        for line in reader.lines() {
            let line = line?;
            // each line should have at last one separator
            if let Some((abbr, expended)) = line.split_once(' ') {
                if let Some(ch) = abbr.chars().nth(0) {
                    if !expended.is_empty() {
                        table.insert(ch, expended.to_owned());
                    }
                }
            }
        }
        Ok(AbbrevTable { table })
    }

    pub fn find_abbrev(&self, ch: char) -> Option<&String> {
        self.table.get(&ch)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;

    use super::AbbrevTable;
    use tempfile::NamedTempFile;

    #[test]
    fn load_good_abbrev_file() -> Result<(), Box<dyn Error>> {
        let file = NamedTempFile::new()?;
        let abbrev_dat = file.into_temp_path();
        fs::write(&abbrev_dat, "A A\nBB\nC\nD D\n  E E\n")?;

        let abbrev = AbbrevTable::open(&abbrev_dat)?;

        assert_eq!(Some(&"A".to_string()), abbrev.find_abbrev('A'));
        assert_eq!(Some(&"D".to_string()), abbrev.find_abbrev('D'));
        assert_eq!(None, abbrev.find_abbrev('B'));
        assert_eq!(None, abbrev.find_abbrev('C'));
        Ok(())
    }
}
