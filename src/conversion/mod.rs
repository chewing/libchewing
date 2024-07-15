//! Algorithms to convert syllables to Chinese characters.

mod chewing;
mod fuzzy;
mod simple;
mod symbol;

use std::{
    borrow::Cow,
    cmp::{max, min},
    fmt::Debug,
};

use crate::{
    dictionary::Dictionary,
    zhuyin::{Syllable, SyllableSlice},
};

pub use self::chewing::ChewingEngine;
pub use self::fuzzy::FuzzyChewingEngine;
pub use self::simple::SimpleEngine;
pub(crate) use self::symbol::{full_width_symbol_input, special_symbol_input};

/// Converts a composition buffer to list of intervals.
///
/// [`Composition`] contains all user inputs and selection information. The out
/// put intervals should cover the whole range of inputs, sorted in first in
/// first out order.
pub trait ConversionEngine: Debug {
    fn convert<'a>(
        &'a self,
        dict: &'a dyn Dictionary,
        comp: &'a Composition,
    ) -> Box<dyn Iterator<Item = Vec<Interval>> + 'a>;
}

/// Output of conversion.
///
/// Interval represents a segment of input buffer converted to a phrase.
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Interval {
    /// The starting offset of the interval.
    pub start: usize,
    /// The end (exclusive) of the interval.
    pub end: usize,
    /// Whether the output is a phrase from dictionary or just symbols.
    pub is_phrase: bool,
    /// The output string.
    pub str: Box<str>,
}

impl Debug for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("I")
            .field(&(self.start..self.end))
            .field(&self.str)
            .finish()
    }
}

impl Interval {
    /// Whether the interval covers the whole range of the other interval.
    pub fn contains(&self, other: &Interval) -> bool {
        self.contains_range(other.start, other.end)
    }
    fn contains_range(&self, start: usize, end: usize) -> bool {
        self.start <= start && self.end >= end
    }
    fn is_contained_by(&self, start: usize, end: usize) -> bool {
        start <= self.start && end >= self.end
    }
    /// Whether the interval covers the part of the other interval.
    pub fn intersect(&self, other: &Interval) -> bool {
        self.intersect_range(other.start, other.end)
    }
    fn intersect_range(&self, start: usize, end: usize) -> bool {
        max(self.start, start) < min(self.end, end)
    }
    /// The length of the interval.
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    /// Whether the interval is empty (no output).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Represents the gap between symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Gap {
    /// Beginning of Buffer.
    Begin,
    /// Explicitly marked break point.
    Break,
    /// Explicitly marked connection point.
    Glue,
    /// Normal, default, gap.
    Normal,
}

/// A smallest unit of input in the pre-edit buffer.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Symbol {
    /// Chinese syllable
    Syllable(Syllable),
    /// Any direct character
    Char(char),
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Syllable(syl) => f.debug_tuple("S").field(&syl.to_string()).finish(),
            Symbol::Char(ch) => f.debug_tuple("C").field(&ch).finish(),
        }
    }
}

impl Symbol {
    pub fn is_syllable(&self) -> bool {
        matches!(self, Symbol::Syllable(_))
    }
    pub fn is_char(&self) -> bool {
        matches!(self, Symbol::Char(_))
    }
    pub fn to_syllable(self) -> Option<Syllable> {
        match self {
            Symbol::Syllable(syllable) => Some(syllable),
            Symbol::Char(_) => None,
        }
    }
    pub fn to_char(self) -> Option<char> {
        match self {
            Symbol::Syllable(_) => None,
            Symbol::Char(c) => Some(c),
        }
    }
}

impl From<Syllable> for Symbol {
    fn from(value: Syllable) -> Self {
        Symbol::Syllable(value)
    }
}

impl From<char> for Symbol {
    fn from(value: char) -> Self {
        Symbol::Char(value)
    }
}

