#![no_main]

use konekode_flow::{Content, Engine};
use konekode_record::read_events;
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|bytes: &[u8]| {
    let Ok(Value::Array(values)) = serde_json::from_slice::<Value>(bytes) else {
        return;
    };
    let events = read_events(&values);
    let content = Content::default();
    let whole = Engine::new(&content, &events).state();
    let mut step = Engine::new(&content, &[]);
    for event in events.iter().rev() {
        step.push(event.clone());
    }
    assert_eq!(step.state(), whole);
});
