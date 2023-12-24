//! TODO: doc

pub mod composition_editor;
mod estimate;
pub mod keyboard;
mod selection;
pub mod syllable;

use std::{io, mem};

pub use estimate::{EstimateError, SqliteUserFreqEstimate, UserFreqEstimate};
pub use syllable::SyllableEditor;
use tracing::{debug, trace};

use crate::{
    conversion::{
        full_width_symbol_input, special_symbol_input, ConversionEngine, Interval, Symbol,
    },
    dictionary::{Dictionary, Phrases},
    editor::keyboard::KeyCode,
    zhuyin::Syllable,
};

use self::{
    composition_editor::CompositionEditor,
    keyboard::KeyEvent,
    selection::{
        phrase::PhraseSelector,
        symbol::{SpecialSymbolSelector, SymbolSelector},
    },
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

#[derive(Debug, Clone, Copy)]
pub enum UserPhraseAddDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy)]
pub struct EditorOptions {
    pub esc_clear_all_buffer: bool,
    pub space_is_select_key: bool,
    pub auto_shift_cursor: bool,
    pub phrase_choice_rearward: bool,
    pub auto_learn_phrase: bool,
    pub auto_commit_threshold: usize,
    pub language_mode: LanguageMode,
    pub character_form: CharacterForm,
    pub user_phrase_add_dir: UserPhraseAddDirection,
}

impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            esc_clear_all_buffer: false,
            space_is_select_key: true,
            auto_shift_cursor: true,
            phrase_choice_rearward: false,
            auto_learn_phrase: true,
            auto_commit_threshold: 16,
            language_mode: LanguageMode::Chinese,
            character_form: CharacterForm::Halfwidth,
            user_phrase_add_dir: UserPhraseAddDirection::Backward,
        }
    }
}