impl SyllableSlice for &[Symbol] {
    fn to_slice(&self) -> Cow<'static, [Syllable]> {
        self.iter()
            .map_while(|&sym| match sym {
                Symbol::Syllable(syl) => Some(syl),
                Symbol::Char(_) => None,
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl SyllableSlice for Vec<Symbol> {
    fn to_slice(&self) -> Cow<'static, [Syllable]> {
        self.iter()
            .map_while(|&sym| match sym {
                Symbol::Syllable(syl) => Some(syl),
                Symbol::Char(_) => None,
            })
            .collect::<Vec<_>>()
            .into()
    }
}

/// Input data collected by the Editor.
#[derive(Debug, Default, Clone)]
pub struct Composition {
    /// Pre-edit inputs either syllables or symbols.
    symbols: Vec<Symbol>,
    /// User indicates offset that shouldn't form a phrase.
    gaps: Vec<Gap>,
    /// User set constraint on that output must match.
    selections: Vec<Interval>,
}

impl Composition {
    pub fn new() -> Composition {
        Default::default()
    }
    pub fn len(&self) -> usize {
        assert_eq!(self.symbols.len(), self.gaps.len());
        self.symbols.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Get the symbol under the cursor
    ///
    /// The cursor always indicates the gap between symbols.
    ///
    /// ```text
    /// |S0|S1|S2|S3|S4|S5|
    /// ^                 ^
    /// |                 `--- cursor 6 and end of buffer
    /// `--- cursor 0
    /// ```
    ///
    /// When returning the symbol under the cursor, we always
    /// return the symbol that has the same index with the cursor.
    /// So cursor 0 will return `Some(S0)` and cursor 6 will return
    /// `None`.
    ///
    /// When inserting a new symbol, it is always inserted to the
    /// gap indicated by the cursor. When cursor is at the end of the
    /// buffer, new symbols are appended.
    pub fn symbol(&self, index: usize) -> Option<Symbol> {
        if index >= self.len() {
            return None;
        }
        Some(self.symbols[index])
    }
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }
    pub fn selections(&self) -> &[Interval] {
        &self.selections
    }
    pub fn gap(&self, index: usize) -> Option<Gap> {
        if index >= self.len() {
            return None;
        }
        Some(self.gaps[index])
    }
    pub fn gap_after(&self, index: usize) -> Option<Gap> {
        if index + 1 >= self.len() {
            return None;
        }
        Some(self.gaps[index + 1])
    }
    pub fn set_gap(&mut self, index: usize, gap: Gap) {
        assert!(index < self.len());
        assert_ne!(gap, Gap::Begin);
        if index == 0 {
            return;
        }
        if gap == Gap::Break {
            let mut to_remove = vec![];
            for (i, selection) in self.selections.iter_mut().enumerate() {
                if selection.start < index && index < selection.end {
                    to_remove.push(i);
                }
            }
            for i in to_remove.into_iter().rev() {
                self.selections.swap_remove(i);
            }
        }
        self.gaps[index] = gap;
    }
    pub fn push(&mut self, sym: Symbol) {
        self.insert(self.len(), sym);
    }
    pub fn insert(&mut self, index: usize, sym: Symbol) {
        assert!(index <= self.len());
        let mut to_remove = vec![];
        for (i, selection) in self.selections.iter_mut().enumerate() {
            if selection.start < index && index < selection.end {
                to_remove.push(i);
            }
            if selection.start >= index {
                selection.start += 1;
                selection.end += 1;
            }
        }
        for i in to_remove.into_iter().rev() {
            self.selections.swap_remove(i);
        }
        self.symbols.insert(index, sym);
        if !self.gaps.is_empty() && index != self.gaps.len() {
            self.gaps[index] = Gap::Normal;
        }
        self.gaps.insert(index, Gap::Normal);
        self.gaps[0] = Gap::Begin;
    }
    pub fn replace(&mut self, index: usize, sym: Symbol) {
        assert!(index < self.len());
        self.symbols[index] = sym;
        self.set_gap(index, Gap::Normal);
    }
    pub fn push_selection(&mut self, interval: Interval) {
        assert!(interval.end <= self.len());
        let mut to_remove = vec![];
        for (i, selection) in self.selections.iter().enumerate() {
            if selection.intersect(&interval) {
                to_remove.push(i);
            }
        }
        for i in to_remove.into_iter().rev() {
            self.selections.swap_remove(i);
        }
        for i in (interval.start..interval.end).skip(1) {
            self.gaps[i] = Gap::Normal;
        }
        self.selections.push(interval);
    }
    pub fn remove_front(&mut self, n: usize) {
        assert!(n <= self.len());
        let mut to_remove = vec![];
        for (i, selection) in self.selections.iter_mut().enumerate() {
            if selection.start < n {
                to_remove.push(i);
            } else {
                selection.start -= n;
                selection.end -= n;
            }
        }
        for i in to_remove.into_iter().rev() {
            self.selections.swap_remove(i);
        }
        self.symbols.drain(0..n);
        self.gaps.drain(0..n);
        if !self.gaps.is_empty() {
            self.gaps[0] = Gap::Begin;
        }
    }
    pub fn remove(&mut self, index: usize) {
        assert!(index < self.len());
        let mut to_remove = vec![];
        for (i, selection) in self.selections.iter_mut().enumerate() {
            if selection.start <= index {
                if index < selection.end {
                    to_remove.push(i);
                }
            } else {
                selection.start -= 1;
                selection.end -= 1;
            }
        }
        for i in to_remove.into_iter().rev() {
            self.selections.swap_remove(i);
        }
        self.symbols.remove(index);
        self.gaps.remove(index);
        if !self.gaps.is_empty() {
            self.gaps[0] = Gap::Begin;
        }
    }
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.gaps.clear();
        self.selections.clear();
    }
}
