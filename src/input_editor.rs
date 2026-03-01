use ratatui::crossterm::event::{KeyCode, KeyEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEditMode {
    Insert,
    Normal,
    Visual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorCommand {
    None,
    Submit,
    ExitInputMode,
    IncrementFocus,
    DecrementFocus,
    Yanked { start: usize, end: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    LineStart,
    LineEnd,
    WordForward,
    WordBackward,
}

/// Reusable line editor with Vim-style insert/normal/visual modes and yank/paste support.
///
/// Cursor semantics:
/// - `Insert`: cursor is between characters (`0..=len`)
/// - `Normal`/`Visual`: cursor is on a character (`0..len-1`, or `0` when empty)
pub struct InputEditor {
    input: String,
    cursor: usize,
    mode: InputEditMode,
    register: String,
    visual_anchor: Option<usize>,
}

impl Default for InputEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEditor {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            mode: InputEditMode::Insert,
            register: String::new(),
            visual_anchor: None,
        }
    }

    pub fn with_input(input: String) -> Self {
        let cursor = input.chars().count();
        Self {
            input,
            cursor,
            mode: InputEditMode::Insert,
            register: String::new(),
            visual_anchor: None,
        }
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn mode(&self) -> InputEditMode {
        self.mode
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn register(&self) -> &str {
        &self.register
    }

    pub fn visual_selection_range(&self) -> Option<(usize, usize)> {
        self.visual_range()
    }

    pub fn set_input(&mut self, input: String) {
        self.input = input;
        self.cursor = self.input.chars().count();
        self.mode = InputEditMode::Insert;
        self.visual_anchor = None;
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.mode = InputEditMode::Insert;
        self.visual_anchor = None;
    }

    pub fn switch_to_insert_mode(&mut self) {
        self.mode = InputEditMode::Insert;
        self.cursor = self.cursor.min(self.char_len());
        self.visual_anchor = None;
    }

    pub fn switch_to_normal_mode(&mut self) {
        let previous_mode = self.mode;
        self.mode = InputEditMode::Normal;
        self.visual_anchor = None;

        let len = self.char_len();
        if len == 0 {
            self.cursor = 0;
        } else {
            self.cursor = match previous_mode {
                InputEditMode::Insert => self.cursor.saturating_sub(1).min(len - 1),
                InputEditMode::Normal | InputEditMode::Visual => self.cursor.min(len - 1),
            };
        }
    }

    pub fn switch_to_visual_mode(&mut self) {
        self.mode = InputEditMode::Visual;
        if self.char_len() == 0 {
            self.cursor = 0;
            self.visual_anchor = None;
        } else {
            self.cursor = self.cursor.min(self.char_len() - 1);
            self.visual_anchor = Some(self.cursor);
        }
    }

    pub fn move_insert_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_insert_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.char_len());
    }

    pub fn enter_char(&mut self, ch: char) {
        let byte_index = self.byte_index_from_char_index(self.cursor);
        self.input.insert(byte_index, ch);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let from_left = self.cursor - 1;
        let before = self.input.chars().take(from_left);
        let after = self.input.chars().skip(self.cursor);
        self.input = before.chain(after).collect();
        self.cursor -= 1;
    }

    pub fn delete_under_cursor(&mut self) {
        let len = self.char_len();
        if len == 0 || self.cursor >= len {
            return;
        }

        let before = self.input.chars().take(self.cursor);
        let after = self.input.chars().skip(self.cursor + 1);
        self.input = before.chain(after).collect();

        let new_len = self.char_len();
        if new_len == 0 {
            self.cursor = 0;
        } else if self.cursor >= new_len {
            self.cursor = new_len - 1;
        }
    }

    pub fn apply_motion(&mut self, motion: Motion) {
        self.cursor = self.motion_target(motion);
    }

    pub fn yank_line(&mut self) {
        self.register = self.input.clone();
    }

    pub fn yank_visual_selection(&mut self) {
        if let Some((from, to_inclusive)) = self.visual_range() {
            self.register = self.slice_char_range(from, to_inclusive + 1);
        }
    }

    pub fn delete_visual_selection(&mut self) {
        let Some((from, to_inclusive)) = self.visual_range() else {
            return;
        };

        let before = self.input.chars().take(from);
        let after = self.input.chars().skip(to_inclusive + 1);
        self.input = before.chain(after).collect();

        let new_len = self.char_len();
        if new_len == 0 {
            self.cursor = 0;
        } else {
            self.cursor = from.min(new_len - 1);
        }
    }

    pub fn yank_with_motion(&mut self, motion: Motion) {
        let chars: Vec<char> = self.input.chars().collect();
        let len = chars.len();
        if len == 0 {
            self.register.clear();
            return;
        }

        let start = self.cursor.min(len - 1);

        let (from, to_exclusive) = match motion {
            Motion::WordForward => {
                // Vim-like `yw`: copy to end of current word (without trailing separator).
                let mut end = start;
                if Self::is_word_char(chars[end]) {
                    while end < len && Self::is_word_char(chars[end]) {
                        end += 1;
                    }
                } else {
                    // If cursor is on separator, copy through next word.
                    while end < len && !Self::is_word_char(chars[end]) {
                        end += 1;
                    }
                    while end < len && Self::is_word_char(chars[end]) {
                        end += 1;
                    }
                }
                (start, end.max(start + 1).min(len))
            }
            Motion::WordBackward => {
                let mut begin = start;
                if begin > 0 {
                    while begin > 0 && !Self::is_word_char(chars[begin]) {
                        begin -= 1;
                    }
                    while begin > 0 && Self::is_word_char(chars[begin - 1]) {
                        begin -= 1;
                    }
                }
                (begin, start + 1)
            }
            _ => {
                let end = self.motion_target(motion);
                let (from, to) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                (from, (to + 1).min(len))
            }
        };

        self.register = self.slice_char_range(from, to_exclusive);
    }

    pub fn paste_after(&mut self) {
        if self.register.is_empty() {
            return;
        }

        let register = self.register.clone();
        let reg_len = register.chars().count();
        let len = self.char_len();
        let insert_at = if len == 0 {
            0
        } else {
            (self.cursor + 1).min(len)
        };

        self.insert_str_at_char_index(insert_at, &register);

        if reg_len > 0 {
            // Vim-like in normal mode: cursor lands on last pasted char.
            self.cursor = insert_at + reg_len - 1;
        }

        self.clamp_cursor_for_mode();
    }

    pub fn paste_before(&mut self) {
        if self.register.is_empty() {
            return;
        }

        let register = self.register.clone();
        let reg_len = register.chars().count();
        let insert_at = self.cursor.min(self.char_len());

        self.insert_str_at_char_index(insert_at, &register);

        if reg_len > 0 {
            self.cursor = insert_at + reg_len - 1;
        }

        self.clamp_cursor_for_mode();
    }

    pub fn motion_from_key(code: KeyCode) -> Option<Motion> {
        match code {
            KeyCode::Left | KeyCode::Char('h') => Some(Motion::Left),
            KeyCode::Right | KeyCode::Char('l') => Some(Motion::Right),
            KeyCode::Char('0') => Some(Motion::LineStart),
            KeyCode::Char('$') => Some(Motion::LineEnd),
            KeyCode::Char('w') => Some(Motion::WordForward),
            KeyCode::Char('b') => Some(Motion::WordBackward),
            _ => None,
        }
    }

    /// Handles only input-editor concerns.
    /// Caller can route `EditorCommand` to app-level actions.
    pub fn handle_key_event(&mut self, key: KeyEvent) -> EditorCommand {
        match self.mode {
            InputEditMode::Insert => self.handle_insert_key(key.code),
            InputEditMode::Normal => self.handle_normal_key(key.code),
            InputEditMode::Visual => self.handle_visual_key(key.code),
        }
    }

    fn handle_insert_key(&mut self, code: KeyCode) -> EditorCommand {
        match code {
            KeyCode::Esc => {
                self.switch_to_normal_mode();
                EditorCommand::None
            }
            KeyCode::Enter => EditorCommand::Submit,
            KeyCode::Char(ch) => {
                self.enter_char(ch);
                EditorCommand::None
            }
            KeyCode::Backspace => {
                self.backspace();
                EditorCommand::None
            }
            KeyCode::Left => {
                self.move_insert_left();
                EditorCommand::None
            }
            KeyCode::Right => {
                self.move_insert_right();
                EditorCommand::None
            }
            _ => EditorCommand::None,
        }
    }

    fn handle_normal_key(&mut self, code: KeyCode) -> EditorCommand {
        match code {
            KeyCode::Esc => EditorCommand::ExitInputMode,
            KeyCode::Enter => EditorCommand::Submit,
            KeyCode::Tab => EditorCommand::IncrementFocus,
            KeyCode::BackTab => EditorCommand::DecrementFocus,

            KeyCode::Char('i') => {
                self.switch_to_insert_mode();
                EditorCommand::None
            }
            KeyCode::Char('a') => {
                let len = self.char_len();
                if len == 0 {
                    self.cursor = 0;
                } else {
                    self.cursor = (self.cursor + 1).min(len);
                }
                self.switch_to_insert_mode();
                EditorCommand::None
            }
            KeyCode::Char('I') => {
                self.cursor = 0;
                self.switch_to_insert_mode();
                EditorCommand::None
            }
            KeyCode::Char('A') => {
                self.cursor = self.char_len();
                self.switch_to_insert_mode();
                EditorCommand::None
            }

            KeyCode::Char('x') => {
                self.delete_under_cursor();
                EditorCommand::None
            }
            KeyCode::Char('v') => {
                self.switch_to_visual_mode();
                EditorCommand::None
            }
            KeyCode::Char('p') => {
                self.paste_after();
                EditorCommand::None
            }
            KeyCode::Char('P') => {
                self.paste_before();
                EditorCommand::None
            }

            other => {
                if let Some(motion) = Self::motion_from_key(other) {
                    self.apply_motion(motion);
                }
                EditorCommand::None
            }
        }
    }

    fn handle_visual_key(&mut self, code: KeyCode) -> EditorCommand {
        match code {
            KeyCode::Esc | KeyCode::Char('v') => {
                self.switch_to_normal_mode();
                EditorCommand::None
            }
            KeyCode::Enter => EditorCommand::Submit,
            KeyCode::Char('y') => {
                let yanked_range = self.visual_range();
                self.yank_visual_selection();
                self.switch_to_normal_mode();
                if let Some((start, end)) = yanked_range {
                    EditorCommand::Yanked { start, end }
                } else {
                    EditorCommand::None
                }
            }
            KeyCode::Char('d') | KeyCode::Char('x') => {
                self.delete_visual_selection();
                self.switch_to_normal_mode();
                EditorCommand::None
            }
            other => {
                if let Some(motion) = Self::motion_from_key(other) {
                    self.apply_motion(motion);
                }
                EditorCommand::None
            }
        }
    }

    fn visual_range(&self) -> Option<(usize, usize)> {
        let len = self.char_len();
        if len == 0 {
            return None;
        }

        let anchor = self.visual_anchor?.min(len - 1);
        let cursor = self.cursor.min(len - 1);

        if anchor <= cursor {
            Some((anchor, cursor))
        } else {
            Some((cursor, anchor))
        }
    }

    fn motion_target(&self, motion: Motion) -> usize {
        let chars: Vec<char> = self.input.chars().collect();
        let len = chars.len();
        if len == 0 {
            return 0;
        }

        let i = self.cursor.min(len - 1);

        match motion {
            Motion::Left => i.saturating_sub(1),
            Motion::Right => (i + 1).min(len - 1),
            Motion::LineStart => 0,
            Motion::LineEnd => len - 1,
            Motion::WordForward => {
                let mut j = i;

                if Self::is_word_char(chars[j]) {
                    while j < len && Self::is_word_char(chars[j]) {
                        j += 1;
                    }
                }
                while j < len && !Self::is_word_char(chars[j]) {
                    j += 1;
                }

                if j >= len { len - 1 } else { j }
            }
            Motion::WordBackward => {
                let mut j = i;
                if j == 0 {
                    return 0;
                }

                j -= 1;
                while j > 0 && !Self::is_word_char(chars[j]) {
                    j -= 1;
                }
                while j > 0 && Self::is_word_char(chars[j - 1]) {
                    j -= 1;
                }
                j
            }
        }
    }

    fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    fn char_len(&self) -> usize {
        self.input.chars().count()
    }

    fn clamp_cursor_for_mode(&mut self) {
        let len = self.char_len();
        self.cursor = match self.mode {
            InputEditMode::Insert => self.cursor.min(len),
            InputEditMode::Normal | InputEditMode::Visual => {
                if len == 0 {
                    0
                } else {
                    self.cursor.min(len - 1)
                }
            }
        };
    }

    fn byte_index_from_char_index(&self, char_index: usize) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(char_index)
            .unwrap_or(self.input.len())
    }

    fn insert_str_at_char_index(&mut self, char_index: usize, to_insert: &str) {
        let idx = self.byte_index_from_char_index(char_index);
        self.input.insert_str(idx, to_insert);
    }

    fn slice_char_range(&self, start_inclusive: usize, end_exclusive: usize) -> String {
        self.input
            .chars()
            .skip(start_inclusive)
            .take(end_exclusive.saturating_sub(start_inclusive))
            .collect()
    }
}
