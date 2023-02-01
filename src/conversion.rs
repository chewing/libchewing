//! TODO: docs

use crate::zhuyin::Syllable;

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

/// TODO: doc
#[derive(Debug)]
pub struct ChineseSequence {
    /// TODO: doc
    pub syllables: Vec<Syllable>,
    /// TODO: doc
    pub selections: Vec<Interval>,
    /// TODO: doc
    pub breaks: Vec<Break>,
}

/// TODO: doc
pub trait ConversionEngine {
    /// TODO: doc
    fn convert(&self, segment: &ChineseSequence) -> Vec<Interval>;
    /// TODO: doc
    fn convert_next(&self, segment: &ChineseSequence, next: usize) -> Vec<Interval>;
}

mod chewing_conversion;
pub use chewing_conversion::ChewingConversionEngine;
