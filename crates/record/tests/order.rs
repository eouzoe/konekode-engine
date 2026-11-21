use std::collections::{BTreeMap, BTreeSet};

use konekode_record::{Appended, Data, DataValue, Event, Log, order};

const ORDERS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

fn event(at: i64, session: &str, seq: i64, id: &str, mark: i32) -> Event {
    let mut data = Data::new();
    if mark != 0 {
        data.insert("n".to_owned(), DataValue::Number(f64::from(mark)));
    }
    Event {
        actor: "a".to_owned(),
        at,
        content: String::new(),
        data,
        id: id.to_owned(),
        kind: "k".to_owned(),
        reference: String::new(),
        seq,
        session: session.to_owned(),
        site: String::new(),
        unit: String::new(),
        version: 3,
    }
}

fn domain() -> Vec<Event> {
    let mut all = Vec::new();
    for at in 0..2 {
        for session in ["a", "b"] {
            for seq in 0..2 {
                for id in ["x", "y", "z"] {
                    for mark in 0..2 {
                        all.push(event(at, session, seq, id, mark));
                    }
                }
            }
        }
    }
    all
}

fn key(event: &Event) -> (i64, &[u8], i64, &[u8]) {
    (
        event.at,
        event.session.as_bytes(),
        event.seq,
        event.id.as_bytes(),
    )
}

fn expected_conflicts(list: &[Event]) -> Vec<String> {
    let mut variants: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for event in list {
        variants
            .entry(event.id.as_str())
            .or_default()
            .insert(serde_json::to_string(event).unwrap_or_default());
    }
    variants
        .into_iter()
        .filter(|(_, copies)| copies.len() > 1)
        .map(|(id, _)| id.to_owned())
        .collect()
}

fn check(list: &[Event]) {
    let base = order(list);
    let mut seen = BTreeSet::new();
    for pair in base.events.windows(2) {
        assert!(
            key(&pair[0]) < key(&pair[1]),
            "not strictly ordered: {list:?}"
        );
    }
    for event in &base.events {
        assert!(seen.insert(event.id.clone()), "an id twice: {list:?}");
        assert!(list.contains(event), "an event nobody sent: {list:?}");
    }
    let sent: BTreeSet<String> = list.iter().map(|event| event.id.clone()).collect();
    assert_eq!(seen, sent);
    assert_eq!(base.conflicts, expected_conflicts(list));
    let mut log = Log::new(&[]);
    for event in list {
        let before: Vec<Event> = log.events().to_vec();
        let fresh = !before.iter().any(|held| held.id == event.id);
        let after_all = before.iter().all(|held| key(held) < key(event));
        let answer = log.append(event.clone());
        if fresh {
            assert_eq!(answer == Appended::Last, after_all, "{list:?}");
        }
    }
    assert_eq!(log.events(), base.events.as_slice(), "appending: {list:?}");
    assert_eq!(log.conflicts(), base.conflicts, "appending: {list:?}");
    assert_eq!(Log::new(list).events(), base.events.as_slice());
}

#[test]
fn every_arrival_order_of_three_gives_one_log() {
    let all = domain();
    for first in &all {
        check(std::slice::from_ref(first));
        for second in &all {
            let pair = [first.clone(), second.clone()];
            check(&pair);
            assert_eq!(order(&pair), order(&[second.clone(), first.clone()]));
            for third in &all {
                let three = [first.clone(), second.clone(), third.clone()];
                let base = order(&three);
                check(&three);
                for places in ORDERS {
                    let shuffled: Vec<Event> =
                        places.iter().map(|place| three[*place].clone()).collect();
                    assert_eq!(order(&shuffled), base, "{three:?}");
                }
            }
        }
    }
}

#[test]
fn the_cycle_the_old_order_made_is_one_line_now() {
    let crossing = [
        event(5, "s1", 1, "z", 0),
        event(5, "s2", 1, "m", 0),
        event(5, "s1", 2, "a", 0),
    ];
    let ids: Vec<String> = order(&crossing)
        .events
        .into_iter()
        .map(|event| event.id)
        .collect();
    assert_eq!(ids, ["z", "a", "m"]);
}

#[test]
fn copies_count_as_duplicates_and_differences_as_conflicts() {
    let same = event(1, "a", 1, "x", 0);
    let other = event(1, "a", 1, "x", 1);
    let reading = order(&[same.clone(), same.clone(), other]);
    assert_eq!(reading.duplicates, 2);
    assert_eq!(reading.conflicts, ["x"]);
    assert_eq!(reading.events.len(), 1);
}
