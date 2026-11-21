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

#[must_use]
pub fn is_js_space(char: char) -> bool {
    (char.is_whitespace() && char != '\u{85}') || char == '\u{feff}'
}

#[must_use]
pub fn js_trim(text: &str) -> &str {
    text.trim_matches(is_js_space)
}

#[must_use]
pub fn js_trim_start(text: &str) -> &str {
    text.trim_start_matches(is_js_space)
}

#[must_use]
pub fn js_trim_end(text: &str) -> &str {
    text.trim_end_matches(is_js_space)
}

#[must_use]
pub fn js_squash(text: &str) -> String {
    text.chars().filter(|char| !is_js_space(*char)).collect()
}

fn substitution(replacement: &str, matched: &str, before: &str, after: &str) -> String {
    let mut out = String::new();
    let mut chars = replacement.chars().peekable();
    while let Some(char) = chars.next() {
        if char != '$' {
            out.push(char);
            continue;
        }
        match chars.peek().copied() {
            Some('$') => {
                out.push('$');
                chars.next();
            }
            Some('&') => {
                out.push_str(matched);
                chars.next();
            }
            Some('`') => {
                out.push_str(before);
                chars.next();
            }
            Some('\'') => {
                out.push_str(after);
                chars.next();
            }
            _ => out.push('$'),
        }
    }
    out
}

#[must_use]
pub fn js_replace_all(text: &str, pattern: &str, replacement: &str) -> String {
    let mut out = String::new();
    let mut cut = 0;
    for (at, found) in text.match_indices(pattern) {
        out.push_str(text.get(cut..at).unwrap_or(""));
        out.push_str(&substitution(
            replacement,
            found,
            text.get(..at).unwrap_or(""),
            text.get(at + found.len()..).unwrap_or(""),
        ));
        cut = at + found.len();
    }
    out.push_str(text.get(cut..).unwrap_or(""));
    out
}

#[must_use]
pub fn js_sorted(lines: &[String]) -> Vec<String> {
    let mut sorted = lines.to_vec();
    sorted.sort_by(|left, right| left.encode_utf16().cmp(right.encode_utf16()));
    sorted
}
