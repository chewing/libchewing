//! TODO: doc

pub mod composition_editor;
mod estimate;
pub mod keyboard;
pub mod syllable;

use std::{fmt::Debug, rc::Rc};

pub use estimate::{EstimateError, SqliteUserFreqEstimate, UserFreqEstimate};
pub use syllable::SyllableEditor;
use tracing::warn;

use crate::{
    conversion::{full_width_symbol_input, ChewingConversionEngine, ConversionEngine, Symbol},
    dictionary::Dictionary,
    editor::keyboard::KeyCode,
    zhuyin::Syllable,
};

use self::{
    composition_editor::CompositionEditor,
    keyboard::KeyEvent,
    syllable::{KeyBehavior, Standard},
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
    fn process_keyevent(&mut self, key_event: KeyEvent) -> EditorKeyBehavior;
}

trait ChewingEditorState<C, D>: Debug {
    fn process_keyevent(
        &self,
        editor: &mut Editor<C, D>,
        key_event: KeyEvent,
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

    fn process_keyevent(&mut self, key_event: KeyEvent) -> EditorKeyBehavior {
        let (key_behavior, new_state) = self.state.process_keyevent(self, key_event);
        self.state = new_state;
        key_behavior
    }
}

impl<C, D> ChewingEditorState<C, D> for Entering {
    fn process_keyevent(
        &self,
        editor: &mut Editor<C, D>,
        key_event: KeyEvent,
    ) -> (EditorKeyBehavior, &'static dyn ChewingEditorState<C, D>) {
        use KeyCode::*;

        match key_event.code {
            Backspace => {
                editor.composition.remove_before_cursor();

                (EditorKeyBehavior::Absorb, &Entering)
            }
            Unknown if key_event.modifiers.capslock => {
                editor.switch_language_mode();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            code @ (N0 | N1 | N2 | N3 | N4 | N5 | N6 | N7 | N8 | N9)
                if key_event.modifiers.ctrl =>
            {
                if code == N0 || code == N1 {
                    editor.start_hanin_symbol_input();
                    return (EditorKeyBehavior::Absorb, &Selecting);
                }

                todo!("handle add new phrases with ctrl-num");
                (EditorKeyBehavior::Absorb, &Entering)
            }
            // DoubleTab => {
            //     // editor.reset_user_break_and_connect_at_cursor();
            //     (EditorKeyBehavior::Absorb, &Entering)
            // }
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
            _ => match editor.language_mode {
                LanguageMode::Chinese => match editor.syllable_editor.key_press(key_event) {
                    KeyBehavior::Absorb => (EditorKeyBehavior::Absorb, &EnteringSyllable),
                    _ => (EditorKeyBehavior::Bell, &EnteringSyllable),
                },
                LanguageMode::English => {
                    match editor.character_form {
                        CharacterForm::Halfwidth => {
                            editor.composition.push(Symbol::Char(key_event.unicode))
                        }
                        CharacterForm::Fullwidth => {
                            let char_ = full_width_symbol_input(key_event.unicode).unwrap();
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
    fn process_keyevent(
        &self,
        editor: &mut Editor<C, D>,
        key_event: KeyEvent,
    ) -> (EditorKeyBehavior, &'static dyn ChewingEditorState<C, D>) {
        use KeyCode::*;

        match key_event.code {
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
            Unknown if key_event.modifiers.capslock => {
                editor.syllable_editor.clear();
                editor.switch_language_mode();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Esc => {
                editor.syllable_editor.clear();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            _ => match editor.syllable_editor.key_press(key_event) {
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
    fn process_keyevent(
        &self,
        editor: &mut Editor<C, D>,
        key_event: KeyEvent,
    ) -> (EditorKeyBehavior, &'static dyn ChewingEditorState<C, D>) {
        use KeyCode::*;

        match key_event.code {
            Backspace => {
                editor.cancel_selecting();

                (EditorKeyBehavior::Absorb, &Entering)
            }
            Unknown if key_event.modifiers.capslock => {
                editor.switch_language_mode();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Down => (EditorKeyBehavior::Absorb, &Selecting),
            _ => {
                todo!("Handle selecting num");
                (EditorKeyBehavior::Absorb, &Entering)
            }
            _ => (EditorKeyBehavior::Ignore, &Selecting),
        }
    }
}

impl<C, D> ChewingEditorState<C, D> for Highlighting {
    fn process_keyevent(
        &self,
        editor: &mut Editor<C, D>,
        key_event: KeyEvent,
    ) -> (EditorKeyBehavior, &'static dyn ChewingEditorState<C, D>) {
        use KeyCode::*;

        match key_event.code {
            Unknown if key_event.modifiers.capslock => {
                editor.switch_language_mode();
                (EditorKeyBehavior::Absorb, &Entering)
            }
            Enter => {
                todo!("Handle learn");
                (EditorKeyBehavior::Absorb, &Entering)
            }
            _ => {
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
        conversion::ChewingConversionEngine,
        dictionary::Dictionary,
        editor::{keyboard::Modifiers, EditorKeyBehavior},
        syl,
        zhuyin::Bopomofo,
    };

    use super::{
        keyboard::{KeyCode, KeyboardLayout, Qwerty},
        BasicEditor, Editor,
    };

    #[test]
    fn editing_mode_input_bopomofo() {
        let keyboard = Qwerty;
        let dict: Rc<dyn Dictionary> = Rc::new(HashMap::new());
        let conversion_engine = ChewingConversionEngine::new(dict.clone());
        let mut editor = Editor::new(conversion_engine, dict);

        let ev = keyboard.map_keycode(KeyCode::H, Modifiers::default());
        let key_behavior = editor.process_keyevent(ev);

        assert_eq!(EditorKeyBehavior::Absorb, key_behavior);
        assert_eq!(syl![Bopomofo::C], editor.syllable_buffer());

        let ev = keyboard.map_keycode(KeyCode::K, Modifiers::default());
        let key_behavior = editor.process_keyevent(ev);

        assert_eq!(EditorKeyBehavior::Absorb, key_behavior);
        assert_eq!(syl![Bopomofo::C, Bopomofo::E], editor.syllable_buffer());
    }

    #[test]
    fn editing_mode_input_bopomofo_commit() {
        let keyboard = Qwerty;
        let dict: Rc<dyn Dictionary> = Rc::new(HashMap::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]));

        let conversion_engine = ChewingConversionEngine::new(dict.clone());
        let mut editor = Editor::new(conversion_engine, dict);

        let keys = [KeyCode::H, KeyCode::K, KeyCode::N4];
        let key_behaviors: Vec<_> = keys
            .into_iter()
            .map(|key| keyboard.map_keycode(key, Modifiers::default()))
            .map(|ev| editor.process_keyevent(ev))
            .collect();

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
        let keyboard = Qwerty;
        let dict: Rc<dyn Dictionary> = Rc::new(HashMap::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]));

        let conversion_engine = ChewingConversionEngine::new(dict.clone());
        let mut editor = Editor::new(conversion_engine, dict);

        let keys = [
            keyboard.map_keycode(KeyCode::H, Default::default()),
            keyboard.map_keycode(KeyCode::K, Default::default()),
            keyboard.map_keycode(KeyCode::N4, Default::default()),
            keyboard.map_keycode(
                KeyCode::Unknown,
                Modifiers {
                    shift: false,
                    ctrl: false,
                    capslock: true,
                },
            ),
            keyboard.map_keycode(KeyCode::Z, Default::default()),
        ];

        let key_behaviors: Vec<_> = keys
            .iter()
            .map(|&key| editor.process_keyevent(key))
            .collect();

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
        let keyboard = Qwerty;
        let dict: Rc<dyn Dictionary> = Rc::new(HashMap::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]));

        let conversion_engine = ChewingConversionEngine::new(dict.clone());
        let mut editor = Editor::new(conversion_engine, dict);

        let keys = [
            keyboard.map_keycode(
                KeyCode::Unknown,
                Modifiers {
                    shift: false,
                    ctrl: false,
                    capslock: true,
                },
            ),
            keyboard.map_keycode(KeyCode::X, Default::default()),
            keyboard.map_keycode(
                KeyCode::Unknown,
                Modifiers {
                    shift: false,
                    ctrl: false,
                    capslock: true,
                },
            ),
            keyboard.map_keycode(KeyCode::H, Default::default()),
            keyboard.map_keycode(KeyCode::K, Default::default()),
            keyboard.map_keycode(KeyCode::N4, Default::default()),
        ];

        let key_behaviors: Vec<_> = keys
            .iter()
            .map(|&key| editor.process_keyevent(key))
            .collect();

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
        let keyboard = Qwerty;
        let dictionary = Rc::new(HashMap::new());
        let conversion_engine = ChewingConversionEngine::new(dictionary.clone());
        let mut editor = Editor::new(conversion_engine, dictionary);
        editor.switch_character_form();

        let keys = [
            keyboard.map_keycode(
                KeyCode::Unknown,
                Modifiers {
                    shift: false,
                    ctrl: false,
                    capslock: true,
                },
            ),
            keyboard.map_keycode(KeyCode::N0, Default::default()),
            keyboard.map_keycode(KeyCode::Minus, Default::default()),
        ];

        let key_behaviors: Vec<_> = keys
            .iter()
            .map(|&key| editor.process_keyevent(key))
            .collect();

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
