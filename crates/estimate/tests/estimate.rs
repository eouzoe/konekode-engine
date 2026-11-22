#![expect(
    clippy::expect_used,
    reason = "a test stops at the first broken assumption"
)]

use konekode_estimate::{Needs, Verdict, careless, estimate};
use konekode_learner::{Observation, Tally};

const DAY: i64 = 86_400_000;

fn close(got: f64, want: f64) {
    assert!((got - want).abs() < 1e-9, "got {got}, want {want}");
}

fn shown(id: &str) -> (String, Tally) {
    (
        id.to_owned(),
        Tally {
            shown: 1,
            ..Tally::default()
        },
    )
}

fn unseen(id: &str) -> (String, Tally) {
    (id.to_owned(), Tally::default())
}

fn tried(component: &str, at: i64, right: bool, helped: bool) -> Observation {
    Observation {
        at,
        component: component.to_owned(),
        helped,
        opportunity: 1,
        right,
        session: "s".to_owned(),
        step: 0.0,
        task: "t".to_owned(),
        unit: "u".to_owned(),
        waited: 0,
    }
}

fn free(component: &str, at: i64, right: bool) -> Observation {
    tried(component, at, right, false)
}

#[test]
fn with_no_answers_the_course_position_is_the_estimate() {
    let result = estimate(&[shown("a"), unseen("b")], &[], &Needs::new(), 0);
    close(result.careless, 0.1);
    let [a, b] = result.components.as_slice() else {
        panic!("two components");
    };
    close(a.known, 0.5);
    close(b.known, 0.1);
    assert_eq!(a.verdict, Verdict::Untried);
    assert_eq!(b.verdict, Verdict::Untried);
    assert_eq!(a.recall, None);
}

#[test]
fn one_answer_of_his_own_is_only_provisional() {
    let result = estimate(&[shown("a")], &[free("a", 0, true)], &Needs::new(), DAY);
    let a = &result.components[0];
    close(a.known, 0.981_521_739_130_434_8);
    assert!(!a.certain);
    assert_eq!(a.verdict, Verdict::Unsure);
    close(a.recall.expect("reviewed"), 0.944_266_798_779_931);
    assert_eq!((a.tries, a.reviews), (1, 1));
}

#[test]
fn a_second_answer_confirms_it() {
    let answers = [free("a", 0, true), free("a", DAY, true)];
    let result = estimate(&[shown("a")], &answers, &Needs::new(), 3 * DAY);
    let a = &result.components[0];
    close(a.known, 0.999_644_543_933_877_8);
    assert!(a.certain);
    close(a.recall.expect("reviewed"), 0.948_599_225_622_967_2);
    assert_eq!(a.verdict, Verdict::Known);
}

#[test]
fn what_he_knows_comes_due_as_time_passes() {
    let answers = [free("a", 0, true), free("a", DAY, true)];
    let result = estimate(&[shown("a")], &answers, &Needs::new(), 61 * DAY);
    let a = &result.components[0];
    close(a.recall.expect("reviewed"), 0.858_073_499_562_775_3);
    assert_eq!(a.verdict, Verdict::Due);
}

#[test]
fn a_guided_answer_is_weak_evidence_and_not_a_retrieval() {
    let result = estimate(
        &[shown("a")],
        &[tried("a", 0, true, true)],
        &Needs::new(),
        DAY,
    );
    let a = &result.components[0];
    close(a.known, 0.696_428_571_428_571_5);
    assert_eq!(a.verdict, Verdict::Learning);
    assert_eq!((a.tries, a.reviews), (1, 0));
    assert_eq!(a.recall, None);
}

#[test]
fn carelessness_is_read_from_slips_on_what_he_already_has() {
    let answers = [
        free("a", 0, true),
        free("a", 1, true),
        free("a", 2, true),
        free("a", 3, false),
        free("a", 4, true),
        tried("a", 5, false, true),
        free("b", 6, false),
    ];
    close(careless(&answers), 2.0 / 12.0);
    close(careless(&[]), 0.1);
    let slips: Vec<Observation> = (0..40).map(|at| free("a", at, at < 3)).collect();
    close(careless(&slips), 0.3);
    let steady: Vec<Observation> = (0..400).map(|at| free("a", at, true)).collect();
    close(careless(&steady), 0.02);
}

