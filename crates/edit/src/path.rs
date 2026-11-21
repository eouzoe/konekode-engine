use konekode_text::units;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum ExpectedKind {
    Command,
    Run,
    Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Expected {
    pub key: String,
    pub kind: ExpectedKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub enum Cause {
    #[cfg_attr(feature = "serde", serde(rename = ""))]
    None,
    Case,
    EarlyLeave,
    EarlyRun,
    Mistyped,
    NeedInsert,
    NoDelete,
    Other,
    Taught,
    Untaught,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Slip {
    pub expected: Expected,
    pub inserting: bool,
    pub key: String,
    pub taught: bool,
    pub upcoming: String,
}

const TYPE_PREFIX: &str = "type:";

fn expected(key: &str, kind: ExpectedKind) -> Expected {
    Expected {
        key: key.to_owned(),
        kind,
    }
}

fn units_of(text: &str, kind: ExpectedKind) -> Vec<Expected> {
    text.chars()
        .map(|char| expected(&char.to_string(), kind))
        .collect()
}

fn expand_token(token: &str) -> Vec<Expected> {
    match token {
        "reset" => Vec::new(),
        "BS" => vec![expected("Backspace", ExpectedKind::Command)],
        "Enter" => vec![expected("Enter", ExpectedKind::Command)],
        "Esc" => vec![expected("Escape", ExpectedKind::Command)],
        "run" => vec![expected("Enter", ExpectedKind::Run)],
        _ => token.strip_prefix(TYPE_PREFIX).map_or_else(
            || units_of(token, ExpectedKind::Command),
            |text| units_of(text, ExpectedKind::Text),
        ),
    }
}

#[must_use]
pub fn expand(path: &[String]) -> Vec<Expected> {
    path.iter().flat_map(|token| expand_token(token)).collect()
}

#[must_use]
pub fn keys_of(path: &[String]) -> Vec<String> {
    expand(path)
        .into_iter()
        .filter(|step| step.kind != ExpectedKind::Run)
        .map(|step| step.key)
        .collect()
}

fn is_char(key: &str) -> bool {
    units(key) == 1
}

#[must_use]
pub fn cause_of(slip: &Slip) -> Cause {
    let key = slip.key.as_str();
    if key == "Backspace" || key == "Delete" {
        return Cause::NoDelete;
    }
    if key.to_lowercase() == slip.expected.key.to_lowercase() {
        return Cause::Case;
    }
    if key == "Escape" && slip.expected.kind == ExpectedKind::Text {
        return Cause::EarlyLeave;
    }
    if key == "Enter" && !slip.inserting {
        return Cause::EarlyRun;
    }
    if !slip.inserting && key == slip.upcoming {
        return Cause::NeedInsert;
    }
    if !slip.inserting && is_char(key) && slip.taught {
        return Cause::Taught;
    }
    if !slip.inserting && is_char(key) {
        return Cause::Untaught;
    }
    if slip.inserting && is_char(key) {
        return Cause::Mistyped;
    }
    Cause::Other
}
