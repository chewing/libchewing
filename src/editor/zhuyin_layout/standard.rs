//! Standard keyboard layout
//!
//! Also known as the Dai Chien (大千) layout. It's the default layout on almost
//! all platforms and the most commonly used one.

use crate::{
    input::{KeyboardEvent, Keycode},
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
    fn key_press(&mut self, key: KeyboardEvent) -> KeyBehavior {
        let bopomofo = match key.code {
            Keycode::KEY_1 => Bopomofo::B,
            Keycode::KEY_2 => Bopomofo::D,
            Keycode::KEY_3 => Bopomofo::TONE3,
            Keycode::KEY_4 => Bopomofo::TONE4,
            Keycode::KEY_5 => Bopomofo::ZH,
            Keycode::KEY_6 => Bopomofo::TONE2,
            Keycode::KEY_7 => Bopomofo::TONE5,
            Keycode::KEY_8 => Bopomofo::A,
            Keycode::KEY_9 => Bopomofo::AI,
            Keycode::KEY_0 => Bopomofo::AN,
            Keycode::KEY_MINUS => Bopomofo::ER,
            Keycode::KEY_Q => Bopomofo::P,
            Keycode::KEY_W => Bopomofo::T,
            Keycode::KEY_E => Bopomofo::G,
            Keycode::KEY_R => Bopomofo::J,
            Keycode::KEY_T => Bopomofo::CH,
            Keycode::KEY_Y => Bopomofo::Z,
            Keycode::KEY_U => Bopomofo::I,
            Keycode::KEY_I => Bopomofo::O,
            Keycode::KEY_O => Bopomofo::EI,
            Keycode::KEY_P => Bopomofo::EN,
            Keycode::KEY_A => Bopomofo::M,
            Keycode::KEY_S => Bopomofo::N,
            Keycode::KEY_D => Bopomofo::K,
            Keycode::KEY_F => Bopomofo::Q,
            Keycode::KEY_G => Bopomofo::SH,
            Keycode::KEY_H => Bopomofo::C,
            Keycode::KEY_J => Bopomofo::U,
            Keycode::KEY_K => Bopomofo::E,
            Keycode::KEY_L => Bopomofo::AU,
            Keycode::KEY_SEMICOLON => Bopomofo::ANG,
            Keycode::KEY_Z => Bopomofo::F,
            Keycode::KEY_X => Bopomofo::L,
            Keycode::KEY_C => Bopomofo::H,
            Keycode::KEY_V => Bopomofo::X,
            Keycode::KEY_B => Bopomofo::R,
            Keycode::KEY_N => Bopomofo::S,
            Keycode::KEY_M => Bopomofo::IU,
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

    use super::Standard;

    #[test]
    fn space() {
        let mut editor = Standard::new();
        let behavior = editor.key_press(KeyboardEvent {
            code: Keycode::KEY_SPACE,
            ksym: Keysym::from(' '),
            state: 0,
        });
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
