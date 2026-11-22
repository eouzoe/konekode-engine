use konekode_text::units;

use crate::drill::{Drill, Feedback, Next, escape, press, remind};
use crate::keyboard::ESCAPE_KEY;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyInput {
    pub key: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub alt: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub ctrl: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub meta: bool,
}

fn typed_by(input: &KeyInput) -> Option<String> {
    if input.ctrl || input.meta || input.alt {
        return None;
    }
    if input.key == "Enter" {
        return Some("\n".to_owned());
    }
    if units(&input.key) == 1 {
        return Some(input.key.clone());
    }
    None
}

#[must_use]
pub fn type_into(drill: &Drill, text: &str) -> Drill {
    let mut next = drill.clone();
    for char in text.chars() {
        next = press(&next, &char.to_string()).resolve(&next);
    }
    next
}

#[must_use]
pub fn judge(drill: &Drill, input: &KeyInput) -> Next {
    let typed = typed_by(input);
    if input.key == ESCAPE_KEY {
        return Next::New(escape(drill));
    }
    if input.key == "Backspace" || input.key == "Delete" {
        return remind(drill, Feedback::NoDelete);
    }
    typed.map_or(Next::Same, |char| press(drill, &char))
}
