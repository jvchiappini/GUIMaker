//! Text/number editing state shared by every editable widget.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextFilter {
    Any,
    Numeric,
}

impl TextFilter {
    pub fn accepts(self, c: char) -> bool {
        match self {
            TextFilter::Any => !c.is_control(),
            TextFilter::Numeric => c.is_ascii_digit() || c == '.' || c == ',' || c == '-',
        }
    }

    pub fn sanitize(self, s: &str) -> String {
        match self {
            TextFilter::Any => s.chars().filter(|c| !c.is_control()).collect(),
            TextFilter::Numeric => {
                let mut out = String::new();
                let mut dot = false;
                for c in s.chars() {
                    if c.is_ascii_digit() {
                        out.push(c);
                    } else if (c == '.' || c == ',') && !dot {
                        out.push('.');
                        dot = true;
                    } else if c == '-' && out.is_empty() {
                        out.push('-');
                    }
                }
                out
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextEvent {
    pub changed: bool,
    pub submitted: bool,
    pub cancelled: bool,
    pub clicked: bool,
}

const UNDO_LIMIT: usize = 128;
const PASTE_TIMEOUT: u32 = 180;

#[derive(Clone, Debug, Default)]
pub struct TextEditState {
    text: String,
    seed: String,
    caret: usize,
    anchor: usize,
    pub(crate) scroll: f32,
    pub(crate) dragging: bool,
    pub(crate) pending_paste: bool,
    paste_wait: u32,
    undo: Vec<(String, usize, usize)>,
    redo: Vec<(String, usize, usize)>,
}

impl TextEditState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn seed(&self) -> &str {
        &self.seed
    }

    pub fn set_text(&mut self, value: impl Into<String>, select_all: bool) {
        self.text = value.into();
        self.seed = self.text.clone();
        let n = self.char_len();
        self.undo.clear();
        self.redo.clear();
        self.scroll = 0.0;
        self.dragging = false;
        self.pending_paste = false;
        self.paste_wait = 0;
        if select_all {
            self.anchor = 0;
            self.caret = n;
        } else {
            self.anchor = n;
            self.caret = n;
        }
    }

    pub fn restore_seed(&mut self) {
        self.text = self.seed.clone();
        let n = self.char_len();
        self.caret = n;
        self.anchor = n;
    }

    pub fn char_len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn caret(&self) -> usize {
        self.caret
    }

    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.caret = self.char_len();
    }

    pub fn has_selection(&self) -> bool {
        self.anchor != self.caret
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        if self.anchor == self.caret {
            None
        } else if self.anchor < self.caret {
            Some((self.anchor, self.caret))
        } else {
            Some((self.caret, self.anchor))
        }
    }

    pub(crate) fn byte(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.text.len())
    }

    pub(crate) fn slice(&self, a: usize, b: usize) -> String {
        self.text.chars().skip(a).take(b.saturating_sub(a)).collect()
    }

    fn snapshot(&mut self) {
        self.undo.push((self.text.clone(), self.caret, self.anchor));
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub(crate) fn delete_selection(&mut self) -> bool {
        if let Some((a, b)) = self.selection() {
            let (ba, bb) = (self.byte(a), self.byte(b));
            self.text.replace_range(ba..bb, "");
            self.caret = a;
            self.anchor = a;
            true
        } else {
            false
        }
    }

    pub(crate) fn apply_insert(&mut self, raw: &str, filter: TextFilter) -> bool {
        let mut ins = String::new();
        for c in raw.chars() {
            if filter.accepts(c) {
                ins.push(c);
            }
        }
        let had_sel = self.has_selection();
        if ins.is_empty() && !had_sel {
            return false;
        }
        self.snapshot();
        self.delete_selection();
        let b = self.byte(self.caret);
        self.text.insert_str(b, &ins);
        self.caret += ins.chars().count();
        self.anchor = self.caret;
        if filter == TextFilter::Numeric {
            let clean = filter.sanitize(&self.text);
            if clean != self.text {
                self.text = clean;
                self.caret = self.caret.min(self.char_len());
                self.anchor = self.caret;
            }
        }
        true
    }

    pub(crate) fn delete_backward(&mut self, by_word: bool) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.caret == 0 {
            return false;
        }
        let from = if by_word { self.prev_word(self.caret) } else { self.caret - 1 };
        self.snapshot();
        let (bf, bc) = (self.byte(from), self.byte(self.caret));
        self.text.replace_range(bf..bc, "");
        self.caret = from;
        self.anchor = from;
        true
    }

    pub(crate) fn delete_forward(&mut self, by_word: bool) -> bool {
        if self.delete_selection() {
            return true;
        }
        let n = self.char_len();
        if self.caret >= n {
            return false;
        }
        let to = if by_word { self.next_word(self.caret) } else { self.caret + 1 };
        self.snapshot();
        let (bc, bt) = (self.byte(self.caret), self.byte(to));
        self.text.replace_range(bc..bt, "");
        true
    }

    pub fn move_caret(&mut self, to: usize, extend: bool) {
        self.caret = to.min(self.char_len());
        if !extend {
            self.anchor = self.caret;
        }
    }

    fn chars(&self) -> Vec<char> {
        self.text.chars().collect()
    }

    pub(crate) fn prev_word(&self, from: usize) -> usize {
        let cs = self.chars();
        let mut i = from.min(cs.len());
        while i > 0 && !cs[i - 1].is_alphanumeric() {
            i -= 1;
        }
        while i > 0 && cs[i - 1].is_alphanumeric() {
            i -= 1;
        }
        i
    }

    pub(crate) fn next_word(&self, from: usize) -> usize {
        let cs = self.chars();
        let n = cs.len();
        let mut i = from.min(n);
        while i < n && cs[i].is_alphanumeric() {
            i += 1;
        }
        while i < n && !cs[i].is_alphanumeric() {
            i += 1;
        }
        i
    }

    pub fn undo(&mut self) -> bool {
        if let Some((t, c, a)) = self.undo.pop() {
            self.redo.push((std::mem::take(&mut self.text), self.caret, self.anchor));
            self.text = t;
            self.caret = c;
            self.anchor = a;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some((t, c, a)) = self.redo.pop() {
            self.undo.push((std::mem::take(&mut self.text), self.caret, self.anchor));
            self.text = t;
            self.caret = c;
            self.anchor = a;
            true
        } else {
            false
        }
    }

    pub(crate) fn index_at_x(&self, x: f32, font_size: f32, measure: &mut dyn FnMut(&str) -> f32) -> usize {
        let mut acc = 0.0;
        for (i, c) in self.text.chars().enumerate() {
            let cw = measure(&c.to_string());
            let _ = font_size;
            if x < acc + cw * 0.5 {
                return i;
            }
            acc += cw;
        }
        self.char_len()
    }

    pub(crate) fn x_at_char(&self, idx: usize, font_size: f32, measure: &mut dyn FnMut(&str) -> f32) -> f32 {
        let _ = font_size;
        measure(&self.slice(0, idx))
    }

    pub(crate) fn tick_paste(&mut self) {
        if !self.pending_paste {
            return;
        }
        self.paste_wait += 1;
        if self.paste_wait > PASTE_TIMEOUT {
            self.pending_paste = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_sanitize_keeps_one_separator_and_leading_sign() {
        assert_eq!(TextFilter::Numeric.sanitize("1a2,3.4.5"), "12.345");
        assert_eq!(TextFilter::Numeric.sanitize("-5.0"), "-5.0");
        assert_eq!(TextFilter::Numeric.sanitize("5-0"), "50");
    }

    #[test]
    fn insert_replaces_selection_and_tracks_caret() {
        let mut e = TextEditState::new();
        e.set_text("hello", false);
        e.select_all();
        assert!(e.apply_insert("ñ", TextFilter::Any));
        assert_eq!(e.text(), "ñ");
        assert_eq!(e.caret, 1);
        assert!(!e.has_selection());
    }

    #[test]
    fn escape_restores_the_seed() {
        let mut e = TextEditState::new();
        e.set_text("12.5", true);
        assert!(e.apply_insert("3", TextFilter::Numeric));
        e.restore_seed();
        assert_eq!(e.text(), "12.5");
    }
}
