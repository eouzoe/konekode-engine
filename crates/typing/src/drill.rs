use konekode_text::{char_at, len};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Mode {
    Insert,
    Normal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub enum Feedback {
    NeedInsert,
    NoDelete,
    None,
    WrongKey,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Drill {
    pub feedback: Feedback,
    pub mistakes: i64,
    pub mode: Mode,
    pub position: i64,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Next {
    Same,
    New(Drill),
}

impl Next {
    #[must_use]
    pub fn resolve(self, before: &Drill) -> Drill {
        match self {
            Self::Same => before.clone(),
            Self::New(drill) => drill,
        }
    }
}

pub const INSERT_KEY: &str = "i";

fn skip_indent(target: &str, start: i64) -> i64 {
    let mut index = start;
    while char_at(target, index) == " " {
        index += 1;
    }
    index
}

#[must_use]
pub fn create_drill(target: &str) -> Drill {
    Drill {
        feedback: Feedback::None,
        mistakes: 0,
        mode: Mode::Normal,
        position: skip_indent(target, 0),
        target: target.to_owned(),
    }
}

#[must_use]
pub fn expected_char(drill: &Drill) -> &str {
    char_at(&drill.target, drill.position)
}

#[must_use]
pub fn is_typed(drill: &Drill) -> bool {
    drill.position >= len(&drill.target)
}

#[must_use]
pub fn is_done(drill: &Drill) -> bool {
    is_typed(drill) && drill.mode == Mode::Normal
}

#[must_use]
pub fn has_started(drill: &Drill) -> bool {
    drill.position > skip_indent(&drill.target, 0)
}

fn command(drill: &Drill, input: &str) -> Next {
    if is_typed(drill) {
        return Next::Same;
    }
    if input == INSERT_KEY {
        return Next::New(Drill {
            feedback: Feedback::None,
            mode: Mode::Insert,
            ..drill.clone()
        });
    }
    Next::New(Drill {
        feedback: Feedback::NeedInsert,
        ..drill.clone()
    })
}

fn insert(drill: &Drill, input: &str) -> Drill {
    let expected = expected_char(drill);
    if is_typed(drill) || input != expected {
        return Drill {
            feedback: Feedback::WrongKey,
            mistakes: drill.mistakes + 1,
            ..drill.clone()
        };
    }
    let next = drill.position + 1;
    if expected == "\n" {
        return Drill {
            feedback: Feedback::None,
            position: skip_indent(&drill.target, next),
            ..drill.clone()
        };
    }
    Drill {
        feedback: Feedback::None,
        position: next,
        ..drill.clone()
    }
}

#[must_use]
pub fn press(drill: &Drill, input: &str) -> Next {
    if drill.mode == Mode::Normal {
        return command(drill, input);
    }
    Next::New(insert(drill, input))
}

#[must_use]
pub fn escape(drill: &Drill) -> Drill {
    Drill {
        feedback: Feedback::None,
        mode: Mode::Normal,
        ..drill.clone()
    }
}

#[must_use]
pub fn remind(drill: &Drill, feedback: Feedback) -> Next {
    if is_done(drill) {
        return Next::Same;
    }
    Next::New(Drill {
        feedback,
        ..drill.clone()
    })
}
