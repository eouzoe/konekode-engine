use std::collections::BTreeSet;

use crate::buffer::{Buffer, Mode, Rules, is_plain};
use crate::insert::insert_key;
use crate::normal::{PREFIXES, apply, command, command_keys, drop_two_lines, find_char};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Outcome {
    pub buffer: Buffer,
    pub refused: bool,
}

fn continued(buffer: &Buffer, key: &str, rules: &Rules) -> Option<Buffer> {
    let ready = Buffer {
        pending: String::new(),
        ..buffer.clone()
    };
    if key == "Escape" {
        return Some(ready);
    }
    if buffer.pending == "d" && key == "j" && rules.keys.contains(key) {
        return drop_two_lines(&ready);
    }
    if buffer.pending == "f" && is_plain(key) {
        return find_char(&ready, key);
    }
    None
}

fn normal_key(buffer: &Buffer, key: &str, rules: &Rules) -> Option<Buffer> {
    if !buffer.pending.is_empty() {
        return continued(buffer, key, rules);
    }
    if key == "Escape" {
        return Some(buffer.clone());
    }
    if !rules.keys.contains(key) {
        return None;
    }
    if PREFIXES.contains(&key) {
        return Some(Buffer {
            pending: key.to_owned(),
            ..buffer.clone()
        });
    }
    command(key).and_then(|found| apply(found, buffer))
}

fn basic(buffer: &Buffer, key: &str, rules: &Rules) -> Option<Buffer> {
    if buffer.mode == Mode::Insert {
        return insert_key(buffer, key, rules);
    }
    normal_key(buffer, key, rules)
}

fn cleared(buffer: &Buffer) -> Buffer {
    if buffer.pending.is_empty() {
        return buffer.clone();
    }
    Buffer {
        pending: String::new(),
        ..buffer.clone()
    }
}

fn repeat(buffer: &Buffer, rules: &Rules) -> Option<Buffer> {
    if buffer.change.is_empty() {
        return None;
    }
    let mut now = buffer.clone();
    for key in &buffer.change {
        now = basic(&now, key, rules).unwrap_or_else(|| cleared(&now));
    }
    Some(now)
}

fn repeats(buffer: &Buffer, key: &str, rules: &Rules) -> bool {
    key == "."
        && buffer.mode == Mode::Normal
        && buffer.pending.is_empty()
        && rules.keys.contains(key)
}

fn advance(buffer: &Buffer, key: &str, rules: &Rules) -> Option<Buffer> {
    if repeats(buffer, key, rules) {
        return repeat(buffer, rules);
    }
    basic(buffer, key, rules)
}

#[must_use]
pub fn known_keys() -> BTreeSet<String> {
    command_keys()
        .iter()
        .chain(PREFIXES.iter())
        .chain(std::iter::once(&"."))
        .map(|key| (*key).to_owned())
        .collect()
}

#[must_use]
pub fn step(buffer: &Buffer, key: &str, rules: &Rules) -> Outcome {
    advance(buffer, key, rules).map_or_else(
        || Outcome {
            buffer: cleared(buffer),
            refused: true,
        },
        |next| Outcome {
            buffer: next,
            refused: false,
        },
    )
}

#[must_use]
pub fn run(buffer: &Buffer, keys: &[String], rules: &Rules) -> Buffer {
    let mut now = buffer.clone();
    for key in keys {
        now = step(&now, key, rules).buffer;
    }
    now
}
