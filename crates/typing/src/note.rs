use konekode_text::len;

use crate::drill::{Drill, Feedback, INSERT_KEY, Mode, Next, expected_char, is_done, is_typed};
use crate::keyboard::ESCAPE_KEY;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attempt {
    pub at: i64,
    pub cause: String,
    pub expected: String,
    pub key: String,
    pub kind: String,
    pub verdict: String,
}

fn cause_for(feedback: Feedback) -> &'static str {
    match feedback {
        Feedback::NeedInsert => "need-insert",
        Feedback::NoDelete => "no-delete",
        Feedback::None => "other",
        Feedback::WrongKey => "mistyped",
    }
}

fn step_at(drill: &Drill) -> i64 {
    if is_typed(drill) {
        return len(&drill.target) + 1;
    }
    if drill.mode == Mode::Normal {
        return 0;
    }
    drill.position + 1
}

fn expected_of(drill: &Drill) -> String {
    if is_typed(drill) {
        return ESCAPE_KEY.to_owned();
    }
    if drill.mode == Mode::Normal {
        return INSERT_KEY.to_owned();
    }
    expected_char(drill).to_owned()
}

fn kind_of(drill: &Drill) -> &'static str {
    if drill.mode == Mode::Insert && !is_typed(drill) {
        return "text";
    }
    "command"
}

fn left_early(before: &Drill, after: &Drill) -> bool {
    before.mode == Mode::Insert && after.mode == Mode::Normal && !is_typed(before)
}

fn moved(before: &Drill, after: &Drill) -> bool {
    step_at(after) != step_at(before) || (is_typed(before) && is_done(after))
}

fn judged(before: &Drill, after: &Drill) -> (&'static str, &'static str) {
    if left_early(before, after) {
        return ("early-leave", "wrong");
    }
    if moved(before, after) {
        return ("", "right");
    }
    (cause_for(after.feedback), "wrong")
}

#[must_use]
pub fn attempt_of(before: &Drill, after: &Next, key: &str) -> Option<Attempt> {
    let Next::New(after) = after else {
        return None;
    };
    if is_done(before) {
        return None;
    }
    let (cause, verdict) = judged(before, after);
    Some(Attempt {
        at: step_at(before),
        cause: cause.to_owned(),
        expected: expected_of(before),
        key: key.to_owned(),
        kind: kind_of(before).to_owned(),
        verdict: verdict.to_owned(),
    })
}
