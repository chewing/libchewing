//! GinYieh keyboard layout
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
    fn key_press(&mut self, key: KeyboardEvent) -> KeyBehavior {
        let bopomofo = match key.code {
            KEY_1 => Bopomofo::TONE5,
            KEY_2 => Bopomofo::B,
            KEY_3 => Bopomofo::D,
            KEY_6 => Bopomofo::ZH,
            KEY_8 => Bopomofo::A,
            KEY_9 => Bopomofo::AI,
            KEY_0 => Bopomofo::AN,
            KEY_MINUS => Bopomofo::I,
            KEY_EQUAL => Bopomofo::ER,
            KEY_Q => Bopomofo::TONE2,
            KEY_W => Bopomofo::P,
            KEY_E => Bopomofo::T,
            KEY_R => Bopomofo::G,
            KEY_T => Bopomofo::J,
            KEY_Y => Bopomofo::CH,
            KEY_U => Bopomofo::Z,
            KEY_I => Bopomofo::O,
            KEY_O => Bopomofo::EI,
            KEY_P => Bopomofo::EN,
            KEY_LEFTBRACE => Bopomofo::U,
            KEY_A => Bopomofo::TONE3,
            KEY_S => Bopomofo::M,
            KEY_D => Bopomofo::N,
            KEY_F => Bopomofo::K,
            KEY_G => Bopomofo::Q,
            KEY_H => Bopomofo::SH,
            KEY_J => Bopomofo::C,
            KEY_K => Bopomofo::E,
            KEY_L => Bopomofo::AU,
            KEY_SEMICOLON => Bopomofo::ANG,
            KEY_APOSTROPHE => Bopomofo::IU,
            KEY_Z => Bopomofo::TONE4,
            KEY_X => Bopomofo::F,
            KEY_C => Bopomofo::L,
            KEY_V => Bopomofo::H,
            KEY_B => Bopomofo::X,
            KEY_N => Bopomofo::R,
            KEY_M => Bopomofo::S,
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
    use super::GinYieh;
    use crate::{
        editor::zhuyin_layout::{KeyBehavior, SyllableEditor},
        input::{KeyboardEvent, keycode::KEY_SPACE, keysym::SYM_SPACE},
    };

    #[test]
    fn space() {
        let mut editor = GinYieh::new();
        let behavior = editor.key_press(KeyboardEvent {
            code: KEY_SPACE,
            ksym: SYM_SPACE,
            state: 0,
        });
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
