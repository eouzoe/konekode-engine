#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub enum Where {
    Anywhere,
    IncludeLine,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "kind", rename = "pair")
)]
pub struct PairItem {
    pub close: String,
    pub id: String,
    pub open: String,
    #[cfg_attr(feature = "serde", serde(rename = "where"))]
    pub place: Where,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "kind", rename = "key")
)]
pub struct KeyItem {
    pub id: String,
    pub key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum Item {
    Key(KeyItem),
    Pair(PairItem),
}

impl Item {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Key(item) => &item.id,
            Self::Pair(item) => &item.id,
        }
    }
}

pub const COMMAND_KEYS: [&str; 12] = ["$", ".", "0", "G", "O", "d", "f", "i", "j", "k", "o", "x"];

fn pair(open: &str, close: &str, id: &str, place: Where) -> PairItem {
    PairItem {
        close: close.to_owned(),
        id: id.to_owned(),
        open: open.to_owned(),
        place,
    }
}

#[must_use]
pub fn pairs() -> Vec<PairItem> {
    vec![
        pair("(", ")", "pair.paren", Where::Anywhere),
        pair("{", "}", "pair.brace", Where::Anywhere),
        pair("<", ">", "pair.include-angle", Where::IncludeLine),
    ]
}

#[must_use]
pub fn keys() -> Vec<KeyItem> {
    COMMAND_KEYS
        .iter()
        .map(|key| KeyItem {
            id: format!("key.{key}"),
            key: (*key).to_owned(),
        })
        .collect()
}

#[must_use]
pub fn catalog() -> Vec<Item> {
    pairs()
        .into_iter()
        .map(Item::Pair)
        .chain(keys().into_iter().map(Item::Key))
        .collect()
}
