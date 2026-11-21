use std::collections::{BTreeMap, BTreeSet};

use konekode_record::{Event, ordered};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnitState {
    pub passed: BTreeSet<String>,
    pub position: Option<String>,
}

#[must_use]
pub fn unit_state(events: &[Event], unit: &str) -> UnitState {
    let mut state = UnitState::default();
    for event in ordered(events).iter().filter(|event| event.unit == unit) {
        match event.kind.as_str() {
            "unit.restarted" => state = UnitState::default(),
            "beat.reached" => state.position = Some(event.reference.clone()),
            "check.passed" => {
                state.passed.insert(event.reference.clone());
            }
            _ => {}
        }
    }
    state
}

#[must_use]
pub fn unlocked_items(events: &[Event]) -> BTreeSet<String> {
    let mut by_unit: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for event in ordered(events) {
        match event.kind.as_str() {
            "unit.restarted" => {
                by_unit.remove(&event.unit);
            }
            "item.unlocked" => {
                by_unit
                    .entry(event.unit.clone())
                    .or_default()
                    .insert(event.reference.clone());
            }
            _ => {}
        }
    }
    by_unit.into_values().flatten().collect()
}
