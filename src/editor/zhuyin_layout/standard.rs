//! Standard keyboard layout
//!
//! Also known as the Dai Chien (大千) layout. It's the default layout on almost
//! all platforms and the most commonly used one.

use crate::{
    editor::keyboard::{KeyEvent, Keycode},
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
        let bopomofo = match key.code {
            Keycode::K1 => Bopomofo::B,
            Keycode::K2 => Bopomofo::D,
            Keycode::K3 => Bopomofo::TONE3,
            Keycode::K4 => Bopomofo::TONE4,
            Keycode::K5 => Bopomofo::ZH,
            Keycode::K6 => Bopomofo::TONE2,
            Keycode::K7 => Bopomofo::TONE5,
            Keycode::K8 => Bopomofo::A,
            Keycode::K9 => Bopomofo::AI,
            Keycode::K10 => Bopomofo::AN,
            Keycode::K11 => Bopomofo::ER,
            Keycode::K15 => Bopomofo::P,
            Keycode::K16 => Bopomofo::T,
            Keycode::K17 => Bopomofo::G,
            Keycode::K18 => Bopomofo::J,
            Keycode::K19 => Bopomofo::CH,
            Keycode::K20 => Bopomofo::Z,
            Keycode::K21 => Bopomofo::I,
            Keycode::K22 => Bopomofo::O,
            Keycode::K23 => Bopomofo::EI,
            Keycode::K24 => Bopomofo::EN,
            Keycode::K27 => Bopomofo::M,
            Keycode::K28 => Bopomofo::N,
            Keycode::K29 => Bopomofo::K,
            Keycode::K30 => Bopomofo::Q,
            Keycode::K31 => Bopomofo::SH,
            Keycode::K32 => Bopomofo::C,
            Keycode::K33 => Bopomofo::U,
            Keycode::K34 => Bopomofo::E,
            Keycode::K35 => Bopomofo::AU,
            Keycode::K36 => Bopomofo::ANG,
            Keycode::K38 => Bopomofo::F,
            Keycode::K39 => Bopomofo::L,
            Keycode::K40 => Bopomofo::H,
            Keycode::K41 => Bopomofo::X,
            Keycode::K42 => Bopomofo::R,
            Keycode::K43 => Bopomofo::S,
            Keycode::K44 => Bopomofo::IU,
            Keycode::K45 => Bopomofo::EH,
            Keycode::K46 => Bopomofo::OU,
            Keycode::K47 => Bopomofo::ENG,
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

    use super::Standard;

    #[test]
    fn space() {
        let mut editor = Standard::new();
        let keyboard = Qwerty;
        let behavior =
            editor.key_press(keyboard.map_with_mod(Keysym::Space, Modifiers::default()));
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
