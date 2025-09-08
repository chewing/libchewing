//! ET41 keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use crate::{
    input::{KeyboardEvent, Keycode},
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
    fn key_press(&mut self, key: KeyboardEvent) -> KeyBehavior {
        let bopomofo = match key.code {
            Keycode::KEY_1 => Bopomofo::TONE5,
            Keycode::KEY_2 => Bopomofo::TONE2,
            Keycode::KEY_3 => Bopomofo::TONE3,
            Keycode::KEY_4 => Bopomofo::TONE4,
            Keycode::KEY_7 => Bopomofo::Q,
            Keycode::KEY_8 => Bopomofo::AN,
            Keycode::KEY_9 => Bopomofo::EN,
            Keycode::KEY_0 => Bopomofo::ANG,
            Keycode::KEY_MINUS => Bopomofo::ENG,
            Keycode::KEY_EQUAL => Bopomofo::ER,
            Keycode::KEY_Q => Bopomofo::EI,
            Keycode::KEY_W => Bopomofo::EH,
            Keycode::KEY_E => Bopomofo::I,
            Keycode::KEY_R => Bopomofo::E,
            Keycode::KEY_T => Bopomofo::T,
            Keycode::KEY_Y => Bopomofo::OU,
            Keycode::KEY_U => Bopomofo::IU,
            Keycode::KEY_I => Bopomofo::AI,
            Keycode::KEY_O => Bopomofo::O,
            Keycode::KEY_P => Bopomofo::P,
            Keycode::KEY_A => Bopomofo::A,
            Keycode::KEY_S => Bopomofo::S,
            Keycode::KEY_D => Bopomofo::D,
            Keycode::KEY_F => Bopomofo::F,
            Keycode::KEY_G => Bopomofo::J,
            Keycode::KEY_H => Bopomofo::H,
            Keycode::KEY_J => Bopomofo::R,
            Keycode::KEY_K => Bopomofo::K,
            Keycode::KEY_L => Bopomofo::L,
            Keycode::KEY_SEMICOLON => Bopomofo::Z,
            Keycode::KEY_APOSTROPHE => Bopomofo::C,
            Keycode::KEY_Z => Bopomofo::AU,
            Keycode::KEY_X => Bopomofo::U,
            Keycode::KEY_C => Bopomofo::X,
            Keycode::KEY_V => Bopomofo::G,
            Keycode::KEY_B => Bopomofo::B,
            Keycode::KEY_N => Bopomofo::N,
            Keycode::KEY_M => Bopomofo::M,
            Keycode::KEY_COMMA => Bopomofo::ZH,
            Keycode::KEY_DOT => Bopomofo::CH,
            Keycode::KEY_SLASH => Bopomofo::SH,
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

    use super::Et;

    #[test]
    fn space() {
        let mut editor = Et::new();
        let behavior = editor.key_press(KeyboardEvent {
            code: Keycode::KEY_SPACE,
            ksym: Keysym::from(' '),
            state: 0,
        });
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
