use crate::{Set, Space};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Error,
    Warning,
    Hint,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Finding {
    NoClause {
        item: usize,
    },
    SelfNeed {
        item: usize,
        clause: usize,
    },
    Unknown {
        item: usize,
        clause: usize,
        need: usize,
    },
    Unreachable {
        item: usize,
    },
    Redundant {
        item: usize,
        clause: usize,
        within: usize,
    },
    Dead {
        item: usize,
        clause: usize,
        need: usize,
    },
    Unsupported {
        item: usize,
        clause: usize,
        need: usize,
    },
}

impl Finding {
    #[must_use]
    pub const fn level(&self) -> Level {
        match self {
            Self::NoClause { .. } | Self::SelfNeed { .. } | Self::Unknown { .. } => Level::Error,
            Self::Unreachable { .. } | Self::Redundant { .. } | Self::Dead { .. } => Level::Warning,
            Self::Unsupported { .. } => Level::Hint,
        }
    }
}

pub(crate) fn errors(clauses: &[Vec<Set>]) -> Vec<Finding> {
    let size = clauses.len();
    let mut found = Vec::new();
    for (item, options) in clauses.iter().enumerate() {
        if options.is_empty() {
            found.push(Finding::NoClause { item });
        }
        for (at, clause) in options.iter().enumerate() {
            if clause.contains(item) {
                found.push(Finding::SelfNeed { item, clause: at });
            }
            if let Some(need) = clause.iter().find(|&need| need >= size) {
                found.push(Finding::Unknown {
                    item,
                    clause: at,
                    need,
                });
            }
        }
    }
    found
}

#[must_use]
pub fn check(clauses: &[Vec<Set>]) -> Vec<Finding> {
    let mut found = errors(clauses);
    if !found.is_empty() {
        return found;
    }
    let space = Space::trusted(clauses.to_vec());
    let reach = space.reachable();
    found.extend(
        (0..clauses.len())
            .filter(|&item| !reach.contains(item))
            .map(|item| Finding::Unreachable { item }),
    );
    for (item, options) in clauses.iter().enumerate() {
        for (at, clause) in options.iter().enumerate() {
            if let Some(within) = (0..options.len()).find(|&other| {
                other != at
                    && options[other].is_subset(clause)
                    && (other < at || options[other] != *clause)
            }) {
                found.push(Finding::Redundant {
                    item,
                    clause: at,
                    within,
                });
            }
            if let Some(need) = clause.iter().find(|&need| !reach.contains(need)) {
                found.push(Finding::Dead {
                    item,
                    clause: at,
                    need,
                });
            } else if let Some(need) = clause.iter().find(|&need| {
                !clauses[need]
                    .iter()
                    .any(|support| support.is_subset(clause))
            }) {
                found.push(Finding::Unsupported {
                    item,
                    clause: at,
                    need,
                });
            }
        }
    }
    found
}
