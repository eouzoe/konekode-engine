use std::collections::BTreeMap;

use konekode_learner::{Domains, unit_domain};
use konekode_unit::{Beat, Task};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UnitContent {
    pub beats: Vec<Beat>,
    #[serde(default)]
    pub starter: String,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Content {
    #[serde(default)]
    pub catalog: Value,
    pub units: BTreeMap<String, UnitContent>,
    #[serde(default)]
    pub version: String,
}

impl Content {
    #[must_use]
    pub fn domains(&self) -> Domains {
        self.units
            .iter()
            .map(|(name, unit)| (name.clone(), unit_domain(&unit.beats, &unit.tasks)))
            .collect()
    }

    #[must_use]
    pub fn catalog_matches(&self) -> bool {
        if self.catalog.is_null() {
            return true;
        }
        serde_json::to_value(konekode_complete::catalog()).is_ok_and(|own| own == self.catalog)
    }
}
