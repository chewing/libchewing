//! TODO: docs

mod chewing;
mod symbol;

use crate::zhuyin::Syllable;

pub use self::chewing::ChewingConversionEngine;
pub(crate) use self::symbol::full_width_symbol_input;

/// TODO: doc
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Interval {
    /// TODO: doc
    pub start: usize,
    /// TODO: doc
    pub end: usize,
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
#[derive(Debug)]
pub struct Break(pub usize);

/// A smallest unit of input in the pre-edit buffer.
#[derive(Debug, Clone, Copy)]
pub enum Symbol {
    /// Chinese syllable
    Syllable(Syllable),
    /// Any direct character
    Char(char),
}

impl Symbol {
    pub(crate) fn is_syllable(&self) -> bool {
        match self {
            Symbol::Syllable(_) => true,
            Symbol::Char(_) => false,
        }
    }
    pub(crate) fn as_syllable(&self) -> Syllable {
        match self {
            Symbol::Syllable(syllable) => *syllable,
            Symbol::Char(_) => panic!(),
        }
    }
}

/// TODO: doc
#[derive(Debug, Default)]
pub struct Composition {
    /// TODO: doc
    pub buffer: Vec<Symbol>,
    /// TODO: doc
    pub selections: Vec<Interval>,
    /// TODO: doc
    pub breaks: Vec<Break>,
}

/// TODO: doc
pub trait ConversionEngine {
    /// TODO: doc
    fn convert(&self, composition: &Composition) -> Vec<Interval>;
    /// TODO: doc
    fn convert_next(&self, composition: &Composition, next: usize) -> Vec<Interval>;
}
