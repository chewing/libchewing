use std::{
    fs::File,
    io::{BufRead, BufReader, Result},
    path::Path,
};

use crate::conversion::Symbol;

#[derive(Debug, Default, Clone)]
pub struct SymbolSelector {
    category: Vec<(String, usize)>,
    table: Vec<String>,
    cursor: Option<u8>,
}

impl SymbolSelector {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<SymbolSelector> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        SymbolSelector::new(reader)
    }
    pub(crate) fn new<R: BufRead>(reader: R) -> Result<SymbolSelector> {
        let mut category = Vec::new();
        let mut table = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.contains('=') {
                let (cat, tab) = line.split_once('=').expect("at last one separator");
                category.push((cat.to_owned(), table.len()));
                table.push(tab.to_owned());
            } else {
                category.push((line, usize::MAX));
            }
        }

        Ok(SymbolSelector {
            category,
            table,
            cursor: None,
        })
    }
    pub(crate) fn menu(&self) -> Vec<String> {
        match self.cursor {
            Some(cursor) => self.table[cursor as usize]
                .chars()
                .map(|c| c.to_string())
                .collect(),
            None => self.category.iter().map(|cat| cat.0.clone()).collect(),
        }
    }
    pub(crate) fn select(&mut self, n: usize) -> Option<Symbol> {
        match self.cursor {
            None => {
                if self.category.len() <= n {
                    return None;
                }
                let cat = &self.category[n];
                if cat.1 == usize::MAX {
                    self.cursor = None;
                    Some(Symbol::new_char(cat.0.chars().next().unwrap()))
                } else {
                    self.cursor = Some(cat.1 as u8);
                    None
                }
            }
            Some(cursor) => {
                self.cursor = None;
                self.table[cursor as usize]
                    .chars()
                    .nth(n)
                    .map(Symbol::new_char)
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct SpecialSymbolSelector {
    symbol: Symbol,
}

impl SpecialSymbolSelector {
    pub(crate) fn new(symbol: Symbol) -> SpecialSymbolSelector {
        SpecialSymbolSelector { symbol }
    }
    pub(crate) fn menu(&self) -> Vec<String> {
        match self.find_category() {
            Some(cat) => cat.chars().map(|c| c.to_string()).collect(),
            None => Vec::new(),
        }
    }
    pub(crate) fn select(&self, n: usize) -> Option<Symbol> {
        self.find_category()
            .and_then(|cat| cat.chars().nth(n).map(Symbol::new_char))
    }
    fn find_category(&self) -> Option<&str> {
        Self::TABLE
            .iter()
            .find(|cat| cat.contains(self.symbol.as_char()))
            .copied()
    }
    const TABLE: &'static [&'static str; 48] = &[
        "ø",
        "「『《〈【〔",
        "」』》〉】〕",
        "{",
        "}",
        "，←",
        "。→．",
        "？¿",
        "！Ⅰ¡",
        "＠Ⅱ⊕⊙㊣﹫",
        "＃Ⅲ﹟",
        "＄Ⅳ€﹩￠∮￡￥",
        "％Ⅴ",
        "︿Ⅵ﹀︽︾",
        "＆Ⅶ﹠",
        "＊Ⅷ×※╳﹡☯☆★",
        "（Ⅸ",
        "）Ⅹ",
        "—－―–←→＿￣﹍﹉﹎﹊﹏﹋…‥¯",
        "／÷↗↙∕",
        "↑↓∣∥︱︳︴",
        "ÅΑα├╠╟╞",
        "Ββ∵",
        "Χχ┘╯╝╜╛㏄℃㎝♣©",
        "Δδ◇◆┤╣╢╡♦",
        "Εε┐╮╗╓╕",
        "Φψ│║♀",
        "Γγ",
        "Ηη♥",
        "Ιι",
        "φ",
        "Κκ㎞㏎",
        "Λλ㏒㏑",
        "Μμ♂ℓ㎎㏕㎜㎡",
        "Νν№",
        "Οο",
        "Ππ",
        "ΘθД┌╭╔╓╒",
        "Ρρ─═",
        "®",
        "Σσ∴□■┼╬╪╫∫§♠",
        "Ττθ△▲▽▼™⊿™",
        "Υυμ∪∩",
        "ν",
        "Ωω┬╦╤╥",
        "Ξξ┴╩╧╨",
        "Ψ",
        "Ζζ└╰╚╙╘",
    ];
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::conversion::Symbol;

    use super::SymbolSelector;

    #[test]
    fn select_level_one_leaf() {
        let reader = io::Cursor::new("…\n※\n常用符號=，、。\n");
        let mut sel = SymbolSelector::new(reader).expect("should parse");

        assert_eq!(vec!["…", "※", "常用符號"], sel.menu());
        assert_eq!(Symbol::new_char('…'), sel.select(0).unwrap());
    }

    #[test]
    fn select_level_two_leaf() {
        let reader = io::Cursor::new("…\n※\n常用符號=，、。\n");
        let mut sel = SymbolSelector::new(reader).expect("should parse");

        assert_eq!(vec!["…", "※", "常用符號"], sel.menu());
        assert_eq!(None, sel.select(2));
        assert_eq!(vec!["，", "、", "。"], sel.menu());
        assert_eq!(Symbol::new_char('，'), sel.select(0).unwrap());
    }
}
