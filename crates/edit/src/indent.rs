use konekode_text::{char_at, count, len, slice};

use crate::buffer::{line_at, line_start};

const INDENT_UNIT: &str = "    ";
const CLOSERS: [&str; 4] = [";", "{", "}", ":"];

fn leading(line: &str) -> &str {
    let kept = len(line.trim_start());
    slice(line, 0, len(line) - kept)
}

fn nearest_above(text: &str, from: i64) -> Option<i64> {
    let mut start = from;
    while line_at(text, start).trim().is_empty() {
        if start == 0 {
            return None;
        }
        start = line_start(text, start - 1);
    }
    Some(start)
}

fn previous(text: &str, start: i64) -> Option<i64> {
    if start == 0 {
        return None;
    }
    nearest_above(text, line_start(text, start - 1))
}

fn is_open(line: &str) -> bool {
    let trimmed = line.trim();
    let closed = CLOSERS.iter().any(|closer| trimmed.ends_with(closer));
    !trimmed.is_empty() && !trimmed.starts_with('#') && !closed
}

fn statement_start(text: &str, start: i64) -> i64 {
    let mut now = start;
    let mut above = previous(text, now);
    while let Some(line) = above {
        if !is_open(line_at(text, line)) {
            break;
        }
        now = line;
        above = previous(text, now);
    }
    now
}

fn is_inside(text: &str, end: i64) -> bool {
    let before = slice(text, 0, end);
    count(before, "{") > count(before, "}")
}

fn continues(text: &str, anchor: i64) -> bool {
    let line = line_at(text, anchor);
    if !is_open(line) || statement_start(text, anchor) != anchor {
        return false;
    }
    is_inside(text, anchor) || line.trim_end().ends_with(')')
}

fn deepens(text: &str, anchor: i64) -> bool {
    line_at(text, anchor).trim_end().ends_with('{') || continues(text, anchor)
}

fn base_of(text: &str, anchor: i64) -> &str {
    let line = line_at(text, anchor);
    if is_open(line) {
        return leading(line);
    }
    leading(line_at(text, statement_start(text, anchor)))
}

#[must_use]
pub fn indent_below(text: &str, at: i64) -> String {
    let Some(anchor) = nearest_above(text, line_start(text, at)) else {
        return String::new();
    };
    let base = base_of(text, anchor);
    if deepens(text, anchor) {
        return format!("{base}{INDENT_UNIT}");
    }
    base.to_owned()
}

fn opens_paren(line: &str) -> bool {
    count(line, "(") > count(line, ")")
}

fn matching_brace(text: &str) -> Option<i64> {
    let mut depth = 0;
    let mut at = len(text) - 1;
    while at >= 0 {
        let char = char_at(text, at);
        if char == "{" && depth == 0 {
            return Some(at);
        }
        depth += i64::from(char == "}") - i64::from(char == "{");
        at -= 1;
    }
    None
}

fn closing(head: &str) -> String {
    let Some(brace) = matching_brace(head) else {
        return String::new();
    };
    let opener = statement_start(head, line_start(head, brace));
    leading(line_at(head, opener)).to_owned()
}

fn by_tail(head: &str, tail: &str, anchor: i64) -> String {
    if tail.starts_with('}') {
        return closing(head);
    }
    if tail.starts_with('{') {
        return base_of(head, anchor).to_owned();
    }
    indent_below(head, len(head))
}

#[must_use]
pub fn indent_split(head: &str, tail: &str) -> String {
    let Some(anchor) = nearest_above(head, line_start(head, len(head))) else {
        return String::new();
    };
    let line = line_at(head, anchor);
    if opens_paren(line) {
        return format!("{}{INDENT_UNIT}{INDENT_UNIT}", leading(line));
    }
    by_tail(head, tail, anchor)
}