/// An editor can react to KeyEvents and change its state.
pub trait BasicEditor {
    /// Handles a KeyEvent
    fn process_keyevent(&mut self, key_event: KeyEvent) -> EditorKeyBehavior;
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

#[derive(Debug)]
pub struct Editor<C, D>
where
    C: ConversionEngine,
    D: Dictionary + Clone,
{
    com: CompositionEditor,
    syl: Box<dyn SyllableEditor>,
    conv: C,
    dict: D,
    options: EditorOptions,
    state: Transition,

    commit_buffer: String,
}

impl<C, D> Editor<C, D>
where
    C: ConversionEngine,
    D: Dictionary + Clone,
{
    pub fn new(conv: C, dict: D) -> Editor<C, D> {
        Editor {
            com: CompositionEditor::default(),
            syl: Box::new(Standard::new()),
            conv,
            dict,
            options: EditorOptions::default(),
            state: Transition::Entering(EditorKeyBehavior::Ignore, Entering),
            commit_buffer: String::new(),
        }
    }
    pub fn language_mode(&self) -> LanguageMode {
        self.options.language_mode
    }
    pub fn set_language_mode(&mut self, language_mode: LanguageMode) {
        self.syl.clear();
        self.options.language_mode = language_mode;
    }
    pub fn character_form(&self) -> CharacterForm {
        self.options.character_form
    }
    pub fn set_character_form(&mut self, charactor_form: CharacterForm) {
        self.options.character_form = charactor_form;
    }
    pub fn last_key_behavior(&self) -> EditorKeyBehavior {
        match self.state {
            Transition::Entering(ekb, _) => ekb,
            Transition::EnteringSyllable(ekb, _) => ekb,
            Transition::Selecting(ekb, _) => ekb,
            Transition::Highlighting(ekb, _) => ekb,
            Transition::Invalid => EditorKeyBehavior::Ignore,
        }
    }
    pub fn entering_syllable(&self) -> bool {
        !self.syl.is_empty()
    }
    pub fn cursor(&self) -> usize {
        self.com.cursor()
    }
    pub fn intervals(&self) -> impl Iterator<Item = Interval> {
        self.conv.convert(self.com.as_ref()).into_iter()
    }
    /// TODO: doc, rename this to `render`?
    pub fn display(&self) -> String {
        self.conv
            .convert(self.com.as_ref())
            .into_iter()
            .map(|interval| interval.phrase)
            .collect::<String>()
    }
    // pub fn commit(&mut self) {
    //     let output = self.display();
    //     self.core_mut().commit_buffer.push_str(&output);
    // }
    // TODO: decide the return type
    pub fn display_commit(&self) -> &str {
        &self.commit_buffer
    }
    pub fn syllable_buffer(&self) -> Syllable {
        self.syl.read()
    }
    pub fn list_candidates(&self) -> Result<Vec<String>, ()> {
        debug!("state {:?}", self.state);
        match &self.state {
            Transition::Selecting(_, sub_state) => match sub_state {
                Selecting::Phrase(sel) => Ok(sel.candidates(&self.dict)),
                Selecting::Symbol(sel) => Ok(sel.menu()),
                Selecting::SpecialSymmbol(sel) => Ok(sel.menu()),
            },
            _ => Err(()),
        }
    }
    pub fn switch_character_form(&mut self) {
        self.options = EditorOptions {
            character_form: match self.options.character_form {
                CharacterForm::Halfwidth => CharacterForm::Fullwidth,
                CharacterForm::Fullwidth => CharacterForm::Halfwidth,
            },
            ..self.options
        };
    }
    pub fn switch_language_mode(&mut self) {
        self.options = EditorOptions {
            language_mode: match self.options.language_mode {
                LanguageMode::English => LanguageMode::Chinese,
                LanguageMode::Chinese => LanguageMode::English,
            },
            ..self.options
        };
    }
    pub fn editor_options(&self) -> EditorOptions {
        self.options
    }
    pub fn set_editor_options(&mut self, options: EditorOptions) {
        self.options = options;
    }
    // fn check_and_reset_range(&mut self) {
    //     todo!()
    // }
    // fn is_entering(&self) -> bool {
    //     todo!()
    // }
    // fn is_selecting(&self) -> bool {
    //     todo!()
    // }
    // fn start_selecting(&mut self) {
    //     todo!()
    // }
    fn cancel_selecting(&mut self) {
        // pop cursor?
    }
    fn start_hanin_symbol_input(&mut self) {
        todo!()
    }
}

impl<C, D> BasicEditor for Editor<C, D>
where
    C: ConversionEngine,
    D: Dictionary + Clone,
{
    fn process_keyevent(&mut self, key_event: KeyEvent) -> EditorKeyBehavior {
        trace!("process_keyevent: {}", &key_event);
        let old_state = mem::replace(&mut self.state, Transition::Invalid);
        self.state = match old_state {
            Transition::Entering(_, s) => s.next(self, key_event),
            Transition::EnteringSyllable(_, s) => s.next(self, key_event),
            Transition::Selecting(_, s) => s.next(self, key_event),
            Transition::Highlighting(_, s) => s.next(self, key_event),
            Transition::Invalid => Transition::Invalid,
        };
        self.last_key_behavior()
    }
}

#[derive(Debug)]
enum Transition {
    Entering(EditorKeyBehavior, Entering),
    EnteringSyllable(EditorKeyBehavior, EnteringSyllable),
    Selecting(EditorKeyBehavior, Selecting),
    Highlighting(EditorKeyBehavior, Highlighting),
    Invalid,
}

#[derive(Debug)]
struct Entering;

#[derive(Debug)]
struct EnteringSyllable;

#[derive(Debug)]
enum Selecting {
    Phrase(PhraseSelector),
    Symbol(SymbolSelector),
    SpecialSymmbol(SpecialSymbolSelector),
}

#[derive(Debug)]
struct SelectingSymbols {
    sel: SymbolSelector,
}

#[derive(Debug)]
struct Highlighting;

impl From<EnteringSyllable> for Entering {
    fn from(_: EnteringSyllable) -> Self {
        Entering
    }
}

impl From<Selecting> for Entering {
    fn from(_: Selecting) -> Self {
        Entering
    }
}

impl From<SelectingSymbols> for Entering {
    fn from(_: SelectingSymbols) -> Self {
        Entering
    }
}

impl From<Highlighting> for Entering {
    fn from(_: Highlighting) -> Self {
        Entering
    }
}

impl Entering {
    fn next<C, D>(self, editor: &mut Editor<C, D>, ev: KeyEvent) -> Transition
    where
        C: ConversionEngine,
        D: Dictionary + Clone,
    {
        use KeyCode::*;

        match ev.code {
            Backspace => {
                editor.com.remove_before_cursor();

                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Unknown if ev.modifiers.capslock => {
                editor.switch_language_mode();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            code @ (N0 | N1 | N2 | N3 | N4 | N5 | N6 | N7 | N8 | N9) if ev.modifiers.ctrl => {
                if code == N0 || code == N1 {
                    editor.start_hanin_symbol_input();
                    return Transition::Selecting(
                        EditorKeyBehavior::Absorb,
                        Selecting::new_phrase(editor, self),
                    );
                }

                todo!("handle add new phrases with ctrl-num");
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            // DoubleTab => {
            //     // editor.reset_user_break_and_connect_at_cursor();
            //     (EditorKeyBehavior::Absorb, &Entering)
            // }
            Del => {
                if editor.com.is_end_of_buffer() {
                    Transition::Entering(EditorKeyBehavior::Ignore, self)
                } else {
                    editor.com.remove_after_cursor();
                    Transition::Entering(EditorKeyBehavior::Absorb, self)
                }
            }
            Home => {
                editor.com.move_cursor_to_beginning();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Left => {
                editor.com.move_cursor_left();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Right => {
                editor.com.move_cursor_right();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Down => {
                debug!("buffer {:?}", editor.com);
                match editor.com.symbol_for_select() {
                    Some(symbol) => match symbol {
                        Symbol::Syllable(_) => Transition::Selecting(
                            EditorKeyBehavior::Absorb,
                            Selecting::new_phrase(editor, self),
                        ),
                        Symbol::Char(_) => Transition::Selecting(
                            EditorKeyBehavior::Absorb,
                            Selecting::new_special_symbol(symbol, self),
                        ),
                    },
                    None => Transition::Entering(EditorKeyBehavior::Ignore, self),
                }
            }
            End => {
                editor.com.move_cursor_to_end();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Enter => {
                let output = editor
                    .conv
                    .convert(editor.com.as_ref())
                    .into_iter()
                    .map(|interval| interval.phrase)
                    .collect::<String>();
                editor.commit_buffer.push_str(&output);
                Transition::Entering(EditorKeyBehavior::Commit, self)
            }
            Esc => {
                if editor.options.esc_clear_all_buffer && !editor.com.is_empty() {
                    editor.com.clear();
                    Transition::Entering(EditorKeyBehavior::Absorb, self)
                } else {
                    Transition::Entering(EditorKeyBehavior::Ignore, self)
                }
            }
            _ => match editor.options.language_mode {
                LanguageMode::Chinese if ev.modifiers.shift => {
                    match special_symbol_input(ev.unicode) {
                        Some(symbol) => {
                            editor.com.push(Symbol::Char(symbol));
                            Transition::Entering(EditorKeyBehavior::Absorb, self)
                        }
                        None => Transition::Entering(EditorKeyBehavior::Ignore, self),
                    }
                }
                LanguageMode::Chinese if ev.code == Grave => Transition::Selecting(
                    EditorKeyBehavior::Absorb,
                    Selecting::new_symbol(editor, Entering),
                ),
                LanguageMode::Chinese => match editor.syl.key_press(ev) {
                    KeyBehavior::Absorb => {
                        Transition::EnteringSyllable(EditorKeyBehavior::Absorb, self.into())
                    }
                    _ => Transition::Entering(EditorKeyBehavior::Bell, self),
                },
                LanguageMode::English => {
                    match editor.options.character_form {
                        CharacterForm::Halfwidth => {
                            if editor.com.is_empty() {
                                editor.commit_buffer.clear();
                                editor.commit_buffer.push(ev.unicode);
                            } else {
                                editor.com.push(Symbol::Char(ev.unicode));
                            }
                        }
                        CharacterForm::Fullwidth => {
                            let char_ = full_width_symbol_input(ev.unicode).unwrap();
                            if editor.com.is_empty() {
                                editor.commit_buffer.clear();
                                editor.commit_buffer.push(char_);
                            } else {
                                editor.com.push(Symbol::Char(char_));
                            }
                        }
                    }
                    Transition::Entering(EditorKeyBehavior::Commit, self)
                }
            },
        }
    }
}

impl From<Entering> for EnteringSyllable {
    fn from(_: Entering) -> Self {
        EnteringSyllable
    }
}

impl EnteringSyllable {
    fn next<C, D>(self, editor: &mut Editor<C, D>, ev: KeyEvent) -> Transition
    where
        C: ConversionEngine,
        D: Dictionary + Clone,
    {
        use KeyCode::*;

        match ev.code {
            Backspace => {
                editor.syl.remove_last();

                if !editor.syl.is_empty() {
                    Transition::EnteringSyllable(EditorKeyBehavior::Absorb, self)
                } else {
                    Transition::Entering(EditorKeyBehavior::Absorb, self.into())
                }
            }
            Unknown if ev.modifiers.capslock => {
                editor.syl.clear();
                editor.switch_language_mode();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Esc => {
                editor.syl.clear();
                if editor.options.esc_clear_all_buffer {
                    editor.com.clear();
                }
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            _ => match editor.syl.key_press(ev) {
                KeyBehavior::Absorb => {
                    Transition::EnteringSyllable(EditorKeyBehavior::Absorb, self)
                }
                KeyBehavior::Commit => {
                    editor.com.push(Symbol::Syllable(editor.syl.read()));
                    editor.syl.clear();
                    Transition::Entering(EditorKeyBehavior::Absorb, self.into())
                }
                _ => Transition::EnteringSyllable(EditorKeyBehavior::Bell, self),
            },
        }
    }
}

impl Selecting {
    fn new_phrase<C, D>(editor: &mut Editor<C, D>, _state: Entering) -> Self
    where
        C: ConversionEngine,
        D: Dictionary + Clone,
    {
        // TODO maintain cursor stack in composition
        // let saved_cursor = editor.com.cursor();
        // debug!("saved_cursor {}", saved_cursor);

        let mut sel = PhraseSelector::new(
            !editor.options.phrase_choice_rearward,
            editor.com.inner.clone(),
        );
        sel.init(editor.cursor(), &editor.dict);

        Selecting::Phrase(sel)
    }
    fn new_symbol<C, D>(editor: &mut Editor<C, D>, _state: Entering) -> Self
    where
        C: ConversionEngine,
        D: Dictionary + Clone,
    {
        // FIXME load from data
        let reader = io::Cursor::new("…\n※\n常用符號=，、。\n");
        let sel = SymbolSelector::new(reader).expect("parse symbols table");
        Selecting::Symbol(sel)
    }
    fn new_special_symbol(symbol: Symbol, _state: Entering) -> Self {
        let sel = SpecialSymbolSelector::new(symbol);
        Selecting::SpecialSymmbol(sel)
    }
    fn next<C, D>(mut self, editor: &mut Editor<C, D>, ev: KeyEvent) -> Transition
    where
        C: ConversionEngine,
        D: Dictionary + Clone,
    {
        use KeyCode::*;

        match ev.code {
            Backspace => {
                editor.cancel_selecting();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Unknown if ev.modifiers.capslock => {
                editor.switch_language_mode();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Up => Transition::Selecting(EditorKeyBehavior::Absorb, self),
            Down => {
                match self {
                    Selecting::Phrase(ref mut sel) => {
                        sel.next(&editor.dict);
                    }
                    Selecting::Symbol(sel) => todo!("next page"),
                    Selecting::SpecialSymmbol(sel) => todo!("next page"),
                }
                Transition::Selecting(EditorKeyBehavior::Absorb, self)
            }
            PageUp => Transition::Selecting(EditorKeyBehavior::Absorb, self),
            PageDown => Transition::Selecting(EditorKeyBehavior::Absorb, self),
            code @ (N1 | N2 | N3 | N4 | N5 | N6 | N7 | N8 | N9) => {
                // TODO allocate less
                let n = code.to_digit().unwrap().saturating_sub(1) as usize;
                match self {
                    Selecting::Phrase(ref sel) => {
                        let candidates = sel.candidates(&editor.dict);
                        match candidates.get(n) {
                            Some(phrase) => {
                                editor.com.select(sel.interval(phrase.into()));
                                debug!("Auto Shift {}", editor.options.auto_shift_cursor);
                                if editor.options.auto_shift_cursor {
                                    editor.com.move_cursor_right();
                                }
                                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
                            }
                            None => Transition::Selecting(EditorKeyBehavior::Bell, self),
                        }
                    }
                    Selecting::Symbol(ref mut sel) => match sel.select(n) {
                        Some(s) => {
                            editor.com.insert(s);
                            Transition::Entering(EditorKeyBehavior::Absorb, self.into())
                        }
                        None => Transition::Selecting(EditorKeyBehavior::Absorb, self),
                    },
                    Selecting::SpecialSymmbol(ref sel) => match sel.select(n) {
                        Some(s) => {
                            editor.com.insert(s);
                            editor.com.remove_before_cursor();
                            Transition::Entering(EditorKeyBehavior::Absorb, self.into())
                        }
                        None => Transition::Selecting(EditorKeyBehavior::Absorb, self),
                    },
                }
            }
            Esc => {
                editor.cancel_selecting();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Del => {
                // NB: should be Ignore but return Absorb for backward compat
                Transition::Selecting(EditorKeyBehavior::Absorb, self)
            }
            _ => {
                unreachable!("invalid state")
            }
        }
    }
}

impl Highlighting {
    fn next<C, D>(self, editor: &mut Editor<C, D>, ev: KeyEvent) -> Transition
    where
        C: ConversionEngine,
        D: Dictionary + Clone,
    {
        use KeyCode::*;

        match ev.code {
            Unknown if ev.modifiers.capslock => {
                editor.switch_language_mode();
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
        let dict = Rc::new(HashMap::new());
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
        let dict = Rc::new(HashMap::from([(
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
        let dict = Rc::new(HashMap::from([(
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
        let dict = Rc::new(HashMap::from([(
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
