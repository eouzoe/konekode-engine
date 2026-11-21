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

fn close(got: f64, want: f64) {
    assert!((got - want).abs() < 1e-12, "got {got}, want {want}");
}

fn chain() -> Vec<Set> {
    Space::new(vec![vec![set(&[])], vec![set(&[0])]])
        .expect("valid")
        .states(10)
        .expect("small")
}

fn weight_of(belief: &Belief, state: &[usize]) -> f64 {
    let wanted = set(state);
    belief
        .states()
        .iter()
        .zip(belief.weights())
        .find(|(state, _)| **state == wanted)
        .map(|(_, weight)| *weight)
        .expect("a state of the space")
}

#[test]
fn noise_reports_what_it_was_given_and_the_four_likelihoods() {
    let noise = open_ended();
    close(noise.careless(), 0.1);
    close(noise.lucky(), 0.02);
    close(noise.likelihood(true, true), 0.9);
    close(noise.likelihood(true, false), 0.1);
    close(noise.likelihood(false, true), 0.02);
    close(noise.likelihood(false, false), 0.98);
}

#[test]
fn a_belief_needs_states_and_usable_weights() {
    assert!(matches!(Belief::uniform(Vec::new()), Err(Error::Empty)));
    let two = vec![set(&[]), set(&[0])];
    assert!(matches!(
        Belief::weighted(two.clone(), vec![1.0]),
        Err(Error::Lengths)
    ));
    for bad in [
        vec![-1.0, 2.0],
        vec![f64::NAN, 1.0],
        vec![f64::INFINITY, 1.0],
        vec![0.0, 0.0],
    ] {
        assert!(
            matches!(
                Belief::weighted(two.clone(), bad.clone()),
                Err(Error::Weights)
            ),
            "{bad:?}"
        );
    }
    let belief = Belief::weighted(two, vec![1.0, 3.0]).expect("usable");
    close(belief.weights()[0], 0.25);
    close(belief.weights()[1], 0.75);
    let zero = Belief::weighted(vec![set(&[]), set(&[0])], vec![0.0, 2.0]).expect("usable");
    close(zero.weights()[0], 0.0);
    close(zero.weights()[1], 1.0);
}

#[test]
fn learning_moves_weight_only_to_states_that_exist() {
    let mut belief = Belief::uniform(chain()).expect("non-empty");
    belief.learn(1, 0.5);
    close(weight_of(&belief, &[]), 1.0 / 3.0);
    close(weight_of(&belief, &[0]), 1.0 / 6.0);
    close(weight_of(&belief, &[0, 1]), 0.5);
    belief.learn(0, 0.5);
    close(weight_of(&belief, &[]), 1.0 / 6.0);
    close(weight_of(&belief, &[0]), 1.0 / 3.0);
    close(weight_of(&belief, &[0, 1]), 0.5);
    belief.learn(0, 3.0);
    close(weight_of(&belief, &[]), 0.0);
    close(weight_of(&belief, &[0]), 0.5);
    belief.learn(1, -1.0);
    close(weight_of(&belief, &[0, 1]), 0.5);
    let total: f64 = belief.weights().iter().sum();
    close(total, 1.0);
}

#[test]
fn the_best_state_is_the_first_of_the_heaviest() {
    let states = chain();
    let even = Belief::uniform(states.clone()).expect("non-empty");
    let (first, chance) = even.best();
    assert_eq!(first, &states[0]);
    close(chance, 1.0 / 3.0);
    let tied = Belief::weighted(states.clone(), vec![1.0, 2.0, 2.0]).expect("usable");
    assert_eq!(tied.best().0, &states[1]);
    close(tied.best().1, 0.4);
}

#[test]
fn entropy_is_measured_in_nats_and_ignores_impossible_states() {
    let states = chain();
    close(
        Belief::uniform(states.clone())
            .expect("non-empty")
            .entropy(),
        3.0_f64.ln(),
    );
    let two = Belief::weighted(states.clone(), vec![0.0, 1.0, 1.0]).expect("usable");
    close(two.entropy(), 2.0_f64.ln());
    let sure = Belief::weighted(states, vec![0.0, 0.0, 5.0]).expect("usable");
    close(sure.entropy(), 0.0);
}

#[test]
fn one_answer_moves_the_weights_by_bayes_rule() {
    let mut belief = Belief::uniform(chain()).expect("non-empty");
    belief.observe(answer(0, true), open_ended());
    let total = 0.02 + 0.9 + 0.9;
    close(weight_of(&belief, &[]), 0.02 / total);
    close(weight_of(&belief, &[0]), 0.9 / total);
    close(weight_of(&belief, &[0, 1]), 0.9 / total);
    close(belief.marginal(0), 1.8 / total);
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
