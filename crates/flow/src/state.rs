use std::collections::{BTreeMap, BTreeSet};

use konekode_learner::{Fold, Observation, Standing, Tally, UnitState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub standing: Standing,
    #[serde(flatten)]
    pub tally: Tally,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub components: Vec<Component>,
    pub conflicts: Vec<String>,
    pub events: usize,
    pub legacy: i64,
    pub observations: Vec<Observation>,
    pub unlocked: BTreeSet<String>,
    pub units: BTreeMap<String, UnitState>,
    pub unread: i64,
}

impl State {
    #[must_use]
    pub fn of(fold: &Fold, conflicts: Vec<String>, events: usize) -> Self {
        let count = fold.evidence.count();
        Self {
            components: fold
                .evidence
                .tallies()
                .into_iter()
                .map(|(id, tally)| Component {
                    id,
                    standing: tally.standing(),
                    tally,
                })
                .collect(),
            conflicts,
            events,
            legacy: count.legacy,
            observations: fold.evidence.observations().to_vec(),
            unlocked: fold.progress.unlocked(),
            units: fold.progress.units().clone(),
            unread: count.unread,
        }
    }
}
