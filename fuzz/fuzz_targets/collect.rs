#![no_main]

use konekode_flow::{Row, collect};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let Ok(rows) = serde_json::from_slice::<Vec<Row>>(bytes) else {
        return;
    };
    let collected = collect(&rows);
    let report = &collected.report;
    assert_eq!(report.rows, rows.len());
    assert_eq!(report.actors, collected.actors.len());
    let kept: usize = collected.actors.values().map(Vec::len).sum();
    assert_eq!(report.events, kept);
    for (actor, events) in &collected.actors {
        for pair in events.windows(2) {
            assert!(pair[0].id != pair[1].id);
        }
        for event in events {
            assert_eq!(&event.actor, actor);
        }
    }
});
