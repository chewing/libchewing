//! Input handling modules

use std::fmt::Display;

use keycode::Keycode;
use keysym::Keysym;

use crate::input::keysym::SYM_NONE;

pub mod keycode;
pub mod keymap;
pub mod keysym;

/// Modifier keys and key press/release state
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum KeyState {
    /// Shift is activated.
    Shift = 1 << 0,
    /// Caps Lock is activated.
    CapsLock = 1 << 1,
    /// Control is activated.
    Control = 1 << 2,
    /// Alt or Meta is activated.
    Alt = 1 << 3,
    /// Num Lock is activated.
    NumLock = 1 << 4,
    /// Super is activated.
    Super = 1 << 6,
    /// Key is released.
    Release = 1 << 30,
}

/// Keyboard layout independent KeyboardEvent
///
/// Use `code` to identify a physical key on a keyboard, use `ksym` to
/// represent the output after keymap translation.
///
/// With this abstraction, chewing can handle any keyboard layout supported
/// by the operating system.
///
/// # Examples
///
/// Use the [`KeyboardEventBuilder`] to construct a KeyboardEvent:
///
/// ```rust
/// use chewing::input::KeyboardEvent;
/// use chewing::input::KeyState;
/// use chewing::input::keycode;
/// use chewing::input::keysym;
///
/// let control_down = true;
/// let caps_lock = false;
/// let evt = KeyboardEvent::builder()
///     .code(keycode::KEY_A)
///     .ksym(keysym::SYM_LOWER_A)
///     .shift()
///     .control_if(control_down)
///     .caps_lock_if(caps_lock)
///     .build();
/// # assert_eq!(keycode::KEY_A, evt.code);
/// # assert_eq!(keysym::SYM_LOWER_A, evt.ksym);
/// # assert!(evt.is_state_on(KeyState::Shift));
/// # assert!(evt.is_state_on(KeyState::Control));
/// # assert!(!evt.is_state_on(KeyState::CapsLock));
/// ```
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct KeyboardEvent {
    /// Code that identifies a physical key on a keyboard.
    ///
    /// Keycodes are the result of the low-level processing of the data that
    /// keyboards send to a computer. For instance 36 may represent the return
    /// key.
    ///
    /// Symbolic names are assigned to raw keycodes in order to facilitate
    /// their mapping to symbols. By convention keycode names are based on US
    /// QWERTY layout. For example the keycode for the return key is
    /// Keycode::RETURN.
    ///
    /// Chewing keycodes have same numeric encoding as X11 or xkbcommon
    /// keycodes.
    pub code: Keycode,

    /// The symbol on the cap of a key.
    ///
    /// Keysyms (short for "key symbol") are translated from keycodes via a
    /// keymap. On different layout (qwerty, dvorak, etc.) all keyboards emit
    /// the same keycodes but produce different keysyms after translation.
    pub ksym: Keysym,

    /// The key press / release state and state of modifier keys.
    ///
    /// Use the state mask to read whether a modifier key is active and
    /// whether the key is pressed.
    pub state: u32,
}

impl KeyboardEvent {
    /// Create a builder to construct the KeyboardEvent.
    ///
    /// # Example
    ///
    /// ```
    /// use chewing::input::KeyboardEvent;
    ///
    /// let shift_pressed = true;
    /// let evt = KeyboardEvent::builder()
    ///     .shift_if(shift_pressed)
    ///     .build();
    /// ```
    pub const fn builder() -> KeyboardEventBuilder {
        KeyboardEventBuilder {
            evt: KeyboardEvent {
                code: Keycode(0),
                ksym: Keysym(0),
                state: 0,
            },
        }
    }
    pub fn is_invalid(&self) -> bool {
        self.code.0 == 0 && self.ksym == SYM_NONE && self.state == 0
    }
    /// Determine whether a modifier is down or toggled
    pub fn is_state_on(&self, m: KeyState) -> bool {
        match m {
            KeyState::Shift => self.is_flag_on(KeyState::Shift as u32),
            KeyState::Control => self.is_flag_on(KeyState::Control as u32),
            KeyState::Alt => self.is_flag_on(KeyState::Alt as u32),
            KeyState::Super => self.is_flag_on(KeyState::Super as u32),
            KeyState::CapsLock => self.is_flag_on(KeyState::CapsLock as u32),
            KeyState::NumLock => self.is_flag_on(KeyState::NumLock as u32),
            KeyState::Release => self.is_flag_on(KeyState::Release as u32),
        }
    }
    fn is_flag_on(&self, mask: u32) -> bool {
        self.state & mask == mask
    }
    pub fn has_modifiers(&self) -> bool {
        self.is_state_on(KeyState::Shift)
            || self.is_state_on(KeyState::Control)
            || self.is_state_on(KeyState::Alt)
            || self.is_state_on(KeyState::Super)
    }
}

