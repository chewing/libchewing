//! TODO: doc

mod estimate;
pub mod keymap;
pub mod layout;
pub mod preedit_buffer;

use std::{collections::HashMap, fmt::Debug, marker::PhantomData, rc::Rc};

pub use estimate::{EstimateError, SqliteUserFreqEstimate, UserFreqEstimate};
pub use layout::SyllableEditor;
use tracing::{instrument, warn};

use crate::{
    conversion::{ChewingConversionEngine, ChineseSequence, ConversionEngine},
    dictionary::{self, Dictionary},
    editor::keymap::KeyCode,
    zhuyin::Syllable,
};

use self::{
    keymap::KeyEvent,
    layout::{KeyBehavior, Standard},
    preedit_buffer::PreeditBuffer,
};

/// Indicates the state change of the editor.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EditorKeyBehavior {
    /// The key has no effect so it was ignored.
    Ignore,
    /// The key caused a conversion update.
    Commit,
    /// The key is an error.
    Bell,
    /// The key changed the editing state so was absorbed.
    Absorb,
}

/// TODO doc.
#[derive(Debug)]
struct Editor<C: 'static, D: 'static> {
    state: &'static dyn ChewingEditorState<C, D>,
    preedit_buffer: PreeditBuffer,
    commit_buffer: String,
    feedback_buffer: String,
    candidate_selector: CandidateSelector,
    syllable_editor: Box<dyn SyllableEditor>,
    conversion_engine: C,
    dictionary: D,
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
    fn key_press(&mut self, key_event: EditorKeyEvent) -> EditorKeyBehavior;
}

