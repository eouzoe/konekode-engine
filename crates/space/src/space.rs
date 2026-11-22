use std::collections::{BTreeSet, VecDeque};

use crate::structure::{atoms_of, without};
use crate::{Finding, Set, check};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    NoClause { item: usize },
    SelfNeed { item: usize },
    Unknown { item: usize, need: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Space {
    clauses: Vec<Vec<Set>>,
    pub(crate) atoms: Vec<Vec<Set>>,
}

impl Space {
    pub fn new(rules: &[Vec<Set>]) -> Result<Self, Error> {
        match check::errors(rules).into_iter().next() {
            Some(Finding::NoClause { item }) => Err(Error::NoClause { item }),
            Some(Finding::SelfNeed { item, .. }) => Err(Error::SelfNeed { item }),
            Some(Finding::Unknown { item, need, .. }) => Err(Error::Unknown { item, need }),
            _ => Ok(Self::from_atoms(atoms_of(rules))),
        }
    }

    pub(crate) fn from_atoms(atoms: Vec<Vec<Set>>) -> Self {
        let clauses = atoms
            .iter()
            .enumerate()
            .map(|(item, list)| list.iter().map(|atom| without(atom, item)).collect())
            .collect();
        Self { clauses, atoms }
    }

    #[must_use]
    pub fn atoms(&self, item: usize) -> &[Set] {
        &self.atoms[item]
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.clauses.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    #[must_use]
    pub fn clauses(&self, item: usize) -> &[Set] {
        &self.clauses[item]
    }

    #[must_use]
    pub fn ready(&self, item: usize, state: &Set) -> bool {
        !state.contains(item)
            && self.clauses[item]
                .iter()
                .any(|clause| clause.is_subset(state))
    }

    #[must_use]
    pub fn outer(&self, state: &Set) -> Set {
        (0..self.len())
            .filter(|&item| self.ready(item, state))
            .collect()
    }

    #[must_use]
    pub fn inner(&self, state: &Set) -> Set {
        state
            .iter()
            .filter(|&item| {
                let mut rest = state.clone();
                rest.remove(item);
                self.is_state(&rest)
            })
            .collect()
    }

    #[must_use]
    pub fn grow(&self, start: &Set, within: Option<&Set>) -> Set {
        let mut state = start.clone();
        loop {
            let next: Vec<usize> = (0..self.len())
                .filter(|&item| {
                    within.is_none_or(|limit| limit.contains(item)) && self.ready(item, &state)
                })
                .collect();
            if next.is_empty() {
                return state;
            }
            for item in next {
                state.insert(item);
            }
        }
    }

    #[must_use]
    pub fn reachable(&self) -> Set {
        self.grow(&Set::new(), None)
    }

    #[must_use]
    pub fn is_state(&self, set: &Set) -> bool {
        set.iter().all(|item| item < self.len()) && self.grow(&Set::new(), Some(set)) == *set
    }

    #[must_use]
    pub fn states(&self, limit: usize) -> Option<Vec<Set>> {
        let mut seen = BTreeSet::from([Set::new()]);
        let mut queue = VecDeque::from([Set::new()]);
        while let Some(state) = queue.pop_front() {
            for item in self.outer(&state).iter() {
                let mut next = state.clone();
                next.insert(item);
                if seen.insert(next.clone()) {
                    if seen.len() > limit {
                        return None;
                    }
                    queue.push_back(next);
                }
            }
        }
        Some(seen.into_iter().collect())
    }

    #[must_use]
    pub fn levels(&self) -> Vec<Option<usize>> {
        let mut levels: Vec<Option<usize>> = vec![None; self.len()];
        let mut moved = true;
        while moved {
            moved = false;
            for item in 0..self.len() {
                let best = self.clauses[item]
                    .iter()
                    .filter_map(|clause| {
                        clause
                            .iter()
                            .map(|need| levels[need])
                            .collect::<Option<Vec<usize>>>()
                            .map(|found| found.into_iter().max().map_or(0, |top| top + 1))
                    })
                    .min();
                if let Some(level) = best
                    && levels[item].is_none_or(|old| level < old)
                {
                    levels[item] = Some(level);
                    moved = true;
                }
            }
        }
        levels
    }

    #[must_use]
    pub fn gap(&self, state: &Set, target: usize) -> Option<Set> {
        if state.contains(target) {
            return Some(Set::new());
        }
        self.atoms[target]
            .iter()
            .map(|atom| atom.difference(state))
            .min_by_key(Set::len)
    }
}

#[must_use]
pub fn project(states: &[Set], kept: &Set) -> Vec<Set> {
    let mut traces: Vec<Set> = states
        .iter()
        .map(|state| state.intersection(kept))
        .collect();
    traces.sort();
    traces.dedup();
    traces
}
