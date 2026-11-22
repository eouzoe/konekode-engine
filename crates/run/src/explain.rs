use std::sync::LazyLock;

use konekode_text::js_trim;
use regex::{Captures, Regex};

use crate::say::{Missing, Text, say};

#[expect(
    clippy::expect_used,
    reason = "the pattern is a literal that the tests compile"
)]
static HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"main\.cpp:(?P<line>[0-9]+):[0-9]+: (?:fatal )?error: (?P<text>[^\n\r\x{2028}\x{2029}]+)",
    )
    .expect("the header pattern is valid")
});

#[expect(
    clippy::expect_used,
    reason = "the pattern is a literal that the tests compile"
)]
static ANOTHER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:\S[^\n\r\x{2028}\x{2029}]*:[0-9]+:[0-9]+: (?:fatal error|error|warning|note): |fatal error: |[0-9]+ (?:error|warning)s? )",
    )
    .expect("the pattern for the next message is valid")
});

#[derive(Clone, Copy)]
enum Shape {
    Bare,
    Line,
    LineName,
    GuessLineName,
}

const RULES: [(&str, &str, Shape); 10] = [
    (r"^expected ';'", "explain.semicolon", Shape::Line),
    (
        r"^use of undeclared identifier '(?P<name>[^']+)'; did you mean '(?P<guess>[^']+)'",
        "explain.undeclaredGuess",
        Shape::GuessLineName,
    ),
    (
        r"^use of undeclared identifier '(?P<name>[^']+)'",
        "explain.undeclared",
        Shape::LineName,
    ),
    (
        r"^unknown type name '(?P<name>[^']+)'",
        "explain.unknownType",
        Shape::LineName,
    ),
    (
        r"^invalid operands to binary expression",
        "explain.operands",
        Shape::Line,
    ),
    (r"^expected '\}'", "explain.missingBrace", Shape::Bare),
    (
        r"^extraneous closing brace",
        "explain.extraBrace",
        Shape::Line,
    ),
    (r"^expected expression", "explain.expression", Shape::Line),
    (r"^missing terminating", "explain.quote", Shape::Line),
    (r"^expected '\)'", "explain.paren", Shape::Line),
];

#[expect(
    clippy::expect_used,
    reason = "the patterns are literals that the tests compile"
)]
static COMPILED: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    RULES
        .iter()
        .map(|(pattern, _, _)| Regex::new(pattern).expect("every rule pattern is valid"))
        .collect()
});

const BLOCK_LINES: usize = 3;
const BRIEF_LINES: usize = 6;

struct Problem {
    line: String,
    text: String,
}

fn group<'a>(found: &Captures<'a>, name: &str) -> &'a str {
    found.name(name).map_or("", |item| item.as_str())
}

fn find(message: &str) -> Option<Problem> {
    let found = HEADER.captures(message)?;
    Some(Problem {
        line: group(&found, "line").to_owned(),
        text: group(&found, "text").to_owned(),
    })
}

fn tell(
    text: &Text,
    key: &str,
    shape: Shape,
    line: &str,
    found: &Captures<'_>,
) -> Result<String, Missing> {
    let name = group(found, "name");
    match shape {
        Shape::Bare => say(text, key, &[]),
        Shape::Line => say(text, key, &[("line", line)]),
        Shape::LineName => say(text, key, &[("line", line), ("name", name)]),
        Shape::GuessLineName => say(
            text,
            key,
            &[
                ("guess", group(found, "guess")),
                ("line", line),
                ("name", name),
            ],
        ),
    }
}

fn translate(text: &Text, problem: &Problem) -> Result<String, Missing> {
    for ((_, key, shape), pattern) in RULES.iter().zip(COMPILED.iter()) {
        if let Some(found) = pattern.captures(&problem.text) {
            return tell(text, key, *shape, &problem.line, &found);
        }
    }
    say(text, "explain.other", &[("line", &problem.line)])
}

fn own_lines<'a>(rest: &[&'a str]) -> Vec<&'a str> {
    rest.iter()
        .take_while(|line| !ANOTHER.is_match(line))
        .copied()
        .collect()
}

fn first_block(message: &str) -> String {
    let lines: Vec<&str> = message.split('\n').collect();
    let from = lines
        .iter()
        .position(|line| HEADER.is_match(line))
        .unwrap_or_else(|| lines.len().saturating_sub(1));
    let tail = lines.get(from..).unwrap_or(&[]);
    let (head, rest) = tail
        .split_first()
        .map_or(("", &[][..]), |(head, rest)| (*head, rest));
    std::iter::once(head)
        .chain(own_lines(rest))
        .filter(|line| !js_trim(line).is_empty())
        .take(BLOCK_LINES)
        .collect::<Vec<_>>()
        .join("\n")
}

fn brief(message: &str) -> String {
    js_trim(message)
        .split('\n')
        .take(BRIEF_LINES)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn explain(text: &Text, message: &str) -> Result<String, Missing> {
    let Some(problem) = find(message) else {
        return Ok(brief(message));
    };
    Ok(format!(
        "{}\n\n{}\n{}",
        translate(text, &problem)?,
        say(text, "explain.original", &[])?,
        first_block(message)
    ))
}
