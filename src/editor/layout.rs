//! Conversion from key events to phonetic keys
//!
//! This module contains engines for phonetic key conversions.
//!
//! Traditionally the keyboards sold in Chinese speaking region have
//! both the English alphabets and Zhuyin symbols printed on the keys.
//! Like English keyboards can have different layouts (QWERTY, Dvorak, etc.),
//! Zhuyin keyboards also have different layouts.
//!
//! The most widely used Zhuyin layout is the one directly printed on the keyboards.
//! It is a one to one mapping from keys to Zhuyin symbols. However, some layouts
//! have smarter mapping from keys to Zhuyin symbols, taking advantage of impossible
//! combinations, to reduce the total keys required.
//!
//! Chewing currently supports the default layout, Hsu's layout, ET26 layout,
//! DaChen CP26 layout, and the Pinyin layout.

use std::fmt::Debug;

use crate::zhuyin::Syllable;

pub use self::{
    dc26::DaiChien26,
    et::Et,
    et26::Et26,
    ginyieh::GinYieh,
    hsu::Hsu,
    ibm::Ibm,
    pinyin::{Pinyin, PinyinVariant},
    standard::Standard,
};

use super::keymap::KeyEvent;

mod dc26;
mod et;
mod et26;
mod ginyieh;
mod hsu;
mod ibm;
mod pinyin;
mod standard;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum KeyboardLayoutCompat {
    Default = 0,
    Hsu,
    Ibm,
    GinYieh,
    Et,
    Et26,
    Dvorak,
    DvorakHsu,
    DachenCp26,
    HanyuPinyin,
    ThlPinyin,
    Mps2Pinyin,
    Carpalx,
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum KeyBehavior {
    Ignore = 0,
    Absorb,
    Commit,
    KeyError,
    Error,
    NoWord,
    OpenSymbolTable,
}

pub trait SyllableEditor {
    /// Handles a key press event and returns the behavior of the layout.
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior;
    /// Removes the last input from the buffer.
    fn remove_last(&mut self);
    /// Clears the phonetic key buffer, removing all values.
    fn clear(&mut self);
    /// Returns true if the editor buffer is empty.
    fn is_empty(&self) -> bool;
    /// Returns the current syllable without changing the editor buffer.
    fn read(&self) -> Syllable;
    /// Returns the current key seq buffer as printable string, if supported by the layout.
    fn key_seq(&self) -> Option<String>;
}