#[test]
fn evidence_on_a_later_item_lifts_what_it_needs() {
    let needs = Needs::from([("b".to_owned(), vec![vec!["a".to_owned()]])]);
    let tallies = [unseen("a"), unseen("b")];
    let before = estimate(&tallies, &[], &needs, 0);
    let after = estimate(&tallies, &[free("b", 0, true)], &needs, DAY);
    close(before.components[0].known, 0.1 / 0.91);
    close(after.components[0].known, 0.4);
    assert!(after.components[1].known <= after.components[0].known);
    let alone = estimate(&tallies, &[free("b", 0, true)], &Needs::new(), DAY);
    close(alone.components[0].known, 0.1);
}

fn histories(length: u32) -> Vec<Vec<(bool, bool)>> {
    (0..4_u32.pow(length))
        .map(|code| {
            (0..length)
                .map(|at| {
                    let pair = (code >> (2 * at)) & 3;
                    (pair & 1 == 1, pair & 2 == 2)
                })
                .collect()
        })
        .collect()
}

fn played(history: &[(bool, bool)]) -> Vec<Observation> {
    history
        .iter()
        .enumerate()
        .map(|(at, (right, helped))| {
            let at = i64::try_from(at).expect("small") * DAY;
            tried("a", at, *right, *helped)
        })
        .collect()
}

#[test]
fn every_short_history_stays_a_probability_and_rewards_getting_it_right() {
    for length in 0..=5 {
        for history in histories(length) {
            let now = 10 * DAY;
            let result = estimate(&[shown("a")], &played(&history), &Needs::new(), now);
            let a = &result.components[0];
            assert!((0.0..=1.0).contains(&a.known), "{history:?}");
            if let Some(recall) = a.recall {
                assert!(recall > 0.0 && recall < 1.0, "{history:?}");
            }
            let free = history.iter().filter(|(_, helped)| !helped).count();
            assert_eq!(
                a.reviews,
                i64::try_from(free).expect("small"),
                "{history:?}"
            );
            assert_eq!(a.tries, i64::from(length));
            for flip in 0..history.len() {
                if history[flip].0 {
                    continue;
                }
                let mut better = history.clone();
                better[flip].0 = true;
                let improved = estimate(&[shown("a")], &played(&better), &Needs::new(), now);
                assert!(
                    improved.components[0].known >= a.known - 1e-12,
                    "{history:?} -> {better:?}"
                );
            }
            let mut longer = history.clone();
            longer.push((true, false));
            let more = estimate(&[shown("a")], &played(&longer), &Needs::new(), now);
            assert!(more.components[0].known >= a.known - 1e-12, "{history:?}");
        }
    }
}

#[test]
fn the_prior_comes_from_the_course_not_from_his_answers() {
    let done = (
        "a".to_owned(),
        Tally {
            completed: 1,
            first_right: 1,
            opportunities: 1,
            ..Tally::default()
        },
    );
    close(
        estimate(&[done], &[], &Needs::new(), 0).components[0].known,
        0.1,
    );
}

#[test]
fn a_structure_that_cannot_be_a_space_falls_back_to_each_item_alone() {
    let needs = Needs::from([("a".to_owned(), vec![vec!["a".to_owned(), "b".to_owned()]])]);
    let result = estimate(
        &[shown("a"), unseen("b")],
        &[free("a", 0, true)],
        &needs,
        DAY,
    );
    close(result.components[0].known, 0.981_521_739_130_434_8);
    close(result.components[1].known, 0.1);
}

#[test]
fn unrelated_items_do_not_crowd_out_the_structure() {
    let needs = Needs::from([("b".to_owned(), vec![vec!["a".to_owned()]])]);
    let mut tallies = vec![unseen("a"), unseen("b")];
    tallies.extend((0..12).map(|at| unseen(&format!("other{at}"))));
    let after = estimate(&tallies, &[free("b", 0, true)], &needs, DAY);
    close(after.components[0].known, 0.4);
}

#[test]
fn wrong_answers_keep_it_learning() {
    let answers = [free("a", 0, false), free("a", DAY, false)];
    let result = estimate(&[shown("a")], &answers, &Needs::new(), 2 * DAY);
    let a = &result.components[0];
    assert!(a.known < 0.5);
    assert_eq!(a.verdict, Verdict::Learning);
    assert!(a.memory.expect("reviewed").difficulty > 4.0);
}
