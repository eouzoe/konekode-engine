use konekode_space::{Set, SkillMap, Space, project};

fn set(items: &[usize]) -> Set {
    items.iter().copied().collect()
}

fn subsets(pool: &[usize]) -> Vec<Set> {
    (0..1_u32 << pool.len())
        .map(|mask| {
            pool.iter()
                .enumerate()
                .filter(|(at, _)| (mask >> at) & 1 == 1)
                .map(|(_, &item)| item)
                .collect()
        })
        .collect()
}

fn families(others: &[usize]) -> Vec<Vec<Set>> {
    let all = subsets(others);
    (1..1_u32 << all.len())
        .map(|mask| {
            all.iter()
                .enumerate()
                .filter(|(at, _)| (mask >> at) & 1 == 1)
                .map(|(_, chosen)| chosen.clone())
                .collect::<Vec<Set>>()
        })
        .filter(|family| {
            family.iter().all(|one| {
                family
                    .iter()
                    .all(|other| one == other || !one.is_subset(other))
            })
        })
        .collect()
}

fn without(set: &Set, item: usize) -> Set {
    let mut rest = set.clone();
    rest.remove(item);
    rest
}

fn with(set: &Set, item: usize) -> Set {
    let mut more = set.clone();
    more.insert(item);
    more
}

fn check(space: &Space, size: usize) {
    let domain: Vec<usize> = (0..size).collect();
    let states: Vec<Set> = subsets(&domain)
        .into_iter()
        .filter(|candidate| space.is_state(candidate))
        .collect();
    let has = |candidate: &Set| states.contains(candidate);
    assert!(has(&Set::new()));
    let mut listed = states.clone();
    listed.sort();
    assert_eq!(space.states(usize::MAX), Some(listed));
    for left in &states {
        for right in &states {
            assert!(has(&left.union(right)), "union of {left:?} and {right:?}");
        }
        if !left.is_empty() {
            assert!(left.iter().any(|item| has(&without(left, item))));
        }
        let ready: Set = domain
            .iter()
            .copied()
            .filter(|&item| !left.contains(item) && has(&with(left, item)))
            .collect();
        assert_eq!(space.outer(left), ready);
    }
    let mut fringes: Vec<(Set, Set)> = states
        .iter()
        .map(|state| (space.inner(state), space.outer(state)))
        .collect();
    fringes.sort();
    fringes.dedup();
    assert_eq!(fringes.len(), states.len());
    for kept in subsets(&domain) {
        let traces = project(&states, &kept);
        for left in &traces {
            for right in &traces {
                assert!(traces.contains(&left.union(right)));
            }
            if !left.is_empty() {
                assert!(
                    left.iter()
                        .any(|item| traces.contains(&without(left, item)))
                );
            }
        }
    }
    for state in &states {
        for target in 0..size {
            let exact = states
                .iter()
                .filter(|other| other.contains(target) && state.is_subset(other))
                .map(|other| other.len() - state.len())
                .min();
            match space.gap(state, target) {
                None => assert!(exact.is_none()),
                Some(found) => {
                    let reached = state.union(&found);
                    assert!(reached.contains(target) && has(&reached));
                    let exact = exact.expect("a state above contains the target");
                    assert_eq!(reached.len() - state.len(), exact);
                }
            }
        }
    }
}

fn every_system(size: usize) -> usize {
    let domain: Vec<usize> = (0..size).collect();
    let options: Vec<Vec<Vec<Set>>> = domain
        .iter()
        .map(|&item| {
            let others: Vec<usize> = domain
                .iter()
                .copied()
                .filter(|&other| other != item)
                .collect();
            families(&others)
        })
        .collect();
    let total: usize = options.iter().map(Vec::len).product();
    for index in 0..total {
        let mut rest = index;
        let clauses: Vec<Vec<Set>> = options
            .iter()
            .map(|list| {
                let chosen = list[rest % list.len()].clone();
                rest /= list.len();
                chosen
            })
            .collect();
        let space = Space::new(&clauses).expect("every generated system is valid");
        check(&space, size);
    }
    total
}

#[test]
fn every_rule_system_on_three_items_is_a_learning_space() {
    assert_eq!(every_system(3), 125);
}

#[test]
fn every_rule_system_on_four_items_is_a_learning_space() {
    assert_eq!(every_system(4), 130_321);
}

#[test]
fn example_22_of_the_survey() {
    let space = Space::new(&[
        vec![set(&[])],
        vec![set(&[])],
        vec![set(&[0, 1]), set(&[1, 3])],
        vec![set(&[1])],
    ])
    .expect("valid");
    let mut expected = vec![
        set(&[]),
        set(&[0]),
        set(&[1]),
        set(&[0, 1]),
        set(&[1, 3]),
        set(&[0, 1, 3]),
        set(&[0, 1, 2]),
        set(&[1, 2, 3]),
        set(&[0, 1, 2, 3]),
    ];
    expected.sort();
    assert_eq!(space.states(100), Some(expected));
    assert_eq!(strings(&space, &Set::new(), 4), 6);
}

fn strings(space: &Space, done: &Set, goal: usize) -> usize {
    if done.len() == goal {
        return 1;
    }
    space
        .outer(done)
        .iter()
        .map(|item| strings(space, &with(done, item), goal))
        .sum()
}

#[test]
fn example_16_is_not_reachable_one_item_at_a_time() {
    let space = Space::new(&[
        vec![set(&[])],
        vec![set(&[3]), set(&[0, 2]), set(&[2, 4])],
        vec![set(&[0, 1]), set(&[1, 4])],
        vec![set(&[1])],
        vec![set(&[1, 2])],
    ])
    .expect("valid");
    assert_eq!(space.states(100), Some(vec![set(&[]), set(&[0])]));
}

#[test]
fn levels_and_errors() {
    let space =
        Space::new(&[vec![set(&[])], vec![set(&[0])], vec![set(&[1]), set(&[0])]]).expect("valid");
    assert_eq!(space.levels(), vec![Some(0), Some(1), Some(1)]);
    assert!(Space::new(&[vec![]]).is_err());
    assert!(Space::new(&[vec![set(&[0])]]).is_err());
    assert!(Space::new(&[vec![set(&[3])]]).is_err());
}

#[test]
fn a_problem_solved_two_ways_has_two_explanations() {
    let map = SkillMap::new(vec![vec![set(&[0])], vec![set(&[0, 1]), set(&[2])]]).expect("valid");
    assert_eq!(map.solvable(&set(&[2])), set(&[1]));
    assert_eq!(map.solvable(&set(&[0, 1])), set(&[0, 1]));
    let skill_states = subsets(&[0, 1, 2]);
    let mut expected = vec![set(&[2]), set(&[0, 1])];
    expected.sort();
    assert_eq!(map.explanations(&set(&[1]), &skill_states), expected);
}
