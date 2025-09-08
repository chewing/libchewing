//! IBM keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use crate::{
    editor::keyboard::{KeyEvent, Keycode},
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
        let bopomofo = match key.code {
            Keycode::K1 => Bopomofo::B,
            Keycode::K2 => Bopomofo::P,
            Keycode::K3 => Bopomofo::M,
            Keycode::K4 => Bopomofo::F,
            Keycode::K5 => Bopomofo::D,
            Keycode::K6 => Bopomofo::T,
            Keycode::K7 => Bopomofo::N,
            Keycode::K8 => Bopomofo::L,
            Keycode::K9 => Bopomofo::G,
            Keycode::K10 => Bopomofo::K,
            Keycode::K11 => Bopomofo::H,
            Keycode::K15 => Bopomofo::J,
            Keycode::K16 => Bopomofo::Q,
            Keycode::K17 => Bopomofo::X,
            Keycode::K18 => Bopomofo::ZH,
            Keycode::K19 => Bopomofo::CH,
            Keycode::K20 => Bopomofo::SH,
            Keycode::K21 => Bopomofo::R,
            Keycode::K22 => Bopomofo::Z,
            Keycode::K23 => Bopomofo::C,
            Keycode::K24 => Bopomofo::S,
            Keycode::K27 => Bopomofo::I,
            Keycode::K28 => Bopomofo::U,
            Keycode::K29 => Bopomofo::IU,
            Keycode::K30 => Bopomofo::A,
            Keycode::K31 => Bopomofo::O,
            Keycode::K32 => Bopomofo::E,
            Keycode::K33 => Bopomofo::EH,
            Keycode::K34 => Bopomofo::AI,
            Keycode::K35 => Bopomofo::EI,
            Keycode::K36 => Bopomofo::AU,
            Keycode::K38 => Bopomofo::OU,
            Keycode::K39 => Bopomofo::AN,
            Keycode::K40 => Bopomofo::EN,
            Keycode::K41 => Bopomofo::ANG,
            Keycode::K42 => Bopomofo::ENG,
            Keycode::K43 => Bopomofo::ER,
            Keycode::K44 => Bopomofo::TONE2,
            Keycode::K45 => Bopomofo::TONE3,
            Keycode::K46 => Bopomofo::TONE4,
            Keycode::K47 => Bopomofo::TONE5,
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
        keyboard::{Keysym, KeyboardLayout, Modifiers, Qwerty},
        zhuyin_layout::{KeyBehavior, SyllableEditor},
    };

    use super::Ibm;

    #[test]
    fn space() {
        let mut editor = Ibm::new();
        let keyboard = Qwerty;
        let behavior =
            editor.key_press(keyboard.map_with_mod(Keysym::Space, Modifiers::default()));
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
