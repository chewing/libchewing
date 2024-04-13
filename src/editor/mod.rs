//! Abstract input method editors.

pub mod abbrev;
mod composition_editor;
mod estimate;
pub mod keyboard;
mod selection;
pub mod syllable;

use std::{
    any::{Any, TypeId},
    cmp::{max, min},
    error::Error,
    fmt::{Debug, Display},
};

pub use self::selection::symbol::SymbolSelector;
pub use estimate::{EstimateError, LaxUserFreqEstimate, UserFreqEstimate};
use log::{debug, trace, warn};

use crate::{
    conversion::{full_width_symbol_input, special_symbol_input, ChewingEngine, Interval, Symbol},
    dictionary::{Dictionary, LayeredDictionary, SystemDictionaryLoader, UserDictionaryLoader},
    editor::keyboard::KeyCode,
    zhuyin::{Syllable, SyllableSlice},
};

use self::{
    abbrev::AbbrevTable,
    composition_editor::CompositionEditor,
    keyboard::KeyEvent,
    selection::{phrase::PhraseSelector, symbol::SpecialSymbolSelector},
    syllable::{KeyBehavior, Standard, SyllableEditor},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageMode {
    Chinese,
    English,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterForm {
    Halfwidth,
    Fullwidth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub disable_auto_learn_phrase: bool,
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
            auto_shift_cursor: false,
            phrase_choice_rearward: false,
            disable_auto_learn_phrase: false,
            auto_commit_threshold: 39,
            candidates_per_page: 10,
            language_mode: LanguageMode::Chinese,
            character_form: CharacterForm::Halfwidth,
            user_phrase_add_dir: UserPhraseAddDirection::Forward,
        }
    }
}

/// An editor can react to KeyEvents and change its state.
pub trait BasicEditor {
    /// Handles a KeyEvent
    fn process_keyevent(&mut self, key_event: KeyEvent) -> EditorKeyBehavior;
}

/// The internal state of the editor.
trait State: Debug {
    /// Transits the state to next state with the key event.
    fn next(&mut self, shared: &mut SharedState, ev: KeyEvent) -> Transition;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn spin_ignore(&self) -> Transition {
        Transition::Spin(EditorKeyBehavior::Ignore)
    }
    fn spin_absorb(&self) -> Transition {
        Transition::Spin(EditorKeyBehavior::Absorb)
    }
    fn spin_bell(&self) -> Transition {
        Transition::Spin(EditorKeyBehavior::Bell)
    }
}

#[derive(Debug)]
enum Transition {
    ToState(Box<dyn State>),
    Spin(EditorKeyBehavior),
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
pub struct Editor {
    shared: SharedState,
    state: Box<dyn State>,
}

#[derive(Debug)]
pub struct EditorError;

impl Display for EditorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Some error happened")
    }
}

impl Error for EditorError {}

#[derive(Debug)]
pub(crate) struct SharedState {
    com: CompositionEditor,
    syl: Box<dyn SyllableEditor>,
    conv: ChewingEngine,
    dict: LayeredDictionary,
    abbr: AbbrevTable,
    sym_sel: SymbolSelector,
    estimate: LaxUserFreqEstimate,
    options: EditorOptions,
    last_key_behavior: EditorKeyBehavior,

    dirty_level: u16,
    nth_conversion: usize,
    commit_buffer: String,
    notice_buffer: String,
}

