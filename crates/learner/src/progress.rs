use std::collections::{BTreeMap, BTreeSet};

use konekode_record::{Event, ordered};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnitState {
    pub passed: BTreeSet<String>,
    pub position: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    units: BTreeMap<String, UnitState>,
    unlocked: BTreeMap<String, BTreeSet<String>>,
}

impl Progress {
    pub fn apply(&mut self, event: &Event) {
        match event.kind.as_str() {
            "unit.restarted" => {
                self.units.remove(&event.unit);
                self.unlocked.remove(&event.unit);
            }
            "beat.reached" => {
                self.units.entry(event.unit.clone()).or_default().position =
                    Some(event.reference.clone());
            }
            "check.passed" => {
                self.units
                    .entry(event.unit.clone())
                    .or_default()
                    .passed
                    .insert(event.reference.clone());
            }
            "item.unlocked" => {
                self.unlocked
                    .entry(event.unit.clone())
                    .or_default()
                    .insert(event.reference.clone());
            }
            _ => {}
        }
    }

    #[must_use]
    pub fn unit(&self, unit: &str) -> UnitState {
        self.units.get(unit).cloned().unwrap_or_default()
    }

    #[must_use]
    pub const fn units(&self) -> &BTreeMap<String, UnitState> {
        &self.units
    }

    #[must_use]
    pub fn unlocked(&self) -> BTreeSet<String> {
        self.unlocked.values().flatten().cloned().collect()
    }
}

fn progress_of(events: &[Event]) -> Progress {
    let mut progress = Progress::default();
    for event in ordered(events) {
        progress.apply(&event);
    }
    progress
}

#[must_use]
pub fn unit_state(events: &[Event], unit: &str) -> UnitState {
    progress_of(events).unit(unit)
}

#[must_use]
pub fn unlocked_items(events: &[Event]) -> BTreeSet<String> {
    progress_of(events).unlocked()
}
