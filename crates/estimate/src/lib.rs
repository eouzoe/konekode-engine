use std::collections::BTreeMap;

use konekode_assess::{Belief, Noise, Response};
use konekode_learner::{Evidence, Observation, Tally};
use konekode_memory::{Memory, Weights, days_between};
use konekode_space::{Set, Space};

pub const LEVEL: f64 = 0.9;
pub const STEADY: f64 = 0.9;
pub const FRESH: f64 = 0.9;
pub const GAIN: f64 = 0.15;
pub const SHOWN_PRIOR: f64 = 0.5;
pub const UNSEEN_PRIOR: f64 = 0.1;
pub const LUCKY_FREE: f64 = 0.02;
pub const LUCKY_HELPED: f64 = 0.5;
pub const SETTLED_AFTER: i64 = 3;
pub const CARELESS_START: (f64, f64) = (1.0, 10.0);
pub const CARELESS_RANGE: (f64, f64) = (0.02, 0.3);
const PART_LIMIT: usize = 4096;

pub type Needs = BTreeMap<String, Vec<Vec<String>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Verdict {
    Untried,
    Learning,
    Unsure,
    Due,
    Known,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Estimate {
    pub id: String,
    pub known: f64,
    pub certain: bool,
    pub recall: Option<f64>,
    pub memory: Option<Memory>,
    pub reviews: i64,
    pub tries: i64,
    pub verdict: Verdict,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Estimates {
    pub careless: f64,
    pub components: Vec<Estimate>,
}

#[must_use]
pub fn careless(observations: &[Observation]) -> f64 {
    let mut rights: BTreeMap<&str, i64> = BTreeMap::new();
    let mut wrong = 0.0;
    let mut total = 0.0;
    for observation in observations
        .iter()
        .filter(|observation| !observation.helped)
    {
        let before = rights
            .get(observation.component.as_str())
            .copied()
            .unwrap_or(0);
        if before >= SETTLED_AFTER {
            total += 1.0;
            if !observation.right {
                wrong += 1.0;
            }
        }
        if observation.right {
            *rights.entry(observation.component.as_str()).or_default() += 1;
        }
    }
    ((CARELESS_START.0 + wrong) / (CARELESS_START.1 + total))
        .clamp(CARELESS_RANGE.0, CARELESS_RANGE.1)
}

fn find(parent: &mut [usize], at: usize) -> usize {
    let mut root = at;
    while parent[root] != root {
        root = parent[root];
    }
    parent[at] = root;
    root
}

fn groups(ids: &[String], needs: &Needs) -> Vec<Vec<String>> {
    let index: BTreeMap<&str, usize> = ids
        .iter()
        .enumerate()
        .map(|(at, id)| (id.as_str(), at))
        .collect();
    let mut parent: Vec<usize> = (0..ids.len()).collect();
    for (item, clauses) in needs {
        let Some(&from) = index.get(item.as_str()) else {
            continue;
        };
        for need in clauses.iter().flatten() {
            if let Some(&to) = index.get(need.as_str()) {
                let left = find(&mut parent, from);
                let right = find(&mut parent, to);
                parent[left] = right;
            }
        }
    }
    let mut grouped: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for (at, id) in ids.iter().enumerate() {
        let root = find(&mut parent, at);
        grouped.entry(root).or_default().push(id.clone());
    }
    grouped.into_values().collect()
}

fn space_of(items: &[String], needs: &Needs) -> Option<Space> {
    let local: BTreeMap<&str, usize> = items
        .iter()
        .enumerate()
        .map(|(at, id)| (id.as_str(), at))
        .collect();
    let clauses = items
        .iter()
        .map(|id| {
            let options: Vec<Set> = needs
                .get(id)
                .into_iter()
                .flatten()
                .map(|clause| {
                    clause
                        .iter()
                        .filter_map(|need| local.get(need.as_str()).copied())
                        .collect()
                })
                .collect();
            if options.is_empty() {
                vec![Set::new()]
            } else {
                options
            }
        })
        .collect();
    Space::new(clauses).ok()
}

struct Part {
    items: Vec<String>,
    belief: Belief,
}

fn prior(state: &Set, priors: &[f64]) -> f64 {
    priors
        .iter()
        .enumerate()
        .map(|(at, chance)| {
            if state.contains(at) {
                *chance
            } else {
                1.0 - chance
            }
        })
        .product()
}

fn part(items: Vec<String>, needs: &Needs, priors: &BTreeMap<String, f64>) -> Vec<Part> {
    let chances: Vec<f64> = items
        .iter()
        .map(|id| priors.get(id).copied().unwrap_or(UNSEEN_PRIOR))
        .collect();
    let states = space_of(&items, needs).and_then(|space| space.states(PART_LIMIT));
    let built = states.and_then(|states| {
        let weights = states.iter().map(|state| prior(state, &chances)).collect();
        Belief::weighted(states, weights).ok()
    });
    if let Some(belief) = built {
        return vec![Part { items, belief }];
    }
    if items.len() == 1 {
        return Vec::new();
    }
    items
        .into_iter()
        .flat_map(|id| part(vec![id], &Needs::new(), priors))
        .collect()
}

struct Located {
    part: usize,
    item: usize,
}

fn noise(careless: f64, helped: bool) -> Option<Noise> {
    Noise::new(careless, if helped { LUCKY_HELPED } else { LUCKY_FREE }).ok()
}

#[must_use]
pub fn estimate_evidence(evidence: &Evidence, needs: &Needs, now: i64) -> Estimates {
    estimate(&evidence.tallies(), evidence.observations(), needs, now)
}

#[must_use]
pub fn estimate(
    tallies: &[(String, Tally)],
    observations: &[Observation],
    needs: &Needs,
    now: i64,
) -> Estimates {
    let ids: Vec<String> = tallies.iter().map(|(id, _)| id.clone()).collect();
    let priors: BTreeMap<String, f64> = tallies
        .iter()
        .map(|(id, tally)| {
            let chance = if tally.shown > 0 {
                SHOWN_PRIOR
            } else {
                UNSEEN_PRIOR
            };
            (id.clone(), chance)
        })
        .collect();
    let mut parts: Vec<Part> = groups(&ids, needs)
        .into_iter()
        .flat_map(|items| part(items, needs, &priors))
        .collect();
    let mut where_is: BTreeMap<String, Located> = BTreeMap::new();
    for (at, built) in parts.iter().enumerate() {
        for (item, id) in built.items.iter().enumerate() {
            where_is.insert(id.clone(), Located { part: at, item });
        }
    }
    let careless = careless(observations);
    let mut tries: BTreeMap<&str, i64> = BTreeMap::new();
    let mut memories: BTreeMap<&str, Remembered> = BTreeMap::new();
    for observation in observations {
        let id = observation.component.as_str();
        *tries.entry(id).or_default() += 1;
        if let (Some(found), Some(noise)) = (where_is.get(id), noise(careless, observation.helped))
        {
            let belief = &mut parts[found.part].belief;
            belief.observe(
                Response {
                    item: found.item,
                    correct: observation.right,
                },
                noise,
            );
            belief.learn(found.item, GAIN);
        }
        if !observation.helped {
            let next = remember(memories.get(id), observation);
            memories.insert(id, next);
        }
    }
    let components = ids
        .iter()
        .map(|id| {
            let located = where_is
                .get(id)
                .map(|found| (&parts[found.part].belief, found.item));
            judge(
                id,
                located,
                memories.get(id.as_str()),
                tries.get(id.as_str()).copied().unwrap_or(0),
                careless,
                now,
            )
        })
        .collect();
    Estimates {
        careless,
        components,
    }
}

type Remembered = (Memory, i64, i64);

fn remember(before: Option<&Remembered>, observation: &Observation) -> Remembered {
    match before {
        None => (
            Memory::first(&Weights::BINARY, observation.right),
            observation.at,
            1,
        ),
        Some((memory, last, count)) => (
            memory.review(
                &Weights::BINARY,
                days_between(*last, observation.at),
                observation.right,
            ),
            observation.at,
            count + 1,
        ),
    }
}

fn survives_a_slip(belief: &Belief, item: usize, careless: f64) -> bool {
    let mut doubt = belief.clone();
    noise(careless, false).is_some_and(|noise| {
        doubt.observe(
            Response {
                item,
                correct: false,
            },
            noise,
        );
        doubt.marginal(item) >= STEADY
    })
}

fn judge(
    id: &str,
    located: Option<(&Belief, usize)>,
    remembered: Option<&Remembered>,
    attempts: i64,
    careless: f64,
    now: i64,
) -> Estimate {
    let known = located.map_or(UNSEEN_PRIOR, |(belief, item)| belief.marginal(item));
    let certain = located.is_some_and(|(belief, item)| survives_a_slip(belief, item, careless));
    let recall = remembered
        .map(|(memory, last, _)| memory.recall(&Weights::BINARY, days_between(*last, now)));
    let verdict = match recall {
        _ if attempts == 0 => Verdict::Untried,
        _ if known < LEVEL => Verdict::Learning,
        None => Verdict::Unsure,
        _ if !certain => Verdict::Unsure,
        Some(chance) if chance < FRESH => Verdict::Due,
        Some(_) => Verdict::Known,
    };
    Estimate {
        id: id.to_owned(),
        known,
        certain,
        recall,
        memory: remembered.map(|(memory, _, _)| *memory),
        reviews: remembered.map_or(0, |(_, _, count)| *count),
        tries: attempts,
        verdict,
    }
}
