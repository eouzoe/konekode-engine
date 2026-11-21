use std::collections::BTreeSet;

use konekode_flow::{Content, Engine, Row, collect};
use konekode_learner::{Standing, UnitState};
use konekode_record::read_events;
use serde_json::{Value, json};

fn start_content() -> Content {
    serde_json::from_str(include_str!("fixtures/start-content.json")).unwrap_or_default()
}

#[test]
fn every_lookup_of_the_start_unit_matches_the_site() {
    let content = start_content();
    assert!(!content.units.is_empty());
    let engine = Engine::new(&content, &[]);
    let golden: Value =
        serde_json::from_str(include_str!("fixtures/start-domain.json")).unwrap_or(Value::Null);
    let shows = golden["shows"].as_array().cloned().unwrap_or_default();
    let uses = golden["uses"].as_array().cloned().unwrap_or_default();
    assert!(!shows.is_empty() && !uses.is_empty());
    for row in shows {
        let beat = row[0].as_str().unwrap_or_default();
        assert_eq!(json!(engine.shows("start", beat)), row[1], "{beat}");
    }
    for row in uses {
        let task = row[0].as_str().unwrap_or_default();
        let step = row[1].as_f64().unwrap_or(f64::NAN);
        assert_eq!(
            json!(engine.uses("start", task, step)),
            row[2],
            "{task} {step}"
        );
    }
    assert_eq!(engine.shows("nowhere", "b"), Vec::<String>::new());
}

#[test]
fn the_catalogue_check_tells_a_different_catalogue_apart() {
    let mut content = start_content();
    assert!(content.catalog_matches());
    content.catalog = json!([]);
    assert!(!content.catalog_matches());
    content.catalog = Value::Null;
    assert!(content.catalog_matches());
}

fn event(seq: i64, at: i64, kind: &str, reference: &str, data: &Value) -> Value {
    json!({
        "actor": "a", "at": at, "content": "c", "data": data, "id": format!("e{seq}"),
        "kind": kind, "ref": reference, "seq": seq, "session": "s", "site": "v",
        "unit": "start", "version": 3,
    })
}

fn known_history() -> [Value; 7] {
    [
        event(1, 1, "beat.reached", "b1", &json!({"shows": "key.i"})),
        event(2, 2, "task.entered", "t1", &json!({"mode": "free"})),
        event(
            3,
            5,
            "step.tried",
            "t1",
            &json!({"at": 0, "verdict": "wrong", "uses": "key.i"}),
        ),
        event(
            4,
            9,
            "step.tried",
            "t1",
            &json!({"at": 0, "verdict": "right", "uses": "key.i"}),
        ),
        event(
            5,
            20,
            "step.tried",
            "t1",
            &json!({"at": 1, "verdict": "right", "uses": "key.esc"}),
        ),
        event(6, 21, "item.unlocked", "pair.paren", &json!({})),
        event(7, 22, "check.passed", "t1", &json!({})),
    ]
}

#[test]
fn a_known_history_gives_the_known_state() {
    let values = known_history();
    let state = Engine::new(&Content::default(), &read_events(&values)).state();
    assert_eq!(state.events, 7);
    assert_eq!(
        state.units.get("start"),
        Some(&UnitState {
            passed: BTreeSet::from(["t1".to_owned()]),
            position: Some("b1".to_owned()),
        })
    );
    assert_eq!(state.unlocked, BTreeSet::from(["pair.paren".to_owned()]));
    let ids: Vec<&str> = state
        .components
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    assert_eq!(ids, ["key.i", "key.esc"]);
    let typed = &state.components[0];
    assert_eq!(
        (
            typed.tally.shown,
            typed.tally.opportunities,
            typed.tally.first_right,
            typed.tally.completed,
            typed.tally.unsettled,
            typed.standing
        ),
        (1, 1, 0, 1, true, Standing::Done)
    );
    let escaped = &state.components[1];
    assert_eq!(
        (
            escaped.tally.shown,
            escaped.tally.opportunities,
            escaped.tally.first_right,
            escaped.tally.completed,
            escaped.tally.unsettled
        ),
        (0, 1, 1, 1, false)
    );
    let seen: Vec<(i64, &str, bool, bool, i64)> = state
        .observations
        .iter()
        .map(|item| {
            (
                item.at,
                item.component.as_str(),
                item.right,
                item.helped,
                item.waited,
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            (5, "key.i", false, false, 3),
            (20, "key.esc", true, false, 11)
        ]
    );
    assert_eq!((state.legacy, state.unread), (0, 0));
}

#[test]
fn a_known_history_gives_the_known_estimates() {
    let values = known_history();
    let engine = Engine::new(&Content::default(), &read_events(&values));
    let state = engine.state();
    assert_eq!(engine.estimates(22), state.estimates);
    assert!((state.estimates.careless - 0.1).abs() < 1e-12);
    let known: Vec<(&str, f64, i64)> = state
        .estimates
        .components
        .iter()
        .map(|item| (item.id.as_str(), item.known, item.reviews))
        .collect();
    assert_eq!(known.len(), 2);
    assert_eq!((known[0].0, known[0].2), ("key.i", 1));
    assert!((known[0].1 - 0.228_703_703_703_703_7).abs() < 1e-12);
    assert_eq!((known[1].0, known[1].2), ("key.esc", 1));
    assert!((known[1].1 - 0.858_333_333_333_333_3).abs() < 1e-12);
}

fn row(id: i64, actor: &str, seq: i64, body: Value) -> Row {
    Row {
        actor: actor.into(),
        body,
        id,
        kind: "events".into(),
        seq,
        session: "s".into(),
    }
}

fn plain(actor: &str, id: &str, seq: i64) -> Value {
    json!({
        "actor": actor, "at": seq, "content": "c", "data": {}, "id": id, "kind": "k",
        "ref": "", "seq": seq, "session": "s", "site": "v", "unit": "", "version": 3,
    })
}

#[test]
fn repeated_numbers_and_copies_across_actors_are_counted() {
    let rows = [
        row(1, "a", 1, json!([plain("a", "x1", 1), plain("a", "x2", 1)])),
        row(2, "a", 1, json!([plain("a", "x1", 1)])),
        row(3, "b", 4, json!([plain("b", "y1", 4)])),
        row(4, "b", 4, json!([plain("b", "y1", 4)])),
    ];
    let collected = collect(&rows);
    assert_eq!(collected.report.misnumbered, [1]);
    assert_eq!(collected.report.duplicates, 2);
    assert_eq!(collected.report.events, 3);
    assert_eq!(collected.report.actors, 2);
}
