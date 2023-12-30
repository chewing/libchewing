//! TODO: doc

mod abbrev;
pub mod composition_editor;
mod estimate;
pub mod keyboard;
mod selection;
pub mod syllable;

use std::{
    cmp::{max, min},
    io, mem,
};

pub use estimate::{EstimateError, SqliteUserFreqEstimate, UserFreqEstimate};
pub use syllable::SyllableEditor;
use tracing::{debug, trace, warn};

use crate::{
    conversion::{
        full_width_symbol_input, special_symbol_input, ConversionEngine, Interval, Symbol,
    },
    dictionary::{AnyDictionary, Dictionary, LayeredDictionary},
    editor::keyboard::KeyCode,
    syl,
    zhuyin::Syllable,
};

use self::{
    abbrev::AbbrevTable,
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
    pub easy_symbol_input: bool,
    pub esc_clear_all_buffer: bool,
    pub space_is_select_key: bool,
    pub auto_shift_cursor: bool,
    pub phrase_choice_rearward: bool,
    pub auto_learn_phrase: bool,
    pub auto_commit_threshold: usize,
    pub candidates_per_page: usize,
    pub language_mode: LanguageMode,
    pub character_form: CharacterForm,
    pub user_phrase_add_dir: UserPhraseAddDirection,
}

impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            easy_symbol_input: false,
            esc_clear_all_buffer: false,
            space_is_select_key: false,
            auto_shift_cursor: true,
            phrase_choice_rearward: false,
            auto_learn_phrase: true,
            auto_commit_threshold: 16,
            candidates_per_page: 10,
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
    C: ConversionEngine<D>,
    D: Dictionary,
{
    com: CompositionEditor,
    syl: Box<dyn SyllableEditor>,
    conv: C,
    dict: D,
    abbr: AbbrevTable,
    options: EditorOptions,
    state: Transition,

    nth_conversion: usize,
    commit_buffer: String,
    notice_buffer: String,
}

