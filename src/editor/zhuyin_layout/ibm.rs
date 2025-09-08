//! IBM keyboard layout
//!
//! Another commonly used keyboard layout on older IBM PC.

use crate::{
    input::{KeyboardEvent, Keycode},
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
            Keycode::KEY_1 => Bopomofo::B,
            Keycode::KEY_2 => Bopomofo::P,
            Keycode::KEY_3 => Bopomofo::M,
            Keycode::KEY_4 => Bopomofo::F,
            Keycode::KEY_5 => Bopomofo::D,
            Keycode::KEY_6 => Bopomofo::T,
            Keycode::KEY_7 => Bopomofo::N,
            Keycode::KEY_8 => Bopomofo::L,
            Keycode::KEY_9 => Bopomofo::G,
            Keycode::KEY_0 => Bopomofo::K,
            Keycode::KEY_MINUS => Bopomofo::H,
            Keycode::KEY_Q => Bopomofo::J,
            Keycode::KEY_W => Bopomofo::Q,
            Keycode::KEY_E => Bopomofo::X,
            Keycode::KEY_R => Bopomofo::ZH,
            Keycode::KEY_T => Bopomofo::CH,
            Keycode::KEY_Y => Bopomofo::SH,
            Keycode::KEY_U => Bopomofo::R,
            Keycode::KEY_I => Bopomofo::Z,
            Keycode::KEY_O => Bopomofo::C,
            Keycode::KEY_P => Bopomofo::S,
            Keycode::KEY_A => Bopomofo::I,
            Keycode::KEY_S => Bopomofo::U,
            Keycode::KEY_D => Bopomofo::IU,
            Keycode::KEY_F => Bopomofo::A,
            Keycode::KEY_G => Bopomofo::O,
            Keycode::KEY_H => Bopomofo::E,
            Keycode::KEY_J => Bopomofo::EH,
            Keycode::KEY_K => Bopomofo::AI,
            Keycode::KEY_L => Bopomofo::EI,
            Keycode::KEY_SEMICOLON => Bopomofo::AU,
            Keycode::KEY_Z => Bopomofo::OU,
            Keycode::KEY_X => Bopomofo::AN,
            Keycode::KEY_C => Bopomofo::EN,
            Keycode::KEY_V => Bopomofo::ANG,
            Keycode::KEY_B => Bopomofo::ENG,
            Keycode::KEY_N => Bopomofo::ER,
            Keycode::KEY_M => Bopomofo::TONE2,
            Keycode::KEY_COMMA => Bopomofo::TONE3,
            Keycode::KEY_DOT => Bopomofo::TONE4,
            Keycode::KEY_SLASH => Bopomofo::TONE5,
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

    use super::Ibm;

    #[test]
    fn space() {
        let mut editor = Ibm::new();
        let behavior = editor.key_press(KeyboardEvent {
            code: Keycode::KEY_SPACE,
            ksym: Keysym::from(' '),
            state: 0,
        });
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
