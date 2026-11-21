use std::collections::BTreeSet;

use konekode_complete::PairItem;
use konekode_text::{char_at, index_of, is_ascii_text, last_index_of, len, rest, slice};

pub const FAR: i64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Mode {
    Insert,
    Normal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Buffer {
    pub auto: i64,
    pub change: Vec<String>,
    pub cursor: i64,
    pub fresh: bool,
    pub mode: Mode,
    pub pending: String,
    pub text: String,
    pub typing: Vec<String>,
    pub want: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rules {
    pub keys: BTreeSet<String>,
    pub pairs: Vec<PairItem>,
}

#[must_use]
pub fn line_start(text: &str, at: i64) -> i64 {
    if at == 0 {
        return 0;
    }
    last_index_of(text, "\n", at - 1) + 1
}

#[must_use]
pub fn line_end(text: &str, at: i64) -> i64 {
    let end = index_of(text, "\n", at);
    if end == -1 { len(text) } else { end }
}

#[must_use]
pub fn line_at(text: &str, at: i64) -> &str {
    slice(text, line_start(text, at), line_end(text, at))
}

#[must_use]
pub fn column_of(text: &str, at: i64) -> i64 {
    at - line_start(text, at)
}

#[must_use]
pub fn last_column(text: &str, at: i64) -> i64 {
    line_start(text, at).max(line_end(text, at) - 1)
}

#[must_use]
pub fn first_non_blank(text: &str, start: i64) -> i64 {
    let mut at = start;
    while char_at(text, at) == " " {
        at += 1;
    }
    at.min(last_column(text, start))
}

#[must_use]
pub fn is_plain(key: &str) -> bool {
    matches!(key.as_bytes(), [byte] if (b' '..=b'~').contains(byte))
}

#[must_use]
pub fn splice(text: &str, from: i64, insert: &str, to: i64) -> String {
    format!("{}{insert}{}", slice(text, 0, from), rest(text, to))
}

#[must_use]
pub fn place(buffer: &Buffer, cursor: i64) -> Buffer {
    Buffer {
        auto: 0,
        cursor,
        fresh: false,
        want: column_of(&buffer.text, cursor),
        ..buffer.clone()
    }
}

#[must_use]
pub fn create_buffer(text: &str, cursor: i64) -> Option<Buffer> {
    if !is_ascii_text(text) {
        return None;
    }
    Some(Buffer {
        auto: 0,
        change: Vec::new(),
        cursor,
        fresh: false,
        mode: Mode::Normal,
        pending: String::new(),
        text: text.to_owned(),
        typing: Vec::new(),
        want: column_of(text, cursor),
    })
}
