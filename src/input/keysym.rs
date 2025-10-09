//! Key symbols.
//!
//! Keysyms (short for "key symbol") are translated from keycodes via a
//! keymap. On different layout (qwerty, dvorak, etc.) all keyboards emit
//! the same keycodes but produce different keysyms after translation.
//!
//! Keysyms has numerical encoding compatible with X11 / xkbcommon.

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Keysym(pub u32);

pub const SYM_NONE: Keysym = Keysym(0x000000);
pub const SYM_VOID: Keysym = Keysym(0xffffff);

pub const SYM_BACKSPACE: Keysym = Keysym(0xff08);
pub const SYM_TAB: Keysym = Keysym(0xff09);
pub const SYM_LINEFEED: Keysym = Keysym(0xff0a);
pub const SYM_RETURN: Keysym = Keysym(0xff0d);
pub const SYM_ESC: Keysym = Keysym(0xff1b);
pub const SYM_DELETE: Keysym = Keysym(0xffff);

pub const SYM_HOME: Keysym = Keysym(0xff50);
pub const SYM_LEFT: Keysym = Keysym(0xff51);
pub const SYM_UP: Keysym = Keysym(0xff52);
pub const SYM_RIGHT: Keysym = Keysym(0xff53);
pub const SYM_DOWN: Keysym = Keysym(0xff54);
pub const SYM_PAGEUP: Keysym = Keysym(0xff55);
pub const SYM_PAGEDOWN: Keysym = Keysym(0xff56);
pub const SYM_END: Keysym = Keysym(0xff57);

pub const SYM_NUMLOCK: Keysym = Keysym(0xff7f);
pub const SYM_KPENTER: Keysym = Keysym(0xff8d);
pub const SYM_KPEQUAL: Keysym = Keysym(0xffbd);
pub const SYM_KPMULTIPLY: Keysym = Keysym(0xffaa);
pub const SYM_KPADD: Keysym = Keysym(0xffab);
pub const SYM_KPSUBTRACT: Keysym = Keysym(0xffad);
pub const SYM_KPDECIMAL: Keysym = Keysym(0xffae);
pub const SYM_KPDIVIDE: Keysym = Keysym(0xffaf);
pub const SYM_KP0: Keysym = Keysym(0xffb0);
pub const SYM_KP1: Keysym = Keysym(0xffb1);
pub const SYM_KP2: Keysym = Keysym(0xffb2);
pub const SYM_KP3: Keysym = Keysym(0xffb3);
pub const SYM_KP4: Keysym = Keysym(0xffb4);
pub const SYM_KP5: Keysym = Keysym(0xffb5);
pub const SYM_KP6: Keysym = Keysym(0xffb6);
pub const SYM_KP7: Keysym = Keysym(0xffb7);
pub const SYM_KP8: Keysym = Keysym(0xffb8);
pub const SYM_KP9: Keysym = Keysym(0xffb9);

pub const SYM_F1: Keysym = Keysym(0xffbe);
pub const SYM_F2: Keysym = Keysym(0xffbf);
pub const SYM_F3: Keysym = Keysym(0xffc0);
pub const SYM_F4: Keysym = Keysym(0xffc1);
pub const SYM_F5: Keysym = Keysym(0xffc2);
pub const SYM_F6: Keysym = Keysym(0xffc3);
pub const SYM_F7: Keysym = Keysym(0xffc4);
pub const SYM_F8: Keysym = Keysym(0xffc5);
pub const SYM_F9: Keysym = Keysym(0xffc6);
pub const SYM_F10: Keysym = Keysym(0xffc7);
pub const SYM_F11: Keysym = Keysym(0xffc8);
pub const SYM_F12: Keysym = Keysym(0xffc9);

pub const SYM_LEFTSHIFT: Keysym = Keysym(0xffe1);
pub const SYM_RIGHTSHIFT: Keysym = Keysym(0xffe2);
pub const SYM_LEFTCTRL: Keysym = Keysym(0xffe3);
pub const SYM_RIGHTCTRL: Keysym = Keysym(0xffe4);
pub const SYM_CAPSLOCK: Keysym = Keysym(0xffe5);

pub const SYM_LEFTMETA: Keysym = Keysym(0xffe7);
pub const SYM_RIGHTMETA: Keysym = Keysym(0xffe8);
pub const SYM_LEFTALT: Keysym = Keysym(0xffe9);
pub const SYM_RIGHTALT: Keysym = Keysym(0xffea);

pub const SYM_SPACE: Keysym = Keysym(0x0020);
pub const SYM_GRAVE: Keysym = Keysym(0x0060);

