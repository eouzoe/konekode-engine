use std::collections::BTreeSet;

use konekode_text::{char_at, len, slice, to_i64};

use crate::buffer::Rules;
use crate::path::ExpectedKind;
use crate::session::Session;
use crate::step::{known_keys, run};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "kind", rename_all = "lowercase")
)]
pub enum Hint {
    Drop {
        line: i64,
    },
    Insert {
        column: i64,
        line: i64,
        text: String,
    },
    #[cfg_attr(feature = "serde", serde(rename = "line"))]
    NewLine {
        before: i64,
        text: String,
    },
    Remove {
        from: i64,
        line: i64,
        to: i64,
    },
}

impl Hint {
    fn line(&self) -> Option<i64> {
        match self {
            Self::Drop { line } | Self::Insert { line, .. } | Self::Remove { line, .. } => {
                Some(*line)
            }
            Self::NewLine { .. } => None,
        }
    }
}

struct Gap {
    inner: i64,
    outer: i64,
    text: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OpKind {
    Add,
    Drop,
    Keep,
}

type Op = (OpKind, String);

#[derive(Default)]
struct Hunk {
    adds: Vec<String>,
    drops: Vec<i64>,
}

#[must_use]
pub fn final_text(session: &Session, rules: &Rules) -> String {
    let size = to_i64(session.path.len());
    let from = if session.at < 0 {
        (size + session.at).max(0)
    } else {
        session.at.min(size)
    };
    let start = usize::try_from(from).unwrap_or(usize::MAX);
    let keys: Vec<String> = session
        .path
        .iter()
        .skip(start)
        .filter(|expected| expected.kind != ExpectedKind::Run)
        .map(|expected| expected.key.clone())
        .collect();
    let guided = Rules {
        keys: known_keys(),
        pairs: rules.pairs.clone(),
    };
    run(&session.buffer, &keys, &guided).text
}

fn shared_start(left: &str, right: &str) -> i64 {
    let mut at = 0;
    while at < len(left) && at < len(right) && char_at(left, at) == char_at(right, at) {
        at += 1;
    }
    at
}

fn widen(gaps: &mut Vec<Gap>, gap: Gap) {
    if let Some(nearest) = gaps.first_mut()
        && nearest.inner == gap.inner
    {
        nearest.outer = gap.outer;
        nearest.text = format!("{}{}", gap.text, nearest.text);
        return;
    }
    gaps.insert(0, gap);
}

fn scan(inner: &str, outer: &str, start: i64) -> (i64, Vec<Gap>) {
    let mut gaps = Vec::new();
    let mut at = len(inner) - 1;
    let mut from = len(outer) - 1;
    while from >= start {
        let text = char_at(outer, from);
        if at >= start && char_at(inner, at) == text {
            at -= 1;
        } else {
            widen(
                &mut gaps,
                Gap {
                    inner: at + 1,
                    outer: from,
                    text: text.to_owned(),
                },
            );
        }
        from -= 1;
    }
    (at, gaps)
}

fn embed(inner: &str, outer: &str) -> Option<Vec<Gap>> {
    let start = shared_start(inner, outer);
    let (at, gaps) = scan(inner, outer, start);
    if at >= start {
        return None;
    }
    Some(gaps)
}

fn kept(ops: &[Op]) -> usize {
    ops.iter().filter(|(kind, _)| *kind == OpKind::Keep).count()
}

fn better(first: Vec<Op>, second: Vec<Op>) -> Vec<Op> {
    if kept(&second) > kept(&first) {
        return second;
    }
    first
}

fn prefixed(op: Op, mut ops: Vec<Op>) -> Vec<Op> {
    ops.insert(0, op);
    ops
}

fn align(old: &[String], next: &[String]) -> Vec<Op> {
    let Some((first, rest_old)) = old.split_first() else {
        return next
            .iter()
            .map(|text| (OpKind::Add, text.clone()))
            .collect();
    };
    let Some((head, tail)) = next.split_first() else {
        return old
            .iter()
            .map(|text| (OpKind::Drop, text.clone()))
            .collect();
    };
    if first == head {
        return prefixed((OpKind::Keep, first.clone()), align(rest_old, tail));
    }
    better(
        prefixed((OpKind::Drop, first.clone()), align(rest_old, next)),
        prefixed((OpKind::Add, head.clone()), align(old, tail)),
    )
}

fn inline(line: i64, old: &str, next: &str) -> Option<Vec<Hint>> {
    if let Some(added) = embed(old, next) {
        return Some(
            added
                .into_iter()
                .map(|gap| Hint::Insert {
                    column: gap.inner,
                    line,
                    text: gap.text,
                })
                .collect(),
        );
    }
    let removed = embed(next, old)?;
    Some(
        removed
            .into_iter()
            .map(|gap| Hint::Remove {
                from: gap.outer,
                line,
                to: gap.outer + len(&gap.text),
            })
            .collect(),
    )
}

fn replaced(line: i64, after: i64, text: &str) -> Vec<Hint> {
    vec![
        Hint::Drop { line },
        Hint::NewLine {
            before: after,
            text: text.to_owned(),
        },
    ]
}

fn settle(old: &[String], hunk: &Hunk, here: i64) -> Vec<Hint> {
    let after = hunk.drops.last().copied().unwrap_or(here - 1) + 1;
    let mut hints = Vec::new();
    for (at, line) in hunk.drops.iter().enumerate() {
        match hunk.adds.get(at) {
            None => hints.push(Hint::Drop { line: *line }),
            Some(next) => {
                let current = usize::try_from(*line)
                    .ok()
                    .and_then(|index| old.get(index))
                    .map_or("", String::as_str);
                hints.extend(
                    inline(*line, current, next).unwrap_or_else(|| replaced(*line, after, next)),
                );
            }
        }
    }
    for text in hunk.adds.iter().skip(hunk.drops.len()) {
        hints.push(Hint::NewLine {
            before: after,
            text: text.clone(),
        });
    }
    hints
}

fn compare(old: &[String], next: &[String]) -> Vec<Hint> {
    let mut hints = Vec::new();
    let mut hunk = Hunk::default();
    let mut line = 0;
    let mut ops = align(old, next);
    ops.push((OpKind::Keep, String::new()));
    for (kind, text) in ops {
        match kind {
            OpKind::Add => {
                hunk.adds.push(text);
                continue;
            }
            OpKind::Drop => hunk.drops.push(line),
            OpKind::Keep => {
                hints.extend(settle(old, &hunk, line));
                hunk = Hunk::default();
            }
        }
        line += 1;
    }
    hints
}

fn lines_still(hints: Vec<Hint>) -> Vec<Hint> {
    let dropped: BTreeSet<i64> = hints
        .iter()
        .filter_map(|hint| match hint {
            Hint::Drop { line } => Some(*line),
            _ => None,
        })
        .collect();
    hints
        .into_iter()
        .filter(|hint| match hint {
            Hint::Drop { .. } | Hint::NewLine { .. } => true,
            Hint::Insert { line, .. } | Hint::Remove { line, .. } => !dropped.contains(line),
        })
        .collect()
}

fn lines(text: &str) -> Vec<String> {
    text.split('\n').map(str::to_owned).collect()
}

#[must_use]
pub fn between(text: &str, goal: &str) -> Vec<Hint> {
    lines_still(compare(&lines(text), &lines(goal)))
}

#[must_use]
pub fn goal_of(session: &Session, rules: &Rules) -> Vec<Hint> {
    if session.path.is_empty() {
        return Vec::new();
    }
    between(&session.buffer.text, &final_text(session, rules))
}

fn rewrite(text: &str, hints: &[&Hint]) -> String {
    let mut column = len(text);
    let mut result = String::new();
    for hint in hints.iter().rev() {
        match hint {
            Hint::Insert {
                column: at,
                text: added,
                ..
            } => {
                result = format!("{added}{}{result}", slice(text, *at, column));
                column = *at;
            }
            Hint::Remove { from, to, .. } => {
                result = format!("{}{result}", slice(text, *to, column));
                column = *from;
            }
            Hint::Drop { .. } | Hint::NewLine { .. } => {}
        }
    }
    format!("{}{result}", slice(text, 0, column))
}

#[must_use]
pub fn fulfil(text: &str, hints: &[Hint]) -> String {
    let old = lines(text);
    let mut result: Vec<String> = Vec::new();
    for (index, line) in (0..=old.len()).zip(0_i64..) {
        let here: Vec<&Hint> = hints
            .iter()
            .filter(|hint| hint.line() == Some(line))
            .collect();
        for hint in hints {
            if let Hint::NewLine {
                before,
                text: added,
            } = hint
                && *before == line
            {
                result.push(added.clone());
            }
        }
        let dropped = here.iter().any(|hint| matches!(hint, Hint::Drop { .. }));
        if let Some(current) = old.get(index)
            && !dropped
        {
            let inside: Vec<&Hint> = here
                .iter()
                .copied()
                .filter(|hint| matches!(hint, Hint::Insert { .. } | Hint::Remove { .. }))
                .collect();
            result.push(rewrite(current, &inside));
        }
    }
    result.join("\n")
}
