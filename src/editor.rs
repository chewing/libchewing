//! TODO: doc

pub mod composition_editor;
mod estimate;
pub mod keymap;
pub mod layout;

use std::{fmt::Debug, rc::Rc};

pub use estimate::{EstimateError, SqliteUserFreqEstimate, UserFreqEstimate};
pub use layout::SyllableEditor;
use tracing::warn;

use crate::{
    conversion::{full_width_symbol_input, ChewingConversionEngine, ConversionEngine, Symbol},
    dictionary::Dictionary,
    zhuyin::Syllable,
};

use self::{
    composition_editor::CompositionEditor,
    keymap::KeyEvent,
    layout::{KeyBehavior, Standard},
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
    composition: CompositionEditor,
    commit_buffer: String,
    feedback_buffer: String,
    candidate_selector: CandidateSelector,
    syllable_editor: Box<dyn SyllableEditor>,
    conversion_engine: C,
    dictionary: D,

    language_mode: LanguageMode,
    character_form: CharacterForm,
}

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

#[derive(Debug)]
enum LanguageMode {
    Chinese,
    English,
}

#[derive(Debug)]
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
                editor.composition.remove_before_cursor();

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
                editor.composition.remove_after_cursor();

                (EditorKeyBehavior::Absorb, &Entering)
            }
            Down => {
                todo!("Handle new selection");
                (EditorKeyBehavior::Absorb, &Selecting)
            }
            End => {
                editor.composition.move_cursor_to_end();
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
            Default(evt) => match editor.language_mode {
                LanguageMode::Chinese => match editor.syllable_editor.key_press(evt) {
                    KeyBehavior::Absorb => (EditorKeyBehavior::Absorb, &EnteringSyllable),
                    _ => (EditorKeyBehavior::Bell, &EnteringSyllable),
                },
                LanguageMode::English => {
                    let char_ = evt.code.to_char();
                    match editor.character_form {
                        CharacterForm::Halfwidth => editor.composition.push(Symbol::Char(char_)),
                        CharacterForm::Fullwidth => {
                            let char_ = full_width_symbol_input(char_).unwrap();
                            editor.composition.push(Symbol::Char(char_));
                        }
                    }
                    (EditorKeyBehavior::Commit, &Entering)
                }
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
                    editor
                        .composition
                        .push(Symbol::Syllable(editor.syllable_buffer()));
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
            composition: CompositionEditor::default(),
            commit_buffer: String::new(),
            feedback_buffer: String::new(),
            candidate_selector: CandidateSelector,
            syllable_editor: Box::new(Standard::new()),
            dictionary,
            conversion_engine,
            language_mode: LanguageMode::Chinese,
            character_form: CharacterForm::Halfwidth,
        }
    }
    fn display(&self) -> String {
        self.conversion_engine
            .convert(self.composition.as_ref())
            .into_iter()
            .map(|interval| interval.phrase)
            .collect::<String>()
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
        self.language_mode = match self.language_mode {
            LanguageMode::English => LanguageMode::Chinese,
            LanguageMode::Chinese => LanguageMode::English,
        }
    }
    fn switch_character_form(&mut self) {
        self.character_form = match self.character_form {
            CharacterForm::Fullwidth => CharacterForm::Halfwidth,
            CharacterForm::Halfwidth => CharacterForm::Fullwidth,
        }
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
        assert_eq!("冊", editor.display());
    }

    #[test]
    fn editing_mode_input_chinese_to_english_mode() {
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
            EditorKeyEvent::CapsLock,
            EditorKeyEvent::Default(KeyEvent {
                index: KeyIndex::K39,
                code: KeyCode::Z,
            }),
        ];

        let key_behaviors: Vec<_> = keys.iter().map(|&key| editor.key_press(key)).collect();

        assert_eq!(
            vec![
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Commit
            ],
            key_behaviors
        );
        assert!(editor.syllable_buffer().is_empty());
        assert_eq!("冊z", editor.display());
    }

    #[test]
    fn editing_mode_input_english_to_chinese_mode() {
        let dict: Rc<dyn Dictionary> = Rc::new(HashMap::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]));

        let conversion_engine = ChewingConversionEngine::new(dict.clone());
        let mut editor = Editor::new(conversion_engine, dict);

        let keys = [
            EditorKeyEvent::CapsLock,
            EditorKeyEvent::Default(KeyEvent {
                index: KeyIndex::K39,
                code: KeyCode::X,
            }),
            EditorKeyEvent::CapsLock,
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
                EditorKeyBehavior::Commit,
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Absorb,
            ],
            key_behaviors
        );
        assert!(editor.syllable_buffer().is_empty());
        assert_eq!("x冊", editor.display());
    }

    #[test]
    fn editing_mode_input_special_symbol() {}

    #[test]
    fn editing_mode_input_full_shape_symbol() {
        let dictionary = Rc::new(HashMap::new());
        let conversion_engine = ChewingConversionEngine::new(dictionary.clone());
        let mut editor = Editor::new(conversion_engine, dictionary);
        editor.switch_character_form();
        let keys = [
            EditorKeyEvent::CapsLock,
            EditorKeyEvent::Default(KeyEvent {
                index: KeyIndex::K10,
                code: KeyCode::N0,
            }),
            EditorKeyEvent::Default(KeyEvent {
                index: KeyIndex::K11,
                code: KeyCode::Minus,
            }),
        ];

        let key_behaviors: Vec<_> = keys.iter().map(|&key| editor.key_press(key)).collect();

        assert_eq!(
            vec![
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Commit,
                EditorKeyBehavior::Commit,
            ],
            key_behaviors
        );
        assert!(editor.syllable_buffer().is_empty());
        assert_eq!("０－", editor.display());
    }

    #[test]
    fn editing_mode_input_symbol() {}
}
