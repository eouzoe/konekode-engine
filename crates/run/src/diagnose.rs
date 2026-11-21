use konekode_text::{js_sorted, js_squash, js_trim};

use crate::say::{Missing, Text, line, say};

fn lines_of(text: &str) -> Vec<String> {
    js_trim(text).split('\n').map(str::to_owned).collect()
}

fn same_lines(want: &[String], got: &[String]) -> bool {
    js_sorted(want).join("\n") == js_sorted(got).join("\n")
}

fn first_difference(text: &Text, want: &[String], got: &[String]) -> Result<String, Missing> {
    let at = want
        .iter()
        .enumerate()
        .position(|(index, line)| got.get(index) != Some(line));
    let shown = at.map_or_else(|| "0".to_owned(), |index| (index + 1).to_string());
    let pick = |lines: &[String]| {
        at.and_then(|index| lines.get(index))
            .cloned()
            .unwrap_or_default()
    };
    say(
        text,
        "wrong.line",
        &[("got", &pick(got)), ("line", &shown), ("want", &pick(want))],
    )
}

fn counted(text: &Text, want: &[String], got: &[String]) -> Result<String, Missing> {
    let have = got.len().to_string();
    let need = want.len().to_string();
    let values = [("got", have.as_str()), ("want", need.as_str())];
    if got.len() < want.len() {
        return say(text, "wrong.fewer", &values);
    }
    if got.len() > want.len() {
        return say(text, "wrong.more", &values);
    }
    first_difference(text, want, got)
}

pub fn diagnose(text: &Text, expected: &str, actual: &str) -> Result<String, Missing> {
    let want = lines_of(expected);
    let got = lines_of(actual);
    if js_trim(actual).is_empty() {
        return line(text, "wrong.nothing");
    }
    if js_squash(actual) == js_squash(expected) {
        return line(text, "wrong.spacing");
    }
    if same_lines(&want, &got) {
        return line(text, "wrong.order");
    }
    counted(text, &want, &got)
}
