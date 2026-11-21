use std::collections::{BTreeMap, BTreeSet};

use konekode_complete::keys;
use konekode_edit::{ExpectedKind, expand};
use konekode_typing::INSERT_KEY;
use konekode_unit::{Beat, Task, Wait, path_of};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnitDomain {
    pub shows: BTreeMap<String, Vec<String>>,
    pub uses: BTreeMap<String, Vec<Vec<String>>>,
}

#[must_use]
pub fn components() -> Vec<String> {
    keys().into_iter().map(|item| item.id).collect()
}

struct Catalog {
    by_key: BTreeMap<String, String>,
    known: BTreeSet<String>,
}

impl Catalog {
    fn new() -> Self {
        let mut by_key = BTreeMap::new();
        let mut known = BTreeSet::new();
        for item in keys() {
            known.insert(item.id.clone());
            by_key.insert(item.key, item.id);
        }
        Self { by_key, known }
    }

    fn used(&self, tokens: &[String]) -> Vec<Vec<String>> {
        expand(tokens)
            .into_iter()
            .map(|step| {
                if step.kind != ExpectedKind::Command {
                    return Vec::new();
                }
                self.by_key.get(&step.key).cloned().into_iter().collect()
            })
            .collect()
    }

    fn shown_by(&self, beat: &Beat) -> Vec<String> {
        let labelled = beat
            .line
            .unlock
            .iter()
            .flatten()
            .filter(|id| self.known.contains(*id))
            .cloned();
        let keys = beat.line.keys.clone().unwrap_or_default();
        let mut seen = BTreeSet::new();
        labelled
            .chain(self.used(&keys).into_iter().flatten())
            .filter(|id| seen.insert(id.clone()))
            .collect()
    }
}

#[must_use]
pub fn unit_domain(beats: &[Beat], tasks: &[Task]) -> UnitDomain {
    let catalog = Catalog::new();
    let drill_uses = catalog.used(&[INSERT_KEY.to_owned()]);
    let shows = beats
        .iter()
        .map(|beat| (beat.line.id.clone(), catalog.shown_by(beat)))
        .collect();
    let mut uses = BTreeMap::new();
    for beat in beats
        .iter()
        .filter(|beat| beat.line.wait == Some(Wait::Drill))
    {
        uses.insert(beat.line.id.clone(), drill_uses.clone());
    }
    for task in tasks {
        uses.insert(task.id.clone(), catalog.used(&path_of(task)));
    }
    UnitDomain { shows, uses }
}
