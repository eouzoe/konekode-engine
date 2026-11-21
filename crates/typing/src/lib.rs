mod drill;
mod keyboard;
mod keypress;
mod note;

pub use drill::{
    Drill, Feedback, INSERT_KEY, Mode, Next, create_drill, escape, expected_char, has_started,
    is_done, is_typed, press, remind,
};
pub use keyboard::{
    ENTER_KEY, ESCAPE_KEY, KeyCap, KeyPress, SHIFT_KEY, SPACE_KEY, find_key, needs_telling, rows,
};
pub use keypress::{KeyInput, judge, type_into};
pub use note::{Attempt, attempt_of};
