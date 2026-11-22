use std::collections::BTreeSet;

use crate::{Item, PairItem, Where};

#[derive(Clone, Debug)]
pub struct Engine {
    items: Vec<Item>,
}

impl Engine {
    #[must_use]
    pub const fn new(items: Vec<Item>) -> Self {
        Self { items }
    }

    fn available<'a>(&'a self, unlocked: &'a BTreeSet<String>) -> impl Iterator<Item = &'a Item> {
        self.items
            .iter()
            .filter(move |item| unlocked.contains(item.id()))
    }

    #[must_use]
    pub fn pairs(&self, unlocked: &BTreeSet<String>) -> Vec<PairItem> {
        self.available(unlocked)
            .filter_map(|item| match item {
                Item::Pair(pair) => Some(pair.clone()),
                Item::Key(_) => None,
            })
            .collect()
    }

    #[must_use]
    pub fn brackets(&self, unlocked: &BTreeSet<String>) -> Vec<String> {
        self.pairs(unlocked)
            .into_iter()
            .filter(|pair| pair.place == Where::Anywhere)
            .map(|pair| pair.open)
            .collect()
    }

    #[must_use]
    pub fn keys(&self, unlocked: &BTreeSet<String>) -> BTreeSet<String> {
        self.available(unlocked)
            .filter_map(|item| match item {
                Item::Key(key) => Some(key.key.clone()),
                Item::Pair(_) => None,
            })
            .collect()
    }

    #[must_use]
    pub fn line_pair(&self, unlocked: &BTreeSet<String>, open: &str) -> Option<PairItem> {
        self.pairs(unlocked)
            .into_iter()
            .find(|pair| pair.place == Where::IncludeLine && pair.open == open)
    }
}
