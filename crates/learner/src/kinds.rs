use std::collections::BTreeSet;

pub const PROGRESS_KINDS: [&str; 4] = [
    "beat.reached",
    "check.passed",
    "item.unlocked",
    "unit.restarted",
];
pub const MODEL_KINDS: [&str; 3] = ["beat.reached", "step.tried", "task.entered"];

#[must_use]
pub fn kept_kinds() -> BTreeSet<&'static str> {
    PROGRESS_KINDS
        .iter()
        .chain(MODEL_KINDS.iter())
        .copied()
        .collect()
}
