//! Standard keyboard layout
//!
//! Also known as the Dai Chien (大千) layout. It's the default layout on almost
//! all platforms and the most commonly used one.

use super::{KeyBehavior, SyllableEditor};
use crate::input::keycode::*;
use crate::{
    input::KeyboardEvent,
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

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
    fn key_press(&mut self, key: KeyboardEvent) -> KeyBehavior {
        let bopomofo = match key.code {
            KEY_1 => Bopomofo::B,
            KEY_2 => Bopomofo::D,
            KEY_3 => Bopomofo::TONE3,
            KEY_4 => Bopomofo::TONE4,
            KEY_5 => Bopomofo::ZH,
            KEY_6 => Bopomofo::TONE2,
            KEY_7 => Bopomofo::TONE5,
            KEY_8 => Bopomofo::A,
            KEY_9 => Bopomofo::AI,
            KEY_0 => Bopomofo::AN,
            KEY_MINUS => Bopomofo::ER,
            KEY_Q => Bopomofo::P,
            KEY_W => Bopomofo::T,
            KEY_E => Bopomofo::G,
            KEY_R => Bopomofo::J,
            KEY_T => Bopomofo::CH,
            KEY_Y => Bopomofo::Z,
            KEY_U => Bopomofo::I,
            KEY_I => Bopomofo::O,
            KEY_O => Bopomofo::EI,
            KEY_P => Bopomofo::EN,
            KEY_A => Bopomofo::M,
            KEY_S => Bopomofo::N,
            KEY_D => Bopomofo::K,
            KEY_F => Bopomofo::Q,
            KEY_G => Bopomofo::SH,
            KEY_H => Bopomofo::C,
            KEY_J => Bopomofo::U,
            KEY_K => Bopomofo::E,
            KEY_L => Bopomofo::AU,
            KEY_SEMICOLON => Bopomofo::ANG,
            KEY_Z => Bopomofo::F,
            KEY_X => Bopomofo::L,
            KEY_C => Bopomofo::H,
            KEY_V => Bopomofo::X,
            KEY_B => Bopomofo::R,
            KEY_N => Bopomofo::S,
            KEY_M => Bopomofo::IU,
            KEY_COMMA => Bopomofo::EH,
            KEY_DOT => Bopomofo::OU,
            KEY_SLASH => Bopomofo::ENG,
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
    use super::Standard;
    use crate::{
        editor::zhuyin_layout::{KeyBehavior, SyllableEditor},
        input::{KeyboardEvent, keycode::KEY_SPACE, keysym::SYM_SPACE},
    };

    #[test]
    fn space() {
        let mut editor = Standard::new();
        let behavior = editor.key_press(KeyboardEvent {
            code: KEY_SPACE,
            ksym: SYM_SPACE,
            state: 0,
        });
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
