use crate::{Error, Set};

#[derive(Clone, Debug)]
pub struct SkillMap {
    competencies: Vec<Vec<Set>>,
}

impl SkillMap {
    pub fn new(competencies: Vec<Vec<Set>>) -> Result<Self, Error> {
        if let Some(item) = competencies.iter().position(Vec::is_empty) {
            return Err(Error::NoClause { item });
        }
        Ok(Self { competencies })
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.competencies.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.competencies.is_empty()
    }

    #[must_use]
    pub fn solvable(&self, skills: &Set) -> Set {
        self.competencies
            .iter()
            .enumerate()
            .filter(|(_, options)| options.iter().any(|option| option.is_subset(skills)))
            .map(|(problem, _)| problem)
            .collect()
    }

    #[must_use]
    pub fn explanations(&self, solved: &Set, skill_states: &[Set]) -> Vec<Set> {
        let fitting: Vec<&Set> = skill_states
            .iter()
            .filter(|state| solved.is_subset(&self.solvable(state)))
            .collect();
        let mut minimal: Vec<Set> = fitting
            .iter()
            .copied()
            .filter(|state| {
                !fitting
                    .iter()
                    .any(|other| other.len() < state.len() && other.is_subset(state))
            })
            .cloned()
            .collect();
        minimal.sort();
        minimal.dedup();
        minimal
    }
}
