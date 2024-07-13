//! ET41 keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use crate::{
    editor::keyboard::{KeyEvent, KeyIndex},
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

use super::{KeyBehavior, SyllableEditor};

/// TODO: docs
#[derive(Debug, Clone, Copy)]
pub struct Et {
    syllable: Syllable,
}

impl Et {
    /// TODO: docs
    pub fn new() -> Et {
        Et {
            syllable: Syllable::new(),
        }
    }
}

impl Default for Et {
    fn default() -> Self {
        Self::new()
    }
}

impl SyllableEditor for Et {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        let bopomofo = match key.index {
            KeyIndex::K1 => Bopomofo::TONE5,
            KeyIndex::K2 => Bopomofo::TONE2,
            KeyIndex::K3 => Bopomofo::TONE3,
            KeyIndex::K4 => Bopomofo::TONE4,
            KeyIndex::K7 => Bopomofo::Q,
            KeyIndex::K8 => Bopomofo::AN,
            KeyIndex::K9 => Bopomofo::EN,
            KeyIndex::K10 => Bopomofo::ANG,
            KeyIndex::K11 => Bopomofo::ENG,
            KeyIndex::K12 => Bopomofo::ER,
            KeyIndex::K15 => Bopomofo::EI,
            KeyIndex::K16 => Bopomofo::EH,
            KeyIndex::K17 => Bopomofo::I,
            KeyIndex::K18 => Bopomofo::E,
            KeyIndex::K19 => Bopomofo::T,
            KeyIndex::K20 => Bopomofo::OU,
            KeyIndex::K21 => Bopomofo::IU,
            KeyIndex::K22 => Bopomofo::AI,
            KeyIndex::K23 => Bopomofo::O,
            KeyIndex::K24 => Bopomofo::P,
            KeyIndex::K27 => Bopomofo::A,
            KeyIndex::K28 => Bopomofo::S,
            KeyIndex::K29 => Bopomofo::D,
            KeyIndex::K30 => Bopomofo::F,
            KeyIndex::K31 => Bopomofo::J,
            KeyIndex::K32 => Bopomofo::H,
            KeyIndex::K33 => Bopomofo::R,
            KeyIndex::K34 => Bopomofo::K,
            KeyIndex::K35 => Bopomofo::L,
            KeyIndex::K36 => Bopomofo::Z,
            KeyIndex::K37 => Bopomofo::C,
            KeyIndex::K38 => Bopomofo::AU,
            KeyIndex::K39 => Bopomofo::U,
            KeyIndex::K40 => Bopomofo::X,
            KeyIndex::K41 => Bopomofo::G,
            KeyIndex::K42 => Bopomofo::B,
            KeyIndex::K43 => Bopomofo::N,
            KeyIndex::K44 => Bopomofo::M,
            KeyIndex::K45 => Bopomofo::ZH,
            KeyIndex::K46 => Bopomofo::CH,
            KeyIndex::K47 => Bopomofo::SH,
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

    use super::Et;

    #[test]
    fn space() {
        let mut editor = Et::new();
        let keyboard = Qwerty;
        let behavior =
            editor.key_press(keyboard.map_with_mod(KeyCode::Space, Modifiers::default()));
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
