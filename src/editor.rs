//! TODO: doc

mod estimate;
pub mod keymap;
pub mod layout;
pub mod preedit_buffer;

use std::{fmt::Debug, marker::PhantomData};

pub use estimate::{EstimateError, SqliteUserFreqEstimate, UserFreqEstimate};
pub use layout::SyllableEditor;
use tracing::{instrument, warn};

use crate::editor::keymap::KeyCode;

use self::{keymap::KeyEvent, layout::KeyBehavior, preedit_buffer::PreeditBuffer};

/// TODO doc.
#[derive(Debug)]
struct Editor {
    state: &'static dyn ChewingEditorState,
    preedit_buffer: PreeditBuffer,
    commit_buffer: String,
    feedback_buffer: String,
    candidate_selector: CandidateSelector,
    syllable_editor: Box<dyn SyllableEditor>,
}

struct EditLineBuffer;
struct CommitBuffer;
#[derive(Debug)]
struct CandidateSelector;

/// An editor can react to KeyEvents and change its state.
pub trait BasicEditor {
    /// The type representing the observable state of the editor.
    type State;
    /// Handles a KeyEvent
    fn key_press(&mut self, key_event: EditorKeyEvent) -> KeyBehavior;
}

trait ChewingEditorState: Debug {
    fn process_event(
        &self,
        editor: &mut Editor,
        key_event: EditorKeyEvent,
    ) -> (KeyBehavior, &'static dyn ChewingEditorState);
}

#[derive(Debug)]
struct Entering;

#[derive(Debug)]
struct EnteringSyllable;

#[derive(Debug)]
struct Selecting;

#[derive(Debug)]
struct Highlighting;

/// List of possible key events that can be handled by an editor.
#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Debug)]
pub enum EditorKeyEvent {
    Space,
    Esc,
    Enter,
    Del,
    Backspace,
    Tab,
    ShiftLeft,
    Left,
    ShiftRight,
    Right,
    Up,
    Home,
    End,
    PageUp,
    PageDown,
    Down,
    CapsLock,
    Default(KeyEvent),
    CtrlNum(u8),
    ShiftSpace,
    DoubleTab,
    NumLock,
}

enum LanguageMode {
    Chinese,
    English,
}

enum CharacterForm {
    Halfwidth,
    Fullwidth,
}

enum UserPhraseAddDirection {
    Forward,
    Backward,
}

struct EditorOptions {
    esc_clear_all_buffer: bool,
    space_is_select_key: bool,
    auto_shift_cursor: bool,
    phrase_choice_rearward: bool,
    auto_learn_phrase: bool,
}

impl BasicEditor for Editor {
    type State = ();

