//! IBM keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use crate::input::keycode::*;
use crate::{
    input::KeyboardEvent,
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
    fn key_press(&mut self, key: KeyboardEvent) -> KeyBehavior {
        let bopomofo = match key.code {
            KEY_1 => Bopomofo::B,
            KEY_2 => Bopomofo::P,
            KEY_3 => Bopomofo::M,
            KEY_4 => Bopomofo::F,
            KEY_5 => Bopomofo::D,
            KEY_6 => Bopomofo::T,
            KEY_7 => Bopomofo::N,
            KEY_8 => Bopomofo::L,
            KEY_9 => Bopomofo::G,
            KEY_0 => Bopomofo::K,
            KEY_MINUS => Bopomofo::H,
            KEY_Q => Bopomofo::J,
            KEY_W => Bopomofo::Q,
            KEY_E => Bopomofo::X,
            KEY_R => Bopomofo::ZH,
            KEY_T => Bopomofo::CH,
            KEY_Y => Bopomofo::SH,
            KEY_U => Bopomofo::R,
            KEY_I => Bopomofo::Z,
            KEY_O => Bopomofo::C,
            KEY_P => Bopomofo::S,
            KEY_A => Bopomofo::I,
            KEY_S => Bopomofo::U,
            KEY_D => Bopomofo::IU,
            KEY_F => Bopomofo::A,
            KEY_G => Bopomofo::O,
            KEY_H => Bopomofo::E,
            KEY_J => Bopomofo::EH,
            KEY_K => Bopomofo::AI,
            KEY_L => Bopomofo::EI,
            KEY_SEMICOLON => Bopomofo::AU,
            KEY_Z => Bopomofo::OU,
            KEY_X => Bopomofo::AN,
            KEY_C => Bopomofo::EN,
            KEY_V => Bopomofo::ANG,
            KEY_B => Bopomofo::ENG,
            KEY_N => Bopomofo::ER,
            KEY_M => Bopomofo::TONE2,
            KEY_COMMA => Bopomofo::TONE3,
            KEY_DOT => Bopomofo::TONE4,
            KEY_SLASH => Bopomofo::TONE5,
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
    use crate::{
        editor::zhuyin_layout::{KeyBehavior, SyllableEditor},
        input::{KeyboardEvent, keycode::KEY_SPACE, keysym::SYM_SPACE},
    };

    use super::Ibm;

    #[test]
    fn space() {
        let mut editor = Ibm::new();
        let behavior = editor.key_press(KeyboardEvent {
            code: KEY_SPACE,
            ksym: SYM_SPACE,
            state: 0,
        });
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
