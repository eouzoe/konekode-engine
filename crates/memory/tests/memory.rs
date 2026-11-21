use konekode_memory::{DAY_MS, Memory, Weights, days_between};

const W: Weights = Weights::BINARY;

fn close(got: f64, want: f64) {
    assert!((got - want).abs() < 1e-9, "got {got}, want {want}");
}

#[test]
fn a_first_success_starts_from_the_papers_weights() {
    let memory = Memory::first(&W, true);
    close(memory.stability, 2.445_347_326_090_037);
    close(memory.brittleness, 0.020_436_896_414_182_396);
    close(memory.difficulty, 0.6506);
    close(memory.recall(&W, 1.0), 0.944_266_798_779_931);
    close(memory.recall(&W, 30.0), 0.851_272_905_508_353_7);
}

#[test]
fn a_first_failure_starts_low_and_harder() {
    let memory = Memory::first(&W, false);
    close(memory.stability, 0.1583);
    close(memory.brittleness, 0.3157);
    close(memory.difficulty, 2.6506);
    close(memory.recall(&W, 1.0), 0.558_402_566_970_602_5);
}

#[test]
fn a_review_moves_stability_the_way_the_paper_says() {
    let first = Memory::first(&W, true);
    let passed = first.review(&W, 1.0, true);
    close(passed.stability, 3.668_898_908_504_477_4);
    close(passed.brittleness, first.brittleness);
    close(passed.difficulty, first.difficulty);
    close(passed.recall(&W, 10.0), 0.914_513_819_045_715_4);
    let failed = first.review(&W, 1.0, false);
    close(failed.stability, 0.651_907_756_821_117_6);
    close(failed.brittleness, 0.3157);
    close(failed.difficulty, 2.6506);
    close(failed.recall(&W, 1.0), 0.729_572_179_374_469_7);
}

#[test]
fn recall_only_falls_with_time() {
    for pass in [true, false] {
        let memory = Memory::first(&W, pass);
        let mut last = memory.recall(&W, 0.0);
        for day in 1..400 {
            let now = memory.recall(&W, f64::from(day) / 4.0);
            assert!(now < last, "day {day}: {now} after {last}");
            last = now;
        }
        assert!(last > 0.0 && last < 1.0);
    }
}

#[test]
fn passing_never_lowers_stability_and_failing_never_raises_it() {
    let mut memory = Memory::first(&W, true);
    for (at, pass) in [true, true, false, true, false, false, true]
        .into_iter()
        .enumerate()
    {
        let days = f64::from(u8::try_from(at).expect("small")) * 1.5;
        let next = memory.review(&W, days, pass);
        if pass {
            assert!(next.stability >= memory.stability);
        } else {
            assert!(next.stability <= memory.stability);
            assert!(next.difficulty > memory.difficulty);
        }
        memory = next;
    }
}

#[test]
fn states_stay_inside_their_bounds() {
    let mut memory = Memory::first(&W, false);
    for _ in 0..200 {
        memory = memory.review(&W, 0.0, false);
    }
    assert!(memory.stability >= 0.001);
    close(memory.difficulty, 100.6506);
    let mut strong = Memory::first(&W, true);
    for _ in 0..200 {
        strong = strong.review(&W, 10_000.0, true);
    }
    assert!(strong.stability <= 36_500.0);
}

#[test]
fn time_is_counted_in_days_and_never_backwards() {
    close(days_between(0, 86_400_000), 1.0);
    close(days_between(0, 43_200_000), 0.5);
    close(days_between(5, 0), 0.0);
    close(DAY_MS, 86_400_000.0);
}
