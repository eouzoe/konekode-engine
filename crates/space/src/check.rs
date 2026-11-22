use crate::structure::{generated, with, without};
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
    Dead {
        item: usize,
        clause: usize,
    },
    Covered {
        item: usize,
        clause: usize,
    },
    Expanded {
        item: usize,
        clause: usize,
        into: Vec<Set>,
    },
}

impl Finding {
    #[must_use]
    pub const fn level(&self) -> Level {
        match self {
            Self::NoClause { .. } | Self::SelfNeed { .. } | Self::Unknown { .. } => Level::Error,
            Self::Unreachable { .. } | Self::Dead { .. } | Self::Covered { .. } => Level::Warning,
            Self::Expanded { .. } => Level::Hint,
        }
    }
}

pub fn errors(rules: &[Vec<Set>]) -> Vec<Finding> {
    let size = rules.len();
    let mut found = Vec::new();
    for (item, options) in rules.iter().enumerate() {
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
pub fn check(rules: &[Vec<Set>]) -> Vec<Finding> {
    let mut found = errors(rules);
    if !found.is_empty() {
        return found;
    }
    let Ok(space) = Space::new(rules) else {
        return found;
    };
    for (item, options) in rules.iter().enumerate() {
        if space.atoms[item].is_empty() {
            found.push(Finding::Unreachable { item });
        }
        let real: Vec<Vec<Set>> = options
            .iter()
            .map(|clause| {
                generated(&space.atoms, item, clause)
                    .into_iter()
                    .filter(|atom| space.atoms[item].contains(atom))
                    .collect()
            })
            .collect();
        let mut active: Vec<usize> = (0..options.len()).collect();
        for (at, clause) in options.iter().enumerate() {
            if generated(&space.atoms, item, clause).is_empty() {
                found.push(Finding::Dead { item, clause: at });
                active.retain(|&other| other != at);
            } else if real[at].iter().all(|atom| {
                active
                    .iter()
                    .any(|&other| other != at && real[other].contains(atom))
            }) {
                found.push(Finding::Covered { item, clause: at });
                active.retain(|&other| other != at);
            } else if real[at] != [with(clause, item)] {
                found.push(Finding::Expanded {
                    item,
                    clause: at,
                    into: real[at].iter().map(|atom| without(atom, item)).collect(),
                });
            }
        }
    }
    found
}
