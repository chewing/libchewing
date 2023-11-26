//! TODO: doc

pub mod composition_editor;
mod estimate;
pub mod keyboard;
pub mod syllable;

use std::mem;

pub use estimate::{EstimateError, SqliteUserFreqEstimate, UserFreqEstimate};
pub use syllable::SyllableEditor;

use crate::{
    conversion::{full_width_symbol_input, special_symbol_input, ConversionEngine, Symbol},
    dictionary::Dictionary,
    editor::keyboard::KeyCode,
    zhuyin::Syllable,
};

use self::{
    composition_editor::CompositionEditor,
    keyboard::KeyEvent,
    syllable::{KeyBehavior, Standard},
};

#[derive(Debug, Clone, Copy)]
pub enum LanguageMode {
    Chinese,
    English,
}

#[derive(Debug, Clone, Copy)]
pub enum CharacterForm {
    Halfwidth,
    Fullwidth,
}

enum UserPhraseAddDirection {
    Forward,
    Backward,
}

#[derive(Debug)]
pub struct EditorOptions {
    pub esc_clear_all_buffer: bool,
    pub space_is_select_key: bool,
    pub auto_shift_cursor: bool,
    pub phrase_choice_rearward: bool,
    pub auto_learn_phrase: bool,
    pub auto_commit_threshold: usize,
}

impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            esc_clear_all_buffer: true,
            space_is_select_key: true,
            auto_shift_cursor: true,
            phrase_choice_rearward: true,
            auto_learn_phrase: true,
            auto_commit_threshold: 16,
        }
    }
}

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

/// An editor can react to KeyEvents and change its state.
pub trait BasicEditor {
    /// Handles a KeyEvent
    fn process_keyevent(&mut self, key_event: KeyEvent) -> EditorKeyBehavior;
}

#[derive(Debug)]
pub struct Editor<C, D>
where
    C: ConversionEngine,
    D: Dictionary,
{
    state: Transition<C, D>,
}

#[derive(Debug)]
struct EditorCore<C, D>
where
    C: ConversionEngine,
    D: Dictionary,
{
    composition: CompositionEditor,
    commit_buffer: String,
    feedback_buffer: String,
    candidate_selector: CandidateSelector,
    syllable_editor: Box<dyn SyllableEditor>,
    conversion_engine: C,
    dictionary: D,
    language_mode: LanguageMode,
    character_form: CharacterForm,
    options: EditorOptions,
}

/// TODO doc.
#[derive(Debug)]
struct EditorState<C, D, S>
where
    C: ConversionEngine,
    D: Dictionary,
{
    core: EditorCore<C, D>,
    state: S,
}

#[derive(Debug)]
struct CandidateSelector;

#[derive(Debug)]
enum Transition<C, D>
where
    C: ConversionEngine,
    D: Dictionary,
{
    Entering(EditorKeyBehavior, EditorState<C, D, Entering>),
    EnteringSyllable(EditorKeyBehavior, EditorState<C, D, EnteringSyllable>),
    Selecting(EditorKeyBehavior, EditorState<C, D, Selecting>),
    Highlighting(EditorKeyBehavior, EditorState<C, D, Highlighting>),
    Invalid,
}

#[derive(Debug)]
struct Entering;

#[derive(Debug)]
struct EnteringSyllable;

#[derive(Debug)]
struct Selecting;

#[derive(Debug)]
struct Highlighting;

impl<C, D> From<EditorState<C, D, Entering>> for EditorState<C, D, Selecting>
where
    C: ConversionEngine,
    D: Dictionary,
{
    fn from(value: EditorState<C, D, Entering>) -> Self {
        EditorState {
            core: value.core,
            state: Selecting,
        }
    }
}

impl<C, D> From<EditorState<C, D, Selecting>> for EditorState<C, D, Entering>
where
    C: ConversionEngine,
    D: Dictionary,
{
    fn from(value: EditorState<C, D, Selecting>) -> Self {
        EditorState {
            core: value.core,
            state: Entering,
        }
    }
}

