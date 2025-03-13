use super::{INDEX_MAP, KeyCode, KeyEvent, KeyboardLayout, Modifiers, dvorak, qwerty};

/// A Dvorak keyboard.
#[derive(Debug)]
pub struct DvorakOnQwerty;

impl KeyboardLayout for DvorakOnQwerty {
    fn map_with_mod(&self, keycode: KeyCode, modifiers: Modifiers) -> KeyEvent {
        let index = qwerty::KEYCODE_INDEX
            .iter()
            .position(|key| *key == keycode)
            .expect("invalid keycode");
        let unicode = if modifiers.capslock || modifiers.shift {
            dvorak::SHIFT_MAP[index]
        } else {
            dvorak::UNICODE_MAP[index]
        };
        KeyEvent {
            index: INDEX_MAP[index],
            code: dvorak::KEYCODE_INDEX[index],
            unicode,
            modifiers,
        }
    }
}
