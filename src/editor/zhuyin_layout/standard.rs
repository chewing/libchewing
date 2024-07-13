//! Standard keyboard layout
//!
//! Also known as the Dai Chien (大千) layout. It's the default layout on almost
//! all platforms and the most commonly used one.

use crate::{
    editor::keyboard::{KeyEvent, KeyIndex},
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

use super::{KeyBehavior, SyllableEditor};

/// TODO: docs
#[derive(Debug, Clone, Copy)]
pub struct Standard {
    syllable: Syllable,
}

impl Standard {
    /// TODO: docs
    pub fn new() -> Standard {
        Standard {
            syllable: Syllable::new(),
        }
    }
}

impl Default for Standard {
    fn default() -> Self {
        Self::new()
    }
}

impl SyllableEditor for Standard {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        let bopomofo = match key.index {
            KeyIndex::K1 => Bopomofo::B,
            KeyIndex::K2 => Bopomofo::D,
            KeyIndex::K3 => Bopomofo::TONE3,
            KeyIndex::K4 => Bopomofo::TONE4,
            KeyIndex::K5 => Bopomofo::ZH,
            KeyIndex::K6 => Bopomofo::TONE2,
            KeyIndex::K7 => Bopomofo::TONE5,
            KeyIndex::K8 => Bopomofo::A,
            KeyIndex::K9 => Bopomofo::AI,
            KeyIndex::K10 => Bopomofo::AN,
            KeyIndex::K11 => Bopomofo::ER,
            KeyIndex::K15 => Bopomofo::P,
            KeyIndex::K16 => Bopomofo::T,
            KeyIndex::K17 => Bopomofo::G,
            KeyIndex::K18 => Bopomofo::J,
            KeyIndex::K19 => Bopomofo::CH,
            KeyIndex::K20 => Bopomofo::Z,
            KeyIndex::K21 => Bopomofo::I,
            KeyIndex::K22 => Bopomofo::O,
            KeyIndex::K23 => Bopomofo::EI,
            KeyIndex::K24 => Bopomofo::EN,
            KeyIndex::K27 => Bopomofo::M,
            KeyIndex::K28 => Bopomofo::N,
            KeyIndex::K29 => Bopomofo::K,
            KeyIndex::K30 => Bopomofo::Q,
            KeyIndex::K31 => Bopomofo::SH,
            KeyIndex::K32 => Bopomofo::C,
            KeyIndex::K33 => Bopomofo::U,
            KeyIndex::K34 => Bopomofo::E,
            KeyIndex::K35 => Bopomofo::AU,
            KeyIndex::K36 => Bopomofo::ANG,
            KeyIndex::K38 => Bopomofo::F,
            KeyIndex::K39 => Bopomofo::L,
            KeyIndex::K40 => Bopomofo::H,
            KeyIndex::K41 => Bopomofo::X,
            KeyIndex::K42 => Bopomofo::R,
            KeyIndex::K43 => Bopomofo::S,
            KeyIndex::K44 => Bopomofo::IU,
            KeyIndex::K45 => Bopomofo::EH,
            KeyIndex::K46 => Bopomofo::OU,
            KeyIndex::K47 => Bopomofo::ENG,
            KeyIndex::K48 => Bopomofo::TONE1,
            _ => return KeyBehavior::KeyError,
        };
        if bopomofo.kind() == BopomofoKind::Tone {
            if !self.syllable.is_empty() {
                if bopomofo != Bopomofo::TONE1 {
                    self.syllable.update(bopomofo);
                }
                return KeyBehavior::Commit;
            }
        } else {
            self.syllable.remove_tone();
        }

        // In C libchewing TONE1 / Space is not a phonetic symbol
        if bopomofo == Bopomofo::TONE1 {
            return KeyBehavior::KeyError;
        }

        self.syllable.update(bopomofo);
        KeyBehavior::Absorb
    }

    fn is_empty(&self) -> bool {
        self.syllable.is_empty()
    }

    fn remove_last(&mut self) {
        self.syllable.pop();
    }

    fn clear(&mut self) {
        self.syllable.clear()
    }

    fn read(&self) -> Syllable {
        self.syllable
    }

    fn key_seq(&self) -> Option<String> {
        None
    }

    fn clone(&self) -> Box<dyn SyllableEditor> {
        Box::new(Clone::clone(self))
    }
}

#[cfg(test)]
mod test {
    use crate::editor::{
        keyboard::{KeyCode, KeyboardLayout, Modifiers, Qwerty},
        zhuyin_layout::{KeyBehavior, SyllableEditor},
    };

    use super::Standard;

    #[test]
    fn space() {
        let mut editor = Standard::new();
        let keyboard = Qwerty;
        let behavior =
            editor.key_press(keyboard.map_with_mod(KeyCode::Space, Modifiers::default()));
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