impl<C, D> Editor<C, D>
where
    C: ConversionEngine<D>,
    D: Dictionary,
{
    pub fn new(conv: C, dict: D) -> Editor<C, D> {
        Editor {
            com: CompositionEditor::default(),
            syl: Box::new(Standard::new()),
            conv,
            dict,
            abbr: AbbrevTable::new().expect("unable to init abbrev table"),
            options: EditorOptions::default(),
            state: Transition::Entering(EditorKeyBehavior::Ignore, Entering),
            nth_conversion: 0,
            commit_buffer: String::new(),
            notice_buffer: String::new(),
        }
    }
    pub fn clear(&mut self) {
        self.com.clear();
        self.syl.clear();
        self.commit_buffer.clear();
    }
    pub fn clear_syllable_editor(&mut self) {
        self.syl.clear();
    }
    pub fn set_syllable_editor(&mut self, syl: Box<dyn SyllableEditor>) {
        self.syl = syl;
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
    fn conversion(&self) -> Vec<Interval> {
        if self.nth_conversion == 0 {
            self.conv.convert(&self.dict, self.com.as_ref())
        } else {
            self.conv
                .convert_next(&self.dict, self.com.as_ref(), self.nth_conversion)
        }
    }
    pub fn intervals(&self) -> impl Iterator<Item = Interval> {
        self.conversion().into_iter()
    }
    /// TODO: doc, rename this to `render`?
    pub fn display(&self) -> String {
        self.conversion()
            .into_iter()
            .map(|interval| interval.phrase)
            .collect::<String>()
    }
    // TODO: decide the return type
    pub fn display_commit(&self) -> &str {
        &self.commit_buffer
    }
    pub fn syllable_buffer(&self) -> Syllable {
        self.syl.read()
    }
    pub fn notification(&self) -> &str {
        &self.notice_buffer
    }
    pub fn symbols(&self) -> &[Symbol] {
        &self.com.inner.buffer
    }
    pub fn len(&self) -> usize {
        self.com.inner.buffer.len()
    }
    /// All candidates after current page
    pub fn paginated_candidates(&self) -> Result<Vec<String>, ()> {
        debug!("state {:?}", self.state);
        match &self.state {
            Transition::Selecting(_, sub_state) => Ok(sub_state
                .candidates(self, &self.dict)
                .into_iter()
                .skip(sub_state.page_no * self.options.candidates_per_page)
                .collect()),
            _ => Err(()),
        }
    }
    pub fn all_candidates(&self) -> Result<Vec<String>, ()> {
        debug!("state {:?}", self.state);
        match &self.state {
            Transition::Selecting(_, sub_state) => Ok(sub_state.candidates(self, &self.dict)),
            _ => Err(()),
        }
    }
    pub fn current_page_no(&self) -> Result<usize, ()> {
        debug!("state {:?}", self.state);
        match &self.state {
            Transition::Selecting(_, sub_state) => Ok(sub_state.page_no),
            _ => Err(()),
        }
    }
    pub fn total_page(&self) -> Result<usize, ()> {
        match &self.state {
            Transition::Selecting(_, sub_state) => Ok(sub_state.total_page(self, &self.dict)),
            _ => Err(()),
        }
    }
    pub fn select(&mut self, n: usize) -> Result<(), ()> {
        if !self.is_selecting() {
            return Err(());
        }
        let old_state = mem::replace(&mut self.state, Transition::Invalid);
        self.state = match old_state {
            Transition::Selecting(_, s) => s.select(self, n),
            _ => old_state,
        };
        if self.last_key_behavior() == EditorKeyBehavior::Absorb {
            self.try_auto_commit();
        }
        if self.last_key_behavior() == EditorKeyBehavior::Bell {
            Err(())
        } else {
            Ok(())
        }
    }
    #[tracing::instrument(skip(self), ret)]
    fn learn_phrase_in_range(&mut self, start: usize, end: usize) -> Result<String, &'static str> {
        let result = self.learn_phrase_in_range_quiet(start, end);
        match result {
            Ok(ref phrase) => self.notice_buffer = format!("加入：{}", phrase),
            Err(msg) => self.notice_buffer = msg.to_owned(),
        }
        result
    }
    fn learn_phrase_in_range_quiet(
        &mut self,
        start: usize,
        end: usize,
    ) -> Result<String, &'static str> {
        if end > self.com.inner.buffer.len() {
            return Err("");
        }
        if self.com.inner.buffer[start..end]
            .iter()
            .any(Symbol::is_char)
        {
            return Err("加詞失敗：字數不符或夾雜符號");
        }
        let phrase = self
            .display()
            .chars()
            .skip(start)
            .take(end - start)
            .collect::<String>();
        self.dict
            .insert(&self.com.inner.buffer[start..end], (&phrase, 100).into())
            .map(|_| phrase)
            .map_err(|_| "已有：")
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
    pub fn is_entering(&self) -> bool {
        match self.state {
            Transition::Entering(_, _) => true,
            _ => false,
        }
    }
    pub fn is_selecting(&self) -> bool {
        match self.state {
            Transition::Selecting(_, _) => true,
            _ => false,
        }
    }
    pub fn start_selecting(&mut self) -> Result<(), ()> {
        let old_state = mem::replace(&mut self.state, Transition::Invalid);
        self.state = match old_state {
            Transition::Entering(_, s) => s.start_selecting(self),
            // Force entering selection
            Transition::EnteringSyllable(_, s) => Entering::from(s).start_selecting(self),
            _ => old_state,
        };
        if self.is_selecting() {
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn has_next_selection_point(&self) -> bool {
        match &self.state {
            Transition::Selecting(_, s) => match &s.sel {
                Selector::Phrase(s) => s.next_selection_point(&self.dict).is_some(),
                Selector::Symbol(_) => false,
                Selector::SpecialSymmbol(_) => false,
            },
            _ => false,
        }
    }
    pub fn has_prev_selection_point(&self) -> bool {
        match &self.state {
            Transition::Selecting(_, s) => match &s.sel {
                Selector::Phrase(s) => s.prev_selection_point(&self.dict).is_some(),
                Selector::Symbol(_) => false,
                Selector::SpecialSymmbol(_) => false,
            },
            _ => false,
        }
    }
    pub fn jump_to_next_selection_point(&mut self) -> Result<(), ()> {
        match &mut self.state {
            Transition::Selecting(_, s) => match &mut s.sel {
                Selector::Phrase(s) => s.jump_to_next_selection_point(&self.dict),
                Selector::Symbol(_) => Err(()),
                Selector::SpecialSymmbol(_) => Err(()),
            },
            _ => Err(()),
        }
    }
    pub fn jump_to_prev_selection_point(&mut self) -> Result<(), ()> {
        match &mut self.state {
            Transition::Selecting(_, s) => match &mut s.sel {
                Selector::Phrase(s) => s.jump_to_prev_selection_point(&self.dict),
                Selector::Symbol(_) => Err(()),
                Selector::SpecialSymmbol(_) => Err(()),
            },
            _ => Err(()),
        }
    }
    pub fn jump_to_first_selection_point(&mut self) {
        match &mut self.state {
            Transition::Selecting(_, s) => match &mut s.sel {
                Selector::Phrase(s) => s.jump_to_first_selection_point(&self.dict),
                Selector::Symbol(_) => {}
                Selector::SpecialSymmbol(_) => {}
            },
            _ => {}
        }
    }
    pub fn jump_to_last_selection_point(&mut self) {
        match &mut self.state {
            Transition::Selecting(_, s) => match &mut s.sel {
                Selector::Phrase(s) => s.jump_to_last_selection_point(&self.dict),
                Selector::Symbol(_) => {}
                Selector::SpecialSymmbol(_) => {}
            },
            _ => {}
        }
    }
    fn cancel_selecting(&mut self) {
        // pop cursor?
    }
    fn start_hanin_symbol_input(&mut self) {
        todo!()
    }
    fn try_auto_commit(&mut self) {
        let intervals: Vec<_> = self.intervals().collect();
        let len: usize = intervals.iter().map(|it| it.len()).sum();
        if len <= self.options.auto_commit_threshold {
            return;
        }

        let mut remove = 0;
        self.commit_buffer.clear();
        for it in intervals {
            self.commit_buffer.push_str(&it.phrase);
            remove += it.len();
            if len - remove <= self.options.auto_commit_threshold {
                break;
            }
        }
        self.com.inner.buffer.splice(0..remove, []);
        self.state = Transition::Entering(EditorKeyBehavior::Commit, Entering);
        // FIXME fix selections and breaks
    }
}

impl<C> Editor<C, LayeredDictionary<AnyDictionary, ()>>
where
    C: ConversionEngine<LayeredDictionary<AnyDictionary, ()>>,
{
    pub fn user_dict(&mut self) -> &mut AnyDictionary {
        self.dict.base()
    }
}

impl<C, D> BasicEditor for Editor<C, D>
where
    C: ConversionEngine<D>,
    D: Dictionary,
{
    fn process_keyevent(&mut self, key_event: KeyEvent) -> EditorKeyBehavior {
        trace!("process_keyevent: {}", &key_event);
        // reset?
        self.notice_buffer.clear();
        let old_state = mem::replace(&mut self.state, Transition::Invalid);
        self.state = match old_state {
            Transition::Entering(_, s) => s.next(self, key_event),
            Transition::EnteringSyllable(_, s) => s.next(self, key_event),
            Transition::Selecting(_, s) => s.next(self, key_event),
            Transition::Highlighting(_, s) => s.next(self, key_event),
            Transition::Invalid => Transition::Invalid,
        };
        if self.last_key_behavior() == EditorKeyBehavior::Absorb {
            self.try_auto_commit();
        }
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
struct Selecting {
    page_no: usize,
    action: SelectingAction,
    sel: Selector,
}

#[derive(Debug)]
enum SelectingAction {
    Insert,
    Replace,
}

#[derive(Debug)]
enum Selector {
    Phrase(PhraseSelector),
    Symbol(SymbolSelector),
    SpecialSymmbol(SpecialSymbolSelector),
}

#[derive(Debug)]
struct Highlighting {
    moving_cursor: usize,
}

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

impl From<Highlighting> for Entering {
    fn from(_: Highlighting) -> Self {
        Entering
    }
}

impl Entering {
    fn start_selecting<C, D>(self, editor: &mut Editor<C, D>) -> Transition
    where
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        match editor.com.symbol_for_select() {
            Some(symbol) => match symbol {
                Symbol::Syllable(_) => Transition::Selecting(
                    EditorKeyBehavior::Absorb,
                    Selecting::new_phrase(editor, self),
                ),
                Symbol::Char(_) => Transition::Selecting(
                    EditorKeyBehavior::Absorb,
                    Selecting::new_special_symbol(editor, symbol, self),
                ),
            },
            None => Transition::Entering(EditorKeyBehavior::Ignore, self),
        }
    }
    fn next<C, D>(mut self, editor: &mut Editor<C, D>, ev: KeyEvent) -> Transition
    where
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        use KeyCode::*;

        match ev.code {
            Backspace => {
                if editor.com.is_empty() {
                    Transition::Entering(EditorKeyBehavior::Ignore, self)
                } else {
                    editor.com.remove_before_cursor();

                    Transition::Entering(EditorKeyBehavior::Absorb, self)
                }
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

                debug!("handle add new phrases with ctrl-num");
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Tab if editor.com.is_end_of_buffer() => {
                editor.nth_conversion += 1;
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Tab => {
                let interval_ends: Vec<_> = editor.conversion().iter().map(|it| it.end).collect();
                if interval_ends.contains(&editor.cursor()) {
                    editor.com.insert_glue();
                } else {
                    editor.com.insert_break();
                }
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
            Left if ev.modifiers.shift => {
                if editor.com.is_empty() || editor.cursor() == 0 {
                    return Transition::Entering(EditorKeyBehavior::Ignore, self);
                }
                Transition::Highlighting(
                    EditorKeyBehavior::Absorb,
                    Highlighting::new(editor.cursor() - 1, editor, self),
                )
            }
            Right if ev.modifiers.shift => {
                if editor.com.is_empty() || editor.com.is_end_of_buffer() {
                    return Transition::Entering(EditorKeyBehavior::Ignore, self);
                }
                Transition::Highlighting(
                    EditorKeyBehavior::Absorb,
                    Highlighting::new(editor.cursor() + 1, editor, self),
                )
            }
            Left => {
                editor.com.move_cursor_left();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Right => {
                editor.com.move_cursor_right();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Up => Transition::Entering(EditorKeyBehavior::Ignore, self),
            Space if ev.modifiers.shift => {
                editor.options.character_form = match editor.options.character_form {
                    CharacterForm::Halfwidth => CharacterForm::Fullwidth,
                    CharacterForm::Fullwidth => CharacterForm::Halfwidth,
                };
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Space if editor.options.space_is_select_key => {
                debug!("buffer {:?}", editor.com);
                match editor.com.symbol_for_select() {
                    Some(symbol) => match symbol {
                        Symbol::Syllable(_) => Transition::Selecting(
                            EditorKeyBehavior::Absorb,
                            Selecting::new_phrase(editor, self),
                        ),
                        Symbol::Char(_) => Transition::Selecting(
                            EditorKeyBehavior::Absorb,
                            Selecting::new_special_symbol(editor, symbol, self),
                        ),
                    },
                    None if editor.com.is_empty() => {
                        match editor.options.character_form {
                            CharacterForm::Halfwidth => editor.commit_buffer.push(' '),
                            CharacterForm::Fullwidth => editor.commit_buffer.push('　'),
                        }
                        Transition::Entering(EditorKeyBehavior::Absorb, self)
                    }
                    None => Transition::Entering(EditorKeyBehavior::Ignore, self),
                }
            }
            Down => {
                debug!("buffer {:?}", editor.com);
                self.start_selecting(editor)
            }
            End | PageUp | PageDown => {
                editor.com.move_cursor_to_end();
                Transition::Entering(EditorKeyBehavior::Absorb, self)
            }
            Enter => {
                editor.commit_buffer.clear();
                let output = editor
                    .conversion()
                    .into_iter()
                    .map(|interval| interval.phrase)
                    .collect::<String>();
                editor.commit_buffer.push_str(&output);
                editor.com.clear();
                editor.nth_conversion = 0;
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
            _ if ev.modifiers.numlock => {
                if editor.com.is_empty() {
                    editor.commit_buffer.clear();
                    editor.commit_buffer.push(ev.unicode);
                } else {
                    editor.com.push(Symbol::Char(ev.unicode));
                }
                Transition::Entering(EditorKeyBehavior::Commit, self)
            }
            _ => match editor.options.language_mode {
                LanguageMode::Chinese if ev.code == Grave && ev.modifiers.is_none() => {
                    Transition::Selecting(
                        EditorKeyBehavior::Absorb,
                        Selecting::new_symbol(editor, Entering),
                    )
                }
                LanguageMode::Chinese if ev.code == Space => {
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
                LanguageMode::Chinese if editor.options.easy_symbol_input => {
                    // Priortize symbol input
                    if let Some(expended) = editor.abbr.find_abbrev(ev.unicode) {
                        expended
                            .chars()
                            .for_each(|ch| editor.com.push(Symbol::Char(ch)));
                        return Transition::Entering(EditorKeyBehavior::Absorb, self);
                    }
                    if let Some(symbol) = special_symbol_input(ev.unicode) {
                        editor.com.push(Symbol::Char(symbol));
                        return Transition::Entering(EditorKeyBehavior::Absorb, self);
                    }
                    if ev.modifiers.is_none() && KeyBehavior::Absorb == editor.syl.key_press(ev) {
                        return Transition::EnteringSyllable(
                            EditorKeyBehavior::Absorb,
                            self.into(),
                        );
                    }
                    Transition::Entering(EditorKeyBehavior::Bell, self)
                }
                LanguageMode::Chinese => {
                    if ev.modifiers.is_none() && KeyBehavior::Absorb == editor.syl.key_press(ev) {
                        return Transition::EnteringSyllable(
                            EditorKeyBehavior::Absorb,
                            self.into(),
                        );
                    }
                    if let Some(symbol) = special_symbol_input(ev.unicode) {
                        editor.com.push(Symbol::Char(symbol));
                        return Transition::Entering(EditorKeyBehavior::Absorb, self);
                    }
                    Transition::Entering(EditorKeyBehavior::Bell, self)
                }
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
        C: ConversionEngine<D>,
        D: Dictionary,
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
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        editor.com.push_cursor();
        editor.com.clamp_cursor();

        let mut sel = PhraseSelector::new(
            !editor.options.phrase_choice_rearward,
            editor.com.inner.clone(),
        );
        sel.init(editor.cursor(), &editor.dict);

        Selecting {
            page_no: 0,
            action: SelectingAction::Replace,
            sel: Selector::Phrase(sel),
        }
    }
    fn new_symbol<C, D>(_editor: &mut Editor<C, D>, _state: Entering) -> Self
    where
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        // FIXME load from data
        let reader = io::Cursor::new(include_str!("../data/symbols.dat"));
        let sel = SymbolSelector::new(reader).expect("parse symbols table");
        Selecting {
            page_no: 0,
            action: SelectingAction::Insert,
            sel: Selector::Symbol(sel),
        }
    }
    fn new_special_symbol<C, D>(editor: &mut Editor<C, D>, symbol: Symbol, _state: Entering) -> Self
    where
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        editor.com.push_cursor();
        editor.com.clamp_cursor();

        let sel = SpecialSymbolSelector::new(symbol);
        if sel.menu().is_empty() {
            // If there's no special symbol then fallback to dynamic symbol table
            let mut sel = Self::new_symbol(editor, _state);
            sel.action = SelectingAction::Replace;
            sel
        } else {
            Selecting {
                page_no: 0,
                action: SelectingAction::Replace,
                sel: Selector::SpecialSymmbol(sel),
            }
        }
    }
    fn candidates<C, D>(&self, editor: &Editor<C, D>, dict: &D) -> Vec<String>
    where
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        match &self.sel {
            Selector::Phrase(sel) => sel.candidates(editor, dict),
            Selector::Symbol(sel) => sel.menu(),
            Selector::SpecialSymmbol(sel) => sel.menu(),
        }
    }
    fn total_page<C, D>(&self, editor: &Editor<C, D>, dict: &D) -> usize
    where
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        self.candidates(editor, dict)
            .len()
            .div_ceil(editor.options.candidates_per_page)
    }
    fn select<C, D>(mut self, editor: &mut Editor<C, D>, n: usize) -> Transition
    where
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        let offset = self.page_no * editor.options.candidates_per_page + n;
        match self.sel {
            Selector::Phrase(ref sel) => {
                let candidates = sel.candidates(editor, &editor.dict);
                match candidates.get(n) {
                    Some(phrase) => {
                        editor.com.select(sel.interval(phrase.into()));
                        debug!("Auto Shift {}", editor.options.auto_shift_cursor);
                        editor.com.pop_cursor();
                        if editor.options.auto_shift_cursor {
                            editor.com.move_cursor_right();
                        }
                        Transition::Entering(EditorKeyBehavior::Absorb, self.into())
                    }
                    None => Transition::Selecting(EditorKeyBehavior::Bell, self),
                }
            }
            Selector::Symbol(ref mut sel) => match sel.select(offset) {
                Some(s) => {
                    match self.action {
                        SelectingAction::Insert => editor.com.insert(s),
                        SelectingAction::Replace => editor.com.replace(s),
                    }
                    editor.com.pop_cursor();
                    Transition::Entering(EditorKeyBehavior::Absorb, self.into())
                }
                None => {
                    self.page_no = 0;
                    Transition::Selecting(EditorKeyBehavior::Absorb, self)
                }
            },
            Selector::SpecialSymmbol(ref sel) => match sel.select(offset) {
                Some(s) => {
                    match self.action {
                        SelectingAction::Insert => editor.com.insert(s),
                        SelectingAction::Replace => editor.com.replace(s),
                    }
                    editor.com.pop_cursor();
                    Transition::Entering(EditorKeyBehavior::Absorb, self.into())
                }
                None => {
                    self.page_no = 0;
                    Transition::Selecting(EditorKeyBehavior::Absorb, self)
                }
            },
        }
    }
    fn next<C, D>(mut self, editor: &mut Editor<C, D>, ev: KeyEvent) -> Transition
    where
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        use KeyCode::*;

        match ev.code {
            Backspace => {
                editor.cancel_selecting();
                editor.com.pop_cursor();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Unknown if ev.modifiers.capslock => {
                editor.switch_language_mode();
                editor.com.pop_cursor();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Up => {
                editor.cancel_selecting();
                editor.com.pop_cursor();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Space if editor.options.space_is_select_key => {
                if self.page_no < self.total_page(editor, &editor.dict) - 1 {
                    self.page_no += 1;
                } else {
                    self.page_no = 0;
                    match &mut self.sel {
                        Selector::Phrase(sel) => {
                            sel.next(&editor.dict);
                        }
                        Selector::Symbol(_sel) => (),
                        Selector::SpecialSymmbol(_sel) => (),
                    }
                }
                Transition::Selecting(EditorKeyBehavior::Absorb, self)
            }
            Down => {
                match &mut self.sel {
                    Selector::Phrase(sel) => {
                        sel.next(&editor.dict);
                    }
                    Selector::Symbol(_sel) => (),
                    Selector::SpecialSymmbol(_sel) => (),
                }
                Transition::Selecting(EditorKeyBehavior::Absorb, self)
            }
            J => {
                let begin = match &self.sel {
                    Selector::Phrase(sel) => sel.begin(),
                    Selector::Symbol(_) => editor.com.cursor(),
                    Selector::SpecialSymmbol(_) => editor.com.cursor(),
                };
                editor.com.move_cursor(begin.saturating_sub(1));
                match editor.com.symbol().expect("should have symbol") {
                    Symbol::Syllable(_) => {
                        let mut sel = PhraseSelector::new(
                            !editor.options.phrase_choice_rearward,
                            editor.com.inner.clone(),
                        );
                        sel.init(editor.cursor(), &editor.dict);
                        self.sel = Selector::Phrase(sel);
                    }
                    sym @ Symbol::Char(_) => {
                        let sel = SpecialSymbolSelector::new(*sym);
                        self.sel = Selector::SpecialSymmbol(sel);
                    }
                }
                Transition::Selecting(EditorKeyBehavior::Absorb, self)
            }
            K => {
                let begin = match &self.sel {
                    Selector::Phrase(sel) => sel.begin(),
                    Selector::Symbol(_) => editor.com.cursor(),
                    Selector::SpecialSymmbol(_) => editor.com.cursor(),
                };
                editor.com.move_cursor(begin.saturating_add(1));
                editor.com.clamp_cursor();
                match editor.com.symbol().expect("should have symbol") {
                    Symbol::Syllable(_) => {
                        let mut sel = PhraseSelector::new(
                            !editor.options.phrase_choice_rearward,
                            editor.com.inner.clone(),
                        );
                        sel.init(editor.cursor(), &editor.dict);
                        self.sel = Selector::Phrase(sel);
                    }
                    sym @ Symbol::Char(_) => {
                        let sel = SpecialSymbolSelector::new(*sym);
                        self.sel = Selector::SpecialSymmbol(sel);
                    }
                }
                Transition::Selecting(EditorKeyBehavior::Absorb, self)
            }
            Left | PageUp => {
                if self.page_no > 0 {
                    self.page_no -= 1;
                } else {
                    self.page_no = self.total_page(editor, &editor.dict) - 1;
                }
                Transition::Selecting(EditorKeyBehavior::Absorb, self)
            }
            Right | PageDown => {
                if self.page_no < self.total_page(editor, &editor.dict) - 1 {
                    self.page_no += 1;
                } else {
                    self.page_no = 0;
                }
                Transition::Selecting(EditorKeyBehavior::Absorb, self)
            }
            code @ (N1 | N2 | N3 | N4 | N5 | N6 | N7 | N8 | N9 | N0) => {
                // TODO allocate less
                let n = code.to_digit().unwrap().saturating_sub(1) as usize;
                self.select(editor, n)
            }
            Esc => {
                editor.cancel_selecting();
                editor.com.pop_cursor();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Del => {
                // NB: should be Ignore but return Absorb for backward compat
                Transition::Selecting(EditorKeyBehavior::Absorb, self)
            }
            _ => {
                warn!("Invalid state transition");
                Transition::Selecting(EditorKeyBehavior::Bell, self)
            }
        }
    }
}

impl Highlighting {
    fn new<C, D>(moving_cursor: usize, editor: &mut Editor<C, D>, _state: Entering) -> Self
    where
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        Highlighting { moving_cursor }
    }
    fn next<C, D>(mut self, editor: &mut Editor<C, D>, ev: KeyEvent) -> Transition
    where
        C: ConversionEngine<D>,
        D: Dictionary,
    {
        use KeyCode::*;

        match ev.code {
            Unknown if ev.modifiers.capslock => {
                editor.switch_language_mode();
                Transition::Entering(EditorKeyBehavior::Absorb, self.into())
            }
            Left if ev.modifiers.shift => {
                if self.moving_cursor != 0 {
                    self.moving_cursor -= 1;
                }
                Transition::Highlighting(EditorKeyBehavior::Absorb, self)
            }
            Right if ev.modifiers.shift => {
                if self.moving_cursor != editor.com.inner.buffer.len() {
                    self.moving_cursor += 1;
                }
                Transition::Highlighting(EditorKeyBehavior::Absorb, self)
            }
            Enter => {
                let start = min(self.moving_cursor, editor.com.cursor());
                let end = max(self.moving_cursor, editor.com.cursor());
                editor.com.move_cursor(self.moving_cursor);
                match editor.learn_phrase_in_range(start, end) {
                    Ok(_) => Transition::Entering(EditorKeyBehavior::Absorb, self.into()),
                    Err(_) => Transition::Entering(EditorKeyBehavior::Bell, self.into()),
                }
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
        conversion::ChewingEngine,
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
        let dict = HashMap::new();
        let conversion_engine = ChewingEngine::new();
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
        let dict = HashMap::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]);

        let conversion_engine = ChewingEngine::new();
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
        let dict = HashMap::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]);

        let conversion_engine = ChewingEngine::new();
        let mut editor = Editor::new(conversion_engine, dict);

        let keys = [
            keyboard.map(KeyCode::H),
            keyboard.map(KeyCode::K),
            keyboard.map(KeyCode::N4),
            // TODO: capslock probably shouldn't be a modifier
            // Toggle english mode
            keyboard.map_with_mod(KeyCode::Unknown, Modifiers::capslock()),
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
        let dict = HashMap::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]);

        let conversion_engine = ChewingEngine::new();
        let mut editor = Editor::new(conversion_engine, dict);

        let keys = [
            // Switch to english mode
            keyboard.map_with_mod(KeyCode::Unknown, Modifiers::capslock()),
            keyboard.map(KeyCode::X),
            // Switch to chinese mode
            keyboard.map_with_mod(KeyCode::Unknown, Modifiers::capslock()),
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
        let dictionary = HashMap::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]);
        let conversion_engine = ChewingEngine::new();
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
        let dictionary = HashMap::new();
        let conversion_engine = ChewingEngine::new();
        let mut editor = Editor::new(conversion_engine, dictionary);
        editor.switch_character_form();

        let steps = [
            (
                KeyCode::Unknown,
                Modifiers::capslock(),
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