impl<C, D> From<EditorState<C, D, Entering>> for EditorState<C, D, EnteringSyllable>
where
    C: ConversionEngine,
    D: Dictionary,
{
    fn from(value: EditorState<C, D, Entering>) -> Self {
        EditorState {
            core: value.core,
            state: EnteringSyllable,
        }
    }
}

impl<C, D> From<EditorState<C, D, EnteringSyllable>> for EditorState<C, D, Entering>
where
    C: ConversionEngine,
    D: Dictionary,
{
    fn from(value: EditorState<C, D, EnteringSyllable>) -> Self {
        EditorState {
            core: value.core,
            state: Entering,
        }
    }
}

impl<C, D> From<EditorState<C, D, Highlighting>> for EditorState<C, D, Entering>
where
    C: ConversionEngine,
    D: Dictionary,
{
    fn from(value: EditorState<C, D, Highlighting>) -> Self {
        EditorState {
            core: value.core,
            state: Entering,
        }
    }
}

impl<C, D> Editor<C, D>
where
    C: ConversionEngine,
    D: Dictionary,
{
    pub fn new(conversion_engine: C, dictionary: D) -> Editor<C, D> {
        Editor {
            state: Transition::Entering(
                EditorKeyBehavior::Ignore,
                EditorState::new(conversion_engine, dictionary),
            ),
        }
    }

    pub fn language_mode(&self) -> LanguageMode {
        self.core().language_mode
    }

    pub fn set_language_mode(&mut self, language_mode: LanguageMode) {
        self.core_mut().language_mode = language_mode;
    }

    pub fn character_form(&self) -> CharacterForm {
        self.core().character_form
    }

    pub fn set_character_form(&mut self, charactor_form: CharacterForm) {
        self.core_mut().character_form = charactor_form;
    }

    fn last_key_behavior(&self) -> EditorKeyBehavior {
        match self.state {
            Transition::Entering(ekb, _) => ekb,
            Transition::EnteringSyllable(ekb, _) => ekb,
            Transition::Selecting(ekb, _) => ekb,
            Transition::Highlighting(ekb, _) => ekb,
            Transition::Invalid => EditorKeyBehavior::Ignore,
        }
    }
}

impl<C, D> BasicEditor for Editor<C, D>
where
    C: ConversionEngine,
    D: Dictionary,
{
    fn process_keyevent(&mut self, key_event: KeyEvent) -> EditorKeyBehavior {
        dbg!(&key_event);
        let old_state = mem::replace(&mut self.state, Transition::Invalid);
        self.state = match old_state {
            Transition::Entering(_, s) => s.process_keyevent(key_event),
            Transition::EnteringSyllable(_, s) => s.process_keyevent(key_event),
            Transition::Selecting(_, s) => s.process_keyevent(key_event),
            Transition::Highlighting(_, s) => s.process_keyevent(key_event),
            Transition::Invalid => Transition::Invalid,
        };
        self.last_key_behavior()
    }
}

