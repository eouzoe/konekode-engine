use konekode_text::to_i64;

use crate::buffer::{Buffer, Mode, Rules};
use crate::path::{Cause, Expected, ExpectedKind, Slip, cause_of};
use crate::step::{known_keys, step};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Session {
    pub at: i64,
    pub buffer: Buffer,
    pub path: Vec<Expected>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Verdict {
    Moved,
    Refused,
    Right,
    Wrong,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pressed {
    pub at: i64,
    pub cause: Cause,
    pub expected: Option<Expected>,
    pub key: String,
    pub runs: bool,
    pub session: Session,
    pub verdict: Verdict,
}

fn path_len(session: &Session) -> i64 {
    to_i64(session.path.len())
}

fn get(session: &Session, at: i64) -> Option<&Expected> {
    usize::try_from(at)
        .ok()
        .and_then(|index| session.path.get(index))
}

#[must_use]
pub fn expected_now(session: &Session) -> Option<Expected> {
    let last = session.path.last();
    if session.at >= path_len(session) && last.is_some_and(|step| step.kind == ExpectedKind::Run) {
        return last.cloned();
    }
    get(session, session.at).cloned()
}

#[must_use]
pub fn is_guided(session: &Session) -> bool {
    !session.path.is_empty()
}

fn upcoming_text(session: &Session) -> String {
    let size = path_len(session);
    let from = if session.at < 0 {
        (size + session.at).max(0)
    } else {
        session.at.min(size)
    };
    let start = usize::try_from(from).unwrap_or(usize::MAX);
    session
        .path
        .iter()
        .skip(start)
        .find(|step| step.kind == ExpectedKind::Text)
        .map_or_else(String::new, |step| step.key.clone())
}

fn free(session: &Session, key: &str, rules: &Rules) -> Pressed {
    if key == "Enter" && session.buffer.mode == Mode::Normal {
        return Pressed {
            at: 0,
            cause: Cause::None,
            expected: None,
            key: key.to_owned(),
            runs: true,
            session: session.clone(),
            verdict: Verdict::Moved,
        };
    }
    let outcome = step(&session.buffer, key, rules);
    Pressed {
        at: 0,
        cause: Cause::None,
        expected: None,
        key: key.to_owned(),
        runs: false,
        session: Session {
            buffer: outcome.buffer,
            ..session.clone()
        },
        verdict: if outcome.refused {
            Verdict::Refused
        } else {
            Verdict::Moved
        },
    }
}

fn slipped(session: &Session, key: &str, rules: &Rules) -> Cause {
    let Some(expected) = expected_now(session) else {
        return Cause::Other;
    };
    cause_of(&Slip {
        expected,
        inserting: session.buffer.mode == Mode::Insert,
        key: key.to_owned(),
        taught: rules.keys.contains(key),
        upcoming: upcoming_text(session),
    })
}

fn forward(session: &Session, buffer: Buffer) -> Session {
    Session {
        at: (session.at + 1).min(path_len(session)),
        buffer,
        path: session.path.clone(),
    }
}

fn follow(session: &Session, key: &str, rules: &Rules) -> Pressed {
    let expected = expected_now(session);
    let Some(target) = expected.clone().filter(|step| step.key == key) else {
        return Pressed {
            at: session.at,
            cause: slipped(session, key, rules),
            expected,
            key: key.to_owned(),
            runs: false,
            session: session.clone(),
            verdict: Verdict::Wrong,
        };
    };
    if target.kind == ExpectedKind::Run {
        return Pressed {
            at: session.at,
            cause: Cause::None,
            expected,
            key: key.to_owned(),
            runs: true,
            session: forward(session, session.buffer.clone()),
            verdict: Verdict::Right,
        };
    }
    let guided = Rules {
        keys: known_keys(),
        pairs: rules.pairs.clone(),
    };
    let outcome = step(&session.buffer, key, &guided);
    Pressed {
        at: session.at,
        cause: Cause::None,
        expected,
        key: key.to_owned(),
        runs: false,
        session: forward(session, outcome.buffer),
        verdict: Verdict::Right,
    }
}

#[must_use]
pub fn press(session: &Session, key: &str, rules: &Rules) -> Pressed {
    if is_guided(session) {
        return follow(session, key, rules);
    }
    free(session, key, rules)
}
