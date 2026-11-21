use std::collections::BTreeMap;

use konekode_record::{Event, Log};

use crate::domain::UnitDomain;
use crate::evidence::Evidence;
use crate::progress::Progress;

pub type Domains = BTreeMap<String, UnitDomain>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fold {
    pub evidence: Evidence,
    pub progress: Progress,
}

impl Fold {
    #[must_use]
    pub fn over(log: &Log, domains: &Domains) -> Self {
        let mut fold = Self::default();
        for event in log.events() {
            fold.apply(event, domains);
        }
        fold
    }

    pub fn apply(&mut self, event: &Event, domains: &Domains) {
        self.progress.apply(event);
        self.evidence.apply(event, domains.get(&event.unit));
    }
}

#[must_use]
pub fn observe(events: &[Event], domains: &Domains) -> Evidence {
    Fold::over(&Log::new(events), domains).evidence
}
