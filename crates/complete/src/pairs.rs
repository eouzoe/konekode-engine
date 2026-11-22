use std::collections::BTreeSet;

use konekode_text::{char_at, index_of, last_index_of, len, slice};

use crate::{PairItem, Where};

struct Open {
    closer: String,
    index: i64,
}

struct Scan<'a> {
    found: Vec<(i64, i64)>,
    open: Vec<Open>,
    pairs: &'a [PairItem],
    text: &'a str,
}

fn line_end(text: &str, index: i64) -> i64 {
    let end = index_of(text, "\n", index);
    if end == -1 { len(text) } else { end }
}

fn line_around(text: &str, index: i64) -> &str {
    slice(
        text,
        last_index_of(text, "\n", index - 1) + 1,
        line_end(text, index),
    )
}

const fn is_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

const fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[must_use]
pub fn is_include_line(line: &str) -> bool {
    let bytes = line.as_bytes();
    let start = bytes
        .iter()
        .position(|byte| !is_space(*byte))
        .unwrap_or(bytes.len());
    let Some(after) = bytes.get(start..) else {
        return false;
    };
    let word = b"#include";
    after.starts_with(word) && after.get(word.len()).is_none_or(|next| !is_word(*next))
}

fn line_closer(scan: &Scan<'_>, index: i64) -> Option<i64> {
    let char = char_at(scan.text, index);
    let pair = scan
        .pairs
        .iter()
        .find(|item| item.place == Where::IncludeLine && item.open == char)?;
    if !is_include_line(line_around(scan.text, index)) {
        return None;
    }
    let closer = index_of(scan.text, &pair.close, index);
    if closer == -1 || closer > line_end(scan.text, index) {
        return None;
    }
    Some(closer)
}

fn open_bracket(scan: &mut Scan<'_>, index: i64) -> bool {
    let char = char_at(scan.text, index);
    let Some(pair) = scan
        .pairs
        .iter()
        .find(|item| item.place == Where::Anywhere && item.open == char && item.open != item.close)
    else {
        return false;
    };
    scan.open.push(Open {
        closer: pair.close.clone(),
        index,
    });
    true
}

fn close_bracket(scan: &mut Scan<'_>, index: i64) -> bool {
    let Some(last) = scan.open.last() else {
        return false;
    };
    if char_at(scan.text, index) != last.closer {
        return false;
    }
    let opener = last.index;
    scan.open.pop();
    scan.found.push((opener, index));
    true
}

fn visit(scan: &mut Scan<'_>, index: i64) {
    if open_bracket(scan, index) || close_bracket(scan, index) {
        return;
    }
    if let Some(closer) = line_closer(scan, index) {
        scan.found.push((index, closer));
    }
}

#[must_use]
pub fn match_pairs(text: &str, pairs: &[PairItem]) -> Vec<(i64, i64)> {
    let mut scan = Scan {
        found: Vec::new(),
        open: Vec::new(),
        pairs,
        text,
    };
    for index in 0..len(text) {
        visit(&mut scan, index);
    }
    scan.found
}

#[must_use]
pub fn auto_closed(text: &str, position: i64, pairs: &[PairItem]) -> BTreeSet<i64> {
    match_pairs(text, pairs)
        .into_iter()
        .filter(|(opener, closer)| *opener < position && *closer >= position)
        .map(|(_, closer)| closer)
        .collect()
}
