use std::collections::BTreeMap;

use konekode_space::{Finding, Level, Set, Space, check};

type Mask = u32;

fn set(items: &[usize]) -> Set {
    items.iter().copied().collect()
}

fn to_set(mask: Mask) -> Set {
    (0..32).filter(|bit| mask >> bit & 1 == 1).collect()
}

fn to_mask(set: &Set) -> Mask {
    set.iter().map(|item| 1 << item).sum()
}

fn is_subset(small: Mask, big: Mask) -> bool {
    small & !big == 0
}

fn minimal(family: &[Mask]) -> Vec<Mask> {
    let mut kept: Vec<Mask> = family
        .iter()
        .copied()
        .filter(|&one| {
            !family
                .iter()
                .any(|&other| other != one && is_subset(other, one))
        })
        .collect();
    kept.sort_unstable();
    kept.dedup();
    kept
}

fn masks(sets: &[Set]) -> Vec<Mask> {
    let mut list: Vec<Mask> = sets.iter().map(to_mask).collect();
    list.sort_unstable();
    list
}

struct Truth {
    size: usize,
    states: Vec<Mask>,
}

impl Truth {
    fn of(clauses: &[Vec<Mask>]) -> Self {
        let size = clauses.len();
        let mut reach = vec![false; 1 << size];
        reach[0] = true;
        for mask in 1..1_usize << size {
            let mask = Mask::try_from(mask).expect("small");
            reach[mask as usize] = (0..size).any(|item| {
                let before = mask & !(1 << item);
                mask >> item & 1 == 1
                    && reach[before as usize]
                    && clauses[item]
                        .iter()
                        .any(|&clause| is_subset(clause, before))
            });
        }
        let states = (0..1 << size)
            .filter(|&mask| reach[mask as usize])
            .collect();
        Self { size, states }
    }

    fn from_states(size: usize, states: &[Mask]) -> Self {
        let mut states = states.to_vec();
        states.sort_unstable();
        Self { size, states }
    }

    fn has(&self, mask: Mask) -> bool {
        self.states.binary_search(&mask).is_ok()
    }

    fn atoms(&self, item: usize) -> Vec<Mask> {
        let holding: Vec<Mask> = self
            .states
            .iter()
            .copied()
            .filter(|state| state >> item & 1 == 1)
            .collect();
        minimal(&holding)
    }

    fn canonical(&self) -> Vec<Vec<Mask>> {
        (0..self.size)
            .map(|item| {
                let mut list: Vec<Mask> = self
                    .atoms(item)
                    .into_iter()
                    .map(|atom| atom & !(1 << item))
                    .collect();
                list.sort_by_key(|&mask| to_set(mask));
                list
            })
            .collect()
    }

    fn inner(&self, state: Mask) -> Mask {
        (0..self.size)
            .filter(|&item| state >> item & 1 == 1 && self.has(state & !(1 << item)))
            .map(|item| 1 << item)
            .sum()
    }

    fn outer(&self, state: Mask) -> Mask {
        (0..self.size)
            .filter(|&item| state >> item & 1 == 0 && self.has(state | 1 << item))
            .map(|item| 1 << item)
            .sum()
    }

    fn traces(&self, kept: Mask) -> Vec<Mask> {
        let mut list: Vec<Mask> = self.states.iter().map(|state| state & kept).collect();
        list.sort_unstable();
        list.dedup();
        list
    }

    fn class(&self, kept: Mask, trace: Mask) -> Vec<Mask> {
        self.states
            .iter()
            .copied()
            .filter(|state| state & kept == trace)
            .collect()
    }

    fn yielding(&self, kept: Mask) -> bool {
        self.traces(kept).into_iter().all(|trace| {
            let class = self.class(kept, trace);
            let base = class.iter().fold(Mask::MAX, |common, state| common & state);
            minimal(&class)
                .into_iter()
                .all(|bottom| (bottom & !base).count_ones() <= 1)
        })
    }
}

fn child_of(class: &[Mask]) -> (Mask, Vec<Mask>) {
    let base = class.iter().fold(Mask::MAX, |common, state| common & state);
    let mut child: Vec<Mask> = class.iter().map(|state| state & !base).collect();
    child.sort_unstable();
    child.dedup();
    (base, child)
}

fn is_learning_space(family: &[Mask]) -> bool {
    let has = |mask: Mask| family.contains(&mask);
    has(0)
        && family.iter().all(|&left| {
            family.iter().all(|&right| has(left | right))
                && (left == 0
                    || (0..32).any(|item| left >> item & 1 == 1 && has(left & !(1 << item))))
        })
}

