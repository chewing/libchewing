//! GinYieh keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use crate::{
    input::{KeyboardEvent, Keycode},
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
    fn key_press(&mut self, key: KeyboardEvent) -> KeyBehavior {
        let bopomofo = match key.code {
            Keycode::KEY_1 => Bopomofo::TONE5,
            Keycode::KEY_2 => Bopomofo::B,
            Keycode::KEY_3 => Bopomofo::D,
            Keycode::KEY_6 => Bopomofo::ZH,
            Keycode::KEY_8 => Bopomofo::A,
            Keycode::KEY_9 => Bopomofo::AI,
            Keycode::KEY_0 => Bopomofo::AN,
            Keycode::KEY_MINUS => Bopomofo::I,
            Keycode::KEY_EQUAL => Bopomofo::ER,
            Keycode::KEY_Q => Bopomofo::TONE2,
            Keycode::KEY_W => Bopomofo::P,
            Keycode::KEY_E => Bopomofo::T,
            Keycode::KEY_R => Bopomofo::G,
            Keycode::KEY_T => Bopomofo::J,
            Keycode::KEY_Y => Bopomofo::CH,
            Keycode::KEY_U => Bopomofo::Z,
            Keycode::KEY_I => Bopomofo::O,
            Keycode::KEY_O => Bopomofo::EI,
            Keycode::KEY_P => Bopomofo::EN,
            Keycode::KEY_LEFTBRACE => Bopomofo::U,
            Keycode::KEY_A => Bopomofo::TONE3,
            Keycode::KEY_S => Bopomofo::M,
            Keycode::KEY_D => Bopomofo::N,
            Keycode::KEY_F => Bopomofo::K,
            Keycode::KEY_G => Bopomofo::Q,
            Keycode::KEY_H => Bopomofo::SH,
            Keycode::KEY_J => Bopomofo::C,
            Keycode::KEY_K => Bopomofo::E,
            Keycode::KEY_L => Bopomofo::AU,
            Keycode::KEY_SEMICOLON => Bopomofo::ANG,
            Keycode::KEY_APOSTROPHE => Bopomofo::IU,
            Keycode::KEY_Z => Bopomofo::TONE4,
            Keycode::KEY_X => Bopomofo::F,
            Keycode::KEY_C => Bopomofo::L,
            Keycode::KEY_V => Bopomofo::H,
            Keycode::KEY_B => Bopomofo::X,
            Keycode::KEY_N => Bopomofo::R,
            Keycode::KEY_M => Bopomofo::S,
            Keycode::KEY_COMMA => Bopomofo::EH,
            Keycode::KEY_DOT => Bopomofo::OU,
            Keycode::KEY_SLASH => Bopomofo::ENG,
            Keycode::KEY_SPACE => Bopomofo::TONE1,
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
        input::{KeyboardEvent, Keycode, Keysym},
    };

    use super::GinYieh;

    #[test]
    fn space() {
        let mut editor = GinYieh::new();
        let behavior = editor.key_press(KeyboardEvent {
            code: Keycode::KEY_SPACE,
            ksym: Keysym::from(' '),
            state: 0,
        });
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
