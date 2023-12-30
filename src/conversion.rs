//! TODO: docs

mod chewing;
mod symbol;

use crate::zhuyin::Syllable;

pub use self::chewing::ChewingEngine;
pub(crate) use self::symbol::{full_width_symbol_input, special_symbol_input};

/// TODO: doc
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Interval {
    /// TODO: doc
    pub start: usize,
    /// TODO: doc
    pub end: usize,
    // TODO doc, fix alignment
    pub is_phrase: bool,
    /// TODO: doc
    pub phrase: String,
}

impl Interval {
    /// TODO: doc
    pub fn contains(&self, other: &Interval) -> bool {
        self.start <= other.start && self.end >= other.end
    }
    /// TODO: doc
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    /// TODO: doc
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// TODO: doc
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Break(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Glue(pub usize);

/// A smallest unit of input in the pre-edit buffer.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Symbol {
    /// Chinese syllable
    Syllable(Syllable),
    /// Any direct character
    Char(char),
}

impl Symbol {
    pub fn is_syllable(&self) -> bool {
        match self {
            Symbol::Syllable(_) => true,
            Symbol::Char(_) => false,
        }
    }
    pub fn as_syllable(&self) -> Syllable {
        match self {
            Symbol::Syllable(syllable) => *syllable,
            Symbol::Char(_) => panic!(),
        }
    }
    pub fn is_char(&self) -> bool {
        match self {
            Symbol::Syllable(_) => false,
            Symbol::Char(_) => true,
        }
    }
    pub fn as_char(&self) -> char {
        match self {
            Symbol::Syllable(_) => panic!(),
            Symbol::Char(c) => *c,
        }
    }
}

impl AsRef<Syllable> for Symbol {
    fn as_ref(&self) -> &Syllable {
        match self {
            Symbol::Syllable(s) => s,
            Symbol::Char(_) => panic!(),
        }
    }
}

/// TODO: doc
#[derive(Debug, Default, Clone)]
pub struct Composition {
    /// TODO: doc
    pub buffer: Vec<Symbol>,
    /// TODO: doc
    pub selections: Vec<Interval>,
    /// TODO: doc
    /// TODO: merge with symbol?
    pub breaks: Vec<Break>,
    /// TODO doc
    pub glues: Vec<Glue>,
}

/// TODO: doc
pub trait ConversionEngine<C: ?Sized> {
    /// TODO: doc
    fn convert(&self, context: &C, composition: &Composition) -> Vec<Interval>;
    /// TODO: doc
    fn convert_next(&self, context: &C, composition: &Composition, next: usize) -> Vec<Interval>;
}