    fn key_press(&mut self, key_event: EditorKeyEvent) -> KeyBehavior {
        let (key_behavior, new_state) = self.state.process_event(self, key_event);
        self.state = new_state;
        key_behavior
    }
}

impl ChewingEditorState for Entering {
    fn process_event(
        &self,
        editor: &mut Editor,
        key_event: EditorKeyEvent,
    ) -> (KeyBehavior, &'static dyn ChewingEditorState) {
        use EditorKeyEvent::*;

        match key_event {
            Backspace => {
                editor.preedit_buffer.remove_last();
                editor.update_conversion();

                (KeyBehavior::Absorb, &Entering)
            }
            CapsLock => {
                editor.switch_language_mode();
                (KeyBehavior::Absorb, &Entering)
            }
            CtrlNum(num) => {
                if num == 0 || num == 1 {
                    editor.start_hanin_symbol_input();
                    return (KeyBehavior::Absorb, &Selecting);
                }

                if (2..=9).contains(&num) {
                    todo!("handle add new phrases with ctrl-num")
                } else {
                    warn!(num = num, "CtrlNum used with number out of range: {}", num);
                    return (KeyBehavior::Error, &Entering);
                }
                (KeyBehavior::Absorb, &Entering)
            }
            DoubleTab => {
                // editor.reset_user_break_and_connect_at_cursor();
                editor.update_conversion();
                (KeyBehavior::Absorb, &Entering)
            }
            Default(_) => {
                todo!();
                (KeyBehavior::Absorb, &EnteringSyllable)
            }
            _ => (KeyBehavior::Ignore, &Entering),
        }
    }
}

impl ChewingEditorState for EnteringSyllable {
    fn process_event(
        &self,
        editor: &mut Editor,
        key_event: EditorKeyEvent,
    ) -> (KeyBehavior, &'static dyn ChewingEditorState) {
        use EditorKeyEvent::*;

        match key_event {
            Backspace => {
                editor.syllable_editor.remove_last();

                (
                    KeyBehavior::Absorb,
                    if !editor.syllable_editor.is_empty() {
                        &EnteringSyllable
                    } else {
                        &Entering
                    },
                )
            }
            Default(_) => {
                todo!();
                (KeyBehavior::Absorb, &Entering)
            }
            _ => (KeyBehavior::Ignore, &EnteringSyllable),
        }
    }
}

impl ChewingEditorState for Selecting {
    fn process_event(
        &self,
        editor: &mut Editor,
        key_event: EditorKeyEvent,
    ) -> (KeyBehavior, &'static dyn ChewingEditorState) {
        use EditorKeyEvent::*;

        match key_event {
            Backspace => {
                editor.cancel_selecting();

                (KeyBehavior::Absorb, &Entering)
            }
            Default(ev) => {
                todo!("Handle selecting num");
                (KeyBehavior::Absorb, &Entering)
            }
            _ => (KeyBehavior::Ignore, &Selecting),
        }
    }
}

impl ChewingEditorState for Highlighting {
    fn process_event(
        &self,
        editor: &mut Editor,
        key_event: EditorKeyEvent,
    ) -> (KeyBehavior, &'static dyn ChewingEditorState) {
        use EditorKeyEvent::*;

        match key_event {
            Default(_) => {
                todo!();
                (KeyBehavior::Absorb, &EnteringSyllable)
            }
            _ => (KeyBehavior::Ignore, &Highlighting),
        }
    }
}

impl Editor {
    fn check_and_reset_range(&mut self) {
        todo!()
    }
    fn is_entering(&self) -> bool {
        todo!()
    }
    fn is_selecting(&self) -> bool {
        todo!()
    }
    fn start_selecting(&mut self) {
        todo!()
    }
    fn cancel_selecting(&mut self) {
        todo!()
    }
    fn update_conversion(&mut self) {
        todo!()
    }
    fn switch_language_mode(&mut self) {
        todo!()
    }
    fn start_hanin_symbol_input(&mut self) {
        todo!()
    }
    #[instrument]
    fn handle_backspace(&mut self) -> KeyBehavior {
        self.check_and_reset_range();

        if !self.is_entering() {
            return KeyBehavior::Ignore;
        }

        if self.is_selecting() {
            self.cancel_selecting();
        } else {
            if self.syllable_editor.is_empty() {
                self.preedit_buffer.remove_last();
                self.update_conversion();
            } else {
                self.syllable_editor.remove_last();
            }
        }
        // TODO self.make_output_with_rtn()
        KeyBehavior::Absorb
    }
    #[instrument]
    fn handle_caps_lock(&mut self) -> KeyBehavior {
        self.switch_language_mode();
        // TODO self.make_output_with_rtn()
        KeyBehavior::Absorb
    }
    #[instrument]
    fn handle_ctrl_num(&mut self, num: u8) -> KeyBehavior {
        self.check_and_reset_range();

        if self.is_selecting() || !self.syllable_editor.is_empty() {
            return KeyBehavior::Ignore;
        }

        if num == 0 || num == 1 {
            self.start_hanin_symbol_input();
            return KeyBehavior::Absorb;
        }

        if (2..=9).contains(&num) {
            todo!()
        } else {
            warn!(num = num, "CtrlNum used with number out of range: {}", num);
            return KeyBehavior::Error;
        }
        KeyBehavior::Absorb
    }
    #[instrument]
    fn handle_double_tab(&mut self) -> KeyBehavior {
        self.check_and_reset_range();

        if !self.is_entering() {
            return KeyBehavior::Ignore;
        }

        if !self.is_selecting() {
            // self.reset_user_break_and_connect_at_cursor();
            self.update_conversion();
        }

        KeyBehavior::Absorb
    }
    #[instrument]
    fn handle_default(&mut self, key_event: KeyEvent) -> KeyBehavior {
        todo!()
    }
    #[instrument]
    fn handle_del(&mut self) -> KeyBehavior {
        self.check_and_reset_range();

        if !self.is_entering() {
            return KeyBehavior::Ignore;
        }

        if !self.is_selecting() && self.syllable_editor.is_empty() {
            self.preedit_buffer.remove_after_cursor();
            self.update_conversion();
        }
        // TODO self.make_output_with_rtn()
        KeyBehavior::Absorb
    }
    #[instrument]
    fn handle_down(&mut self) -> KeyBehavior {
        self.check_and_reset_range();

        if !self.is_entering() {
            return KeyBehavior::Ignore;
        }

        // TODO
        self.start_selecting();

        KeyBehavior::Absorb
    }
    #[instrument]
    fn handle_end(&mut self) -> KeyBehavior {
        self.check_and_reset_range();

        if !self.is_entering() {
            return KeyBehavior::Ignore;
        }

        if !self.is_selecting() {
            self.preedit_buffer.move_cursor_to_end();
        }

        KeyBehavior::Absorb
    }
    #[instrument]
    fn handle_enter(&mut self) -> KeyBehavior {
        todo!("handle commit")
    }
    #[instrument]
    fn handle_esc(&mut self) -> KeyBehavior {
        self.check_and_reset_range();

        if !self.is_entering() {
            return KeyBehavior::Ignore;
        }

        if self.is_selecting() {
            self.cancel_selecting();
            return KeyBehavior::Absorb;
        }

        if !self.syllable_editor.is_empty() {
            self.syllable_editor.clear();
            return KeyBehavior::Absorb;
        }

        todo!("CleanAllBuf");

        KeyBehavior::Absorb
    }
    #[instrument]
    fn handle_home(&mut self) -> KeyBehavior {
        self.check_and_reset_range();

        if !self.is_entering() {
            return KeyBehavior::Ignore;
        }

        if !self.is_selecting() {
            self.preedit_buffer.move_cursor_to_beginning();
        }

        KeyBehavior::Absorb
    }
}