impl Editor {
    pub fn chewing() -> Result<Editor, Box<dyn Error>> {
        let system_dict = SystemDictionaryLoader::new().load()?;
        let user_dict = UserDictionaryLoader::new().load()?;
        let estimate = LaxUserFreqEstimate::open(user_dict.as_ref())?;
        let dict = LayeredDictionary::new(system_dict, user_dict);
        let conversion_engine = ChewingEngine::new();
        let abbrev = SystemDictionaryLoader::new().load_abbrev()?;
        let sym_sel = SystemDictionaryLoader::new().load_symbol_selector()?;
        let editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);
        Ok(editor)
    }

    pub fn new(
        conv: ChewingEngine,
        dict: LayeredDictionary,
        estimate: LaxUserFreqEstimate,
        abbr: AbbrevTable,
        sym_sel: SymbolSelector,
    ) -> Editor {
        Editor {
            shared: SharedState {
                com: CompositionEditor::default(),
                syl: Box::new(Standard::new()),
                conv,
                dict,
                abbr,
                sym_sel,
                estimate,
                options: EditorOptions::default(),
                last_key_behavior: EditorKeyBehavior::Absorb,
                dirty_level: 0,
                nth_conversion: 0,
                commit_buffer: String::new(),
                notice_buffer: String::new(),
            },
            state: Box::new(Entering),
        }
    }

    pub fn set_syllable_editor(&mut self, syl: Box<dyn SyllableEditor>) {
        self.shared.syl = syl;
    }
    pub fn clear(&mut self) {
        self.state = Box::new(Entering);
        self.shared.clear();
    }
    pub fn clear_syllable_editor(&mut self) {
        self.shared.syl.clear();
    }
    pub fn cursor(&self) -> usize {
        self.shared.cursor()
    }
    pub fn language_mode(&self) -> LanguageMode {
        self.shared.options.language_mode
    }
    pub fn set_language_mode(&mut self, language_mode: LanguageMode) {
        self.shared.syl.clear();
        self.shared.options.language_mode = language_mode;
    }

    pub fn character_form(&self) -> CharacterForm {
        self.shared.options.character_form
    }
    pub fn set_character_form(&mut self, charactor_form: CharacterForm) {
        self.shared.options.character_form = charactor_form;
    }

    // TODO: deprecate other direct set methods
    pub fn editor_options(&self) -> EditorOptions {
        self.shared.options
    }
    pub fn set_editor_options(&mut self, options: EditorOptions) {
        self.shared.options = options;
    }
    pub fn switch_character_form(&mut self) {
        self.shared.options = EditorOptions {
            character_form: match self.shared.options.character_form {
                CharacterForm::Halfwidth => CharacterForm::Fullwidth,
                CharacterForm::Fullwidth => CharacterForm::Halfwidth,
            },
            ..self.shared.options
        };
    }

    pub fn entering_syllable(&self) -> bool {
        !self.shared.syl.is_empty()
    }
    pub fn syllable_buffer(&self) -> Syllable {
        self.shared.syl.read()
    }
    pub fn symbols(&self) -> &[Symbol] {
        self.shared.com.symbols()
    }
    pub fn user_dict(&mut self) -> &mut dyn Dictionary {
        self.shared.dict.user_dict()
    }
    pub fn learn_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: &str,
    ) -> Result<(), String> {
        self.shared.learn_phrase(syllables, phrase)
    }
    pub fn unlearn_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: &str,
    ) -> Result<(), String> {
        self.shared.unlearn_phrase(syllables, phrase)
    }
    /// All candidates after current page
    pub fn paginated_candidates(&self) -> Result<Vec<String>, EditorError> {
        debug!("state {:?}", self.state);
        let any = self.state.as_any();
        if let Some(selecting) = any.downcast_ref::<Selecting>() {
            Ok(selecting
                .candidates(&self.shared, &self.shared.dict)
                .into_iter()
                .skip(selecting.page_no * self.shared.options.candidates_per_page)
                .collect())
        } else {
            Err(EditorError)
        }
    }
    pub fn all_candidates(&self) -> Result<Vec<String>, EditorError> {
        debug!("state {:?}", self.state);
        let any = self.state.as_any();
        if let Some(selecting) = any.downcast_ref::<Selecting>() {
            Ok(selecting.candidates(&self.shared, &self.shared.dict))
        } else {
            Err(EditorError)
        }
    }
    pub fn current_page_no(&self) -> Result<usize, EditorError> {
        debug!("state {:?}", self.state);
        let any = self.state.as_any();
        if let Some(selecting) = any.downcast_ref::<Selecting>() {
            Ok(selecting.page_no)
        } else {
            Err(EditorError)
        }
    }
    pub fn total_page(&self) -> Result<usize, EditorError> {
        let any = self.state.as_any();
        if let Some(selecting) = any.downcast_ref::<Selecting>() {
            Ok(selecting.total_page(&self.shared, &self.shared.dict))
        } else {
            Err(EditorError)
        }
    }
    pub fn select(&mut self, n: usize) -> Result<(), EditorError> {
        let any = self.state.as_any_mut();
        let selecting = match any.downcast_mut::<Selecting>() {
            Some(selecting) => selecting,
            None => return Err(EditorError),
        };
        match selecting.select(&mut self.shared, n) {
            Transition::ToState(to_state) => {
                self.shared.last_key_behavior = EditorKeyBehavior::Absorb;
                self.state = to_state;
            }
            Transition::Spin(behavior) => self.shared.last_key_behavior = behavior,
        }
        if self.shared.last_key_behavior == EditorKeyBehavior::Absorb {
            self.shared.try_auto_commit();
        }
        if self.shared.last_key_behavior == EditorKeyBehavior::Bell {
            Err(EditorError)
        } else {
            Ok(())
        }
    }
    pub fn cancel_selecting(&mut self) -> Result<(), EditorError> {
        if self.is_selecting() {
            self.shared.cancel_selecting();
            self.shared.last_key_behavior = EditorKeyBehavior::Absorb;
            self.state = Box::new(Entering);
        }
        Ok(())
    }
    pub fn last_key_behavior(&self) -> EditorKeyBehavior {
        self.shared.last_key_behavior
    }
    pub fn is_entering(&self) -> bool {
        self.state.as_any().type_id() == TypeId::of::<Entering>()
    }
    pub fn is_selecting(&self) -> bool {
        self.state.as_any().type_id() == TypeId::of::<Selecting>()
    }
    pub fn intervals(&self) -> impl Iterator<Item = Interval> {
        self.shared.intervals()
    }
    /// TODO: doc, rename this to `render`?
    pub fn display(&self) -> String {
        self.shared
            .conversion()
            .into_iter()
            .map(|interval| interval.phrase)
            .collect::<String>()
    }
    // TODO: decide the return type
    pub fn display_commit(&self) -> &str {
        &self.shared.commit_buffer
    }
    pub fn commit(&mut self) -> Result<(), String> {
        if !self.is_entering() || self.shared.com.is_empty() {
            return Err("error".to_string());
        }
        self.shared.commit()
    }
    pub fn has_next_selection_point(&self) -> bool {
        let any = self.state.as_any();
        if let Some(s) = any.downcast_ref::<Selecting>() {
            match &s.sel {
                Selector::Phrase(s) => s.next_selection_point(&self.shared.dict).is_some(),
                Selector::Symbol(_) => false,
                Selector::SpecialSymmbol(_) => false,
            }
        } else {
            false
        }
    }
    pub fn has_prev_selection_point(&self) -> bool {
        let any = self.state.as_any();
        if let Some(s) = any.downcast_ref::<Selecting>() {
            match &s.sel {
                Selector::Phrase(s) => s.prev_selection_point(&self.shared.dict).is_some(),
                Selector::Symbol(_) => false,
                Selector::SpecialSymmbol(_) => false,
            }
        } else {
            false
        }
    }
    pub fn jump_to_next_selection_point(&mut self) -> Result<(), EditorError> {
        let any = self.state.as_any_mut();
        if let Some(s) = any.downcast_mut::<Selecting>() {
            match &mut s.sel {
                Selector::Phrase(s) => s.jump_to_next_selection_point(&self.shared.dict),
                Selector::Symbol(_) => Err(EditorError),
                Selector::SpecialSymmbol(_) => Err(EditorError),
            }
        } else {
            Err(EditorError)
        }
    }
    pub fn jump_to_prev_selection_point(&mut self) -> Result<(), EditorError> {
        let any = self.state.as_any_mut();
        if let Some(s) = any.downcast_mut::<Selecting>() {
            match &mut s.sel {
                Selector::Phrase(s) => s.jump_to_prev_selection_point(&self.shared.dict),
                Selector::Symbol(_) => Err(EditorError),
                Selector::SpecialSymmbol(_) => Err(EditorError),
            }
        } else {
            Err(EditorError)
        }
    }
    pub fn jump_to_first_selection_point(&mut self) {
        let any = self.state.as_any_mut();
        if let Some(s) = any.downcast_mut::<Selecting>() {
            match &mut s.sel {
                Selector::Phrase(s) => s.jump_to_first_selection_point(&self.shared.dict),
                Selector::Symbol(_) => {}
                Selector::SpecialSymmbol(_) => {}
            }
        } else {
            {}
        }
    }
    pub fn jump_to_last_selection_point(&mut self) {
        let any = self.state.as_any_mut();
        if let Some(s) = any.downcast_mut::<Selecting>() {
            match &mut s.sel {
                Selector::Phrase(s) => s.jump_to_last_selection_point(&self.shared.dict),
                Selector::Symbol(_) => {}
                Selector::SpecialSymmbol(_) => {}
            }
        } else {
            {}
        }
    }
    pub fn start_selecting(&mut self) -> Result<(), EditorError> {
        let transition = if let Some(s) = self.state.as_any_mut().downcast_mut::<Entering>() {
            s.start_selecting(&mut self.shared)
        } else if let Some(s) = self.state.as_any_mut().downcast_mut::<EnteringSyllable>() {
            // Force entering selection
            s.start_selecting(&mut self.shared)
        } else {
            Transition::Spin(EditorKeyBehavior::Bell)
        };
        match transition {
            Transition::ToState(to_state) => {
                self.shared.last_key_behavior = EditorKeyBehavior::Absorb;
                self.state = to_state;
            }
            Transition::Spin(behavior) => self.shared.last_key_behavior = behavior,
        }
        if self.is_selecting() {
            Ok(())
        } else {
            Err(EditorError)
        }
    }
    pub fn notification(&self) -> &str {
        &self.shared.notice_buffer
    }
}

