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

fn component_for(key: &str) -> Option<String> {
    keys()
        .into_iter()
        .find(|item| item.key == key)
        .map(|item| item.id)
}

fn used(tokens: &[String]) -> Vec<Vec<String>> {
    expand(tokens)
        .into_iter()
        .map(|step| {
            if step.kind != ExpectedKind::Command {
                return Vec::new();
            }
            component_for(&step.key).into_iter().collect()
        })
        .collect()
}

fn shown_by_beat(beat: &Beat, known: &BTreeSet<String>) -> Vec<String> {
    let labelled = beat
        .line
        .unlock
        .iter()
        .flatten()
        .filter(|id| known.contains(*id))
        .cloned();
    let keys = beat.line.keys.clone().unwrap_or_default();
    let mut seen = BTreeSet::new();
    labelled
        .chain(used(&keys).into_iter().flatten())
        .filter(|id| seen.insert(id.clone()))
        .collect()
}

#[must_use]
pub fn unit_domain(beats: &[Beat], tasks: &[Task]) -> UnitDomain {
    let known: BTreeSet<String> = components().into_iter().collect();
    let drill_uses = used(&[INSERT_KEY.to_owned()]);
    let shows = beats
        .iter()
        .map(|beat| (beat.line.id.clone(), shown_by_beat(beat, &known)))
        .collect();
    let mut uses = BTreeMap::new();
    for beat in beats
        .iter()
        .filter(|beat| beat.line.wait == Some(Wait::Drill))
    {
        uses.insert(beat.line.id.clone(), drill_uses.clone());
    }
    for task in tasks {
        uses.insert(task.id.clone(), used(&path_of(task)));
    }
    UnitDomain { shows, uses }
}
