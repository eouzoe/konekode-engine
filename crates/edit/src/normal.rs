use konekode_text::{index_of, len, slice};

use crate::buffer::{
    Buffer, FAR, Mode, first_non_blank, last_column, line_end, line_start, place, splice,
};
use crate::indent::indent_below;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Down,
    Erase,
    OpenAbove,
    OpenBelow,
    StartInsert,
    ToEnd,
    ToLast,
    ToStart,
    Up,
}

pub const PREFIXES: [&str; 2] = ["d", "f"];

fn settle(buffer: &Buffer, start: i64) -> Buffer {
    Buffer {
        auto: 0,
        cursor: (start + buffer.want).min(last_column(&buffer.text, start)),
        ..buffer.clone()
    }
}

fn up(buffer: &Buffer) -> Option<Buffer> {
    let start = line_start(&buffer.text, buffer.cursor);
    if start == 0 {
        return None;
    }
    Some(settle(buffer, line_start(&buffer.text, start - 1)))
}

fn down(buffer: &Buffer) -> Option<Buffer> {
    let end = line_end(&buffer.text, buffer.cursor);
    if end == len(&buffer.text) {
        return None;
    }
    Some(settle(buffer, end + 1))
}

fn to_start(buffer: &Buffer) -> Buffer {
    place(buffer, line_start(&buffer.text, buffer.cursor))
}

fn to_end(buffer: &Buffer) -> Buffer {
    Buffer {
        want: FAR,
        ..place(buffer, last_column(&buffer.text, buffer.cursor))
    }
}

fn to_last(buffer: &Buffer) -> Buffer {
    let start = line_start(&buffer.text, len(&buffer.text));
    place(buffer, first_non_blank(&buffer.text, start))
}

fn erase(buffer: &Buffer) -> Option<Buffer> {
    let text = &buffer.text;
    let cursor = buffer.cursor;
    if line_start(text, cursor) == line_end(text, cursor) {
        return None;
    }
    let next = splice(text, cursor, "", cursor + 1);
    let last = last_column(&next, line_start(text, cursor));
    let changed = Buffer {
        change: vec!["x".to_owned()],
        text: next,
        ..buffer.clone()
    };
    Some(place(&changed, cursor.min(last)))
}

fn start_insert(buffer: &Buffer) -> Buffer {
    Buffer {
        auto: 0,
        fresh: false,
        mode: Mode::Insert,
        typing: vec!["i".to_owned()],
        ..buffer.clone()
    }
}

fn open_below(buffer: &Buffer) -> Buffer {
    let end = line_end(&buffer.text, buffer.cursor);
    let indent = indent_below(&buffer.text, buffer.cursor);
    Buffer {
        auto: 0,
        cursor: end + 1 + len(&indent),
        fresh: true,
        mode: Mode::Insert,
        text: splice(&buffer.text, end, &format!("\n{indent}"), end),
        typing: vec!["o".to_owned()],
        ..buffer.clone()
    }
}

fn indent_above(text: &str, start: i64) -> String {
    if start == 0 {
        return String::new();
    }
    indent_below(text, start - 1)
}

fn open_above(buffer: &Buffer) -> Buffer {
    let start = line_start(&buffer.text, buffer.cursor);
    let indent = indent_above(&buffer.text, start);
    Buffer {
        auto: 0,
        cursor: start + len(&indent),
        fresh: true,
        mode: Mode::Insert,
        text: splice(&buffer.text, start, &format!("{indent}\n"), start),
        typing: vec!["O".to_owned()],
        ..buffer.clone()
    }
}

fn drop_from(text: &str, start: i64, last_end: i64) -> String {
    if last_end == len(text) {
        return slice(text, 0, (start - 1).max(0)).to_owned();
    }
    splice(text, start, "", last_end + 1)
}

#[must_use]
pub fn drop_two_lines(buffer: &Buffer) -> Option<Buffer> {
    let text = &buffer.text;
    let cursor = buffer.cursor;
    let end = line_end(text, cursor);
    if end == len(text) {
        return None;
    }
    let start = line_start(text, cursor);
    let next = drop_from(text, start, line_end(text, end + 1));
    let landed = line_start(&next, start.min(len(&next)));
    let target = first_non_blank(&next, landed);
    let changed = Buffer {
        change: vec!["d".to_owned(), "j".to_owned()],
        text: next,
        ..buffer.clone()
    };
    Some(place(&changed, target))
}

#[must_use]
pub fn find_char(buffer: &Buffer, char: &str) -> Option<Buffer> {
    let at = index_of(&buffer.text, char, buffer.cursor + 1);
    if at == -1 || at >= line_end(&buffer.text, buffer.cursor) {
        return None;
    }
    Some(place(buffer, at))
}

#[must_use]
pub fn command(key: &str) -> Option<Command> {
    let found = match key {
        "$" => Command::ToEnd,
        "0" => Command::ToStart,
        "G" => Command::ToLast,
        "O" => Command::OpenAbove,
        "i" => Command::StartInsert,
        "j" => Command::Down,
        "k" => Command::Up,
        "o" => Command::OpenBelow,
        "x" => Command::Erase,
        _ => return None,
    };
    Some(found)
}

#[must_use]
pub fn apply(command: Command, buffer: &Buffer) -> Option<Buffer> {
    match command {
        Command::Down => down(buffer),
        Command::Erase => erase(buffer),
        Command::OpenAbove => Some(open_above(buffer)),
        Command::OpenBelow => Some(open_below(buffer)),
        Command::StartInsert => Some(start_insert(buffer)),
        Command::ToEnd => Some(to_end(buffer)),
        Command::ToLast => Some(to_last(buffer)),
        Command::ToStart => Some(to_start(buffer)),
        Command::Up => up(buffer),
    }
}

#[must_use]
pub fn command_keys() -> [&'static str; 9] {
    ["$", "0", "G", "O", "i", "j", "k", "o", "x"]
}
