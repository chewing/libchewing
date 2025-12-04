//! Abstract input method editors.

mod abbrev;
mod composition_editor;
mod estimate;
mod selection;
pub mod zhuyin_layout;

use std::{
    any::Any,
    cmp::{max, min},
    error::Error,
    fmt::{Debug, Display},
    mem,
};

pub use estimate::{LaxUserFreqEstimate, UserFreqEstimate};
use tracing::{debug, error, info, trace, warn};

pub use self::{abbrev::AbbrevTable, selection::symbol::SymbolSelector};
use self::{
    composition_editor::CompositionEditor,
    selection::{phrase::PhraseSelector, symbol::SpecialSymbolSelector},
    zhuyin_layout::{KeyBehavior, Standard, SyllableEditor},
};
use crate::{
    conversion::{
        ChewingEngine, ConversionEngine, Interval, Symbol, full_width_symbol_input,
        special_symbol_input,
    },
    dictionary::{
        DEFAULT_DICT_NAMES, Dictionary, DictionaryMut, Layered, LookupStrategy,
        SystemDictionaryLoader, UpdateDictionaryError, UserDictionaryLoader,
    },
    input::{KeyState, KeyboardEvent, keysym::*},
    zhuyin::Syllable,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionEngineKind {
    SimpleEngine,
    ChewingEngine,
    FuzzyChewingEngine,
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
    pub lookup_strategy: LookupStrategy,
    pub conversion_engine: ConversionEngineKind,
    pub enable_fullwidth_toggle_key: bool,
    pub sort_candidates_by_frequency: bool,
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
            lookup_strategy: LookupStrategy::Standard,
            // FIXME may be out of sync with the engine used
            conversion_engine: ConversionEngineKind::ChewingEngine,
            enable_fullwidth_toggle_key: true,
            sort_candidates_by_frequency: false,
        }
    }
}

/// An editor can react to KeyEvents and change its state.
pub trait BasicEditor {
    /// Handles a KeyEvent
    fn process_keyevent(&mut self, key_event: KeyboardEvent) -> EditorKeyBehavior;
}

/// The internal state of the editor.
trait State: Any + Debug {
    /// Transits the state to next state with the key event.
    fn next(&mut self, shared: &mut SharedState, ev: KeyboardEvent) -> Transition;

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

/// All different errors that may happen when changing editor state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorError {
    /// Requested invalid state change.
    InvalidState,
    /// Requested invalid input.
    InvalidInput,
    /// Requested state change was not possible.
    Impossible,
}

impl Display for EditorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Editor cannot perform requested action.")
    }
}

impl Error for EditorError {}

