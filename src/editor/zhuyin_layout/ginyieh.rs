//! GinYieh keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use crate::{
    editor::keyboard::{KeyEvent, Keycode},
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
        let bopomofo = match key.code {
            Keycode::K1 => Bopomofo::TONE5,
            Keycode::K2 => Bopomofo::B,
            Keycode::K3 => Bopomofo::D,
            Keycode::K6 => Bopomofo::ZH,
            Keycode::K8 => Bopomofo::A,
            Keycode::K9 => Bopomofo::AI,
            Keycode::K10 => Bopomofo::AN,
            Keycode::K11 => Bopomofo::I,
            Keycode::K12 => Bopomofo::ER,
            Keycode::K15 => Bopomofo::TONE2,
            Keycode::K16 => Bopomofo::P,
            Keycode::K17 => Bopomofo::T,
            Keycode::K18 => Bopomofo::G,
            Keycode::K19 => Bopomofo::J,
            Keycode::K20 => Bopomofo::CH,
            Keycode::K21 => Bopomofo::Z,
            Keycode::K22 => Bopomofo::O,
            Keycode::K23 => Bopomofo::EI,
            Keycode::K24 => Bopomofo::EN,
            Keycode::K25 => Bopomofo::U,
            Keycode::K27 => Bopomofo::TONE3,
            Keycode::K28 => Bopomofo::M,
            Keycode::K29 => Bopomofo::N,
            Keycode::K30 => Bopomofo::K,
            Keycode::K31 => Bopomofo::Q,
            Keycode::K32 => Bopomofo::SH,
            Keycode::K33 => Bopomofo::C,
            Keycode::K34 => Bopomofo::E,
            Keycode::K35 => Bopomofo::AU,
            Keycode::K36 => Bopomofo::ANG,
            Keycode::K37 => Bopomofo::IU,
            Keycode::K38 => Bopomofo::TONE4,
            Keycode::K39 => Bopomofo::F,
            Keycode::K40 => Bopomofo::L,
            Keycode::K41 => Bopomofo::H,
            Keycode::K42 => Bopomofo::X,
            Keycode::K43 => Bopomofo::R,
            Keycode::K44 => Bopomofo::S,
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

    use super::GinYieh;

    #[test]
    fn space() {
        let mut editor = GinYieh::new();
        let keyboard = Qwerty;
        let behavior =
            editor.key_press(keyboard.map_with_mod(Keysym::Space, Modifiers::default()));
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
