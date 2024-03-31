//! Algorithms to convert syllables to Chinese characters.

mod chewing;
mod symbol;

use std::{
    borrow::Cow,
    cmp::{max, min},
    fmt::Debug,
};

use crate::zhuyin::{Syllable, SyllableSlice};

pub use self::chewing::ChewingEngine;
pub(crate) use self::symbol::{full_width_symbol_input, special_symbol_input};

/// TODO: doc
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Interval {
    /// TODO: doc
    pub start: usize,
    /// TODO: doc
    pub end: usize,
    // TODO doc
    pub is_phrase: bool,
    /// TODO: doc
    pub phrase: Box<str>,
}

impl Debug for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("I")
            .field(&(self.start..self.end))
            .field(&self.phrase)
            .finish()
    }
}

impl Interval {
    /// TODO: doc
    pub fn contains(&self, other: &Interval) -> bool {
        self.contains_range(other.start, other.end)
    }
    fn contains_range(&self, start: usize, end: usize) -> bool {
        self.start <= start && self.end >= end
    }
    fn is_contained_by(&self, start: usize, end: usize) -> bool {
        start <= self.start && end >= self.end
    }
    pub fn intersect(&self, other: &Interval) -> bool {
        self.intersect_range(other.start, other.end)
    }
    fn intersect_range(&self, start: usize, end: usize) -> bool {
        max(self.start, start) < min(self.end, end)
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

/// A smallest unit of input in the pre-edit buffer.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum SymbolKind {
    /// Chinese syllable
    Syllable(Syllable),
    /// Any direct character
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GapKind {
    // Beginning of Buffer.
    BOB,
    Break,
    Glue,
    Normal,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol {
    kind: SymbolKind,
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            SymbolKind::Syllable(syl) => f.debug_tuple("S").field(&syl.to_string()).finish(),
            SymbolKind::Char(ch) => f.debug_tuple("C").field(&ch).finish(),
        }
    }
}

impl Symbol {
    pub fn new_syl(sym: Syllable) -> Symbol {
        Symbol {
            kind: SymbolKind::Syllable(sym),
        }
    }
    pub fn new_char(sym: char) -> Symbol {
        Symbol {
            kind: SymbolKind::Char(sym),
        }
    }
    pub fn is_syllable(&self) -> bool {
        match self.kind {
            SymbolKind::Syllable(_) => true,
            SymbolKind::Char(_) => false,
        }
    }
    // FIXME return Result<Syllable>
    pub fn as_syllable(&self) -> Syllable {
        match self.kind {
            SymbolKind::Syllable(syllable) => syllable,
            SymbolKind::Char(_) => panic!(),
        }
    }
    pub fn is_char(&self) -> bool {
        match self.kind {
            SymbolKind::Syllable(_) => false,
            SymbolKind::Char(_) => true,
        }
    }
    // FIXME return Result<char>
    pub fn as_char(&self) -> char {
        match self.kind {
            SymbolKind::Syllable(_) => panic!(),
            SymbolKind::Char(c) => c,
        }
    }
}

impl SyllableSlice for &[Symbol] {
    fn as_slice(&self) -> Cow<'static, [Syllable]> {
        self.iter()
            .map_while(|sym| match sym.kind {
                SymbolKind::Syllable(syl) => Some(syl),
                SymbolKind::Char(_) => None,
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl SyllableSlice for Vec<Symbol> {
    fn as_slice(&self) -> Cow<'static, [Syllable]> {
        self.iter()
            .map_while(|sym| match sym.kind {
                SymbolKind::Syllable(syl) => Some(syl),
                SymbolKind::Char(_) => None,
            })
            .collect::<Vec<_>>()
            .into()
    }
}

/// TODO: doc
#[derive(Debug, Default, Clone)]
pub struct Composition {
    /// TODO: doc
    symbols: Vec<Symbol>,
    gaps: Vec<GapKind>,
    /// TODO: doc
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
    pub fn gap(&self, index: usize) -> Option<GapKind> {
        if index >= self.len() {
            return None;
        }
        Some(self.gaps[index])
    }
    pub fn gap_after(&self, index: usize) -> Option<GapKind> {
        if index + 1 >= self.len() {
            return None;
        }
        Some(self.gaps[index + 1])
    }
    pub fn set_gap(&mut self, index: usize, gap: GapKind) {
        assert!(index < self.len());
        assert_ne!(gap, GapKind::BOB);
        if index == 0 {
            return;
        }
        if gap == GapKind::Break {
            let mut to_remove = vec![];
            for (i, selection) in self.selections.iter_mut().enumerate() {
                if selection.start < index && index < selection.end {
                    to_remove.push(i);
                }
            }
            for i in to_remove {
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
        for i in to_remove {
            self.selections.swap_remove(i);
        }
        self.symbols.insert(index, sym);
        if !self.gaps.is_empty() && index != self.gaps.len() {
            self.gaps[index] = GapKind::Normal;
        }
        self.gaps.insert(index, GapKind::Normal);
        self.gaps[0] = GapKind::BOB;
    }
    pub fn replace(&mut self, index: usize, sym: Symbol) {
        assert!(index < self.len());
        self.symbols[index] = sym;
        self.set_gap(index, GapKind::Normal);
    }
    pub fn push_selection(&mut self, interval: Interval) {
        assert!(interval.end <= self.len());
        let mut to_remove = vec![];
        for (i, selection) in self.selections.iter().enumerate() {
            if selection.intersect(&interval) {
                to_remove.push(i);
            }
        }
        for i in to_remove {
            self.selections.swap_remove(i);
        }
        for i in (interval.start..interval.end).skip(1) {
            self.gaps[i] = GapKind::Normal;
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
        for i in to_remove {
            self.selections.swap_remove(i);
        }
        self.symbols.drain(0..n);
        self.gaps.drain(0..n);
        if !self.gaps.is_empty() {
            self.gaps[0] = GapKind::BOB;
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
        for i in to_remove {
            self.selections.swap_remove(i);
        }
        self.symbols.remove(index);
        self.gaps.remove(index);
        if !self.gaps.is_empty() {
            self.gaps[0] = GapKind::BOB;
        }
    }
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.gaps.clear();
        self.selections.clear();
    }
}

/// TODO: doc
pub trait ConversionEngine<C: ?Sized> {
    /// TODO: doc, combine convert_next as an Iterator
    fn convert(&self, context: &C, composition: &Composition) -> Vec<Interval>;
    /// TODO: doc
    fn convert_next(&self, context: &C, composition: &Composition, next: usize) -> Vec<Interval>;
}