impl SharedState {
    fn clear(&mut self) {
        self.last_key_behavior = EditorKeyBehavior::Absorb;
        self.com.clear();
        self.syl.clear();
        self.commit_buffer.clear();
        self.notice_buffer.clear();
        self.nth_conversion = 0;
    }
    fn conversion(&self) -> Vec<Interval> {
        self.conv
            .convert(&self.dict, self.com.as_ref())
            .cycle()
            .nth(self.nth_conversion)
            .unwrap()
    }
    fn intervals(&self) -> impl Iterator<Item = Interval> {
        self.conversion().into_iter()
    }
    fn cursor(&self) -> usize {
        self.com.cursor()
    }
    fn learn_phrase_in_range(&mut self, start: usize, end: usize) -> Result<String, String> {
        let result = self.learn_phrase_in_range_quiet(start, end);
        match &result {
            Ok(phrase) => self.notice_buffer = format!("加入：{}", phrase),
            Err(msg) => msg.clone_into(&mut self.notice_buffer),
        }
        result
    }
    fn learn_phrase_in_range_quiet(&mut self, start: usize, end: usize) -> Result<String, String> {
        if end > self.com.len() {
            return Err("加詞失敗：字數不符或夾雜符號".to_owned());
        }
        let syllables = self.com.symbols()[start..end].to_vec();
        if syllables.iter().any(Symbol::is_char) {
            return Err("加詞失敗：字數不符或夾雜符號".to_owned());
        }
        // FIXME
        let phrase = self
            .conversion()
            .into_iter()
            .map(|interval| interval.phrase)
            .collect::<String>()
            .chars()
            .skip(start)
            .take(end - start)
            .collect::<String>();
        if self
            .dict
            .user_dict()
            .lookup_all_phrases(&syllables)
            .into_iter()
            .any(|it| it.as_str() == phrase)
        {
            return Err(format!("已有：{phrase}"));
        }
        let result = self
            .dict
            .add_phrase(&syllables, (phrase.as_ref(), 100).into())
            .map(|_| phrase)
            .map_err(|_| "加詞失敗：字數不符或夾雜符號".to_owned());
        if result.is_ok() {
            self.dirty_level += 1;
        }
        result
    }
    fn learn_phrase(&mut self, syllables: &dyn SyllableSlice, phrase: &str) -> Result<(), String> {
        if syllables.as_slice().len() != phrase.chars().count() {
            warn!(
                "syllables({:?})[{}] and phrase({})[{}] has different length",
                &syllables,
                syllables.as_slice().len(),
                &phrase,
                phrase.chars().count()
            );
            return Err("".to_string());
        }
        let phrases = self.dict.lookup_all_phrases(syllables);
        if phrases.is_empty() {
            let _ = self.dict.add_phrase(syllables, (phrase, 1).into());
            return Ok(());
        }
        let phrase_freq = phrases
            .iter()
            .find(|p| p.as_str() == phrase)
            .map(|p| p.freq())
            .unwrap_or(0);
        let phrase = (phrase, phrase_freq).into();
        // TODO: fine tune learning curve
        let max_freq = phrases.iter().map(|p| p.freq()).max().unwrap_or(1);
        let user_freq = self.estimate.estimate(&phrase, phrase.freq(), max_freq);
        let time = self.estimate.now().unwrap();

        let _ = self.dict.update_phrase(syllables, phrase, user_freq, time);
        self.dirty_level += 1;
        Ok(())
    }
    fn unlearn_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: &str,
    ) -> Result<(), String> {
        let _ = self.dict.remove_phrase(syllables, phrase);
        self.dirty_level += 1;
        Ok(())
    }
    fn switch_language_mode(&mut self) {
        self.options = EditorOptions {
            language_mode: match self.options.language_mode {
                LanguageMode::English => LanguageMode::Chinese,
                LanguageMode::Chinese => LanguageMode::English,
            },
            ..self.options
        };
    }
    fn cancel_selecting(&mut self) {
        self.com.pop_cursor();
    }
    fn commit(&mut self) -> Result<(), String> {
        self.commit_buffer.clear();
        let intervals = self.conversion();
        debug!("buffer {:?}", self.com);
        if !self.options.disable_auto_learn_phrase {
            self.auto_learn(&intervals);
        }
        let output = intervals
            .into_iter()
            .map(|interval| interval.phrase)
            .collect::<String>();
        self.commit_buffer.push_str(&output);
        self.com.clear();
        self.nth_conversion = 0;
        self.last_key_behavior = EditorKeyBehavior::Commit;
        Ok(())
    }
    fn try_auto_commit(&mut self) {
        let len = self.com.len();
        if len <= self.options.auto_commit_threshold {
            return;
        }
        let intervals: Vec<_> = self.intervals().collect();

        let mut remove = 0;
        self.commit_buffer.clear();
        for it in intervals {
            self.commit_buffer.push_str(&it.phrase);
            remove += it.len();
            if len - remove <= self.options.auto_commit_threshold {
                break;
            }
        }
        self.com.remove_front(remove);
        debug!(
            "buffer has {} symbols left after auto commit",
            self.com.len()
        );
        self.last_key_behavior = EditorKeyBehavior::Commit;
    }
    fn auto_learn(&mut self, intervals: &[Interval]) {
        debug!("intervals {:?}", intervals);
        let mut pending = String::new();
        let mut syllables = Vec::new();
        for interval in intervals {
            if interval.is_phrase && interval.len() == 1 && !is_break_word(&interval.phrase) {
                pending.push_str(&interval.phrase);
                syllables.extend_from_slice(&self.com.symbols()[interval.start..interval.end]);
            } else {
                if !pending.is_empty() {
                    debug!("autolearn-2 {:?} as {}", &syllables, &pending);
                    let _ = self.learn_phrase(&syllables, &pending);
                    pending.clear();
                    syllables.clear();
                }
                if interval.is_phrase {
                    debug!(
                        "autolearn-3 {:?} as {}",
                        &self.com.symbols()[interval.start..interval.end],
                        &interval.phrase
                    );
                    // FIXME avoid copy
                    let _ = self.learn_phrase(
                        &self.com.symbols()[interval.start..interval.end].to_vec(),
                        &interval.phrase,
                    );
                }
            }
        }
        if !pending.is_empty() {
            debug!("autolearn-1 {:?} as {}", &syllables, &pending);
            let _ = self.learn_phrase(&syllables, &pending);
            pending.clear();
            syllables.clear();
        }
    }
}

