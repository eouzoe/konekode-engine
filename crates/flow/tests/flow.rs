use konekode_flow::{Content, Engine, Pushed, Row, collect};
use konekode_record::{Event, read_event};
use serde_json::{Value, json};

struct Dice(u64);

impl Dice {
    fn below(&mut self, limit: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % limit
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        let place = usize::try_from(self.below(items.len() as u64)).unwrap_or(0);
        items[place]
    }
}

fn learner_event(dice: &mut Dice, number: u64, at: u64) -> Value {
    let kind = dice.pick(&[
        "beat.reached",
        "task.entered",
        "step.tried",
        "step.tried",
        "check.passed",
        "item.unlocked",
        "unit.restarted",
        "key.down",
    ]);
    let mut data = serde_json::Map::new();
    match kind {
        "beat.reached" if dice.below(3) > 0 => {
            data.insert(
                "shows".into(),
                json!(dice.pick(&["key.i", "key.i,key.esc", ""])),
            );
        }
        "task.entered" => {
            data.insert("mode".into(), json!(dice.pick(&["free", "guided"])));
        }
        "step.tried" => {
            data.insert("at".into(), json!(dice.below(3)));
            data.insert("verdict".into(), json!(dice.pick(&["right", "wrong"])));
            if dice.below(3) > 0 {
                data.insert(
                    "uses".into(),
                    json!(dice.pick(&["key.i", "key.esc,key.i", "key.o"])),
                );
            }
        }
        _ => {}
    }
    let reference = match kind {
        "task.entered" | "step.tried" => dice.pick(&["t1", "t2"]),
        "item.unlocked" => dice.pick(&["pair.paren", "key.o"]),
        "unit.restarted" => "",
        _ => dice.pick(&["b1", "b2", "b3"]),
    };
    json!({
        "actor": "a",
        "at": at,
        "content": "c",
        "data": data,
        "id": format!("e{number}"),
        "kind": kind,
        "ref": reference,
        "seq": number,
        "session": dice.pick(&["s1", "s1", "s2"]),
        "site": "v",
        "unit": dice.pick(&["start", "start", "other"]),
        "version": 3,
    })
}

fn scenario(dice: &mut Dice) -> Vec<Event> {
    let size = 1 + dice.below(12);
    let mut at = 0;
    let mut events = Vec::new();
    for number in 0..size {
        at += dice.below(3);
        let Some(event) = read_event(&learner_event(dice, number, at)) else {
            continue;
        };
        if dice.below(8) == 0 {
            events.push(event.clone());
        }
        if dice.below(16) == 0 {
            let mut copy = event.clone();
            copy.reference.push('x');
            events.push(copy);
        }
        events.push(event);
    }
    events
}

fn shuffled(dice: &mut Dice, events: &[Event]) -> Vec<Event> {
    let mut out = events.to_vec();
    for place in (1..out.len()).rev() {
        let other = usize::try_from(dice.below(place as u64 + 1)).unwrap_or(0);
        out.swap(place, other);
    }
    out
}

#[test]
fn one_by_one_in_any_order_matches_all_at_once() {
    let content = Content::default();
    let mut dice = Dice(0x9e37_79b9_7f4a_7c15);
    let mut applied = 0;
    for _ in 0..20_000 {
        let events = scenario(&mut dice);
        let whole = Engine::new(&content, &events).state();
        let mut step = Engine::new(&content, &[]);
        for event in &events {
            if step.push(event.clone()) == Pushed::Applied {
                applied += 1;
            }
        }
        assert_eq!(step.state(), whole, "{events:?}");
        let mixed = shuffled(&mut dice, &events);
        let mut later = Engine::new(&content, &[]);
        for event in &mixed {
            later.push(event.clone());
        }
        assert_eq!(later.state(), whole, "{mixed:?}");
        assert_eq!(Engine::new(&content, &mixed).state(), whole);
    }
    assert!(applied > 20_000);
}

fn row(id: i64, session: &str, seq: i64, body: Value) -> Row {
    Row {
        actor: "a".into(),
        body,
        id,
        kind: "events".into(),
        seq,
        session: session.into(),
    }
}

fn event(id: &str, session: &str, seq: i64) -> Value {
    json!({
        "actor": "a", "at": seq, "content": "c", "data": {}, "id": id, "kind": "beat.reached",
        "ref": "b1", "seq": seq, "session": session, "site": "v", "unit": "start", "version": 3,
    })
}

#[test]
fn collecting_checks_every_batch_against_its_ticket() {
    let rows = [
        row(
            1,
            "s1",
            1,
            json!([event("e1", "s1", 1), event("e2", "s1", 2)]),
        ),
        row(
            2,
            "s1",
            1,
            json!([event("e1", "s1", 1), event("e2", "s1", 2)]),
        ),
        row(
            3,
            "s1",
            3,
            json!([event("e3", "s1", 3), event("e9", "s9", 4)]),
        ),
        row(4, "s1", 7, json!([event("e5", "s1", 5)])),
        row(5, "s1", 6, json!({"not": "a list"})),
        row(6, "s1", 8, json!([event("e8", "s1", 8), {"version": 3}])),
        Row {
            kind: "replay".into(),
            ..row(7, "s1", 1, json!([]))
        },
        Row {
            kind: "other".into(),
            ..row(8, "s1", 1, json!([]))
        },
    ];
    let collected = collect(&rows);
    let report = &collected.report;
    assert_eq!(report.rows, 8);
    assert_eq!(report.event_rows, 6);
    assert_eq!(report.replay_rows, 1);
    assert_eq!(report.unknown_rows, [8]);
    assert_eq!(report.bad_bodies, [5]);
    assert_eq!(report.foreign, [3]);
    assert_eq!(report.misnumbered, [4]);
    assert_eq!(report.unreadable, 1);
    assert_eq!(report.duplicates, 2);
    assert_eq!(report.conflicts, Vec::<String>::new());
    let ids: Vec<&str> = collected.actors["a"]
        .iter()
        .map(|event| event.id.as_str())
        .collect();
    assert_eq!(ids, ["e1", "e2", "e3", "e5", "e8"]);
}