pub const SYM_1: Keysym = Keysym(0x0031);
pub const SYM_2: Keysym = Keysym(0x0032);
pub const SYM_3: Keysym = Keysym(0x0033);
pub const SYM_4: Keysym = Keysym(0x0034);
pub const SYM_5: Keysym = Keysym(0x0035);
pub const SYM_6: Keysym = Keysym(0x0036);
pub const SYM_7: Keysym = Keysym(0x0037);
pub const SYM_8: Keysym = Keysym(0x0038);
pub const SYM_9: Keysym = Keysym(0x0039);
pub const SYM_0: Keysym = Keysym(0x0030);
pub const SYM_LOWER_A: Keysym = Keysym(0x0061);
pub const SYM_LOWER_B: Keysym = Keysym(0x0062);
pub const SYM_LOWER_C: Keysym = Keysym(0x0063);
pub const SYM_LOWER_D: Keysym = Keysym(0x0064);
pub const SYM_LOWER_E: Keysym = Keysym(0x0065);
pub const SYM_LOWER_F: Keysym = Keysym(0x0066);
pub const SYM_LOWER_G: Keysym = Keysym(0x0067);
pub const SYM_LOWER_H: Keysym = Keysym(0x0068);
pub const SYM_LOWER_I: Keysym = Keysym(0x0069);
pub const SYM_LOWER_J: Keysym = Keysym(0x006a);
pub const SYM_LOWER_K: Keysym = Keysym(0x006b);
pub const SYM_LOWER_L: Keysym = Keysym(0x006c);
pub const SYM_LOWER_M: Keysym = Keysym(0x006d);
pub const SYM_LOWER_N: Keysym = Keysym(0x006e);
pub const SYM_LOWER_O: Keysym = Keysym(0x006f);
pub const SYM_LOWER_P: Keysym = Keysym(0x0070);
pub const SYM_LOWER_Q: Keysym = Keysym(0x0071);
pub const SYM_LOWER_R: Keysym = Keysym(0x0072);
pub const SYM_LOWER_S: Keysym = Keysym(0x0073);
pub const SYM_LOWER_T: Keysym = Keysym(0x0074);
pub const SYM_LOWER_U: Keysym = Keysym(0x0075);
pub const SYM_LOWER_V: Keysym = Keysym(0x0076);
pub const SYM_LOWER_W: Keysym = Keysym(0x0077);
pub const SYM_LOWER_X: Keysym = Keysym(0x0078);
pub const SYM_LOWER_Y: Keysym = Keysym(0x0079);
pub const SYM_LOWER_Z: Keysym = Keysym(0x007a);

impl Keysym {
    const UNICODE_OFFSET: u32 = 0x01000000;
    pub const fn from_char(value: char) -> Keysym {
        if value.is_ascii_control() {
            // ASCII control characters are not valid Keysm
            return SYM_NONE;
        }
        if value.is_ascii() {
            return Keysym(value as u32);
        }
        Keysym(value as u32 + Self::UNICODE_OFFSET)
    }
    pub fn is_ascii(&self) -> bool {
        self.0 >= 0x20 && self.0 <= 0x7e
    }
    pub fn is_unicode(&self) -> bool {
        self.is_ascii() || (self.0 >= 0x01000100 && self.0 <= 0x0110ffff)
    }
    pub fn is_keypad(&self) -> bool {
        (self.0 >= SYM_KP0.0 && self.0 <= SYM_KP9.0)
            || [
                SYM_KPADD,
                SYM_KPSUBTRACT,
                SYM_KPMULTIPLY,
                SYM_KPDIVIDE,
                SYM_KPDECIMAL,
                SYM_KPEQUAL,
            ]
            .contains(self)
    }
    pub fn to_unicode(&self) -> char {
        if self.is_keypad() {
            return match *self {
                SYM_KP0 => '0',
                SYM_KP1 => '1',
                SYM_KP2 => '2',
                SYM_KP3 => '3',
                SYM_KP4 => '4',
                SYM_KP5 => '5',
                SYM_KP6 => '6',
                SYM_KP7 => '7',
                SYM_KP8 => '8',
                SYM_KP9 => '9',
                SYM_KPADD => '+',
                SYM_KPSUBTRACT => '-',
                SYM_KPMULTIPLY => '*',
                SYM_KPDIVIDE => '/',
                SYM_KPDECIMAL => '.',
                SYM_KPEQUAL => '=',
                _ => unreachable!(),
            };
        }
        if !self.is_unicode() {
            return char::REPLACEMENT_CHARACTER;
        }
        if self.0 > Self::UNICODE_OFFSET {
            return char::from_u32(self.0 - Self::UNICODE_OFFSET)
                .unwrap_or(char::REPLACEMENT_CHARACTER);
        }
        char::from_u32(self.0).unwrap_or(char::REPLACEMENT_CHARACTER)
    }
    pub fn is_digit(&self) -> bool {
        self.to_digit().is_some()
    }
    pub fn to_digit(&self) -> Option<u8> {
        if self.0 >= 0x0030 && self.0 <= 0x0039 {
            return Some((self.0 - 0x0030) as u8);
        }
        if self >= &SYM_KP0 && self <= &SYM_KP9 {
            return Some((self.0 - SYM_KP0.0) as u8);
        }
        None
    }
    pub fn is_atoz(&self) -> bool {
        (self.0 >= 'a' as u32 && self.0 <= 'z' as u32)
            || (self.0 >= 'A' as u32 && self.0 <= 'Z' as u32)
    }
}

impl From<char> for Keysym {
    fn from(value: char) -> Self {
        Self::from_char(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tty_function_keys() {
        assert_eq!(SYM_BACKSPACE, Keysym(0xff08));
    }

    #[test]
    fn latin1_keys() {
        assert_eq!(Keysym::from('a'), Keysym(0x0061));
        assert_eq!(Keysym::from('1'), Keysym(0x0031));
    }

    #[test]
    fn unicode_keys() {
        assert_eq!(Keysym::from('\u{20A0}'), Keysym(0x10020a0));
    }

    #[test]
    fn digits() {
        let ksym = Keysym::from('5');
        assert!(ksym.is_digit());
        assert_eq!(Some(5), ksym.to_digit());
    }

    #[test]
    fn alphabet() {
        let mut ksyms = ('a'..'z').chain('A'..'Z').map(Keysym::from);
        assert!(ksyms.all(|ksym| ksym.is_atoz()))
    }
}
