use konekode_assess::{Belief, Error, Noise, Response};
use konekode_space::{Set, Space};

fn set(items: &[usize]) -> Set {
    items.iter().copied().collect()
}

fn space() -> Space {
    Space::new(vec![
        vec![set(&[])],
        vec![set(&[0])],
        vec![set(&[0])],
        vec![set(&[1, 2])],
        vec![set(&[1]), set(&[2])],
        vec![set(&[3])],
    ])
    .expect("valid")
}

fn open_ended() -> Noise {
    Noise::new(0.1, 0.02).expect("informative")
}

fn answer(item: usize, correct: bool) -> Response {
    Response { item, correct }
}

#[test]
fn noise_that_carries_no_evidence_is_refused() {
    assert_eq!(Noise::new(0.5, 0.5), Err(Error::Noise));
    assert_eq!(Noise::new(0.6, 0.5), Err(Error::Noise));
    assert_eq!(Noise::new(0.0, 0.1), Err(Error::Noise));
    assert_eq!(Noise::new(0.1, 0.0), Err(Error::Noise));
    assert_eq!(Noise::new(f64::NAN, 0.1), Err(Error::Noise));
    assert!(Noise::new(0.1, 0.02).is_ok());
}

#[test]
fn the_order_of_answers_does_not_matter() {
    let states = space().states(1000).expect("small");
    let answers = [
        answer(1, true),
        answer(3, false),
        answer(4, true),
        answer(0, true),
    ];
    let mut forward = Belief::uniform(states.clone()).expect("non-empty");
    let mut backward = Belief::uniform(states).expect("non-empty");
    for given in answers {
        forward.observe(given, open_ended());
    }
    for given in answers.iter().rev() {
        backward.observe(*given, open_ended());
    }
    for (left, right) in forward.weights().iter().zip(backward.weights()) {
        assert!((left - right).abs() < 1e-12);
    }
}

#[test]
fn every_true_state_is_found_by_asking_with_no_prior_data() {
    let space = space();
    let states = space.states(1000).expect("small");
    let everything: Set = (0..space.len()).collect();
    for truth in &states {
        let mut belief = Belief::uniform(states.clone()).expect("non-empty");
        for _ in 0..12 {
            let item = belief.question(&everything).expect("there are items");
            belief.observe(answer(item, truth.contains(item)), open_ended());
        }
        let (best, chance) = belief.best();
        assert_eq!(best, truth);
        assert!(chance > 0.8, "{truth:?} only reached {chance}");
    }
}

#[test]
fn a_success_that_is_easy_to_get_without_knowing_counts_for_little() {
    let states = space().states(1000).expect("small");
    let mut followed = Belief::uniform(states.clone()).expect("non-empty");
    let mut alone = Belief::uniform(states).expect("non-empty");
    let start = followed.marginal(3);
    followed.observe(answer(3, true), Noise::new(0.05, 0.9).expect("informative"));
    alone.observe(answer(3, true), open_ended());
    assert!(followed.marginal(3) > start);
    assert!(alone.marginal(3) > followed.marginal(3));
}
