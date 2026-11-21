#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyCap {
    pub base: String,
    pub shift: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyPress {
    pub key: String,
    pub shift: bool,
}

pub const ENTER_KEY: &str = "Enter";
pub const ESCAPE_KEY: &str = "Escape";
pub const SHIFT_KEY: &str = "Shift";
pub const SPACE_KEY: &str = "Space";

const LAYOUT: [&str; 4] = [
    "`~ 1! 2@ 3# 4$ 5% 6^ 7& 8* 9( 0) -_ =+",
    r"qQ wW eE rR tT yY uU iI oO pP [{ ]} \|",
    "aA sS dD fF gG hH jJ kK lL ;: '\"",
    "zZ xX cC vV bB nN mM ,< .> /?",
];

fn caps(row: &str) -> Vec<KeyCap> {
    row.split(' ')
        .map(|pair| {
            let mut chars = pair.chars();
            KeyCap {
                base: chars.next().map_or_else(String::new, String::from),
                shift: chars.next().map_or_else(String::new, String::from),
            }
        })
        .collect()
}

#[must_use]
pub fn rows() -> Vec<Vec<KeyCap>> {
    LAYOUT.iter().map(|row| caps(row)).collect()
}

#[must_use]
pub fn find_key(char: &str) -> Option<KeyPress> {
    if char == "\n" {
        return Some(KeyPress {
            key: ENTER_KEY.to_owned(),
            shift: false,
        });
    }
    if char == " " {
        return Some(KeyPress {
            key: SPACE_KEY.to_owned(),
            shift: false,
        });
    }
    for cap in rows().into_iter().flatten() {
        if cap.base == char {
            return Some(KeyPress {
                key: cap.base,
                shift: false,
            });
        }
        if cap.shift == char {
            return Some(KeyPress {
                key: cap.base,
                shift: true,
            });
        }
    }
    None
}

#[must_use]
pub fn needs_telling(char: &str) -> bool {
    char == "\n" || char == " " || find_key(char).is_some_and(|found| found.shift)
}