impl Display for KeyboardEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#KeyboardEvent(")?;
        write!(f, ":code {}", self.code.0)?;
        write!(f, " :ksym {}", self.ksym.0)?;
        if self.ksym.is_unicode() {
            write!(f, " :char '{}'", self.ksym.to_unicode())?;
        }
        if self.state != 0 {
            write!(f, " :state '")?;
            if self.is_state_on(KeyState::Control) {
                write!(f, "c")?;
            }
            if self.is_state_on(KeyState::Shift) {
                write!(f, "s")?;
            }
            if self.is_state_on(KeyState::Alt) {
                write!(f, "a")?;
            }
            if self.is_state_on(KeyState::Super) {
                write!(f, "S")?;
            }
            if self.is_state_on(KeyState::CapsLock) {
                write!(f, "C")?;
            }
            if self.is_state_on(KeyState::NumLock) {
                write!(f, "N")?;
            }
            if self.is_state_on(KeyState::Release) {
                write!(f, "R")?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KeyboardEventBuilder {
    evt: KeyboardEvent,
}

impl KeyboardEventBuilder {
    pub const fn code(&mut self, code: Keycode) -> &mut KeyboardEventBuilder {
        self.evt.code = code;
        self
    }
    pub const fn ksym(&mut self, key: Keysym) -> &mut KeyboardEventBuilder {
        self.evt.ksym = key;
        self
    }
    pub const fn shift(&mut self) -> &mut KeyboardEventBuilder {
        self.shift_if(true)
    }
    pub const fn shift_if(&mut self, shift: bool) -> &mut KeyboardEventBuilder {
        if shift {
            self.evt.state |= KeyState::Shift as u32;
        }
        self
    }
    pub const fn caps_lock(&mut self) -> &mut KeyboardEventBuilder {
        self.caps_lock_if(true)
    }
    pub const fn caps_lock_if(&mut self, caps_lock: bool) -> &mut KeyboardEventBuilder {
        if caps_lock {
            self.evt.state |= KeyState::CapsLock as u32;
        }
        self
    }
    pub const fn control(&mut self) -> &mut KeyboardEventBuilder {
        self.control_if(true)
    }
    pub const fn control_if(&mut self, control: bool) -> &mut KeyboardEventBuilder {
        if control {
            self.evt.state |= KeyState::Control as u32;
        }
        self
    }
    pub const fn alt_if(&mut self, alt: bool) -> &mut KeyboardEventBuilder {
        if alt {
            self.evt.state |= KeyState::Alt as u32;
        }
        self
    }
    pub const fn num_lock_if(&mut self, num_lock: bool) -> &mut KeyboardEventBuilder {
        if num_lock {
            self.evt.state |= KeyState::NumLock as u32;
        }
        self
    }
    pub const fn super_if(&mut self, supa: bool) -> &mut KeyboardEventBuilder {
        if supa {
            self.evt.state |= KeyState::Super as u32;
        }
        self
    }
    pub const fn release(&mut self) -> &mut KeyboardEventBuilder {
        self.release_if(true)
    }
    pub const fn release_if(&mut self, release: bool) -> &mut KeyboardEventBuilder {
        if release {
            self.evt.state |= KeyState::Release as u32;
        }
        self
    }
    pub const fn build(&mut self) -> KeyboardEvent {
        self.evt
    }
}

#[cfg(test)]
mod tests {
    use super::KeyboardEvent;
    use super::keycode;
    use super::keysym;
    use crate::input::KeyState;

    #[test]
    fn keyboard_event_builder() {
        let control_down = true;
        let caps_lock = false;
        let evt = KeyboardEvent::builder()
            .code(keycode::KEY_A)
            .ksym(keysym::SYM_LOWER_A)
            .shift()
            .control_if(control_down)
            .caps_lock_if(caps_lock)
            .num_lock_if(false)
            .alt_if(false)
            .super_if(false)
            .release_if(false)
            .build();
        assert_eq!(keycode::KEY_A, evt.code);
        assert_eq!(keysym::SYM_LOWER_A, evt.ksym);
        assert!(evt.is_state_on(KeyState::Control));
        assert!(evt.is_state_on(KeyState::Shift));
        assert!(!evt.is_state_on(KeyState::CapsLock));
    }
}
