use std::collections::{BTreeMap, BTreeSet};

use konekode_record::{Event, order, read_event};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub actor: String,
    pub body: Value,
    pub id: i64,
    pub kind: String,
    pub seq: i64,
    pub session: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub actors: usize,
    #[serde(rename = "badBodies")]
    pub bad_bodies: Vec<i64>,
    pub conflicts: Vec<String>,
    pub duplicates: i64,
    pub events: usize,
    #[serde(rename = "eventRows")]
    pub event_rows: usize,
    pub foreign: Vec<i64>,
    pub misnumbered: Vec<i64>,
    #[serde(rename = "replayRows")]
    pub replay_rows: usize,
    pub rows: usize,
    #[serde(rename = "unknownRows")]
    pub unknown_rows: Vec<i64>,
    pub unreadable: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Collected {
    pub actors: BTreeMap<String, Vec<Event>>,
    pub report: Report,
}

fn numbered(row: &Row, events: &[Event]) -> bool {
    let Some(first) = events.first() else {
        return false;
    };
    first.seq == row.seq && events.windows(2).all(|pair| pair[0].seq < pair[1].seq)
}

fn take_events(row: &Row, report: &mut Report, into: &mut Vec<Event>) {
    let Some(values) = row.body.as_array() else {
        report.bad_bodies.push(row.id);
        return;
    };
    let mut own = Vec::new();
    let mut foreign = false;
    for value in values {
        let Some(event) = read_event(value) else {
            report.unreadable += 1;
            continue;
        };
        if event.actor != row.actor || event.session != row.session {
            foreign = true;
            continue;
        }
        own.push(event);
    }
    if foreign {
        report.foreign.push(row.id);
    }
    if !numbered(row, &own) {
        report.misnumbered.push(row.id);
    }
    into.extend(own);
}

#[must_use]
pub fn collect(rows: &[Row]) -> Collected {
    let mut report = Report {
        rows: rows.len(),
        ..Report::default()
    };
    let mut gathered: BTreeMap<String, Vec<Event>> = BTreeMap::new();
    for row in rows {
        match row.kind.as_str() {
            "events" => {
                report.event_rows += 1;
                let into = gathered.entry(row.actor.clone()).or_default();
                take_events(row, &mut report, into);
            }
            "replay" => report.replay_rows += 1,
            _ => report.unknown_rows.push(row.id),
        }
    }
    let mut conflicts = BTreeSet::new();
    let mut actors = BTreeMap::new();
    for (actor, events) in gathered {
        let ordered = order(&events);
        report.duplicates += ordered.duplicates;
        conflicts.extend(ordered.conflicts);
        report.events += ordered.events.len();
        if !ordered.events.is_empty() {
            actors.insert(actor, ordered.events);
        }
    }
    report.actors = actors.len();
    report.conflicts = conflicts.into_iter().collect();
    Collected { actors, report }
}
