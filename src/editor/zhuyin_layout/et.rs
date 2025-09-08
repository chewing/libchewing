//! ET41 keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use crate::{
    editor::keyboard::{KeyEvent, Keycode},
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
        let bopomofo = match key.code {
            Keycode::K1 => Bopomofo::TONE5,
            Keycode::K2 => Bopomofo::TONE2,
            Keycode::K3 => Bopomofo::TONE3,
            Keycode::K4 => Bopomofo::TONE4,
            Keycode::K7 => Bopomofo::Q,
            Keycode::K8 => Bopomofo::AN,
            Keycode::K9 => Bopomofo::EN,
            Keycode::K10 => Bopomofo::ANG,
            Keycode::K11 => Bopomofo::ENG,
            Keycode::K12 => Bopomofo::ER,
            Keycode::K15 => Bopomofo::EI,
            Keycode::K16 => Bopomofo::EH,
            Keycode::K17 => Bopomofo::I,
            Keycode::K18 => Bopomofo::E,
            Keycode::K19 => Bopomofo::T,
            Keycode::K20 => Bopomofo::OU,
            Keycode::K21 => Bopomofo::IU,
            Keycode::K22 => Bopomofo::AI,
            Keycode::K23 => Bopomofo::O,
            Keycode::K24 => Bopomofo::P,
            Keycode::K27 => Bopomofo::A,
            Keycode::K28 => Bopomofo::S,
            Keycode::K29 => Bopomofo::D,
            Keycode::K30 => Bopomofo::F,
            Keycode::K31 => Bopomofo::J,
            Keycode::K32 => Bopomofo::H,
            Keycode::K33 => Bopomofo::R,
            Keycode::K34 => Bopomofo::K,
            Keycode::K35 => Bopomofo::L,
            Keycode::K36 => Bopomofo::Z,
            Keycode::K37 => Bopomofo::C,
            Keycode::K38 => Bopomofo::AU,
            Keycode::K39 => Bopomofo::U,
            Keycode::K40 => Bopomofo::X,
            Keycode::K41 => Bopomofo::G,
            Keycode::K42 => Bopomofo::B,
            Keycode::K43 => Bopomofo::N,
            Keycode::K44 => Bopomofo::M,
            Keycode::K45 => Bopomofo::ZH,
            Keycode::K46 => Bopomofo::CH,
            Keycode::K47 => Bopomofo::SH,
            Keycode::K48 => Bopomofo::TONE1,
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
        keyboard::{KeyboardLayout, Keysym, Modifiers, Qwerty},
        zhuyin_layout::{KeyBehavior, SyllableEditor},
    };

    use super::Et;

    #[test]
    fn space() {
        let mut editor = Et::new();
        let keyboard = Qwerty;
        let behavior = editor.key_press(keyboard.map_with_mod(Keysym::Space, Modifiers::default()));
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
