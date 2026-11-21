use konekode_estimate::{Estimates, Needs, estimate_evidence};
use konekode_learner::{Domains, Fold, shown_by, used_by};
use konekode_record::{Appended, Event, Log};
use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::state::State;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pushed {
    Applied,
    Refolded,
    Unchanged,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Engine {
    domains: Domains,
    fold: Fold,
    log: Log,
    needs: Needs,
}

impl Engine {
    #[must_use]
    pub fn new(content: &Content, events: &[Event]) -> Self {
        let domains = content.domains();
        let log = Log::new(events);
        let fold = Fold::over(&log, &domains);
        Self {
            domains,
            fold,
            log,
            needs: content.needs.clone(),
        }
    }

    pub fn push(&mut self, event: Event) -> Pushed {
        match self.log.append(event) {
            Appended::Last => {
                if let Some(last) = self.log.events().last() {
                    self.fold.apply(last, &self.domains);
                }
                Pushed::Applied
            }
            Appended::Inside | Appended::Replaced => {
                self.fold = Fold::over(&self.log, &self.domains);
                Pushed::Refolded
            }
            Appended::Same | Appended::Kept => Pushed::Unchanged,
        }
    }

    #[must_use]
    pub fn shows(&self, unit: &str, beat: &str) -> Vec<String> {
        shown_by(self.domains.get(unit), beat).unwrap_or_default()
    }

    #[must_use]
    pub fn uses(&self, unit: &str, task: &str, step: f64) -> Vec<String> {
        used_by(self.domains.get(unit), task, step).unwrap_or_default()
    }

    #[must_use]
    pub fn state(&self) -> State {
        let now = self.log.events().last().map_or(0, |event| event.at);
        State::of(
            &self.fold,
            self.log.conflicts(),
            self.log.events().len(),
            &self.needs,
            now,
        )
    }

    #[must_use]
    pub fn estimates(&self, now: i64) -> Estimates {
        estimate_evidence(&self.fold.evidence, &self.needs, now)
    }
}
