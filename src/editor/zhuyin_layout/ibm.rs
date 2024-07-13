//! IBM keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use crate::{
    editor::keyboard::{KeyEvent, KeyIndex},
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

use super::{KeyBehavior, SyllableEditor};

/// TODO: docs
#[derive(Debug, Clone, Copy)]
pub struct Ibm {
    syllable: Syllable,
}

impl Ibm {
    /// TODO: docs
    pub fn new() -> Ibm {
        Ibm {
            syllable: Syllable::new(),
        }
    }
}

impl Default for Ibm {
    fn default() -> Self {
        Self::new()
    }
}

impl SyllableEditor for Ibm {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        let bopomofo = match key.index {
            KeyIndex::K1 => Bopomofo::B,
            KeyIndex::K2 => Bopomofo::P,
            KeyIndex::K3 => Bopomofo::M,
            KeyIndex::K4 => Bopomofo::F,
            KeyIndex::K5 => Bopomofo::D,
            KeyIndex::K6 => Bopomofo::T,
            KeyIndex::K7 => Bopomofo::N,
            KeyIndex::K8 => Bopomofo::L,
            KeyIndex::K9 => Bopomofo::G,
            KeyIndex::K10 => Bopomofo::K,
            KeyIndex::K11 => Bopomofo::H,
            KeyIndex::K15 => Bopomofo::J,
            KeyIndex::K16 => Bopomofo::Q,
            KeyIndex::K17 => Bopomofo::X,
            KeyIndex::K18 => Bopomofo::S,
            KeyIndex::K19 => Bopomofo::CH,
            KeyIndex::K20 => Bopomofo::SH,
            KeyIndex::K21 => Bopomofo::R,
            KeyIndex::K22 => Bopomofo::Z,
            KeyIndex::K23 => Bopomofo::C,
            KeyIndex::K24 => Bopomofo::S,
            KeyIndex::K27 => Bopomofo::I,
            KeyIndex::K28 => Bopomofo::U,
            KeyIndex::K29 => Bopomofo::IU,
            KeyIndex::K30 => Bopomofo::A,
            KeyIndex::K31 => Bopomofo::O,
            KeyIndex::K32 => Bopomofo::E,
            KeyIndex::K33 => Bopomofo::EH,
            KeyIndex::K34 => Bopomofo::AI,
            KeyIndex::K35 => Bopomofo::EI,
            KeyIndex::K36 => Bopomofo::AU,
            KeyIndex::K38 => Bopomofo::OU,
            KeyIndex::K39 => Bopomofo::AN,
            KeyIndex::K40 => Bopomofo::EN,
            KeyIndex::K41 => Bopomofo::ANG,
            KeyIndex::K42 => Bopomofo::ENG,
            KeyIndex::K43 => Bopomofo::ER,
            KeyIndex::K44 => Bopomofo::TONE2,
            KeyIndex::K45 => Bopomofo::TONE3,
            KeyIndex::K46 => Bopomofo::TONE4,
            KeyIndex::K47 => Bopomofo::TONE5,
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

    use super::Ibm;

    #[test]
    fn space() {
        let mut editor = Ibm::new();
        let keyboard = Qwerty;
        let behavior =
            editor.key_press(keyboard.map_with_mod(KeyCode::Space, Modifiers::default()));
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
