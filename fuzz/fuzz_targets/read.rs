#![no_main]

use konekode_record::{read_counted, read_event};
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|bytes: &[u8]| {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return;
    };
    let one = read_event(&value);
    if let Some(list) = value.as_array() {
        let reading = read_counted(list);
        let total = reading.events.len() + usize::try_from(reading.unreadable).unwrap_or(0);
        assert_eq!(total, list.len());
    }
    if let Some(event) = one {
        let again = serde_json::to_value(&event).ok().and_then(|text| read_event(&text));
        assert_eq!(again.as_ref(), Some(&event));
    }
});
