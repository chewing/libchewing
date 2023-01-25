use crate::zhuyin::Syllable;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Interval {
    pub start: usize,
    pub end: usize,
    pub phrase: String,
}

impl Interval {
    pub fn contains(&self, other: &Interval) -> bool {
        self.start <= other.start && self.end >= other.end
    }
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug)]
pub struct Break(pub usize);

#[derive(Debug)]
pub struct ChineseSequence {
    pub syllables: Vec<Syllable>,
    pub selections: Vec<Interval>,
    pub breaks: Vec<Break>,
}

pub trait ConversionEngine {
    fn convert(&self, segment: &ChineseSequence) -> Vec<Interval>;
    fn convert_next(&self, segment: &ChineseSequence, next: usize) -> Vec<Interval>;
}

mod chewing_conversion;
pub use chewing_conversion::ChewingConversionEngine;
