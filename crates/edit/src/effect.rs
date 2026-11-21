use konekode_text::{last_index_of, slice, to_i64, units};

use crate::buffer::{Buffer, Mode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum EffectKind {
    Columns,
    Dropped,
    Lines,
    Repeat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Effect {
    pub count: i64,
    pub keys: i64,
    pub kind: EffectKind,
}

const WORTH_TELLING: i64 = 2;

fn lines_in(text: &str) -> i64 {
    to_i64(text.split('\n').count())
}

fn line_of(buffer: &Buffer) -> i64 {
    lines_in(slice(&buffer.text, 0, buffer.cursor))
}

fn column_of(buffer: &Buffer) -> i64 {
    buffer.cursor - (last_index_of(&buffer.text, "\n", buffer.cursor - 1) + 1)
}

fn moved(before: &Buffer, after: &Buffer, keys: i64) -> Option<Effect> {
    let lines = (line_of(after) - line_of(before)).abs();
    let columns = (column_of(after) - column_of(before)).abs();
    if lines >= WORTH_TELLING {
        return Some(Effect {
            count: lines,
            keys,
            kind: EffectKind::Lines,
        });
    }
    if lines == 0 && columns >= WORTH_TELLING {
        return Some(Effect {
            count: columns,
            keys,
            kind: EffectKind::Columns,
        });
    }
    None
}

fn changed(before: &Buffer, after: &Buffer, key: &str) -> Option<Effect> {
    let keys = units(&before.pending) + 1;
    if key == "." && before.pending.is_empty() {
        return Some(Effect {
            count: to_i64(before.change.len()),
            keys,
            kind: EffectKind::Repeat,
        });
    }
    let dropped = lines_in(&before.text) - lines_in(&after.text);
    if dropped > 0 {
        return Some(Effect {
            count: dropped,
            keys,
            kind: EffectKind::Dropped,
        });
    }
    None
}

#[must_use]
pub fn effect_of(before: &Buffer, after: &Buffer, key: &str) -> Option<Effect> {
    if before.mode != Mode::Normal || after.mode != Mode::Normal {
        return None;
    }
    if before.text == after.text {
        return moved(before, after, units(&before.pending) + 1);
    }
    changed(before, after, key)
}
