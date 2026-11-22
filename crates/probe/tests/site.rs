use std::fs::File;
use std::io::{BufRead, BufReader};

use flate2::read::GzDecoder;
use serde_json::{Map, Value, json};

fn plain(value: &Value) -> Value {
    match value {
        Value::Number(number) => number.as_f64().map_or(Value::Null, |float| json!(float)),
        Value::Array(items) => Value::Array(items.iter().map(plain).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, item)| (key.clone(), plain(item)))
                .collect::<Map<String, Value>>(),
        ),
        other @ (Value::Null | Value::Bool(_) | Value::String(_)) => other.clone(),
    }
}

#[test]
fn every_answer_recorded_from_the_site_still_holds() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/site.jsonl.gz");
    let file = File::open(path).expect("the recorded answers are missing");
    let mut ui = konekode_probe::new_ui();
    let mut total = 0;
    let mut failed = Vec::new();
    for line in BufReader::new(GzDecoder::new(file)).lines() {
        let line = line.expect("the recorded answers are unreadable");
        let case: Value = serde_json::from_str(&line).expect("a recorded answer is not JSON");
        let request = json!({ "op": case["op"], "args": case["args"] });
        let reply = konekode_probe::reply(&mut ui, &request.to_string());
        let got = reply
            .get("ok")
            .cloned()
            .unwrap_or_else(|| json!({ "__error": reply["error"] }));
        total += 1;
        if plain(&got) != plain(&case["expected"]) {
            failed.push(format!(
                "{}: {}\n  want {}\n  got  {}",
                case["op"], case["args"], case["expected"], got
            ));
        }
    }
    assert!(total > 16_000, "only {total} recorded answers");
    assert!(
        failed.is_empty(),
        "{} of {total} differ:\n{}",
        failed.len(),
        failed
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