fn is_well_graded_and_union_stable(family: &[Mask]) -> bool {
    let has = |mask: Mask| family.contains(&mask);
    family.iter().all(|&left| {
        family.iter().all(|&right| {
            has(left | right)
                && (!is_subset(left, right)
                    || left == right
                    || (0..32).any(|item| {
                        right >> item & 1 == 1 && left >> item & 1 == 0 && has(left | 1 << item)
                    }))
        })
    })
}

fn space_of(clauses: &[Vec<Mask>]) -> Space {
    Space::new(
        &clauses
            .iter()
            .map(|options| options.iter().map(|&mask| to_set(mask)).collect())
            .collect::<Vec<Vec<Set>>>(),
    )
    .expect("valid")
}

fn clause_masks(space: &Space) -> Vec<Vec<Mask>> {
    (0..space.len())
        .map(|item| space.clauses(item).iter().map(to_mask).collect())
        .collect()
}

fn audit(clauses: &[Vec<Mask>], kept_sets: &[Mask]) {
    let truth = Truth::of(clauses);
    let space = space_of(clauses);
    audit_canonical(clauses, &truth, &space);
    audit_fringes(&truth, &space);
    for &kept in kept_sets {
        audit_projection(&truth, &space, kept);
    }
}

fn audit_canonical(clauses: &[Vec<Mask>], truth: &Truth, space: &Space) {
    let size = truth.size;
    for item in 0..size {
        let atoms = space.atoms(item);
        assert_eq!(masks(atoms), truth.atoms(item), "atoms at {item}");
        for atom in atoms {
            let mask = to_mask(atom);
            let owners = (0..size)
                .filter(|&other| truth.atoms(other).contains(&mask))
                .count();
            assert_eq!(owners, 1, "theorem 19: atom {atom:?} belongs to one item");
        }
    }

    assert_eq!(clause_masks(space), truth.canonical());
    assert_eq!(Truth::of(&clause_masks(space)).states, truth.states);
    let mut shuffled: Vec<Vec<Mask>> = clauses
        .iter()
        .map(|list| list.iter().rev().copied().collect())
        .collect();
    for list in &mut shuffled {
        list.extend(list.clone());
    }
    assert_eq!(space_of(&shuffled), *space);
    let unreachable: Vec<Finding> = (0..size)
        .filter(|&item| truth.states.iter().all(|state| state >> item & 1 == 0))
        .map(|item| Finding::NoClause { item })
        .collect();
    let canonical_clauses: Vec<Vec<Set>> =
        (0..size).map(|item| space.clauses(item).to_vec()).collect();
    assert_eq!(check(&canonical_clauses), unreachable);
    if unreachable.is_empty() {
        assert_eq!(Space::new(&canonical_clauses).expect("valid"), *space);
    }
    assert_eq!(space.levels(), authored_levels(clauses));
    for &state in &truth.states {
        for target in 0..size {
            let exact = truth
                .states
                .iter()
                .filter(|&&other| other >> target & 1 == 1 && is_subset(state, other))
                .map(|other| (other & !state).count_ones())
                .min();
            let found = space.gap(&to_set(state), target).map(|gap| {
                assert!(truth.has(state | to_mask(&gap)));
                gap.len()
            });
            assert_eq!(found, exact.map(|count| count as usize));
        }
    }
}

