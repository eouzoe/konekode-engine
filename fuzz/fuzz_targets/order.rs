#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use konekode_record::{Data, DataValue, Event, Log, order};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Seed {
    at: u8,
    id: u8,
    mark: u8,
    seq: u8,
    session: u8,
}

fn event(seed: &Seed) -> Event {
    let mut data = Data::new();
    if seed.mark % 3 != 0 {
        data.insert("n".to_owned(), DataValue::Number(f64::from(seed.mark % 3)));
    }
    Event {
        actor: "a".to_owned(),
        at: i64::from(seed.at % 4),
        content: String::new(),
        data,
        id: format!("e{}", seed.id % 5),
        kind: "k".to_owned(),
        reference: String::new(),
        seq: i64::from(seed.seq % 4),
        session: format!("s{}", seed.session % 3),
        site: String::new(),
        unit: String::new(),
        version: 3,
    }
}

fuzz_target!(|bytes: &[u8]| {
    let mut input = Unstructured::new(bytes);
    let Ok(seeds) = Vec::<Seed>::arbitrary(&mut input) else {
        return;
    };
    let events: Vec<Event> = seeds.iter().take(12).map(event).collect();
    let base = order(&events);
    let mut reversed = events.clone();
    reversed.reverse();
    assert_eq!(order(&reversed), base);
    let mut rotated = events.clone();
    if !rotated.is_empty() {
        rotated.rotate_left(1);
    }
    assert_eq!(order(&rotated), base);
    let mut log = Log::new(&[]);
    for item in &events {
        log.append(item.clone());
    }
    assert_eq!(log.events(), base.events.as_slice());
    assert_eq!(log.conflicts(), base.conflicts);
});