#[rustfmt::skip]
fn is_break_word(word: &str) -> bool {
    ["是", "的", "了", "不",
     "也", "而", "你", "我",
     "他", "與", "它", "她",
     "其", "就", "和", "或",
     "們", "性", "員", "子",
     "上", "下", "中", "內",
     "外", "化", "者", "家",
     "兒", "年", "月", "日",
     "時", "分", "秒", "街",
     "路", "村", "在"].contains(&word)
}

impl BasicEditor for Editor {
    fn process_keyevent(&mut self, key_event: KeyEvent) -> EditorKeyBehavior {
        trace!("process_keyevent: {}", &key_event);
        let _ = self.shared.estimate.tick();
        // reset?
        self.shared.notice_buffer.clear();
        if self.shared.last_key_behavior == EditorKeyBehavior::Commit {
            self.shared.commit_buffer.clear();
        }

        match self.state.next(&mut self.shared, key_event) {
            Transition::ToState(to_state) => {
                self.shared.last_key_behavior = EditorKeyBehavior::Absorb;
                self.state = to_state;
            }
            Transition::Spin(behavior) => self.shared.last_key_behavior = behavior,
        }

        if self.shared.last_key_behavior == EditorKeyBehavior::Absorb {
            self.shared.try_auto_commit();
        }
        trace!("last_key_behavior = {:?}", self.shared.last_key_behavior);
        trace!("comp: {:?}", &self.shared.com);
        const DIRTY_THRESHOLD: u16 = 0;
        if self.shared.dirty_level > DIRTY_THRESHOLD {
            let _ = self.shared.dict.reopen();
            let _ = self.shared.dict.flush();
            self.shared.dirty_level = 0;
        }
        self.shared.last_key_behavior
    }
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

impl Entering {
    fn start_selecting(&self, editor: &mut SharedState) -> Transition {
        match editor.com.symbol_for_select() {
            Some(symbol) => {
                if symbol.is_syllable() {
                    Transition::ToState(Box::new(Selecting::new_phrase(editor)))
                } else {
                    Transition::ToState(Box::new(Selecting::new_special_symbol(editor, symbol)))
                }
            }
            None => self.spin_ignore(),
        }
    }
    fn start_selecting_or_input_space(&self, editor: &mut SharedState) -> Transition {
        debug!("buffer {:?}", editor.com);
        match editor.com.symbol_for_select() {
            Some(symbol) => {
                if symbol.is_syllable() {
                    Transition::ToState(Box::new(Selecting::new_phrase(editor)))
                } else {
                    Transition::ToState(Box::new(Selecting::new_special_symbol(editor, symbol)))
                }
            }
            None if editor.com.is_empty() => {
                match editor.options.character_form {
                    CharacterForm::Halfwidth => editor.commit_buffer.push(' '),
                    CharacterForm::Fullwidth => editor.commit_buffer.push('　'),
                }
                self.spin_commit()
            }
            None => self.spin_ignore(),
        }
    }
    fn start_symbol_input(&self, editor: &mut SharedState) -> Transition {
        Transition::ToState(Box::new(Selecting::new_symbol(editor)))
    }
    fn start_enter_syllable(&self) -> Transition {
        Transition::ToState(Box::new(EnteringSyllable))
    }
    fn start_highlighting(&self, start_cursor: usize) -> Transition {
        Transition::ToState(Box::new(Highlighting::new(start_cursor)))
    }
    fn spin_commit(&self) -> Transition {
        Transition::Spin(EditorKeyBehavior::Commit)
    }
}

impl State for Entering {
    fn next(&mut self, shared: &mut SharedState, ev: KeyEvent) -> Transition {
        use KeyCode::*;

        match ev.code {
            Backspace => {
                if shared.com.is_empty() {
                    self.spin_ignore()
                } else {
                    shared.com.remove_before_cursor();
                    self.spin_absorb()
                }
            }
            Unknown if ev.modifiers.capslock => {
                shared.switch_language_mode();
                self.spin_absorb()
            }
            code @ (N0 | N1 | N2 | N3 | N4 | N5 | N6 | N7 | N8 | N9) if ev.modifiers.ctrl => {
                if code == N0 || code == N1 {
                    return self.start_symbol_input(shared);
                }
                let n = code as usize;
                let result = match shared.options.user_phrase_add_dir {
                    UserPhraseAddDirection::Forward => {
                        shared.learn_phrase_in_range(shared.cursor(), shared.cursor() + n)
                    }
                    UserPhraseAddDirection::Backward => {
                        if shared.cursor() >= n {
                            shared.learn_phrase_in_range(shared.cursor() - n, shared.cursor())
                        } else {
                            "加詞失敗：字數不符或夾雜符號".clone_into(&mut shared.notice_buffer);
                            Err("加詞失敗：字數不符或夾雜符號".to_owned())
                        }
                    }
                };
                match result {
                    Ok(_) => self.spin_absorb(),
                    Err(_) => self.spin_bell(),
                }
            }
            Tab if shared.com.is_end_of_buffer() => {
                shared.nth_conversion += 1;
                self.spin_absorb()
            }
            Tab => {
                let interval_ends: Vec<_> = shared.conversion().iter().map(|it| it.end).collect();
                if interval_ends.contains(&shared.cursor()) {
                    shared.com.insert_glue();
                } else {
                    shared.com.insert_break();
                }
                self.spin_absorb()
            }
            // DoubleTab => {
            //     // editor.reset_user_break_and_connect_at_cursor();
            //     (EditorKeyBehavior::Absorb, &Entering)
            // }
            Del => {
                if shared.com.is_end_of_buffer() {
                    self.spin_ignore()
                } else {
                    shared.com.remove_after_cursor();
                    self.spin_absorb()
                }
            }
            Home => {
                shared.com.move_cursor_to_beginning();
                self.spin_absorb()
            }
            Left if ev.modifiers.shift => {
                if shared.com.is_empty() || shared.cursor() == 0 {
                    return self.spin_ignore();
                }
                self.start_highlighting(shared.cursor() - 1)
            }
            Right if ev.modifiers.shift => {
                if shared.com.is_empty() || shared.com.is_end_of_buffer() {
                    return self.spin_ignore();
                }
                self.start_highlighting(shared.cursor() + 1)
            }
            Left => {
                shared.com.move_cursor_left();
                self.spin_absorb()
            }
            Right => {
                shared.com.move_cursor_right();
                self.spin_absorb()
            }
            Up => self.spin_ignore(),
            Space if ev.modifiers.shift => {
                shared.options.character_form = match shared.options.character_form {
                    CharacterForm::Halfwidth => CharacterForm::Fullwidth,
                    CharacterForm::Fullwidth => CharacterForm::Halfwidth,
                };
                self.spin_absorb()
            }
            Space
                if shared.options.space_is_select_key
                    && shared.options.language_mode == LanguageMode::Chinese =>
            {
                self.start_selecting_or_input_space(shared)
            }
            Down => {
                debug!("buffer {:?}", shared.com);
                self.start_selecting(shared)
            }
            End | PageUp | PageDown => {
                shared.com.move_cursor_to_end();
                self.spin_absorb()
            }
            Enter => {
                let _ = shared.commit();
                self.spin_commit()
            }
            Esc => {
                if shared.options.esc_clear_all_buffer && !shared.com.is_empty() {
                    shared.com.clear();
                    self.spin_absorb()
                } else {
                    self.spin_ignore()
                }
            }
            _ if ev.modifiers.numlock => {
                if shared.com.is_empty() {
                    shared.commit_buffer.clear();
                    shared.commit_buffer.push(ev.unicode);
                    self.spin_commit()
                } else {
                    shared.com.insert(Symbol::new_char(ev.unicode));
                    self.spin_absorb()
                }
            }
            _ => match shared.options.language_mode {
                LanguageMode::Chinese if ev.code == Grave && ev.modifiers.is_none() => {
                    self.start_symbol_input(shared)
                }
                LanguageMode::Chinese if ev.code == Space => match shared.options.character_form {
                    CharacterForm::Halfwidth => {
                        if shared.com.is_empty() {
                            shared.commit_buffer.clear();
                            shared.commit_buffer.push(ev.unicode);
                            self.spin_commit()
                        } else {
                            shared.com.insert(Symbol::new_char(ev.unicode));
                            self.spin_absorb()
                        }
                    }
                    CharacterForm::Fullwidth => {
                        let char_ = full_width_symbol_input(ev.unicode).unwrap();
                        if shared.com.is_empty() {
                            shared.commit_buffer.clear();
                            shared.commit_buffer.push(char_);
                            self.spin_commit()
                        } else {
                            shared.com.insert(Symbol::new_char(char_));
                            self.spin_absorb()
                        }
                    }
                },
                LanguageMode::Chinese if shared.options.easy_symbol_input => {
                    // Priortize symbol input
                    if let Some(expended) = shared.abbr.find_abbrev(ev.unicode) {
                        expended
                            .chars()
                            .for_each(|ch| shared.com.insert(Symbol::new_char(ch)));
                        return self.spin_absorb();
                    }
                    if let Some(symbol) = special_symbol_input(ev.unicode) {
                        shared.com.insert(Symbol::new_char(symbol));
                        return self.spin_absorb();
                    }
                    if ev.modifiers.is_none() && KeyBehavior::Absorb == shared.syl.key_press(ev) {
                        return self.start_enter_syllable();
                    }
                    self.spin_bell()
                }
                LanguageMode::Chinese => {
                    if ev.modifiers.is_none() && KeyBehavior::Absorb == shared.syl.key_press(ev) {
                        return self.start_enter_syllable();
                    }
                    if let Some(symbol) = special_symbol_input(ev.unicode) {
                        shared.com.insert(Symbol::new_char(symbol));
                        return self.spin_absorb();
                    }
                    self.spin_bell()
                }
                LanguageMode::English => match shared.options.character_form {
                    CharacterForm::Halfwidth => {
                        if shared.com.is_empty() {
                            shared.commit_buffer.clear();
                            shared.commit_buffer.push(ev.unicode);
                            self.spin_commit()
                        } else {
                            shared.com.insert(Symbol::new_char(ev.unicode));
                            self.spin_absorb()
                        }
                    }
                    CharacterForm::Fullwidth => {
                        let char_ = full_width_symbol_input(ev.unicode).unwrap();
                        if shared.com.is_empty() {
                            shared.commit_buffer.clear();
                            shared.commit_buffer.push(char_);
                            self.spin_commit()
                        } else {
                            shared.com.insert(Symbol::new_char(char_));
                            self.spin_absorb()
                        }
                    }
                },
            },
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl EnteringSyllable {
    fn start_entering(&self) -> Transition {
        Transition::ToState(Box::new(Entering))
    }
    fn start_selecting(&self, editor: &mut SharedState) -> Transition {
        editor.syl.clear();
        match editor.com.symbol_for_select() {
            Some(symbol) => {
                if symbol.is_syllable() {
                    Transition::ToState(Box::new(Selecting::new_phrase(editor)))
                } else {
                    Transition::ToState(Box::new(Selecting::new_special_symbol(editor, symbol)))
                }
            }
            None => self.spin_ignore(),
        }
    }
}

impl State for EnteringSyllable {
    fn next(&mut self, shared: &mut SharedState, ev: KeyEvent) -> Transition {
        use KeyCode::*;

        match ev.code {
            Backspace => {
                shared.syl.remove_last();

                if !shared.syl.is_empty() {
                    self.spin_absorb()
                } else {
                    self.start_entering()
                }
            }
            Unknown if ev.modifiers.capslock => {
                shared.syl.clear();
                shared.switch_language_mode();
                self.start_entering()
            }
            Esc => {
                shared.syl.clear();
                if shared.options.esc_clear_all_buffer {
                    shared.com.clear();
                }
                self.start_entering()
            }
            _ => match shared.syl.key_press(ev) {
                KeyBehavior::Absorb => self.spin_absorb(),
                KeyBehavior::Commit => {
                    if shared
                        .dict
                        .lookup_first_phrase(&[shared.syl.read()])
                        .is_some()
                    {
                        shared.com.insert(Symbol::new_syl(shared.syl.read()));
                    }
                    shared.syl.clear();
                    self.start_entering()
                }
                _ => self.spin_bell(),
            },
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Selecting {
    fn new_phrase(editor: &mut SharedState) -> Self {
        editor.com.push_cursor();
        editor.com.clamp_cursor();

        let mut sel = PhraseSelector::new(
            !editor.options.phrase_choice_rearward,
            editor.com.to_composition(),
        );
        sel.init(editor.cursor(), &editor.dict);

        Selecting {
            page_no: 0,
            action: SelectingAction::Replace,
            sel: Selector::Phrase(sel),
        }
    }
    fn new_symbol(editor: &mut SharedState) -> Self {
        Selecting {
            page_no: 0,
            action: SelectingAction::Insert,
            sel: Selector::Symbol(editor.sym_sel.clone()),
        }
    }
    fn new_special_symbol(editor: &mut SharedState, symbol: Symbol) -> Self {
        editor.com.push_cursor();
        editor.com.clamp_cursor();

        let sel = SpecialSymbolSelector::new(symbol);
        if sel.menu().is_empty() {
            // If there's no special symbol then fallback to dynamic symbol table
            let mut sel = Self::new_symbol(editor);
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
    fn candidates(&self, editor: &SharedState, dict: &LayeredDictionary) -> Vec<String> {
        match &self.sel {
            Selector::Phrase(sel) => sel.candidates(editor, dict),
            Selector::Symbol(sel) => sel.menu(),
            Selector::SpecialSymmbol(sel) => sel.menu(),
        }
    }
    fn total_page(&self, editor: &SharedState, dict: &LayeredDictionary) -> usize {
        // MSRV: stable after rust 1.73
        fn div_ceil(lhs: usize, rhs: usize) -> usize {
            let d = lhs / rhs;
            let r = lhs % rhs;
            if r > 0 && rhs > 0 {
                d + 1
            } else {
                d
            }
        }
        div_ceil(
            self.candidates(editor, dict).len(),
            editor.options.candidates_per_page,
        )
    }
    fn select(&mut self, editor: &mut SharedState, n: usize) -> Transition {
        let offset = self.page_no * editor.options.candidates_per_page + n;
        match self.sel {
            Selector::Phrase(ref sel) => {
                let candidates = sel.candidates(editor, &editor.dict);
                debug!("candidates: {:?}", &candidates);
                match candidates.get(offset) {
                    Some(phrase) => {
                        editor.com.select(sel.interval(phrase.as_str()));
                        debug!("Auto Shift {}", editor.options.auto_shift_cursor);
                        editor.com.pop_cursor();
                        if editor.options.auto_shift_cursor {
                            editor.com.move_cursor_right();
                        }
                        self.start_entering()
                    }
                    None => self.spin_bell(),
                }
            }
            Selector::Symbol(ref mut sel) => match sel.select(offset) {
                Some(s) => {
                    match self.action {
                        SelectingAction::Insert => editor.com.insert(s),
                        SelectingAction::Replace => editor.com.replace(s),
                    }
                    editor.com.pop_cursor();
                    self.start_entering()
                }
                None => {
                    self.page_no = 0;
                    self.spin_absorb()
                }
            },
            Selector::SpecialSymmbol(ref sel) => match sel.select(offset) {
                Some(s) => {
                    match self.action {
                        SelectingAction::Insert => editor.com.insert(s),
                        SelectingAction::Replace => editor.com.replace(s),
                    }
                    editor.com.pop_cursor();
                    self.start_entering()
                }
                None => {
                    self.page_no = 0;
                    self.spin_absorb()
                }
            },
        }
    }
    fn start_entering(&self) -> Transition {
        Transition::ToState(Box::new(Entering))
    }
}

impl State for Selecting {
    fn next(&mut self, shared: &mut SharedState, ev: KeyEvent) -> Transition {
        use KeyCode::*;

        if ev.modifiers.ctrl || ev.modifiers.shift {
            return self.spin_bell();
        }

        match ev.code {
            Backspace => {
                shared.cancel_selecting();
                self.start_entering()
            }
            Unknown if ev.modifiers.capslock => {
                shared.switch_language_mode();
                shared.cancel_selecting();
                self.start_entering()
            }
            Up => {
                shared.cancel_selecting();
                self.start_entering()
            }
            Space if shared.options.space_is_select_key => {
                if self.page_no + 1 < self.total_page(shared, &shared.dict) {
                    self.page_no += 1;
                } else {
                    self.page_no = 0;
                    match &mut self.sel {
                        Selector::Phrase(sel) => {
                            sel.next(&shared.dict);
                        }
                        Selector::Symbol(_sel) => (),
                        Selector::SpecialSymmbol(_sel) => (),
                    }
                }
                self.spin_absorb()
            }
            Down => {
                match &mut self.sel {
                    Selector::Phrase(sel) => {
                        sel.next(&shared.dict);
                    }
                    Selector::Symbol(_sel) => (),
                    Selector::SpecialSymmbol(_sel) => (),
                }
                self.spin_absorb()
            }
            J => {
                if shared.com.is_empty() {
                    return self.spin_ignore();
                }
                let begin = match &self.sel {
                    Selector::Phrase(sel) => sel.begin(),
                    Selector::Symbol(_) => shared.com.cursor(),
                    Selector::SpecialSymmbol(_) => shared.com.cursor(),
                };
                shared.com.move_cursor(begin.saturating_sub(1));
                let sym = shared.com.symbol().expect("should have symbol");
                if sym.is_syllable() {
                    let mut sel = PhraseSelector::new(
                        !shared.options.phrase_choice_rearward,
                        shared.com.to_composition(),
                    );
                    sel.init(shared.cursor(), &shared.dict);
                    self.sel = Selector::Phrase(sel);
                } else {
                    let sel = SpecialSymbolSelector::new(sym);
                    self.sel = Selector::SpecialSymmbol(sel);
                }
                self.spin_absorb()
            }
            K => {
                if shared.com.is_empty() {
                    return self.spin_ignore();
                }
                let begin = match &self.sel {
                    Selector::Phrase(sel) => sel.begin(),
                    Selector::Symbol(_) => shared.com.cursor(),
                    Selector::SpecialSymmbol(_) => shared.com.cursor(),
                };
                shared.com.move_cursor(begin.saturating_add(1));
                shared.com.clamp_cursor();
                let sym = shared.com.symbol().expect("should have symbol");
                if sym.is_syllable() {
                    let mut sel = PhraseSelector::new(
                        !shared.options.phrase_choice_rearward,
                        shared.com.to_composition(),
                    );
                    sel.init(shared.cursor(), &shared.dict);
                    self.sel = Selector::Phrase(sel);
                } else {
                    let sel = SpecialSymbolSelector::new(sym);
                    self.sel = Selector::SpecialSymmbol(sel);
                }
                self.spin_absorb()
            }
            Left | PageUp => {
                if self.page_no > 0 {
                    self.page_no -= 1;
                } else {
                    self.page_no = self.total_page(shared, &shared.dict).saturating_sub(1);
                }
                self.spin_absorb()
            }
            Right | PageDown => {
                if self.page_no + 1 < self.total_page(shared, &shared.dict) {
                    self.page_no += 1;
                } else {
                    self.page_no = 0;
                }
                self.spin_absorb()
            }
            code @ (N1 | N2 | N3 | N4 | N5 | N6 | N7 | N8 | N9 | N0) => {
                let n = code.to_digit().unwrap().saturating_sub(1) as usize;
                self.select(shared, n)
            }
            Esc => {
                shared.cancel_selecting();
                shared.com.pop_cursor();
                self.start_entering()
            }
            Del => {
                // NB: should be Ignore but return Absorb for backward compat
                self.spin_absorb()
            }
            _ => self.spin_bell(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Highlighting {
    fn new(moving_cursor: usize) -> Self {
        Highlighting { moving_cursor }
    }
    fn start_entering(&self) -> Transition {
        Transition::ToState(Box::new(Entering))
    }
}

impl State for Highlighting {
    fn next(&mut self, shared: &mut SharedState, ev: KeyEvent) -> Transition {
        use KeyCode::*;

        match ev.code {
            Unknown if ev.modifiers.capslock => {
                shared.switch_language_mode();
                self.start_entering()
            }
            Left if ev.modifiers.shift => {
                if self.moving_cursor != 0 {
                    self.moving_cursor -= 1;
                }
                self.spin_absorb()
            }
            Right if ev.modifiers.shift => {
                if self.moving_cursor != shared.com.len() {
                    self.moving_cursor += 1;
                }
                self.spin_absorb()
            }
            Enter => {
                let start = min(self.moving_cursor, shared.com.cursor());
                let end = max(self.moving_cursor, shared.com.cursor());
                shared.com.move_cursor(self.moving_cursor);
                let _ = shared.learn_phrase_in_range(start, end);
                self.start_entering()
            }
            _ => self.start_entering(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use estimate::LaxUserFreqEstimate;

    use crate::{
        conversion::ChewingEngine,
        dictionary::{LayeredDictionary, TrieBufDictionary},
        editor::{
            abbrev::AbbrevTable, estimate, keyboard::Modifiers, EditorKeyBehavior, SymbolSelector,
        },
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
        let dict = LayeredDictionary::new(
            vec![Box::new(TrieBufDictionary::new_in_memory())],
            Box::new(TrieBufDictionary::new_in_memory()),
        );
        let conversion_engine = ChewingEngine::new();
        let estimate = LaxUserFreqEstimate::open_in_memory(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

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
        let dict = TrieBufDictionary::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]);
        let dict = LayeredDictionary::new(
            vec![Box::new(dict)],
            Box::new(TrieBufDictionary::new_in_memory()),
        );
        let conversion_engine = ChewingEngine::new();
        let estimate = LaxUserFreqEstimate::open_in_memory(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

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
        let dict = TrieBufDictionary::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]);
        let dict = LayeredDictionary::new(
            vec![Box::new(dict)],
            Box::new(TrieBufDictionary::new_in_memory()),
        );
        let conversion_engine = ChewingEngine::new();
        let estimate = LaxUserFreqEstimate::open_in_memory(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

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
                EditorKeyBehavior::Absorb,
            ],
            key_behaviors
        );
        assert!(editor.syllable_buffer().is_empty());
        assert_eq!("冊z", editor.display());
    }

    #[test]
    fn editing_mode_input_english_to_chinese_mode() {
        let keyboard = Qwerty;
        let dict = TrieBufDictionary::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]);
        let dict = LayeredDictionary::new(
            vec![Box::new(dict)],
            Box::new(TrieBufDictionary::new_in_memory()),
        );
        let conversion_engine = ChewingEngine::new();
        let estimate = LaxUserFreqEstimate::open_in_memory(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

        let keys = [
            // Switch to english mode
            keyboard.map_with_mod(KeyCode::Unknown, Modifiers::capslock()),
            keyboard.map(KeyCode::X),
        ];

        let key_behaviors: Vec<_> = keys
            .iter()
            .map(|&key| editor.process_keyevent(key))
            .collect();

        assert_eq!(
            vec![EditorKeyBehavior::Absorb, EditorKeyBehavior::Commit],
            key_behaviors
        );
        assert!(editor.syllable_buffer().is_empty());
        assert_eq!("x", editor.display_commit());

        let keys = [
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
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Absorb,
                EditorKeyBehavior::Absorb,
            ],
            key_behaviors
        );
        assert!(editor.syllable_buffer().is_empty());
        assert_eq!("冊", editor.display());
    }

    #[test]
    fn editing_chinese_mode_input_special_symbol() {
        let keyboard = Qwerty;
        let dict = TrieBufDictionary::from([(
            vec![crate::syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("冊", 100).into()],
        )]);
        let dict = LayeredDictionary::new(
            vec![Box::new(dict)],
            Box::new(TrieBufDictionary::new_in_memory()),
        );
        let conversion_engine = ChewingEngine::new();
        let estimate = LaxUserFreqEstimate::open_in_memory(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

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
        let dict = TrieBufDictionary::new_in_memory();
        let dict = LayeredDictionary::new(
            vec![Box::new(dict)],
            Box::new(TrieBufDictionary::new_in_memory()),
        );
        let conversion_engine = ChewingEngine::new();
        let estimate = LaxUserFreqEstimate::open_in_memory(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

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