fn authored_levels(clauses: &[Vec<Mask>]) -> Vec<Option<usize>> {
    let mut levels: Vec<Option<usize>> = vec![None; clauses.len()];
    let mut moved = true;
    while moved {
        moved = false;
        for (item, options) in clauses.iter().enumerate() {
            let best = options
                .iter()
                .filter_map(|&clause| {
                    (0..clauses.len())
                        .filter(|need| clause >> need & 1 == 1)
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

fn audit_fringes(truth: &Truth, space: &Space) {
    let full: Mask = (1 << truth.size) - 1;
    let mut outers: Vec<Mask> = truth
        .states
        .iter()
        .map(|&state| truth.outer(state))
        .collect();
    outers.sort_unstable();
    outers.dedup();
    assert_eq!(
        outers.len(),
        truth.states.len(),
        "the outer fringe alone names the state"
    );
    for &state in &truth.states {
        let inner = to_set(truth.inner(state));
        let outer = to_set(truth.outer(state));
        assert_eq!(space.from_fringes(&inner, &outer), Some(to_set(state)));
    }
    for state in 0..=full {
        if !truth.has(state) {
            let inner = to_set(truth.inner(state));
            let outer = to_set(truth.outer(state));
            let found = space
                .from_fringes(&inner, &outer)
                .map(|found| to_mask(&found));
            let expected = truth.states.iter().copied().find(|&other| {
                truth.inner(other) == truth.inner(state) && truth.outer(other) == truth.outer(state)
            });
            assert_eq!(found, expected);
        }
    }
}

fn audit_projection(truth: &Truth, space: &Space, kept: Mask) {
    let full: Mask = (1 << truth.size) - 1;
    let kept_set = to_set(kept);
    let projected = space.projection(&kept_set);
    let traces = projected.states(usize::MAX).expect("no limit");
    assert_eq!(masks(&traces), truth.traces(kept));
    assert!(is_learning_space(&truth.traces(kept)), "theorem 33 (i)");
    let mut plus_ok = true;
    for trace in truth.traces(kept) {
        let class = truth.class(kept, trace);
        let (base, child) = child_of(&class);
        assert!(is_well_graded_and_union_stable(&child), "theorem 33 (ii)");
        let found = space
            .child(&kept_set, &to_set(trace))
            .expect("trace has a child");
        assert_eq!(to_mask(found.base()), base);
        assert_eq!(masks(found.bottoms()), minimal(&class));
        assert_eq!(
            to_mask(found.top()),
            class.iter().fold(0, |all, state| all | state)
        );
        assert_eq!(to_mask(&found.trace()), trace);
        assert_eq!(found.is_trivial(), child == [0]);
        assert_eq!(masks(&found.states(usize::MAX).expect("no limit")), child);
        for candidate in 0..=full {
            assert_eq!(
                found.contains(&to_set(candidate)),
                child.contains(&candidate)
            );
        }
        if child != [0] {
            let mut plus = child.clone();
            if !plus.contains(&0) {
                plus.push(0);
            }
            plus_ok &= is_learning_space(&plus);
        }
    }
    for trace in 0..=full {
        if trace & !kept == 0 && !truth.traces(kept).contains(&trace) {
            assert!(space.child(&kept_set, &to_set(trace)).is_none());
        }
    }
    assert_eq!(truth.yielding(kept), plus_ok, "theorem 16");
    assert_eq!(
        space.yielding(&kept_set, usize::MAX),
        Some(truth.yielding(kept))
    );
}

fn audit_check(clauses: &[Vec<Mask>]) {
    let truth = Truth::of(clauses);
    let size = clauses.len();
    let reach: Mask = truth.states.iter().fold(0, |all, state| all | state);
    let mut expected = Vec::new();
    for item in 0..size {
        if reach >> item & 1 == 0 {
            expected.push(Finding::Unreachable { item });
        }
    }
    let mut kept_rules: Vec<Vec<Mask>> = vec![Vec::new(); size];
    for (item, options) in clauses.iter().enumerate() {
        let atoms = truth.atoms(item);
        let fired: Vec<Vec<Mask>> = options
            .iter()
            .map(|&clause| {
                let above: Vec<Mask> = truth
                    .states
                    .iter()
                    .copied()
                    .filter(|&state| is_subset(clause, state) && state >> item & 1 == 0)
                    .map(|state| state | 1 << item)
                    .collect();
                minimal(&above)
            })
            .collect();
        let real: Vec<Vec<Mask>> = fired
            .iter()
            .map(|list| {
                list.iter()
                    .copied()
                    .filter(|atom| atoms.contains(atom))
                    .collect()
            })
            .collect();
        let mut active: Vec<usize> = (0..options.len()).collect();
        for (at, &clause) in options.iter().enumerate() {
            if fired[at].is_empty() {
                expected.push(Finding::Dead { item, clause: at });
                active.retain(|&other| other != at);
            } else if real[at].iter().all(|atom| {
                active
                    .iter()
                    .any(|&other| other != at && real[other].contains(atom))
            }) {
                expected.push(Finding::Covered { item, clause: at });
                active.retain(|&other| other != at);
            } else if real[at] != [clause | 1 << item] {
                expected.push(Finding::Expanded {
                    item,
                    clause: at,
                    into: real[at]
                        .iter()
                        .map(|&atom| to_set(atom & !(1 << item)))
                        .collect(),
                });
                kept_rules[item].extend(real[at].iter().map(|&atom| atom & !(1 << item)));
            } else {
                kept_rules[item].push(clause);
            }
        }
    }
    let input: Vec<Vec<Set>> = clauses
        .iter()
        .map(|options| options.iter().map(|&mask| to_set(mask)).collect())
        .collect();
    let mut found = check(&input);
    found.sort();
    expected.sort();
    assert_eq!(found, expected);
    assert_eq!(
        Truth::of(&kept_rules).states,
        truth.states,
        "dropping and rewriting keeps the structure"
    );
}

fn subsets_of(pool: Mask) -> Vec<Mask> {
    (0..=pool).filter(|&mask| is_subset(mask, pool)).collect()
}

fn families(item: usize, size: usize, antichains: bool) -> Vec<Vec<Mask>> {
    let full: Mask = (1 << size) - 1;
    let all = subsets_of(full & !(1 << item));
    (1..1_u64 << all.len())
        .map(|mask| {
            all.iter()
                .enumerate()
                .filter(|(at, _)| mask >> at & 1 == 1)
                .map(|(_, &chosen)| chosen)
                .collect::<Vec<Mask>>()
        })
        .filter(|family| !antichains || minimal(family).len() == family.len())
        .collect()
}

fn every_system(size: usize, antichains: bool, mut visit: impl FnMut(&[Vec<Mask>])) -> usize {
    let options: Vec<Vec<Vec<Mask>>> = (0..size)
        .map(|item| families(item, size, antichains))
        .collect();
    let total: usize = options.iter().map(Vec::len).product();
    for index in 0..total {
        let mut rest = index;
        let clauses: Vec<Vec<Mask>> = options
            .iter()
            .map(|list| {
                let chosen = list[rest % list.len()].clone();
                rest /= list.len();
                chosen
            })
            .collect();
        visit(&clauses);
    }
    total
}

struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % bound as u64).expect("small")
    }

    fn system(&mut self, size: usize) -> Vec<Vec<Mask>> {
        (0..size)
            .map(|item| {
                (0..=self.below(3))
                    .map(|_| {
                        let width = self.below(4);
                        (0..width)
                            .map(|_| self.below(size))
                            .filter(|&need| need != item)
                            .fold(0, |mask, need| mask | 1 << need)
                    })
                    .collect()
            })
            .collect()
    }
}

#[test]
fn every_rule_system_on_three_items_with_redundant_clauses() {
    let all: Vec<Mask> = subsets_of(0b111);
    let mut groups: BTreeMap<Vec<Mask>, Space> = BTreeMap::new();
    let total = every_system(3, false, |clauses| {
        audit(clauses, &all);
        audit_check(clauses);
        let canonical = space_of(clauses);
        let states = Truth::of(clauses).states;
        let kept = groups.entry(states).or_insert_with(|| canonical.clone());
        assert_eq!(*kept, canonical, "one canonical form per structure");
    });
    assert_eq!(total, 3375);
    let distinct: std::collections::BTreeSet<Vec<Vec<Mask>>> =
        groups.values().map(clause_masks).collect();
    assert_eq!(distinct.len(), groups.len());
}

#[test]
fn every_antichain_rule_system_on_four_items() {
    let all: Vec<Mask> = subsets_of(0b1111);
    let mut groups: BTreeMap<Vec<Mask>, Vec<Vec<Mask>>> = BTreeMap::new();
    let total = every_system(4, true, |clauses| {
        audit(clauses, &all);
        audit_check(clauses);
        let canonical = clause_masks(&space_of(clauses));
        let kept = groups
            .entry(Truth::of(clauses).states)
            .or_insert_with(|| canonical.clone());
        assert_eq!(*kept, canonical, "one canonical form per structure");
    });
    assert_eq!(total, 130_321);
}

#[test]
fn sampled_rule_systems_on_five_to_eight_items() {
    let mut random = Random(19);
    for size in 5..=8 {
        let full: Mask = (1 << size) - 1;
        for _ in 0..1000 {
            let clauses = random.system(size);
            let kept: Vec<Mask> = (0..4)
                .map(|_| Mask::try_from(random.below(1 << size)).expect("small") & full)
                .chain([0, full])
                .collect();
            audit(&clauses, &kept);
            audit_check(&clauses);
        }
    }
}

#[test]
fn findings_carry_their_level() {
    let found = check(&[vec![], vec![set(&[1])], vec![set(&[5])]]);
    assert_eq!(
        found,
        vec![
            Finding::NoClause { item: 0 },
            Finding::SelfNeed { item: 1, clause: 0 },
            Finding::Unknown {
                item: 2,
                clause: 0,
                need: 5
            },
        ]
    );
    assert!(found.iter().all(|finding| finding.level() == Level::Error));
    let found = check(&[
        vec![set(&[]), set(&[1])],
        vec![set(&[2])],
        vec![set(&[1])],
        vec![set(&[4])],
        vec![set(&[0])],
        vec![set(&[0]), set(&[0, 4])],
        vec![set(&[0]), set(&[0])],
    ]);
    let expanded = Finding::Expanded {
        item: 3,
        clause: 0,
        into: vec![set(&[0, 4])],
    };
    assert_eq!(
        found,
        vec![
            Finding::Dead { item: 0, clause: 1 },
            Finding::Unreachable { item: 1 },
            Finding::Dead { item: 1, clause: 0 },
            Finding::Unreachable { item: 2 },
            Finding::Dead { item: 2, clause: 0 },
            expanded.clone(),
            Finding::Covered { item: 5, clause: 1 },
            Finding::Covered { item: 6, clause: 0 },
        ]
    );
    assert_eq!(
        Finding::Covered { item: 5, clause: 1 }.level(),
        Level::Warning
    );
    assert_eq!(expanded.level(), Level::Hint);
}

#[test]
fn example_22_of_the_survey_is_its_own_canonical_form() {
    let space = Space::new(&[
        vec![set(&[])],
        vec![set(&[])],
        vec![set(&[0, 1]), set(&[1, 3])],
        vec![set(&[1])],
    ])
    .expect("valid");
    let clauses: Vec<Vec<Set>> = (0..4).map(|item| space.clauses(item).to_vec()).collect();
    assert_eq!(clauses[2], vec![set(&[0, 1]), set(&[1, 3])]);
    assert_eq!(check(&clauses), Vec::new());
    assert_eq!(
        space.from_fringes(&set(&[0, 3]), &set(&[2])),
        Some(set(&[0, 1, 3]))
    );
    assert_eq!(space.from_fringes(&set(&[0]), &set(&[2])), None);
}

#[test]
fn example_16_of_the_survey_is_not_a_learning_space() {
    let states: Vec<Mask> = [
        &[][..],
        &[0],
        &[1, 3],
        &[0, 1, 2],
        &[0, 1, 3],
        &[1, 2, 4],
        &[0, 1, 2, 3],
        &[0, 1, 2, 4],
        &[1, 2, 3, 4],
        &[0, 1, 2, 3, 4],
    ]
    .iter()
    .map(|items| to_mask(&set(items)))
    .collect();
    let truth = Truth::from_states(5, &states);
    let table: Vec<Vec<Mask>> = vec![
        vec![to_mask(&set(&[0]))],
        masks(&[set(&[1, 3]), set(&[0, 1, 2]), set(&[1, 2, 4])]),
        masks(&[set(&[0, 1, 2]), set(&[1, 2, 4])]),
        vec![to_mask(&set(&[1, 3]))],
        vec![to_mask(&set(&[1, 2, 4]))],
    ];
    for (item, atoms) in table.iter().enumerate() {
        assert_eq!(truth.atoms(item), *atoms, "table 3, item {item}");
    }
    assert!(truth.atoms(1).contains(&0b111) && truth.atoms(2).contains(&0b111));
    assert!(!is_learning_space(&states));
}

#[test]
fn example_29_of_the_survey() {
    let space = Space::new(&[
        vec![set(&[])],
        vec![set(&[])],
        vec![set(&[0, 3])],
        vec![set(&[0])],
    ])
    .expect("valid");
    let expected = [
        &[][..],
        &[0],
        &[1],
        &[0, 1],
        &[0, 3],
        &[0, 1, 3],
        &[0, 2, 3],
        &[0, 1, 2, 3],
    ];
    let mut listed: Vec<Set> = expected.iter().map(|items| set(items)).collect();
    listed.sort();
    assert_eq!(space.states(100), Some(listed));
    let kept = set(&[2, 3]);
    let mut traces = vec![set(&[]), set(&[3]), set(&[2, 3])];
    traces.sort();
    assert_eq!(space.projection(&kept).states(100), Some(traces));
    let table = [
        (set(&[2, 3]), set(&[0, 2, 3]), vec![set(&[]), set(&[1])]),
        (set(&[3]), set(&[0, 3]), vec![set(&[]), set(&[1])]),
        (
            set(&[]),
            set(&[]),
            vec![set(&[]), set(&[0]), set(&[1]), set(&[0, 1])],
        ),
    ];
    for (trace, base, mut child) in table {
        let found = space.child(&kept, &trace).expect("a class");
        assert_eq!(*found.base(), base);
        child.sort();
        assert_eq!(found.states(100), Some(child));
    }
    assert_eq!(space.yielding(&kept, 100), Some(true));
}

#[test]
fn example_30_of_the_survey_has_a_child_without_the_empty_set() {
    let space = Space::new(&[
        vec![set(&[1, 2]), set(&[1, 3])],
        vec![set(&[])],
        vec![set(&[1])],
        vec![set(&[1])],
    ])
    .expect("valid");
    let found = space.child(&set(&[0]), &set(&[0])).expect("a class");
    let mut child = vec![set(&[2]), set(&[3]), set(&[2, 3])];
    child.sort();
    assert_eq!(found.states(100), Some(child));
    assert!(!found.contains(&set(&[])));
}

#[test]
fn example_5_of_projections_of_a_learning_space() {
    let named = |word: &str| -> Set {
        word.bytes()
            .map(|letter| usize::from(letter - b'a'))
            .collect()
    };
    let family = [
        "", "b", "c", "ab", "ac", "bc", "bd", "abc", "abd", "bcd", "bce", "bdf", "abcd", "abce",
        "bcde", "bcdf", "bcef", "abdf", "abcde", "abcdf", "abcef", "bcdef", "abcdef", "abcdefg",
    ];
    let states: Vec<Mask> = family.iter().map(|word| to_mask(&named(word))).collect();
    let truth = Truth::from_states(7, &states);
    assert!(is_learning_space(&truth.states));
    let space = space_of(&truth.canonical());
    assert_eq!(masks(&space.states(1000).expect("small")), truth.states);
    let kept = named("adf");
    assert_eq!(space.projection(&kept).states(100).expect("small").len(), 8);
    let children = [
        ("bce", vec!["", "b", "c", "bc", "bce"]),
        ("abce", vec!["b", "c", "bc", "bce"]),
        ("bcde", vec!["", "c", "ce"]),
        ("bcdef", vec!["", "c", "ce"]),
        ("abcde", vec!["", "c", "ce"]),
        ("abcdefg", vec!["", "c", "ce", "ceg"]),
        ("bcef", vec![""]),
        ("abcef", vec![""]),
    ];
    for (state, words) in children {
        let trace = named(state).intersection(&kept);
        let found = space.child(&kept, &trace).expect("a class");
        let mut expected: Vec<Set> = words.iter().map(|word| named(word)).collect();
        expected.sort();
        assert_eq!(found.states(100), Some(expected), "child of {state}");
    }
    assert_eq!(space.yielding(&kept, 100), Some(true));
}

#[test]
fn a_large_structure_is_answered_from_its_rules() {
    let size = 120;
    let clauses: Vec<Vec<Set>> = (0..size)
        .map(|item| {
            if item % 2 == 0 {
                vec![set(&[])]
            } else if item + 1 < size {
                vec![set(&[item - 1]), set(&[item + 1])]
            } else {
                vec![set(&[item - 1])]
            }
        })
        .collect();
    let space = Space::new(&clauses).expect("valid");
    assert_eq!(space.states(10_000), None);
    assert_eq!(check(&clauses), Vec::new());
    let canonical = &space;
    for (item, options) in clauses.iter().enumerate() {
        let mut sorted = options.clone();
        sorted.sort();
        assert_eq!(canonical.clauses(item), sorted.as_slice());
    }
    let evens: Set = (0..size).step_by(2).collect();
    let odds: Set = (1..size).step_by(2).collect();
    assert_eq!(space.from_fringes(&evens, &odds), Some(evens.clone()));
    let projected = space.projection(&evens);
    for item in 0..size {
        let expected = if item % 2 == 0 {
            vec![Set::new()]
        } else {
            Vec::new()
        };
        assert_eq!(projected.clauses(item), expected.as_slice());
    }
    let child = space.child(&evens, &evens).expect("a class");
    assert_eq!(*child.base(), evens);
    assert_eq!(child.bottoms(), std::slice::from_ref(&evens));
    assert_eq!(*child.top(), space.domain());
    assert!(child.contains(&set(&[1, 3])));
    assert!(!child.contains(&set(&[0])));
    assert_eq!(space.yielding(&evens, 10_000), None);
    let few: Set = (0..6).collect();
    assert_eq!(space.yielding(&few, 10_000), Some(true));
}
