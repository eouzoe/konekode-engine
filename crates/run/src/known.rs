use std::collections::BTreeMap;

use crate::outcome::Outcome;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KnownRow {
    pub input: String,
    pub output: String,
    pub program: String,
}

#[derive(Clone, Debug, Default)]
pub struct Known {
    table: BTreeMap<(String, String), String>,
}

impl Known {
    #[must_use]
    pub fn new(rows: &[KnownRow]) -> Self {
        let table = rows
            .iter()
            .map(|row| ((row.program.clone(), row.input.clone()), row.output.clone()))
            .collect();
        Self { table }
    }

    #[must_use]
    pub fn outcome(&self, source: &str, input: &str) -> Option<Outcome> {
        let output = self.table.get(&(source.to_owned(), input.to_owned()))?;
        Some(Outcome::Finished {
            exit_code: 0,
            output: output.clone(),
        })
    }
}
