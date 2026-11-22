use std::collections::{BTreeSet, VecDeque};

use crate::{Set, Space};

#[derive(Clone, Debug)]
pub struct Child<'a> {
    space: &'a Space,
    kept: Set,
    base: Set,
    bottoms: Vec<Set>,
    top: Set,
}

impl<'a> Child<'a> {
    pub(crate) const fn new(
        space: &'a Space,
        kept: Set,
        base: Set,
        bottoms: Vec<Set>,
        top: Set,
    ) -> Self {
        Self {
            space,
            kept,
            base,
            bottoms,
            top,
        }
    }

    #[must_use]
    pub fn trace(&self) -> Set {
        self.base.intersection(&self.kept)
    }

    #[must_use]
    pub const fn base(&self) -> &Set {
        &self.base
    }

    #[must_use]
    pub fn bottoms(&self) -> &[Set] {
        &self.bottoms
    }

    #[must_use]
    pub const fn top(&self) -> &Set {
        &self.top
    }

    #[must_use]
    pub fn is_trivial(&self) -> bool {
        self.bottoms.len() == 1 && self.bottoms[0] == self.top
    }

    #[must_use]
    pub fn contains(&self, set: &Set) -> bool {
        let state = set.union(&self.base);
        set.intersection(&self.base).is_empty()
            && state.intersection(&self.kept) == self.trace()
            && self.space.is_state(&state)
    }

    #[must_use]
    pub fn states(&self, limit: usize) -> Option<Vec<Set>> {
        let free = self.space.domain().difference(&self.kept);
        let mut seen: BTreeSet<Set> = self.bottoms.iter().cloned().collect();
        if seen.len() > limit {
            return None;
        }
        let mut queue: VecDeque<Set> = self.bottoms.iter().cloned().collect();
        while let Some(state) = queue.pop_front() {
            for item in self.space.outer(&state).intersection(&free).iter() {
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
        Some(
            seen.into_iter()
                .map(|state| state.difference(&self.base))
                .collect::<BTreeSet<Set>>()
                .into_iter()
                .collect(),
        )
    }
}
