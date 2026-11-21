use konekode_complete::{PairItem, Where, is_include_line};
use konekode_text::{char_at, len, rest, slice};

use crate::buffer::{
    Buffer, Mode, Rules, column_of, is_plain, line_at, line_end, line_start, splice,
};
use crate::indent::indent_split;

const ROOM_AFTER: [&str; 9] = [")", "]", "}", ":", ";", ">", " ", "\n", ""];

fn applies(pair: &PairItem, buffer: &Buffer) -> bool {
    pair.place == Where::Anywhere || is_include_line(line_at(&buffer.text, buffer.cursor))
}

fn pair_for<'a>(buffer: &Buffer, key: &str, rules: &'a Rules) -> Option<&'a PairItem> {
    if !ROOM_AFTER.contains(&char_at(&buffer.text, buffer.cursor)) {
        return None;
    }
    rules
        .pairs
        .iter()
        .find(|pair| pair.open == key && applies(pair, buffer))
}

fn skips(buffer: &Buffer, key: &str, rules: &Rules) -> bool {
    buffer.auto > 0
        && char_at(&buffer.text, buffer.cursor) == key
        && rules.pairs.iter().any(|pair| pair.close == key)
}

fn with_key(keys: &[String], key: &str) -> Vec<String> {
    let mut all = keys.to_vec();
    all.push(key.to_owned());
    all
}

fn type_char(buffer: &Buffer, key: &str, rules: &Rules) -> Buffer {
    let typing = with_key(&buffer.typing, key);
    if skips(buffer, key, rules) {
        return Buffer {
            auto: buffer.auto - 1,
            cursor: buffer.cursor + 1,
            fresh: false,
            typing,
            ..buffer.clone()
        };
    }
    let closer = pair_for(buffer, key, rules).map_or("", |pair| pair.close.as_str());
    let insert = format!("{key}{closer}");
    Buffer {
        auto: buffer.auto + len(closer),
        cursor: buffer.cursor + 1,
        fresh: false,
        text: splice(&buffer.text, buffer.cursor, &insert, buffer.cursor),
        typing,
        ..buffer.clone()
    }
}

fn tidy(buffer: &Buffer) -> Buffer {
    let start = line_start(&buffer.text, buffer.cursor);
    let end = line_end(&buffer.text, buffer.cursor);
    if !buffer.fresh || !slice(&buffer.text, start, end).trim().is_empty() {
        return buffer.clone();
    }
    Buffer {
        cursor: start,
        text: splice(&buffer.text, start, "", end),
        ..buffer.clone()
    }
}

fn break_line(buffer: &Buffer) -> Buffer {
    let tidied = tidy(buffer);
    let text = &tidied.text;
    let cursor = tidied.cursor;
    let end = line_end(text, cursor);
    let head = slice(text, 0, cursor);
    let tail = slice(text, cursor, end).trim_start();
    let indent = indent_split(head, tail);
    Buffer {
        auto: 0,
        cursor: len(head) + 1 + len(&indent),
        fresh: true,
        text: format!("{head}\n{indent}{tail}{}", rest(text, end)),
        typing: with_key(&buffer.typing, "Enter"),
        ..buffer.clone()
    }
}

fn is_paired(buffer: &Buffer, rules: &Rules) -> bool {
    let before = char_at(&buffer.text, buffer.cursor - 1);
    let after = char_at(&buffer.text, buffer.cursor);
    buffer.auto > 0
        && rules
            .pairs
            .iter()
            .any(|pair| pair.open == before && pair.close == after)
}

fn rub_out(buffer: &Buffer, rules: &Rules) -> Option<Buffer> {
    let cursor = buffer.cursor;
    if cursor == 0 {
        return None;
    }
    let paired = i64::from(is_paired(buffer, rules));
    Some(Buffer {
        auto: buffer.auto - paired,
        cursor: cursor - 1,
        text: splice(&buffer.text, cursor - 1, "", cursor + paired),
        typing: with_key(&buffer.typing, "Backspace"),
        ..buffer.clone()
    })
}

fn leave(buffer: &Buffer) -> Buffer {
    let tidied = tidy(buffer);
    let cursor = line_start(&tidied.text, tidied.cursor).max(tidied.cursor - 1);
    let want = column_of(&tidied.text, cursor);
    Buffer {
        auto: 0,
        change: with_key(&buffer.typing, "Escape"),
        cursor,
        fresh: false,
        mode: Mode::Normal,
        typing: Vec::new(),
        want,
        ..tidied
    }
}

#[must_use]
pub fn insert_key(buffer: &Buffer, key: &str, rules: &Rules) -> Option<Buffer> {
    match key {
        "Backspace" => rub_out(buffer, rules),
        "Enter" => Some(break_line(buffer)),
        "Escape" => Some(leave(buffer)),
        _ if is_plain(key) => Some(type_char(buffer, key, rules)),
        _ => None,
    }
}