#[derive(Debug)]
pub(crate) struct SharedState {
    com: CompositionEditor,
    syl: Box<dyn SyllableEditor>,
    conv: Box<dyn ConversionEngine>,
    dict: Layered,
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
        let sys_loader = SystemDictionaryLoader::new();
        let system_dict = sys_loader.load(DEFAULT_DICT_NAMES)?;
        let user_dict = UserDictionaryLoader::new().load()?;
        let estimate = LaxUserFreqEstimate::max_from(user_dict.as_ref());
        let dict = Layered::new(system_dict, user_dict);
        let conversion_engine = Box::new(ChewingEngine::new());
        let abbrev = sys_loader.load_abbrev()?;
        let sym_sel = sys_loader.load_symbol_selector()?;
        let editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);
        Ok(editor)
    }

    pub fn new(
        conv: Box<dyn ConversionEngine>,
        dict: Layered,
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
        info!("Set syllable editor: {:?}", self.shared.syl);
    }
    pub fn set_conversion_engine(&mut self, engine: Box<dyn ConversionEngine>) {
        self.shared.conv = engine;
        info!("Set conversion engine: {:?}", self.shared.conv);
    }
    pub fn clear(&mut self) {
        self.state = Box::new(Entering);
        self.shared.clear();
    }
    pub fn ack(&mut self) {
        self.shared.commit_buffer.clear();
    }
    pub fn clear_composition_editor(&mut self) {
        self.shared.com.clear();
    }
    pub fn clear_syllable_editor(&mut self) {
        self.shared.syl.clear();
    }
    pub fn cursor(&self) -> usize {
        self.shared.cursor()
    }

    // TODO: deprecate other direct set methods
    pub fn editor_options(&self) -> EditorOptions {
        self.shared.options
    }
    pub fn set_editor_options(&mut self, options: EditorOptions) {
        if self.shared.options.language_mode != options.language_mode {
            self.cancel_entering_syllable();
        }
        self.shared.options = options;
    }
    pub fn entering_syllable(&self) -> bool {
        !self.shared.syl.is_empty()
    }
    pub fn syllable_buffer(&self) -> Syllable {
        self.shared.syl.read()
    }
    pub fn syllable_buffer_display(&self) -> String {
        self.shared
            .syl
            .key_seq()
            .unwrap_or_else(|| self.shared.syl.read().to_string())
    }
    pub fn symbols(&self) -> &[Symbol] {
        self.shared.com.symbols()
    }
    pub fn user_dict(&mut self) -> &mut dyn Dictionary {
        self.shared.dict.user_dict()
    }
    pub fn learn_phrase(
        &mut self,
        syllables: &[Syllable],
        phrase: &str,
    ) -> Result<(), UpdateDictionaryError> {
        self.shared.learn_phrase(syllables, phrase)
    }
    pub fn unlearn_phrase(
        &mut self,
        syllables: &[Syllable],
        phrase: &str,
    ) -> Result<(), UpdateDictionaryError> {
        self.shared.unlearn_phrase(syllables, phrase)
    }
    /// All candidates after current page
    pub fn paginated_candidates(&self) -> Result<Vec<String>, EditorError> {
        let any = self.state.as_ref() as &dyn Any;
        if let Some(selecting) = any.downcast_ref::<Selecting>() {
            Ok(selecting
                .candidates(&self.shared, &self.shared.dict)
                .into_iter()
                .skip(selecting.page_no * self.shared.options.candidates_per_page)
                .collect())
        } else {
            Err(EditorError::InvalidState)
        }
    }
    pub fn all_candidates(&self) -> Result<Vec<String>, EditorError> {
        let any = self.state.as_ref() as &dyn Any;
        if let Some(selecting) = any.downcast_ref::<Selecting>() {
            Ok(selecting.candidates(&self.shared, &self.shared.dict))
        } else {
            Err(EditorError::InvalidState)
        }
    }
    pub fn current_page_no(&self) -> Result<usize, EditorError> {
        let any = self.state.as_ref() as &dyn Any;
        if let Some(selecting) = any.downcast_ref::<Selecting>() {
            Ok(selecting.page_no)
        } else {
            Err(EditorError::InvalidState)
        }
    }
    pub fn total_page(&self) -> Result<usize, EditorError> {
        let any = self.state.as_ref() as &dyn Any;
        if let Some(selecting) = any.downcast_ref::<Selecting>() {
            Ok(selecting.total_page(&self.shared, &self.shared.dict))
        } else {
            Err(EditorError::InvalidState)
        }
    }
    pub fn select(&mut self, n: usize) -> Result<(), EditorError> {
        let any = self.state.as_mut() as &mut dyn Any;
        let selecting = match any.downcast_mut::<Selecting>() {
            Some(selecting) => selecting,
            None => return Err(EditorError::InvalidState),
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
            Err(EditorError::InvalidState)
        } else {
            Ok(())
        }
    }
    pub fn cancel_selecting(&mut self) -> Result<(), EditorError> {
        if self.is_selecting() {
            self.shared.cancel_selecting();
            self.shared.last_key_behavior = EditorKeyBehavior::Absorb;
            self.state = Box::new(Entering);
            Ok(())
        } else {
            Err(EditorError::InvalidState)
        }
    }
    pub fn cancel_entering_syllable(&mut self) {
        self.shared.syl.clear();
        self.shared.last_key_behavior = EditorKeyBehavior::Absorb;
        self.state = Box::new(Entering);
    }
    pub fn last_key_behavior(&self) -> EditorKeyBehavior {
        self.shared.last_key_behavior
    }
    pub fn is_entering(&self) -> bool {
        let any = self.state.as_ref() as &dyn Any;
        any.is::<Entering>()
    }
    pub fn is_selecting(&self) -> bool {
        let any = self.state.as_ref() as &dyn Any;
        any.is::<Selecting>()
    }
    pub fn intervals(&self) -> impl Iterator<Item = Interval> {
        self.shared.intervals()
    }
    pub fn len(&self) -> usize {
        self.shared.com.len()
    }
    pub fn is_empty(&self) -> bool {
        self.shared.com.is_empty()
    }
    /// TODO: doc, rename this to `render`?
    pub fn display(&self) -> String {
        self.shared
            .conversion()
            .into_iter()
            .map(|interval| interval.text)
            .collect::<String>()
    }
    // TODO: decide the return type
    pub fn display_commit(&self) -> &str {
        &self.shared.commit_buffer
    }
    pub fn commit(&mut self) -> Result<(), EditorError> {
        if self.shared.com.is_empty() {
            return Err(EditorError::InvalidState);
        }
        self.shared.commit();
        Ok(())
    }
    pub fn has_next_selection_point(&self) -> bool {
        let any = self.state.as_ref() as &dyn Any;
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
        let any = self.state.as_ref() as &dyn Any;
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
        let any = self.state.as_mut() as &mut dyn Any;
        if let Some(s) = any.downcast_mut::<Selecting>() {
            match &mut s.sel {
                Selector::Phrase(s) => s.jump_to_next_selection_point(&self.shared.dict),
                _ => Err(EditorError::InvalidState),
            }
        } else {
            Err(EditorError::InvalidState)
        }
    }
    pub fn jump_to_prev_selection_point(&mut self) -> Result<(), EditorError> {
        let any = self.state.as_mut() as &mut dyn Any;
        if let Some(s) = any.downcast_mut::<Selecting>() {
            match &mut s.sel {
                Selector::Phrase(s) => s.jump_to_prev_selection_point(&self.shared.dict),
                _ => Err(EditorError::InvalidState),
            }
        } else {
            Err(EditorError::InvalidState)
        }
    }
    pub fn jump_to_first_selection_point(&mut self) -> Result<(), EditorError> {
        let any = self.state.as_mut() as &mut dyn Any;
        if let Some(s) = any.downcast_mut::<Selecting>() {
            match &mut s.sel {
                Selector::Phrase(s) => {
                    s.jump_to_first_selection_point(&self.shared.dict);
                    Ok(())
                }
                _ => Err(EditorError::InvalidState),
            }
        } else {
            Err(EditorError::InvalidState)
        }
    }
    pub fn jump_to_last_selection_point(&mut self) -> Result<(), EditorError> {
        let any = self.state.as_mut() as &mut dyn Any;
        if let Some(s) = any.downcast_mut::<Selecting>() {
            match &mut s.sel {
                Selector::Phrase(s) => {
                    s.jump_to_last_selection_point(&self.shared.dict);
                    Ok(())
                }
                _ => Err(EditorError::InvalidState),
            }
        } else {
            Err(EditorError::InvalidState)
        }
    }
    pub fn start_selecting(&mut self) -> Result<(), EditorError> {
        let any = self.state.as_mut() as &mut dyn Any;
        let transition = if let Some(s) = any.downcast_mut::<Entering>() {
            s.start_selecting(&mut self.shared)
        } else if let Some(s) = any.downcast_mut::<EnteringSyllable>() {
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
            Err(EditorError::InvalidState)
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
        let paths = self.conv.convert(&self.dict, self.com.as_ref());
        if paths.is_empty() {
            return vec![];
        }
        paths[self.nth_conversion % paths.len()].intervals.clone()
    }
    fn intervals(&self) -> impl Iterator<Item = Interval> {
        self.conversion().into_iter()
    }
    fn snapshot(&mut self) {
        // for interval in self.intervals() {
        //     self.com.select(interval);
        // }
        // self.nth_conversion = 0;
    }
    fn cursor(&self) -> usize {
        self.com.cursor()
    }
    fn learn_phrase_in_range_notify(
        &mut self,
        start: usize,
        end: usize,
    ) -> Result<(), UpdateDictionaryError> {
        let result = self.learn_phrase_in_range_quiet(start, end);
        match &result {
            Ok(phrase) => {
                self.notice_buffer = format!("加入：{phrase}");
                Ok(())
            }
            Err(msg) => {
                msg.clone_into(&mut self.notice_buffer);
                Err(UpdateDictionaryError::new())
            }
        }
    }
    // FIXME enhance user visible reporting
    fn learn_phrase_in_range_quiet(&mut self, start: usize, end: usize) -> Result<String, String> {
        if end > self.com.len() {
            return Err("加詞失敗：字數不符或夾雜符號".to_owned());
        }
        let symbols = self.com.symbols()[start..end].to_vec();
        if symbols.iter().any(Symbol::is_char) {
            return Err("加詞失敗：字數不符或夾雜符號".to_owned());
        }
        let syllables: Vec<Syllable> = symbols
            .iter()
            .map(|s| s.to_syllable().unwrap_or_default())
            .collect();
        // FIXME
        let phrase = self
            .conversion()
            .into_iter()
            .map(|interval| interval.text)
            .collect::<String>()
            .chars()
            .skip(start)
            .take(end - start)
            .collect::<String>();
        if self
            .dict
            .user_dict()
            .lookup(&syllables, LookupStrategy::Standard)
            .into_iter()
            .any(|it| it.as_str() == phrase)
        {
            return Err(format!("已有：{phrase}"));
        }
        let result = self
            .learn_phrase(&syllables, &phrase)
            .map_err(|_| "加詞失敗：字數不符或夾雜符號".to_owned());
        if result.is_ok() {
            self.dirty_level += 1;
        }
        result.map(|_| phrase)
    }
    fn learn_phrase(
        &mut self,
        syllables: &[Syllable],
        phrase: &str,
    ) -> Result<(), UpdateDictionaryError> {
        if syllables.len() != phrase.chars().count() {
            warn!(
                "syllables({:?})[{}] and phrase({})[{}] has different length",
                &syllables,
                syllables.len(),
                &phrase,
                phrase.chars().count()
            );
            return Err(UpdateDictionaryError::new());
        }
        let phrases = self.dict.lookup(syllables, LookupStrategy::Standard);
        if phrases.is_empty() {
            self.dict.add_phrase(syllables, (phrase, 10).into())?;
            return Ok(());
        }
        let phrase = phrases
            .iter()
            .find(|p| p.as_str() == phrase)
            .cloned()
            .unwrap_or((phrase, 10).into());
        // TODO: fine tune learning curve
        let max_freq = phrases.iter().map(|p| p.freq()).max().unwrap_or(1);
        let user_freq = self.estimate.estimate(&phrase, max_freq);
        let time = self.estimate.now();

        let _ = self.dict.update_phrase(syllables, phrase, user_freq, time);
        self.dirty_level += 1;
        Ok(())
    }
    fn unlearn_phrase(
        &mut self,
        syllables: &[Syllable],
        phrase: &str,
    ) -> Result<(), UpdateDictionaryError> {
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
    fn switch_character_form(&mut self) {
        self.options = EditorOptions {
            character_form: match self.options.character_form {
                CharacterForm::Halfwidth => CharacterForm::Fullwidth,
                CharacterForm::Fullwidth => CharacterForm::Halfwidth,
            },
            ..self.options
        };
    }
    fn cancel_selecting(&mut self) {
        self.com.pop_cursor();
    }
    fn commit(&mut self) {
        self.commit_buffer.clear();
        let intervals = self.conversion();
        debug!("buffer {:?}", self.com);
        if !self.options.disable_auto_learn_phrase {
            self.auto_learn(&intervals);
        }
        let output = intervals
            .into_iter()
            .map(|interval| interval.text)
            .collect::<String>();
        self.commit_buffer.push_str(&output);
        self.com.clear();
        self.nth_conversion = 0;
        self.last_key_behavior = EditorKeyBehavior::Commit;
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
            self.commit_buffer.push_str(&it.text);
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
        for (syllables, phrase) in collect_new_phrases(intervals, self.com.symbols()) {
            if let Err(error) = self.learn_phrase(&syllables, &phrase) {
                error!("Failed to learn phrase {phrase} from {syllables:?}: {error:#}");
            }
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

fn collect_new_phrases(intervals: &[Interval], symbols: &[Symbol]) -> Vec<(Vec<Syllable>, String)> {
    debug!("intervals {:?}", intervals);
    let mut pending = String::new();
    let mut syllables = Vec::new();
    let mut phrases = vec![];
    let mut collect = |syllables, pending| {
        if !phrases.iter().any(|(_, p)| p == &pending) {
            debug!("autolearn {:?} as {}", &syllables, &pending);
            phrases.push((syllables, pending))
        }
    };
    // Step 1. collect all intervals
    for interval in intervals.iter().filter(|it| it.is_phrase) {
        let syllables = symbols[interval.start..interval.end]
            .iter()
            .map(|s| s.to_syllable().unwrap())
            .collect();
        let pending = interval.text.clone().into_string();
        collect(syllables, pending);
    }
    // Step 2. collect all intervals with length one with break words removed
    for interval in intervals.iter() {
        if interval.is_phrase && interval.len() == 1 && !is_break_word(&interval.text) {
            pending.push_str(&interval.text);
            syllables.extend(
                symbols[interval.start..interval.end]
                    .iter()
                    .map(|s| s.to_syllable().unwrap()),
            );
        } else if !pending.is_empty() {
            collect(mem::take(&mut syllables), mem::take(&mut pending));
        }
    }
    if !pending.is_empty() {
        collect(mem::take(&mut syllables), mem::take(&mut pending));
    }
    // Step 3. collect all intervals with length one including break words
    for interval in intervals {
        if interval.is_phrase && interval.len() == 1 {
            pending.push_str(&interval.text);
            syllables.extend(
                symbols[interval.start..interval.end]
                    .iter()
                    .map(|s| s.to_syllable().unwrap()),
            );
        } else if !pending.is_empty() {
            collect(mem::take(&mut syllables), mem::take(&mut pending));
        }
    }
    if !pending.is_empty() {
        collect(syllables, pending);
    }
    phrases
}

impl BasicEditor for Editor {
    fn process_keyevent(&mut self, key_event: KeyboardEvent) -> EditorKeyBehavior {
        debug!("process {}", key_event);
        self.shared.estimate.tick();
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

        if self.shared.options.conversion_engine == ConversionEngineKind::SimpleEngine
            && self.is_entering()
            && !self.shared.com.is_empty()
        {
            self.shared.commit();
        }

        if self.is_entering() && self.shared.last_key_behavior == EditorKeyBehavior::Absorb {
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
        if editor.sym_sel.is_empty() {
            self.spin_bell()
        } else {
            Transition::ToState(Box::new(Selecting::new_symbol(editor)))
        }
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
    fn next(&mut self, shared: &mut SharedState, ev: KeyboardEvent) -> Transition {
        match ev.ksym {
            SYM_BACKSPACE => {
                if shared.com.is_empty() {
                    self.spin_ignore()
                } else {
                    shared.com.remove_before_cursor();
                    self.spin_absorb()
                }
            }
            SYM_CAPSLOCK => {
                shared.switch_language_mode();
                self.spin_absorb()
            }
            code if ev.ksym.is_digit() && ev.is_state_on(KeyState::Control) => {
                let n = code.to_digit().unwrap_or_default() as usize;
                if n == 0 || n == 1 {
                    return self.start_symbol_input(shared);
                }
                let result = match shared.options.user_phrase_add_dir {
                    UserPhraseAddDirection::Forward => {
                        shared.learn_phrase_in_range_notify(shared.cursor(), shared.cursor() + n)
                    }
                    UserPhraseAddDirection::Backward => {
                        if shared.cursor() >= n {
                            shared
                                .learn_phrase_in_range_notify(shared.cursor() - n, shared.cursor())
                        } else {
                            "加詞失敗：字數不符或夾雜符號".clone_into(&mut shared.notice_buffer);
                            return self.spin_bell();
                        }
                    }
                };
                match result {
                    Ok(_) => self.spin_absorb(),
                    Err(_) => self.spin_bell(),
                }
            }
            SYM_RETURN | SYM_ESC | SYM_TAB | SYM_HOME | SYM_END | SYM_LEFT | SYM_RIGHT | SYM_UP
            | SYM_DOWN | SYM_PAGEUP | SYM_PAGEDOWN
                if shared.com.is_empty() =>
            {
                self.spin_ignore()
            }
            SYM_TAB if shared.com.is_end_of_buffer() => {
                shared.nth_conversion += 1;
                self.spin_absorb()
            }
            SYM_TAB => {
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
            SYM_DELETE => {
                if shared.com.is_end_of_buffer() {
                    self.spin_ignore()
                } else {
                    shared.com.remove_after_cursor();
                    self.spin_absorb()
                }
            }
            SYM_HOME => {
                shared.snapshot();
                shared.com.move_cursor_to_beginning();
                self.spin_absorb()
            }
            SYM_LEFT if ev.is_state_on(KeyState::Shift) => {
                if shared.com.is_beginning_of_buffer() {
                    return self.spin_ignore();
                }
                shared.snapshot();
                self.start_highlighting(shared.cursor() - 1)
            }
            SYM_RIGHT if ev.is_state_on(KeyState::Shift) => {
                if shared.com.is_end_of_buffer() {
                    return self.spin_ignore();
                }
                shared.snapshot();
                self.start_highlighting(shared.cursor() + 1)
            }
            SYM_LEFT => {
                shared.snapshot();
                shared.com.move_cursor_left(1);
                self.spin_absorb()
            }
            SYM_RIGHT => {
                shared.snapshot();
                shared.com.move_cursor_right(1);
                self.spin_absorb()
            }
            SYM_UP => self.spin_ignore(),
            SYM_SPACE
                if ev.is_state_on(KeyState::Shift)
                    && shared.options.enable_fullwidth_toggle_key =>
            {
                shared.switch_character_form();
                self.spin_absorb()
            }
            SYM_SPACE
                if shared.options.space_is_select_key
                    && shared.options.language_mode == LanguageMode::Chinese =>
            {
                self.start_selecting_or_input_space(shared)
            }
            SYM_DOWN => {
                debug!("buffer {:?}", shared.com);
                self.start_selecting(shared)
            }
            SYM_END | SYM_PAGEUP | SYM_PAGEDOWN => {
                shared.snapshot();
                shared.com.move_cursor_to_end();
                self.spin_absorb()
            }
            SYM_RETURN => {
                shared.commit();
                self.spin_commit()
            }
            SYM_ESC => {
                if shared.options.esc_clear_all_buffer && !shared.com.is_empty() {
                    shared.com.clear();
                    self.spin_absorb()
                } else {
                    self.spin_ignore()
                }
            }
            _ if ev.ksym.is_keypad() && ev.is_state_on(KeyState::NumLock) => {
                if shared.com.is_empty() {
                    shared.commit_buffer.clear();
                    shared.commit_buffer.push(ev.ksym.to_unicode());
                    self.spin_commit()
                } else {
                    shared.com.insert(Symbol::from(ev.ksym.to_unicode()));
                    self.spin_absorb()
                }
            }
            _ => {
                if shared.nth_conversion != 0 {
                    shared.snapshot();
                }
                match shared.options.language_mode {
                    LanguageMode::Chinese if ev.ksym == SYM_GRAVE && !ev.has_modifiers() => {
                        self.start_symbol_input(shared)
                    }
                    LanguageMode::Chinese if ev.ksym == SYM_SPACE => {
                        match shared.options.character_form {
                            CharacterForm::Halfwidth => {
                                if shared.com.is_empty() {
                                    shared.commit_buffer.clear();
                                    shared.commit_buffer.push(ev.ksym.to_unicode());
                                    self.spin_commit()
                                } else {
                                    shared.com.insert(Symbol::from(ev.ksym.to_unicode()));
                                    self.spin_absorb()
                                }
                            }
                            CharacterForm::Fullwidth => {
                                let char_ = full_width_symbol_input(ev.ksym.to_unicode()).unwrap();
                                if shared.com.is_empty() {
                                    shared.commit_buffer.clear();
                                    shared.commit_buffer.push(char_);
                                    self.spin_commit()
                                } else {
                                    shared.com.insert(Symbol::from(char_));
                                    self.spin_absorb()
                                }
                            }
                        }
                    }
                    LanguageMode::Chinese => {
                        if shared.options.easy_symbol_input && ev.is_state_on(KeyState::Shift) {
                            // Priortize symbol input
                            if let Some(expended) = shared.abbr.find_abbrev(ev.ksym.to_unicode()) {
                                expended
                                    .chars()
                                    .for_each(|ch| shared.com.insert(Symbol::from(ch)));
                                return self.spin_absorb();
                            }
                        }
                        if !ev.has_modifiers() && KeyBehavior::Absorb == shared.syl.key_press(ev) {
                            return self.start_enter_syllable();
                        }
                        if let Some(symbol) = special_symbol_input(ev.ksym.to_unicode()) {
                            shared.com.insert(Symbol::from(symbol));
                            return self.spin_absorb();
                        }
                        if ev.ksym.is_unicode() {
                            match shared.options.character_form {
                                CharacterForm::Halfwidth => {
                                    if shared.com.is_empty() {
                                        // FIXME we should ignore these keys if pre-edit is empty
                                        shared.commit_buffer.clear();
                                        shared.commit_buffer.push(ev.ksym.to_unicode());
                                        return self.spin_commit();
                                    } else {
                                        shared.com.insert(Symbol::from(ev.ksym.to_unicode()));
                                        return self.spin_absorb();
                                    }
                                }
                                CharacterForm::Fullwidth => {
                                    let char_ =
                                        full_width_symbol_input(ev.ksym.to_unicode()).unwrap();
                                    if shared.com.is_empty() {
                                        shared.commit_buffer.clear();
                                        shared.commit_buffer.push(char_);
                                        return self.spin_commit();
                                    } else {
                                        shared.com.insert(Symbol::from(char_));
                                        return self.spin_absorb();
                                    }
                                }
                            }
                        }
                        self.spin_bell()
                    }
                    LanguageMode::English => {
                        if !ev.ksym.is_unicode() {
                            return self.spin_bell();
                        }
                        match shared.options.character_form {
                            CharacterForm::Halfwidth => {
                                if shared.com.is_empty() {
                                    // FIXME we should ignore these keys if pre-edit is empty
                                    shared.commit_buffer.clear();
                                    shared.commit_buffer.push(ev.ksym.to_unicode());
                                    self.spin_commit()
                                } else {
                                    shared.com.insert(Symbol::from(ev.ksym.to_unicode()));
                                    self.spin_absorb()
                                }
                            }
                            CharacterForm::Fullwidth => {
                                let char_ = full_width_symbol_input(ev.ksym.to_unicode()).unwrap();
                                if shared.com.is_empty() {
                                    shared.commit_buffer.clear();
                                    shared.commit_buffer.push(char_);
                                    self.spin_commit()
                                } else {
                                    shared.com.insert(Symbol::from(char_));
                                    self.spin_absorb()
                                }
                            }
                        }
                    }
                }
            }
        }
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
    fn start_selecting_simple_engine(&self, editor: &mut SharedState) -> Transition {
        editor.syl.clear();
        Transition::ToState(Box::new(Selecting::new_phrase_for_simple_engine(editor)))
    }
}

impl State for EnteringSyllable {
    fn next(&mut self, shared: &mut SharedState, ev: KeyboardEvent) -> Transition {
        match ev.ksym {
            SYM_BACKSPACE => {
                shared.syl.remove_last();

                if !shared.syl.is_empty() {
                    self.spin_absorb()
                } else {
                    self.start_entering()
                }
            }
            SYM_CAPSLOCK => {
                shared.syl.clear();
                shared.switch_language_mode();
                self.start_entering()
            }
            SYM_ESC => {
                shared.syl.clear();
                if shared.options.esc_clear_all_buffer {
                    shared.com.clear();
                }
                self.start_entering()
            }
            _ => {
                let key_behavior = match shared.options.lookup_strategy {
                    LookupStrategy::FuzzyPartialPrefix => shared.syl.fuzzy_key_press(ev),
                    LookupStrategy::Standard => shared.syl.key_press(ev),
                };
                match key_behavior {
                    KeyBehavior::Absorb => self.spin_absorb(),
                    KeyBehavior::Fuzzy(syl) => {
                        if !shared
                            .dict
                            .lookup(&[syl], shared.options.lookup_strategy)
                            .is_empty()
                        {
                            shared.com.insert(Symbol::from(syl));
                        }
                        self.spin_absorb()
                    }
                    KeyBehavior::Commit => {
                        if !shared
                            .dict
                            .lookup(&[shared.syl.read()], shared.options.lookup_strategy)
                            .is_empty()
                        {
                            shared.com.insert(Symbol::from(shared.syl.read()));
                            shared.syl.clear();
                            if shared.options.conversion_engine
                                == ConversionEngineKind::SimpleEngine
                            {
                                self.start_selecting_simple_engine(shared)
                            } else {
                                self.start_entering()
                            }
                        } else {
                            shared.syl.clear();
                            self.start_entering()
                        }
                    }
                    _ => self.spin_bell(),
                }
            }
        }
    }
}

impl Selecting {
    fn new_phrase(editor: &mut SharedState) -> Self {
        editor.com.push_cursor();
        editor.com.clamp_cursor();

        let mut sel = PhraseSelector::new(
            !editor.options.phrase_choice_rearward,
            editor.options.lookup_strategy,
            editor.com.to_composition(),
        );
        sel.init(editor.cursor(), &editor.dict);

        Selecting {
            page_no: 0,
            action: SelectingAction::Replace,
            sel: Selector::Phrase(sel),
        }
    }
    fn new_phrase_for_simple_engine(editor: &mut SharedState) -> Self {
        editor.com.push_cursor();
        // editor.com.clamp_cursor();

        let mut sel = PhraseSelector::new(
            false,
            editor.options.lookup_strategy,
            editor.com.to_composition(),
        );
        sel.init_single_word(editor.cursor());

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
    fn candidates(&self, editor: &SharedState, dict: &Layered) -> Vec<String> {
        match &self.sel {
            Selector::Phrase(sel) => sel.candidates(editor, dict),
            Selector::Symbol(sel) => sel.menu(),
            Selector::SpecialSymmbol(sel) => sel.menu(),
        }
    }
    fn total_page(&self, editor: &SharedState, dict: &Layered) -> usize {
        self.candidates(editor, dict)
            .len()
            .div_ceil(editor.options.candidates_per_page)
    }
    fn select(&mut self, editor: &mut SharedState, n: usize) -> Transition {
        let offset = self.page_no * editor.options.candidates_per_page + n;
        match self.sel {
            Selector::Phrase(ref sel) => {
                let candidates = sel.candidates(editor, &editor.dict);
                debug!("candidates: {:?}", &candidates);
                match candidates.get(offset) {
                    Some(phrase) => {
                        let interval = sel.interval(phrase.as_str());
                        let len = interval.len();
                        editor.com.select(interval);
                        debug!("Auto Shift {}", editor.options.auto_shift_cursor);
                        editor.com.pop_cursor();
                        if editor.options.auto_shift_cursor {
                            if editor.options.phrase_choice_rearward {
                                editor.com.move_cursor_right(1);
                            } else {
                                editor.com.move_cursor_right(len);
                            }
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
    fn next(&mut self, shared: &mut SharedState, ev: KeyboardEvent) -> Transition {
        if ev.is_state_on(KeyState::Control) || ev.is_state_on(KeyState::Shift) {
            return self.spin_bell();
        }

        match ev.ksym {
            SYM_BACKSPACE => {
                shared.cancel_selecting();
                self.start_entering()
            }
            SYM_CAPSLOCK => {
                shared.switch_language_mode();
                shared.cancel_selecting();
                self.start_entering()
            }
            SYM_UP => {
                shared.cancel_selecting();
                self.start_entering()
            }
            SYM_DOWN | SYM_SPACE => {
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
            SYM_LOWER_J => {
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
                        shared.options.lookup_strategy,
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
            SYM_LOWER_K => {
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
                        shared.options.lookup_strategy,
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
            SYM_LEFT | SYM_PAGEUP => {
                if self.page_no > 0 {
                    self.page_no -= 1;
                } else {
                    self.page_no = self.total_page(shared, &shared.dict).saturating_sub(1);
                }
                self.spin_absorb()
            }
            SYM_RIGHT | SYM_PAGEDOWN => {
                if self.page_no + 1 < self.total_page(shared, &shared.dict) {
                    self.page_no += 1;
                } else {
                    self.page_no = 0;
                }
                self.spin_absorb()
            }
            _ if ev.ksym.is_digit() => {
                let n = ev.ksym.to_digit().unwrap() as usize;
                let n = if n == 0 { 9 } else { n - 1 };
                self.select(shared, n)
            }
            SYM_ESC => {
                shared.cancel_selecting();
                shared.com.pop_cursor();
                if shared.options.conversion_engine == ConversionEngineKind::SimpleEngine {
                    shared.com.clear();
                }
                self.start_entering()
            }
            SYM_DELETE => {
                // NB: should be Ignore but return Absorb for backward compat
                self.spin_absorb()
            }
            _ => self.spin_bell(),
        }
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
    fn next(&mut self, shared: &mut SharedState, ev: KeyboardEvent) -> Transition {
        match ev.ksym {
            SYM_CAPSLOCK => {
                shared.switch_language_mode();
                self.start_entering()
            }
            SYM_LEFT if ev.is_state_on(KeyState::Shift) => {
                if self.moving_cursor != 0 {
                    self.moving_cursor -= 1;
                }
                self.spin_absorb()
            }
            SYM_RIGHT if ev.is_state_on(KeyState::Shift) => {
                if self.moving_cursor != shared.com.len() {
                    self.moving_cursor += 1;
                }
                self.spin_absorb()
            }
            SYM_RETURN => {
                let start = min(self.moving_cursor, shared.com.cursor());
                let end = max(self.moving_cursor, shared.com.cursor());
                shared.com.move_cursor(self.moving_cursor);
                let _ = shared.learn_phrase_in_range_notify(start, end);
                self.start_entering()
            }
            _ => self.start_entering(),
        }
    }
}

#[cfg(test)]
mod tests {
    use estimate::LaxUserFreqEstimate;

    use super::collect_new_phrases;
    use super::{BasicEditor, Editor};
    use crate::{
        conversion::{ChewingEngine, Interval, Symbol},
        dictionary::{Layered, TrieBuf},
        editor::{EditorKeyBehavior, EditorOptions, SymbolSelector, abbrev::AbbrevTable, estimate},
        input::{
            KeyboardEvent, keycode,
            keymap::{QWERTY_MAP, map_ascii},
            keysym,
        },
        syl,
        zhuyin::Bopomofo as bpmf,
    };

    const CAPSLOCK_EVENT: KeyboardEvent = KeyboardEvent::builder()
        .code(keycode::KEY_CAPSLOCK)
        .ksym(keysym::SYM_CAPSLOCK)
        .caps_lock_if(true)
        .build();

    #[test]
    fn editing_mode_input_bopomofo() {
        let dict = Layered::new(
            vec![Box::new(TrieBuf::new_in_memory())],
            Box::new(TrieBuf::new_in_memory()),
        );
        let conversion_engine = Box::new(ChewingEngine::new());
        let estimate = LaxUserFreqEstimate::new(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

        let ev = KeyboardEvent {
            code: keycode::KEY_H,
            ksym: keysym::SYM_LOWER_H,
            state: 0,
        };
        let key_behavior = editor.process_keyevent(ev);

        assert_eq!(EditorKeyBehavior::Absorb, key_behavior);
        assert_eq!(syl![bpmf::C], editor.syllable_buffer());

        let ev = KeyboardEvent {
            code: keycode::KEY_K,
            ksym: keysym::SYM_LOWER_K,
            state: 0,
        };
        let key_behavior = editor.process_keyevent(ev);

        assert_eq!(EditorKeyBehavior::Absorb, key_behavior);
        assert_eq!(syl![bpmf::C, bpmf::E], editor.syllable_buffer());
    }

    #[test]
    fn editing_mode_input_bopomofo_commit() {
        let dict = TrieBuf::from([(
            vec![crate::syl![bpmf::C, bpmf::E, bpmf::TONE4]],
            vec![("冊", 100)],
        )]);
        let dict = Layered::new(vec![Box::new(dict)], Box::new(TrieBuf::new_in_memory()));
        let conversion_engine = Box::new(ChewingEngine::new());
        let estimate = LaxUserFreqEstimate::new(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

        let keys = [b'h', b'k', b'4'];
        let key_behaviors: Vec<_> = keys
            .into_iter()
            .map(|key| map_ascii(&QWERTY_MAP, key))
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
    fn editing_mode_input_bopomofo_select() {
        let dict = TrieBuf::from([(
            vec![crate::syl![bpmf::C, bpmf::E, bpmf::TONE4]],
            vec![("冊", 100), ("測", 200)],
        )]);
        let dict = Layered::new(vec![Box::new(dict)], Box::new(TrieBuf::new_in_memory()));
        let conversion_engine = Box::new(ChewingEngine::new());
        let estimate = LaxUserFreqEstimate::new(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

        editor.set_editor_options(EditorOptions {
            sort_candidates_by_frequency: false,
            ..Default::default()
        });

        editor.process_keyevent(
            KeyboardEvent::builder()
                .code(keycode::KEY_H)
                .ksym(keysym::SYM_LOWER_H)
                .build(),
        );
        editor.process_keyevent(
            KeyboardEvent::builder()
                .code(keycode::KEY_K)
                .ksym(keysym::SYM_LOWER_H)
                .build(),
        );
        editor.process_keyevent(
            KeyboardEvent::builder()
                .code(keycode::KEY_4)
                .ksym(keysym::SYM_4)
                .build(),
        );
        editor.process_keyevent(
            KeyboardEvent::builder()
                .code(keycode::KEY_DOWN)
                .ksym(keysym::SYM_DOWN)
                .build(),
        );
        let candidates = editor
            .all_candidates()
            .expect("should be in selection mode");
        assert_eq!(vec!["冊", "測"], candidates);
    }

    #[test]
    fn editing_mode_input_bopomofo_select_sorted() {
        let dict = TrieBuf::from([(
            vec![crate::syl![bpmf::C, bpmf::E, bpmf::TONE4]],
            vec![("冊", 100), ("測", 200)],
        )]);
        let dict = Layered::new(vec![Box::new(dict)], Box::new(TrieBuf::new_in_memory()));
        let conversion_engine = Box::new(ChewingEngine::new());
        let estimate = LaxUserFreqEstimate::new(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

        editor.set_editor_options(EditorOptions {
            sort_candidates_by_frequency: true,
            ..Default::default()
        });

        editor.process_keyevent(
            KeyboardEvent::builder()
                .code(keycode::KEY_H)
                .ksym(keysym::SYM_LOWER_H)
                .build(),
        );
        editor.process_keyevent(
            KeyboardEvent::builder()
                .code(keycode::KEY_K)
                .ksym(keysym::SYM_LOWER_H)
                .build(),
        );
        editor.process_keyevent(
            KeyboardEvent::builder()
                .code(keycode::KEY_4)
                .ksym(keysym::SYM_4)
                .build(),
        );
        editor.process_keyevent(
            KeyboardEvent::builder()
                .code(keycode::KEY_DOWN)
                .ksym(keysym::SYM_DOWN)
                .build(),
        );
        let candidates = editor
            .all_candidates()
            .expect("should be in selection mode");
        assert_eq!(vec!["測", "冊"], candidates);
    }

    #[test]
    fn editing_mode_input_chinese_to_english_mode() {
        let dict = TrieBuf::from([(
            vec![crate::syl![bpmf::C, bpmf::E, bpmf::TONE4]],
            vec![("冊", 100)],
        )]);
        let dict = Layered::new(vec![Box::new(dict)], Box::new(TrieBuf::new_in_memory()));
        let conversion_engine = Box::new(ChewingEngine::new());
        let estimate = LaxUserFreqEstimate::new(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

        let keys = [
            map_ascii(&QWERTY_MAP, b'h'),
            map_ascii(&QWERTY_MAP, b'k'),
            map_ascii(&QWERTY_MAP, b'4'),
            // Toggle english mode
            CAPSLOCK_EVENT,
            map_ascii(&QWERTY_MAP, b'z'),
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
        let dict = TrieBuf::from([(
            vec![crate::syl![bpmf::C, bpmf::E, bpmf::TONE4]],
            vec![("冊", 100)],
        )]);
        let dict = Layered::new(vec![Box::new(dict)], Box::new(TrieBuf::new_in_memory()));
        let conversion_engine = Box::new(ChewingEngine::new());
        let estimate = LaxUserFreqEstimate::new(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

        let keys = [
            // Switch to english mode
            CAPSLOCK_EVENT,
            map_ascii(&QWERTY_MAP, b'x'),
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
            CAPSLOCK_EVENT,
            map_ascii(&QWERTY_MAP, b'h'),
            map_ascii(&QWERTY_MAP, b'k'),
            map_ascii(&QWERTY_MAP, b'4'),
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
        let dict = TrieBuf::from([(
            vec![crate::syl![bpmf::C, bpmf::E, bpmf::TONE4]],
            vec![("冊", 100)],
        )]);
        let dict = Layered::new(vec![Box::new(dict)], Box::new(TrieBuf::new_in_memory()));
        let conversion_engine = Box::new(ChewingEngine::new());
        let estimate = LaxUserFreqEstimate::new(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

        let keys = [
            map_ascii(&QWERTY_MAP, b'!'),
            map_ascii(&QWERTY_MAP, b'h'),
            map_ascii(&QWERTY_MAP, b'k'),
            map_ascii(&QWERTY_MAP, b'4'),
            map_ascii(&QWERTY_MAP, b'<'),
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
        let dict = TrieBuf::new_in_memory();
        let dict = Layered::new(vec![Box::new(dict)], Box::new(TrieBuf::new_in_memory()));
        let conversion_engine = Box::new(ChewingEngine::new());
        let estimate = LaxUserFreqEstimate::new(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

        editor.shared.switch_character_form();

        let steps = [
            (CAPSLOCK_EVENT, EditorKeyBehavior::Absorb, "", "", ""),
            (
                map_ascii(&QWERTY_MAP, b'0'),
                EditorKeyBehavior::Commit,
                "",
                "",
                "０",
            ),
            (
                map_ascii(&QWERTY_MAP, b'-'),
                EditorKeyBehavior::Commit,
                "",
                "",
                "－",
            ),
        ];

        for s in steps {
            let key = s.0;
            let kb = editor.process_keyevent(key);
            assert_eq!(s.1, kb);
            assert_eq!(s.2, editor.syllable_buffer().to_string());
            assert_eq!(s.3, editor.display());
            assert_eq!(s.4, editor.display_commit());
        }
    }

    #[test]
    fn editing_mode_open_empty_symbol_table_then_bell() {
        let dict = Layered::new(
            vec![Box::new(TrieBuf::new_in_memory())],
            Box::new(TrieBuf::new_in_memory()),
        );
        let conversion_engine = Box::new(ChewingEngine::new());
        let estimate = LaxUserFreqEstimate::new(0);
        let abbrev = AbbrevTable::new();
        let sym_sel = SymbolSelector::default();
        let mut editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);

        let ev = map_ascii(&QWERTY_MAP, b'`');
        let key_behavior = editor.process_keyevent(ev);

        assert_eq!(EditorKeyBehavior::Bell, key_behavior);
        assert_eq!(syl![], editor.syllable_buffer());
    }

    #[test]
    fn collect_new_phrases_with_no_break_word() {
        let intervals = [
            Interval {
                start: 0,
                end: 2,
                is_phrase: true,
                text: "今天".into(),
            },
            Interval {
                start: 2,
                end: 4,
                is_phrase: true,
                text: "天氣".into(),
            },
            Interval {
                start: 4,
                end: 6,
                is_phrase: true,
                text: "真好".into(),
            },
        ];
        let symbols = [
            Symbol::Syllable(syl![bpmf::J, bpmf::I, bpmf::EN]),
            Symbol::Syllable(syl![bpmf::T, bpmf::I, bpmf::AN]),
            Symbol::Syllable(syl![bpmf::T, bpmf::I, bpmf::AN]),
            Symbol::Syllable(syl![bpmf::Q, bpmf::I, bpmf::TONE4]),
            Symbol::Syllable(syl![bpmf::ZH, bpmf::EN]),
            Symbol::Syllable(syl![bpmf::H, bpmf::AU, bpmf::TONE3]),
        ];
        let phrases = collect_new_phrases(&intervals, &symbols);
        assert_eq!(
            vec![
                (
                    vec![
                        syl![bpmf::J, bpmf::I, bpmf::EN],
                        syl![bpmf::T, bpmf::I, bpmf::AN],
                    ],
                    "今天".to_string()
                ),
                (
                    vec![
                        syl![bpmf::T, bpmf::I, bpmf::AN],
                        syl![bpmf::Q, bpmf::I, bpmf::TONE4],
                    ],
                    "天氣".to_string()
                ),
                (
                    vec![
                        syl![bpmf::ZH, bpmf::EN],
                        syl![bpmf::H, bpmf::AU, bpmf::TONE3],
                    ],
                    "真好".to_string()
                ),
            ],
            phrases
        );
    }

    #[test]
    fn collect_new_phrases_with_break_word() {
        let intervals = [
            Interval {
                start: 0,
                end: 2,
                is_phrase: true,
                text: "今天".into(),
            },
            Interval {
                start: 2,
                end: 3,
                is_phrase: true,
                text: "也".into(),
            },
            Interval {
                start: 3,
                end: 4,
                is_phrase: true,
                text: "是".into(),
            },
            Interval {
                start: 4,
                end: 7,
                is_phrase: true,
                text: "好天氣".into(),
            },
        ];
        let symbols = [
            Symbol::Syllable(syl![bpmf::J, bpmf::I, bpmf::EN]),
            Symbol::Syllable(syl![bpmf::T, bpmf::I, bpmf::AN]),
            Symbol::Syllable(syl![bpmf::I, bpmf::EH, bpmf::TONE3]),
            Symbol::Syllable(syl![bpmf::SH, bpmf::TONE4]),
            Symbol::Syllable(syl![bpmf::H, bpmf::AU, bpmf::TONE3]),
            Symbol::Syllable(syl![bpmf::T, bpmf::I, bpmf::AN]),
            Symbol::Syllable(syl![bpmf::Q, bpmf::I, bpmf::TONE4]),
        ];
        let phrases = collect_new_phrases(&intervals, &symbols);
        assert_eq!(
            vec![
                (
                    vec![
                        syl![bpmf::J, bpmf::I, bpmf::EN],
                        syl![bpmf::T, bpmf::I, bpmf::AN],
                    ],
                    "今天".to_string()
                ),
                (vec![syl![bpmf::I, bpmf::EH, bpmf::TONE3]], "也".to_string()),
                (vec![syl![bpmf::SH, bpmf::TONE4]], "是".to_string()),
                (
                    vec![
                        syl![bpmf::H, bpmf::AU, bpmf::TONE3],
                        syl![bpmf::T, bpmf::I, bpmf::AN],
                        syl![bpmf::Q, bpmf::I, bpmf::TONE4],
                    ],
                    "好天氣".to_string()
                ),
                (
                    vec![
                        syl![bpmf::I, bpmf::EH, bpmf::TONE3],
                        syl![bpmf::SH, bpmf::TONE4]
                    ],
                    "也是".to_string()
                ),
            ],
            phrases
        );
    }
}