impl<C, D> EditorState<C, D, Entering>
where
    C: ConversionEngine,
    D: Dictionary,
{
    fn process_keyevent(mut self, key_event: KeyEvent) -> Transition<C, D> {
        use KeyCode::*;

        match key_event.code {
            Backspace => {
                self.core.composition.remove_before_cursor();

                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Unknown if key_event.modifiers.capslock => {
                self.switch_language_mode();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            code @ (N0 | N1 | N2 | N3 | N4 | N5 | N6 | N7 | N8 | N9)
                if key_event.modifiers.ctrl =>
            {
                if code == N0 || code == N1 {
                    self.start_hanin_symbol_input();
                    return Transition::Selecting(EditorKeyBehavior::Absorb, self.into());
                }

                todo!("handle add new phrases with ctrl-num");
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            // DoubleTab => {
            //     // editor.reset_user_break_and_connect_at_cursor();
            //     (EditorKeyBehavior::Absorb, &Entering)
            // }
            Del => {
                self.core.composition.remove_after_cursor();

                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Left => {
                self.core.composition.move_cursor_left();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Down => {
                if !self.core.composition.is_cursor_on_syllable() {
                    return Transition::Entering(EditorKeyBehavior::Ignore, self);
                }
                let end = 3; //self.core.composition.cursor();
                if self.core.options.phrase_choice_rearward {
                    // TODO remember current cursor
                    self.core.composition.rewind_cursor_to_break_point();
                }
                let start = self.core.composition.cursor();
                let syllables: Vec<_> = self
                    .core
                    .composition
                    .slice(start, end)
                    .iter()
                    .map(|sym| sym.as_syllable())
                    .collect();
                let phrases = self
                    .core
                    .dictionary
                    .lookup_phrase(&syllables)
                    .collect::<Vec<_>>();

                dbg!(syllables);
                dbg!(phrases);
                Transition::Selecting(EditorKeyBehavior::Absorb, self.into())
            }
            End => {
                self.core.composition.move_cursor_to_end();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Enter => {
                todo!("Handle commit");
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Esc => {
                todo!("Handle clean all buf");
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            _ => match self.core.language_mode {
                LanguageMode::Chinese if key_event.modifiers.shift => {
                    match special_symbol_input(key_event.unicode) {
                        Some(symbol) => {
                            self.core.composition.push(Symbol::Char(symbol));
                            Transition::Entering(EditorKeyBehavior::Absorb, self)
                        }
                        None => Transition::Entering(EditorKeyBehavior::Ignore, self),
                    }
                }
                LanguageMode::Chinese => match self.core.syllable_editor.key_press(key_event) {
                    KeyBehavior::Absorb => {
                        Transition::EnteringSyllable(EditorKeyBehavior::Absorb, self.into())
                    }
                    _ => Transition::Entering(EditorKeyBehavior::Bell, self),
                },
                LanguageMode::English => {
                    match self.core.character_form {
                        CharacterForm::Halfwidth => {
                            if self.core.composition.is_empty() {
                                self.core.commit_buffer.clear();
                                self.core.commit_buffer.push(key_event.unicode);
                            } else {
                                self.core.composition.push(Symbol::Char(key_event.unicode));
                            }
                        }
                        CharacterForm::Fullwidth => {
                            let char_ = full_width_symbol_input(key_event.unicode).unwrap();
                            if self.core.composition.is_empty() {
                                self.core.commit_buffer.clear();
                                self.core.commit_buffer.push(char_);
                            } else {
                                self.core.composition.push(Symbol::Char(char_));
                            }
                        }
                    }
                    Transition::Entering(EditorKeyBehavior::Commit, self)
                }
            },
        }
    }
}

impl<C, D> EditorState<C, D, EnteringSyllable>
where
    C: ConversionEngine,
    D: Dictionary,
{
    fn process_keyevent(mut self, key_event: KeyEvent) -> Transition<C, D> {
        use KeyCode::*;

        match key_event.code {
            Backspace => {
                self.core.syllable_editor.remove_last();

                if !self.core.syllable_editor.is_empty() {
                    Transition::EnteringSyllable(EditorKeyBehavior::Absorb, self)
                } else {
                    Transition::Entering(EditorKeyBehavior::Absorb, self.into())
                }
            }
            Unknown if key_event.modifiers.capslock => {
                self.core.syllable_editor.clear();
                self.switch_language_mode();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Esc => {
                self.core.syllable_editor.clear();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            _ => match self.core.syllable_editor.key_press(key_event) {
                KeyBehavior::Absorb => {
                    Transition::EnteringSyllable(EditorKeyBehavior::Absorb, self)
                }
                KeyBehavior::Commit => {
                    self.core
                        .composition
                        .push(Symbol::Syllable(self.core.syllable_editor.read()));
                    self.core.syllable_editor.clear();
                    Transition::Entering(EditorKeyBehavior::Absorb, self.into())
                }
                _ => Transition::EnteringSyllable(EditorKeyBehavior::Bell, self),
            },
        }
    }
}

impl<C, D> EditorState<C, D, Selecting>
where
    C: ConversionEngine,
    D: Dictionary,
{
    fn process_keyevent(mut self, key_event: KeyEvent) -> Transition<C, D> {
        use KeyCode::*;

        match key_event.code {
            Backspace => {
                self.cancel_selecting();

                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Unknown if key_event.modifiers.capslock => {
                self.switch_language_mode();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Down => Transition::Selecting(EditorKeyBehavior::Absorb, self),
            _ => {
                todo!("Handle selecting num");
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
        }
    }
}

impl<C, D> EditorState<C, D, Highlighting>
where
    C: ConversionEngine,
    D: Dictionary,
{
    fn process_keyevent(mut self, key_event: KeyEvent) -> Transition<C, D> {
        use KeyCode::*;

        match key_event.code {
            Unknown if key_event.modifiers.capslock => {
                self.switch_language_mode();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Enter => {
                todo!("Handle learn");
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            _ => {
                todo!();
                // Transition::EnteringSyllable(EditorKeyBehavior::Absorb, self.into())
            }
        }
    }
}

impl<C, D> EditorState<C, D, Entering>
where
    C: ConversionEngine,
    D: Dictionary,
{
    /// TODO: doc
    fn new(conversion_engine: C, dictionary: D) -> EditorState<C, D, Entering> {
        EditorState {
            core: EditorCore {
                composition: CompositionEditor::default(),
                commit_buffer: String::new(),
                feedback_buffer: String::new(),
                candidate_selector: CandidateSelector,
                syllable_editor: Box::new(Standard::new()),
                dictionary,
                conversion_engine,
                language_mode: LanguageMode::Chinese,
                character_form: CharacterForm::Halfwidth,
                options: Default::default(),
            },
            state: Entering,
        }
    }
}

impl<C, D> Editor<C, D>
where
    C: ConversionEngine,
    D: Dictionary,
{
    /// TODO: doc, rename this to `render`?
    pub fn display(&self) -> String {
        self.core()
            .conversion_engine
            .convert(self.core().composition.as_ref())
            .into_iter()
            .map(|interval| interval.phrase)
            .collect::<String>()
    }

    // TODO: decide the return type
    pub fn display_commit(&self) -> &str {
        &self.core().commit_buffer
    }

    fn syllable_buffer(&self) -> Syllable {
        self.core().syllable_editor.read()
    }

    fn switch_character_form(&mut self) {
        match &mut self.state {
            Transition::Entering(_, s) => s.switch_character_form(),
            Transition::EnteringSyllable(_, s) => s.switch_character_form(),
            Transition::Selecting(_, s) => s.switch_character_form(),
            Transition::Highlighting(_, s) => s.switch_character_form(),
            Transition::Invalid => unreachable!(),
        }
    }

    fn core(&self) -> &EditorCore<C, D> {
        match &self.state {
            Transition::Entering(_, s) => &s.core,
            Transition::EnteringSyllable(_, s) => &s.core,
            Transition::Selecting(_, s) => &s.core,
            Transition::Highlighting(_, s) => &s.core,
            Transition::Invalid => unreachable!(),
        }
    }

    fn core_mut(&mut self) -> &mut EditorCore<C, D> {
        match &mut self.state {
            Transition::Entering(_, s) => &mut s.core,
            Transition::EnteringSyllable(_, s) => &mut s.core,
            Transition::Selecting(_, s) => &mut s.core,
            Transition::Highlighting(_, s) => &mut s.core,
            Transition::Invalid => unreachable!(),
        }
    }
}

impl<C, D, S> EditorState<C, D, S>
where
    C: ConversionEngine,
    D: Dictionary,
{
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
        self.core.language_mode = match self.core.language_mode {
            LanguageMode::English => LanguageMode::Chinese,
            LanguageMode::Chinese => LanguageMode::English,
        }
    }
    fn switch_character_form(&mut self) {
        self.core.character_form = match self.core.character_form {
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

        let ev = keyboard.map(KeyCode::H);
        let key_behavior = editor.process_keyevent(ev);

        assert_eq!(EditorKeyBehavior::Absorb, key_behavior);
        assert_eq!(syl![Bopomofo::C], editor.syllable_buffer());

        let ev = keyboard.map(KeyCode::K);
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
            .map(|key| keyboard.map(key))
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
            keyboard.map(KeyCode::H),
            keyboard.map(KeyCode::K),
            keyboard.map(KeyCode::N4),
            // TODO: capslock probably shouldn't be a modifier
            // Toggle english mode
            keyboard.map_with_mod(
                KeyCode::Unknown,
                Modifiers {
                    shift: false,
                    ctrl: false,
                    capslock: true,
                },
            ),
            keyboard.map(KeyCode::Z),
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
            // Switch to english mode
            keyboard.map_with_mod(
                KeyCode::Unknown,
                Modifiers {
                    shift: false,
                    ctrl: false,
                    capslock: true,
                },
            ),
            keyboard.map(KeyCode::X),
            // Switch to chinese mode
            keyboard.map_with_mod(
                KeyCode::Unknown,
                Modifiers {
                    shift: false,
                    ctrl: false,
                    capslock: true,
                },
            ),
            keyboard.map(KeyCode::H),
            keyboard.map(KeyCode::K),
            keyboard.map(KeyCode::N4),
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
        assert_eq!("x", editor.display_commit());
        assert_eq!("冊", editor.display());
    }

    #[test]
    fn editing_chinese_mode_input_special_symbol() {
        let keyboard = Qwerty;
        let dictionary = Rc::new(HashMap::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]));
        let conversion_engine = ChewingConversionEngine::new(dictionary.clone());
        let mut editor = Editor::new(conversion_engine, dictionary);

        let keys = [
            keyboard.map_with_mod(KeyCode::N1, Modifiers::shift()),
            keyboard.map_with_mod(KeyCode::H, Modifiers::default()),
            keyboard.map_with_mod(KeyCode::K, Modifiers::default()),
            keyboard.map_with_mod(KeyCode::N4, Modifiers::default()),
            keyboard.map_with_mod(KeyCode::Comma, Modifiers::shift()),
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
                EditorKeyBehavior::Absorb,
            ],
            key_behaviors
        );
        assert!(editor.syllable_buffer().is_empty());
        assert_eq!("！冊，", editor.display());
    }

    #[test]
    fn editing_mode_input_full_shape_symbol() {
        let keyboard = Qwerty;
        let dictionary = Rc::new(HashMap::new());
        let conversion_engine = ChewingConversionEngine::new(dictionary.clone());
        let mut editor = Editor::new(conversion_engine, dictionary);
        editor.switch_character_form();

        let steps = [
            (
                KeyCode::Unknown,
                Modifiers {
                    shift: false,
                    ctrl: false,
                    capslock: true,
                },
                EditorKeyBehavior::Absorb,
                "",
                "",
                "",
            ),
            (
                KeyCode::N0,
                Modifiers::default(),
                EditorKeyBehavior::Commit,
                "",
                "",
                "０",
            ),
            (
                KeyCode::Minus,
                Modifiers::default(),
                EditorKeyBehavior::Commit,
                "",
                "",
                "－",
            ),
        ];

        for s in steps {
            let key = keyboard.map_with_mod(s.0, s.1);
            let kb = editor.process_keyevent(key);
            assert_eq!(s.2, kb);
            assert_eq!(s.3, editor.syllable_buffer().to_string());
            assert_eq!(s.4, editor.display());
            assert_eq!(s.5, editor.display_commit());
        }
    }

    #[test]
    fn editing_mode_input_symbol() {}
}
