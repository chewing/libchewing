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

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

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

use super::keyboard::KeyEvent;

mod dc26;
mod et;
mod et26;
mod ginyieh;
mod hsu;
mod ibm;
mod pinyin;
mod standard;

/// TODO: docs
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum KeyboardLayoutCompat {
    /// TODO: docs
    Default = 0,
    /// TODO: docs
    Hsu,
    /// TODO: docs
    Ibm,
    /// TODO: docs
    GinYieh,
    /// TODO: docs
    Et,
    /// TODO: docs
    Et26,
    /// TODO: docs
    Dvorak,
    /// TODO: docs
    DvorakHsu,
    /// TODO: docs
    DachenCp26,
    /// TODO: docs
    HanyuPinyin,
    /// TODO: docs
    ThlPinyin,
    /// TODO: docs
    Mps2Pinyin,
    /// TODO: docs
    Carpalx,
    /// TODO: docs
    ColemakDhAnsi,
    /// TODO: docs
    ColemakDhOrth,
    /// Workman standard layout
    Workman,
    /// TODO: docs
    Colemak,
}

#[derive(Debug)]
pub struct ParseKeyboardLayoutError;

impl Display for ParseKeyboardLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to parse keyboard layout")
    }
}

impl FromStr for KeyboardLayoutCompat {
    type Err = ParseKeyboardLayoutError;

    fn from_str(kb_str: &str) -> Result<Self, Self::Err> {
        let layout = match kb_str {
            "KB_DEFAULT" => Self::Default,
            "KB_HSU" => Self::Hsu,
            "KB_IBM" => Self::Ibm,
            "KB_GIN_YIEH" => Self::GinYieh,
            "KB_ET" => Self::Et,
            "KB_ET26" => Self::Et26,
            "KB_DVORAK" => Self::Dvorak,
            "KB_DVORAK_HSU" => Self::DvorakHsu,
            "KB_DACHEN_CP26" => Self::DachenCp26,
            "KB_HANYU_PINYIN" => Self::HanyuPinyin,
            "KB_THL_PINYIN" => Self::ThlPinyin,
            "KB_MPS2_PINYIN" => Self::Mps2Pinyin,
            "KB_CARPALX" => Self::Carpalx,
            "KB_COLEMAK" => Self::Colemak,
            "KB_COLEMAK_DH_ANSI" => Self::ColemakDhAnsi,
            "KB_COLEMAK_DH_ORTH" => Self::ColemakDhOrth,
            "KB_WORKMAN" => Self::Workman,
            _ => return Err(ParseKeyboardLayoutError),
        };
        Ok(layout)
    }
}

impl Display for KeyboardLayoutCompat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyboardLayoutCompat::Default => f.write_str("KB_DEFAULT"),
            KeyboardLayoutCompat::Hsu => f.write_str("KB_HSU"),
            KeyboardLayoutCompat::Ibm => f.write_str("KB_IBM"),
            KeyboardLayoutCompat::GinYieh => f.write_str("KB_GIN_YIEH"),
            KeyboardLayoutCompat::Et => f.write_str("KB_ET"),
            KeyboardLayoutCompat::Et26 => f.write_str("KB_ET26"),
            KeyboardLayoutCompat::Dvorak => f.write_str("KB_DVORAK"),
            KeyboardLayoutCompat::DvorakHsu => f.write_str("KB_DVORAK_HSU"),
            KeyboardLayoutCompat::DachenCp26 => f.write_str("KB_DACHEN_CP26"),
            KeyboardLayoutCompat::HanyuPinyin => f.write_str("KB_HANYU_PINYIN"),
            KeyboardLayoutCompat::ThlPinyin => f.write_str("KB_THL_PINYIN"),
            KeyboardLayoutCompat::Mps2Pinyin => f.write_str("KB_MPS2_PINYIN"),
            KeyboardLayoutCompat::Carpalx => f.write_str("KB_CARPALX"),
            KeyboardLayoutCompat::Colemak => f.write_str("KB_COLEMAK"),
            KeyboardLayoutCompat::ColemakDhAnsi => f.write_str("KB_COLEMAK_DH_ANSI"),
            KeyboardLayoutCompat::ColemakDhOrth => f.write_str("KB_COLEMAK_DH_ORTH"),
            KeyboardLayoutCompat::Workman => f.write_str("KB_WORKMAN"),
        }
    }
}

impl TryFrom<u8> for KeyboardLayoutCompat {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Default,
            1 => Self::Hsu,
            2 => Self::Ibm,
            3 => Self::GinYieh,
            4 => Self::Et,
            5 => Self::Et26,
            6 => Self::Dvorak,
            7 => Self::DvorakHsu,
            8 => Self::DachenCp26,
            9 => Self::HanyuPinyin,
            10 => Self::ThlPinyin,
            11 => Self::Mps2Pinyin,
            12 => Self::Carpalx,
            13 => Self::ColemakDhAnsi,
            14 => Self::ColemakDhOrth,
            15 => Self::Workman,
            16 => Self::Colemak,
            _ => return Err(()),
        })
    }
}

/// TODO: docs
/// TODO: move this to the editor module
#[derive(Debug, PartialEq)]
pub enum KeyBehavior {
    /// TODO: docs
    Ignore,
    /// TODO: docs
    Absorb,
    /// TODO: docs
    Commit,
    /// TODO: docs
    KeyError,
    /// TODO: docs
    Error,
    /// TODO: docs
    NoWord,
    /// TODO: docs
    OpenSymbolTable,
    /// Fuzzed Syllable
    Fuzzy(Syllable),
}

/// TODO: docs
pub trait SyllableEditor: Debug {
    /// Handles a key press event and returns the behavior of the layout.
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior;
    /// Handles a key press event and returns the behavior of the layout.
    ///
    /// If a syllable is completed prematurely due to fuzzy logic, a
    /// `Fuzzy(Syllable)` will be returned.
    fn fuzzy_key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        if self.is_empty() {
            return self.key_press(key);
        }
        let mut clone = self.clone();
        clone.clear();
        clone.key_press(key);
        let current_syl = self.read();
        let new_syl = clone.read();
        if current_syl.has_initial() && new_syl.has_initial()
            || current_syl.has_medial() && (new_syl.has_initial() || new_syl.has_medial())
            || current_syl.has_rime()
                && (new_syl.has_initial() || new_syl.has_medial() || new_syl.has_rime())
        {
            let ret = KeyBehavior::Fuzzy(current_syl);
            self.clear();
            self.key_press(key);
            return ret;
        }
        self.key_press(key)
    }
    /// Removes the last input from the buffer.
    fn remove_last(&mut self);
    /// Clears the phonetic key buffer, removing all values.
    fn clear(&mut self);
    /// Returns true if the editor buffer is empty.
    fn is_empty(&self) -> bool;
    /// Returns the current syllable without changing the editor buffer.
    fn read(&self) -> Syllable;
    /// Returns the current key seq buffer as printable string, if supported by the layout.
    fn key_seq(&self) -> Option<String> {
        None
    }
    /// Returns the alternative syllable, if supported by the layout.
    fn alt_syllables(&self, syl: Syllable) -> &[Syllable] {
        let _ = syl;
        &[]
    }
    // Returns a copy of the SyllableEditor
    fn clone(&self) -> Box<dyn SyllableEditor>;
}
