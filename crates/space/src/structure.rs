use crate::{Child, Set, Space};

pub(crate) fn keep_minimal(list: &mut Vec<Set>, candidate: Set) -> bool {
    if list.iter().any(|kept| kept.is_subset(&candidate)) {
        return false;
    }
    list.retain(|kept| !candidate.is_subset(kept));
    list.push(candidate);
    true
}

pub(crate) fn unions(items: &Set, atoms: &[Vec<Set>], allow: impl Fn(&Set) -> bool) -> Vec<Set> {
    let mut partial = vec![Set::new()];
    for need in items.iter() {
        let mut next = Vec::new();
        for sofar in &partial {
            if sofar.contains(need) {
                keep_minimal(&mut next, sofar.clone());
                continue;
            }
            for atom in atoms[need].iter().filter(|atom| allow(atom)) {
                keep_minimal(&mut next, sofar.union(atom));
            }
        }
        partial = next;
        if partial.is_empty() {
            break;
        }
    }
    partial.sort();
    partial
}

fn with(set: &Set, item: usize) -> Set {
    let mut more = set.clone();
    more.insert(item);
    more
}

fn without(set: &Set, item: usize) -> Set {
    let mut rest = set.clone();
    rest.remove(item);
    rest
}

impl Space {
    #[must_use]
    pub fn domain(&self) -> Set {
        (0..self.len()).collect()
    }

    #[must_use]
    pub fn atoms(&self) -> Vec<Vec<Set>> {
        let mut atoms: Vec<Vec<Set>> = vec![Vec::new(); self.len()];
        let mut moved = true;
        while moved {
            moved = false;
            for item in 0..self.len() {
                for clause in &self.clauses[item] {
                    for union in unions(clause, &atoms, |atom| !atom.contains(item)) {
                        moved |= keep_minimal(&mut atoms[item], with(&union, item));
                    }
                }
            }
        }
        for list in &mut atoms {
            list.sort();
        }
        atoms
    }

    #[must_use]
    pub fn canonical(&self) -> Self {
        let clauses = self
            .atoms()
            .into_iter()
            .enumerate()
            .map(|(item, list)| list.iter().map(|atom| without(atom, item)).collect())
            .collect();
        Self::trusted(clauses)
    }

    #[must_use]
    pub fn projection(&self, kept: &Set) -> Self {
        let clauses = self
            .atoms()
            .into_iter()
            .enumerate()
            .map(|(item, list)| {
                let mut traces = Vec::new();
                if kept.contains(item) {
                    for atom in &list {
                        keep_minimal(&mut traces, without(&atom.intersection(kept), item));
                    }
                }
                traces.sort();
                traces
            })
            .collect();
        Self::trusted(clauses)
    }

    #[must_use]
    pub fn from_fringes(&self, inner: &Set, outer: &Set) -> Option<Set> {
        let state = self.grow(&Set::new(), Some(&self.domain().difference(outer)));
        (self.inner(&state) == *inner && self.outer(&state) == *outer).then_some(state)
    }

    #[must_use]
    pub fn child(&self, kept: &Set, trace: &Set) -> Option<Child<'_>> {
        self.child_with(&self.atoms(), kept, trace)
    }

    pub(crate) fn child_with(
        &self,
        atoms: &[Vec<Set>],
        kept: &Set,
        trace: &Set,
    ) -> Option<Child<'_>> {
        let kept = kept.intersection(&self.domain());
        if !trace.is_subset(&kept) {
            return None;
        }
        let bottoms = unions(trace, atoms, |atom| {
            atom.intersection(&kept).is_subset(trace)
        });
        let base = bottoms
            .iter()
            .skip(1)
            .fold(bottoms.first()?.clone(), |common, bottom| {
                common.intersection(bottom)
            });
        let top = self.grow(
            &Set::new(),
            Some(&trace.union(&self.domain().difference(&kept))),
        );
        Some(Child::new(self, kept, base, bottoms, top))
    }

    #[must_use]
    pub fn yielding(&self, kept: &Set, limit: usize) -> Option<bool> {
        let atoms = self.atoms();
        let traces = self.projection(kept).states(limit)?;
        Some(traces.iter().all(|trace| {
            self.child_with(&atoms, kept, trace).is_some_and(|child| {
                child
                    .bottoms()
                    .iter()
                    .all(|bottom| bottom.difference(child.base()).len() <= 1)
            })
        }))
    }
}