trait ChewingEditorState<C, D>: Debug {
    fn process_event(
        &self,
        editor: &mut Editor<C, D>,
        key_event: EditorKeyEvent,
    ) -> (EditorKeyBehavior, &'static dyn ChewingEditorState<C, D>);
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
#[derive(Debug, Clone, Copy)]
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

impl<C, D> BasicEditor for Editor<C, D> {
    type State = ();

    fn key_press(&mut self, key_event: EditorKeyEvent) -> EditorKeyBehavior {
        let (key_behavior, new_state) = self.state.process_event(self, key_event);
        self.state = new_state;
        key_behavior
    }
}

impl<C, D> ChewingEditorState<C, D> for Entering {
    fn process_event(
        &self,
        editor: &mut Editor<C, D>,
        key_event: EditorKeyEvent,
    ) -> (EditorKeyBehavior, &'static dyn ChewingEditorState<C, D>) {
        use EditorKeyEvent::*;

        match key_event {
            Backspace => {
                editor.preedit_buffer.remove_before_cursor();

                (EditorKeyBehavior::Absorb, &Entering)
            }
            CapsLock => {
                editor.switch_language_mode();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            CtrlNum(num) => {
                if num == 0 || num == 1 {
                    editor.start_hanin_symbol_input();
                    return (EditorKeyBehavior::Absorb, &Selecting);
                }

                if (2..=9).contains(&num) {
                    todo!("handle add new phrases with ctrl-num")
                } else {
                    warn!(num = num, "CtrlNum used with number out of range: {}", num);
                    return (EditorKeyBehavior::Bell, &Entering);
                }
                (EditorKeyBehavior::Absorb, &Entering)
            }
            DoubleTab => {
                // editor.reset_user_break_and_connect_at_cursor();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Del => {
                editor.preedit_buffer.remove_after_cursor();

                (EditorKeyBehavior::Absorb, &Entering)
            }
            Down => {
                todo!("Handle new selection");
                (EditorKeyBehavior::Absorb, &Selecting)
            }
            End => {
                editor.preedit_buffer.move_cursor_to_end();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Enter => {
                todo!("Handle commit");
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Esc => {
                todo!("Handle clean all buf");
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Default(evt) => match editor.syllable_editor.key_press(evt) {
                KeyBehavior::Absorb => (EditorKeyBehavior::Absorb, &EnteringSyllable),
                _ => (EditorKeyBehavior::Bell, &EnteringSyllable),
            },
            _ => (EditorKeyBehavior::Ignore, &Entering),
        }
    }
}

impl<C, D> ChewingEditorState<C, D> for EnteringSyllable {
    fn process_event(
        &self,
        editor: &mut Editor<C, D>,
        key_event: EditorKeyEvent,
    ) -> (EditorKeyBehavior, &'static dyn ChewingEditorState<C, D>) {
        use EditorKeyEvent::*;

        match key_event {
            Backspace => {
                editor.syllable_editor.remove_last();

                (
                    EditorKeyBehavior::Absorb,
                    if !editor.syllable_editor.is_empty() {
                        &EnteringSyllable
                    } else {
                        &Entering
                    },
                )
            }
            CapsLock => {
                editor.syllable_editor.clear();
                editor.switch_language_mode();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Esc => {
                editor.syllable_editor.clear();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Default(evt) => match editor.syllable_editor.key_press(evt) {
                KeyBehavior::Absorb => (EditorKeyBehavior::Absorb, &EnteringSyllable),
                KeyBehavior::Commit => {
                    editor.preedit_buffer.insert(editor.syllable_buffer());
                    editor.syllable_editor.clear();
                    (EditorKeyBehavior::Absorb, &Entering)
                }
                _ => (EditorKeyBehavior::Bell, &EnteringSyllable),
            },
            _ => (EditorKeyBehavior::Ignore, &EnteringSyllable),
        }
    }
}

impl<C, D> ChewingEditorState<C, D> for Selecting {
    fn process_event(
        &self,
        editor: &mut Editor<C, D>,
        key_event: EditorKeyEvent,
    ) -> (EditorKeyBehavior, &'static dyn ChewingEditorState<C, D>) {
        use EditorKeyEvent::*;

        match key_event {
            Backspace => {
                editor.cancel_selecting();

                (EditorKeyBehavior::Absorb, &Entering)
            }
            CapsLock => {
                editor.switch_language_mode();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Down => (EditorKeyBehavior::Absorb, &Selecting),
            Default(ev) => {
                todo!("Handle selecting num");
                (EditorKeyBehavior::Absorb, &Entering)
            }
            _ => (EditorKeyBehavior::Ignore, &Selecting),
        }
    }
}

impl<C, D> ChewingEditorState<C, D> for Highlighting {
    fn process_event(
        &self,
        editor: &mut Editor<C, D>,
        key_event: EditorKeyEvent,
    ) -> (EditorKeyBehavior, &'static dyn ChewingEditorState<C, D>) {
        use EditorKeyEvent::*;

        match key_event {
            CapsLock => {
                editor.switch_language_mode();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Enter => {
                todo!("Handle learn");
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Default(_) => {
                todo!();
                (EditorKeyBehavior::Absorb, &EnteringSyllable)
            }
            _ => (EditorKeyBehavior::Ignore, &Highlighting),
        }
    }
}

impl Editor<ChewingConversionEngine, Rc<dyn Dictionary>> {
    fn new(
        conversion_engine: ChewingConversionEngine,
        dictionary: Rc<dyn Dictionary>,
    ) -> Editor<ChewingConversionEngine, Rc<dyn Dictionary>> {
        Editor {
            state: &Entering,
            preedit_buffer: PreeditBuffer::default(),
            commit_buffer: String::new(),
            feedback_buffer: String::new(),
            candidate_selector: CandidateSelector,
            syllable_editor: Box::new(Standard::new()),
            dictionary,
            conversion_engine,
        }
    }
    fn preedit_buffer(&self) -> String {
        self.conversion_engine
            .convert(&ChineseSequence {
                syllables: dbg!(self.preedit_buffer.syllables()),
                selections: vec![],
                breaks: vec![],
            })
            .into_iter()
            .map(|interval| interval.phrase)
            .collect()
    }
}

impl<C, D> Editor<C, D> {
    // fn with_syllable_editor(syllable_editor: Box<dyn SyllableEditor>) -> Editor<C, D> {
    //     Editor {
    //         state: &Entering,
    //         preedit_buffer: PreeditBuffer::default(),
    //         commit_buffer: String::new(),
    //         feedback_buffer: String::new(),
    //         candidate_selector: CandidateSelector,
    //         syllable_editor,
    //     }
    // }
    fn syllable_buffer(&self) -> Syllable {
        self.syllable_editor.read()
    }
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
    fn switch_language_mode(&mut self) {
        todo!()
    }
    fn start_hanin_symbol_input(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, rc::Rc};

    use crate::{
        conversion::ChewingConversionEngine, dictionary::Dictionary, editor::EditorKeyBehavior,
        syl, zhuyin::Bopomofo,
    };

    use super::{
        keymap::{KeyCode, KeyEvent, KeyIndex},
        layout::Standard,
        BasicEditor, Editor, EditorKeyEvent,
    };

    #[test]
    fn editing_mode_input_bopomofo() {
        let dict: Rc<dyn Dictionary> = Rc::new(HashMap::new());
        let conversion_engine = ChewingConversionEngine::new(dict.clone());
        let mut editor = Editor::new(conversion_engine, dict);

        let key_behavior = editor.key_press(EditorKeyEvent::Default(KeyEvent {
            index: KeyIndex::K32,
            code: KeyCode::H,
        }));

        assert_eq!(EditorKeyBehavior::Absorb, key_behavior);
        assert_eq!(syl![Bopomofo::C], editor.syllable_buffer());

        let key_behavior = editor.key_press(EditorKeyEvent::Default(KeyEvent {
            index: KeyIndex::K34,
            code: KeyCode::K,
        }));

        assert_eq!(EditorKeyBehavior::Absorb, key_behavior);
        assert_eq!(syl![Bopomofo::C, Bopomofo::E], editor.syllable_buffer());
    }

    #[test]
    fn editing_mode_input_bopomofo_commit() {
        let dict: Rc<dyn Dictionary> = Rc::new(HashMap::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]));

        let conversion_engine = ChewingConversionEngine::new(dict.clone());
        let mut editor = Editor::new(conversion_engine, dict);

        let keys = [
            EditorKeyEvent::Default(KeyEvent {
                index: KeyIndex::K32,
                code: KeyCode::H,
            }),
            EditorKeyEvent::Default(KeyEvent {
                index: KeyIndex::K34,
                code: KeyCode::K,
            }),
            EditorKeyEvent::Default(KeyEvent {
                index: KeyIndex::K4,
                code: KeyCode::N4,
            }),
        ];

        let key_behaviors: Vec<_> = keys.iter().map(|&key| editor.key_press(key)).collect();

        assert_eq!(
            vec![
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Absorb
            ],
            key_behaviors
        );
        assert!(editor.syllable_buffer().is_empty());
        assert_eq!("冊", editor.preedit_buffer());
    }

    #[test]
    fn editing_mode_input_chinese_to_english_mode() {}

    #[test]
    fn editing_mode_input_english_to_chinese_mode() {}

    #[test]
    fn editing_mode_input_special_symbol() {}

    #[test]
    fn editing_mode_input_full_shape_symbol() {}

    #[test]
    fn editing_mode_input_symbol() {}
}
