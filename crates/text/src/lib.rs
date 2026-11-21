#[must_use]
pub fn to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[must_use]
pub fn len(text: &str) -> i64 {
    to_i64(text.len())
}

#[must_use]
pub fn units(text: &str) -> i64 {
    to_i64(text.encode_utf16().count())
}

#[must_use]
pub fn is_ascii_text(text: &str) -> bool {
    text.is_ascii()
}

fn clamp(index: i64, text: &str) -> usize {
    usize::try_from(index.clamp(0, len(text))).unwrap_or(0)
}

fn bound(index: i64, size: i64) -> i64 {
    if index < 0 {
        (size + index).max(0)
    } else {
        index.min(size)
    }
}

#[must_use]
pub fn char_at(text: &str, index: i64) -> &str {
    if index < 0 || index >= len(text) {
        return "";
    }
    let at = clamp(index, text);
    text.get(at..=at).unwrap_or("")
}

#[must_use]
pub fn slice(text: &str, start: i64, end: i64) -> &str {
    let size = len(text);
    let from = bound(start, size);
    let to = bound(end, size);
    if from >= to {
        return "";
    }
    text.get(clamp(from, text)..clamp(to, text)).unwrap_or("")
}

#[must_use]
pub fn rest(text: &str, start: i64) -> &str {
    slice(text, start, len(text))
}

#[must_use]
pub fn index_of(text: &str, needle: &str, from: i64) -> i64 {
    let start = clamp(from, text);
    if needle.is_empty() {
        return to_i64(start);
    }
    text.get(start..)
        .and_then(|after| after.find(needle))
        .map_or(-1, |found| to_i64(start + found))
}

#[must_use]
pub fn last_index_of(text: &str, needle: &str, from: i64) -> i64 {
    let size = len(text);
    let mut at = from.clamp(0, size).min(size - len(needle));
    while at >= 0 {
        if slice(text, at, at + len(needle)) == needle {
            return at;
        }
        at -= 1;
    }
    -1
}

#[must_use]
pub fn count(text: &str, needle: &str) -> i64 {
    to_i64(text.matches(needle).count())
}
