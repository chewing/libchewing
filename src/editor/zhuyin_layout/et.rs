//! ET41 keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use super::{KeyBehavior, SyllableEditor};
use crate::input::keycode::*;
use crate::{
    input::KeyboardEvent,
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

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
    fn key_press(&mut self, key: KeyboardEvent) -> KeyBehavior {
        let bopomofo = match key.code {
            KEY_1 => Bopomofo::TONE5,
            KEY_2 => Bopomofo::TONE2,
            KEY_3 => Bopomofo::TONE3,
            KEY_4 => Bopomofo::TONE4,
            KEY_7 => Bopomofo::Q,
            KEY_8 => Bopomofo::AN,
            KEY_9 => Bopomofo::EN,
            KEY_0 => Bopomofo::ANG,
            KEY_MINUS => Bopomofo::ENG,
            KEY_EQUAL => Bopomofo::ER,
            KEY_Q => Bopomofo::EI,
            KEY_W => Bopomofo::EH,
            KEY_E => Bopomofo::I,
            KEY_R => Bopomofo::E,
            KEY_T => Bopomofo::T,
            KEY_Y => Bopomofo::OU,
            KEY_U => Bopomofo::IU,
            KEY_I => Bopomofo::AI,
            KEY_O => Bopomofo::O,
            KEY_P => Bopomofo::P,
            KEY_A => Bopomofo::A,
            KEY_S => Bopomofo::S,
            KEY_D => Bopomofo::D,
            KEY_F => Bopomofo::F,
            KEY_G => Bopomofo::J,
            KEY_H => Bopomofo::H,
            KEY_J => Bopomofo::R,
            KEY_K => Bopomofo::K,
            KEY_L => Bopomofo::L,
            KEY_SEMICOLON => Bopomofo::Z,
            KEY_APOSTROPHE => Bopomofo::C,
            KEY_Z => Bopomofo::AU,
            KEY_X => Bopomofo::U,
            KEY_C => Bopomofo::X,
            KEY_V => Bopomofo::G,
            KEY_B => Bopomofo::B,
            KEY_N => Bopomofo::N,
            KEY_M => Bopomofo::M,
            KEY_COMMA => Bopomofo::ZH,
            KEY_DOT => Bopomofo::CH,
            KEY_SLASH => Bopomofo::SH,
            KEY_SPACE => Bopomofo::TONE1,
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
    use super::Et;
    use crate::{
        editor::zhuyin_layout::{KeyBehavior, SyllableEditor},
        input::{KeyboardEvent, keycode::KEY_SPACE, keysym::SYM_SPACE},
    };

    #[test]
    fn space() {
        let mut editor = Et::new();
        let behavior = editor.key_press(KeyboardEvent {
            code: KEY_SPACE,
            ksym: SYM_SPACE,
            state: 0,
        });
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
