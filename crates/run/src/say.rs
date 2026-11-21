use std::collections::BTreeMap;
use std::fmt;

use konekode_text::js_replace_all;

pub type Text = BTreeMap<String, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Missing(pub String);

impl fmt::Display for Missing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "no text for {}", self.0)
    }
}

impl std::error::Error for Missing {}

pub fn line(text: &Text, key: &str) -> Result<String, Missing> {
    text.get(key)
        .cloned()
        .ok_or_else(|| Missing(key.to_owned()))
}

pub fn say(text: &Text, key: &str, values: &[(&str, &str)]) -> Result<String, Missing> {
    let mut out = line(text, key)?;
    for (name, value) in values {
        out = js_replace_all(&out, &format!("{{{name}}}"), value);
    }
    Ok(out)
}
