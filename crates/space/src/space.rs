use std::collections::{BTreeSet, VecDeque};

use crate::Set;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    NoClause { item: usize },
    SelfNeed { item: usize },
    Unknown { item: usize, need: usize },
}

#[derive(Clone, Debug)]
pub struct Space {
    clauses: Vec<Vec<Set>>,
}

impl Space {
    pub fn new(clauses: Vec<Vec<Set>>) -> Result<Self, Error> {
        let size = clauses.len();
        for (item, options) in clauses.iter().enumerate() {
            if options.is_empty() {
                return Err(Error::NoClause { item });
            }
            for clause in options {
                if clause.contains(item) {
                    return Err(Error::SelfNeed { item });
                }
                if let Some(need) = clause.iter().find(|&need| need >= size) {
                    return Err(Error::Unknown { item, need });
                }
            }
        }
        Ok(Self { clauses })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.clauses.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
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
        self.solve(target, state, &Set::new())
    }

    fn solve(&self, item: usize, have: &Set, open: &Set) -> Option<Set> {
        if have.contains(item) {
            return Some(Set::new());
        }
        if open.contains(item) {
            return None;
        }
        let mut inside = open.clone();
        inside.insert(item);
        let mut best: Option<Set> = None;
        for clause in &self.clauses[item] {
            let mut added = Some(Set::new());
            for need in clause.iter() {
                added = added.and_then(|sofar| {
                    self.solve(need, &have.union(&sofar), &inside)
                        .map(|more| sofar.union(&more))
                });
            }
            if let Some(found) = added
                && best.as_ref().is_none_or(|kept| found.len() < kept.len())
            {
                best = Some(found);
            }
        }
        best.map(|mut found| {
            found.insert(item);
            found
        })
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
