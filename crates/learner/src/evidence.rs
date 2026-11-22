use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use konekode_record::{DataValue, Event};

use crate::describe::{Count, described, shown_by, used_by};
use crate::domain::UnitDomain;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Standing {
    Done,
    Shown,
    Unseen,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tally {
    pub completed: i64,
    #[cfg_attr(feature = "serde", serde(rename = "firstRight"))]
    pub first_right: i64,
    pub opportunities: i64,
    pub shown: i64,
    pub unsettled: bool,
}

impl Tally {
    #[must_use]
    pub const fn standing(&self) -> Standing {
        if self.completed > 0 {
            return Standing::Done;
        }
        if self.shown > 0 {
            return Standing::Shown;
        }
        Standing::Unseen
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Observation {
    pub at: i64,
    pub component: String,
    pub helped: bool,
    pub opportunity: i64,
    pub right: bool,
    pub session: String,
    pub step: f64,
    pub task: String,
    pub unit: String,
    pub waited: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    helped: bool,
    last: i64,
    steps: BTreeMap<u64, bool>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Evidence {
    count: Count,
    entries: BTreeMap<String, Entry>,
    observations: Vec<Observation>,
    order: Vec<String>,
    seen: BTreeSet<String>,
    tallies: BTreeMap<String, Tally>,
}

fn step_key(step: f64) -> u64 {
    if matches!(step.partial_cmp(&0.0), Some(Ordering::Equal)) {
        return 0.0_f64.to_bits();
    }
    step.to_bits()
}

fn entry_key(event: &Event) -> String {
    format!("{}|{}|{}", event.session, event.unit, event.reference)
}

fn text_is(event: &Event, key: &str, wanted: &str) -> bool {
    matches!(event.data.get(key), Some(DataValue::Text(value)) if value == wanted)
}

impl Evidence {
    pub fn apply(&mut self, event: &Event, domain: Option<&UnitDomain>) {
        match event.kind.as_str() {
            "beat.reached" => self.reach(event, domain),
            "step.tried" => self.attempt(event, domain),
            "task.entered" => self.enter(event),
            _ => {}
        }
    }

    #[must_use]
    pub const fn count(&self) -> Count {
        self.count
    }

    #[must_use]
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    #[must_use]
    pub fn tallies(&self) -> Vec<(String, Tally)> {
        self.order
            .iter()
            .filter_map(|id| Some((id.clone(), self.tallies.get(id)?.clone())))
            .collect()
    }

    #[must_use]
    pub fn tally(&self, id: &str) -> Tally {
        self.tallies.get(id).cloned().unwrap_or_default()
    }

    fn tally_mut(&mut self, id: &str) -> &mut Tally {
        if !self.tallies.contains_key(id) {
            self.order.push(id.to_owned());
        }
        self.tallies.entry(id.to_owned()).or_default()
    }

    fn reach(&mut self, event: &Event, domain: Option<&UnitDomain>) {
        if !self
            .seen
            .insert(format!("{}|{}", event.unit, event.reference))
        {
            return;
        }
        let shown = described(&mut self.count, event.data.get("shows"), || {
            shown_by(domain, &event.reference)
        });
        for id in shown {
            self.tally_mut(&id).shown += 1;
        }
    }

    fn enter(&mut self, event: &Event) {
        self.entries.insert(
            entry_key(event),
            Entry {
                helped: !text_is(event, "mode", "free"),
                last: event.at,
                steps: BTreeMap::new(),
            },
        );
    }

    fn attempt(&mut self, event: &Event, domain: Option<&UnitDomain>) {
        let Some(DataValue::Number(step)) = event.data.get("at") else {
            return;
        };
        let step = *step;
        let Some(entry) = self.entries.get_mut(&entry_key(event)) else {
            return;
        };
        let key = step_key(step);
        if entry.steps.get(&key) == Some(&true) {
            return;
        }
        let right = text_is(event, "verdict", "right");
        let first = !entry.steps.contains_key(&key);
        let helped = entry.helped;
        let waited = event.at - entry.last;
        entry.steps.insert(key, right);
        if right {
            entry.last = event.at;
        }
        let used = described(&mut self.count, event.data.get("uses"), || {
            used_by(domain, &event.reference, step)
        });
        for component in used {
            let tally = self.tally_mut(&component);
            let opportunity = tally.opportunities + 1;
            if first {
                tally.opportunities += 1;
                tally.unsettled = !right;
            }
            if first && right {
                tally.first_right += 1;
            }
            if right {
                tally.completed += 1;
            }
            if first {
                self.observations.push(Observation {
                    at: event.at,
                    component,
                    helped,
                    opportunity,
                    right,
                    session: event.session.clone(),
                    step,
                    task: event.reference.clone(),
                    unit: event.unit.clone(),
                    waited,
                });
            }
        }
    }
}
