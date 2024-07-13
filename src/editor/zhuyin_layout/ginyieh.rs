//! GinYieh keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use crate::{
    editor::keyboard::{KeyEvent, KeyIndex},
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

use super::{KeyBehavior, SyllableEditor};

/// TODO: docs
#[derive(Debug, Clone, Copy)]
pub struct GinYieh {
    syllable: Syllable,
}

impl GinYieh {
    /// TODO: docs
    pub fn new() -> GinYieh {
        GinYieh {
            syllable: Syllable::new(),
        }
    }
}

impl Default for GinYieh {
    fn default() -> Self {
        Self::new()
    }
}

impl SyllableEditor for GinYieh {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        let bopomofo = match key.index {
            KeyIndex::K1 => Bopomofo::TONE5,
            KeyIndex::K2 => Bopomofo::B,
            KeyIndex::K3 => Bopomofo::D,
            KeyIndex::K6 => Bopomofo::ZH,
            KeyIndex::K8 => Bopomofo::A,
            KeyIndex::K9 => Bopomofo::AI,
            KeyIndex::K10 => Bopomofo::AN,
            KeyIndex::K11 => Bopomofo::I,
            KeyIndex::K12 => Bopomofo::ER,
            KeyIndex::K15 => Bopomofo::TONE2,
            KeyIndex::K16 => Bopomofo::P,
            KeyIndex::K17 => Bopomofo::T,
            KeyIndex::K18 => Bopomofo::G,
            KeyIndex::K19 => Bopomofo::J,
            KeyIndex::K20 => Bopomofo::CH,
            KeyIndex::K21 => Bopomofo::Z,
            KeyIndex::K22 => Bopomofo::O,
            KeyIndex::K23 => Bopomofo::EI,
            KeyIndex::K24 => Bopomofo::EN,
            KeyIndex::K27 => Bopomofo::TONE3,
            KeyIndex::K28 => Bopomofo::M,
            KeyIndex::K29 => Bopomofo::N,
            KeyIndex::K30 => Bopomofo::K,
            KeyIndex::K31 => Bopomofo::Q,
            KeyIndex::K32 => Bopomofo::SH,
            KeyIndex::K33 => Bopomofo::C,
            KeyIndex::K34 => Bopomofo::E,
            KeyIndex::K35 => Bopomofo::OU,
            KeyIndex::K36 => Bopomofo::ANG,
            KeyIndex::K37 => Bopomofo::IU,
            KeyIndex::K38 => Bopomofo::TONE4,
            KeyIndex::K39 => Bopomofo::F,
            KeyIndex::K40 => Bopomofo::L,
            KeyIndex::K41 => Bopomofo::H,
            KeyIndex::K42 => Bopomofo::X,
            KeyIndex::K43 => Bopomofo::R,
            KeyIndex::K44 => Bopomofo::S,
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

    use super::GinYieh;

    #[test]
    fn space() {
        let mut editor = GinYieh::new();
        let keyboard = Qwerty;
        let behavior =
            editor.key_press(keyboard.map_with_mod(KeyCode::Space, Modifiers::default()));
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
