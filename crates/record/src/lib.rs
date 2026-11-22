use std::cmp::Ordering;
use std::collections::btree_map::Entry as Slot;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const CURRENT_VERSION: i64 = 3;
pub const FAULT_KIND: &str = "fault";
pub const FAULT_SUMMARY_KIND: &str = "fault.summary";
pub const UNKNOWN_CONTENT: &str = "";
const UNLABELLED_VERSION: i64 = 2;
const LEGACY_VERSION: i64 = 1;
const LEGACY_SESSION: &str = "legacy";
const SAFE: f64 = 9_007_199_254_740_991.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataValue {
    Flag(bool),
    Number(f64),
    Text(String),
}

pub type Data = BTreeMap<String, DataValue>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub actor: String,
    pub at: i64,
    pub content: String,
    pub data: Data,
    pub id: String,
    pub kind: String,
    #[serde(rename = "ref")]
    pub reference: String,
    pub seq: i64,
    pub session: String,
    pub site: String,
    pub unit: String,
    pub version: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    pub events: Vec<Event>,
    pub unlabelled: i64,
    pub unreadable: i64,
}

fn text(object: &Map<String, Value>, key: &str) -> Option<String> {
    object.get(key)?.as_str().map(str::to_owned)
}

fn name(object: &Map<String, Value>, key: &str) -> Option<String> {
    text(object, key).filter(|value| !value.is_empty())
}

fn safe_integer(value: &Value) -> Option<i64> {
    let number = value.as_f64()?;
    let integral = matches!(number.fract().partial_cmp(&0.0), Some(Ordering::Equal));
    if !number.is_finite() || !integral || number.abs() > SAFE {
        return None;
    }
    value
        .as_i64()
        .or_else(|| format!("{number:.0}").parse().ok())
}

fn moment(object: &Map<String, Value>, key: &str) -> Option<i64> {
    safe_integer(object.get(key)?)
}

fn version(object: &Map<String, Value>, wanted: i64) -> bool {
    object.get("version").and_then(safe_integer) == Some(wanted)
}

fn plain_zero(number: f64) -> f64 {
    if matches!(number.partial_cmp(&0.0), Some(Ordering::Equal)) {
        return 0.0;
    }
    number
}

fn data(object: &Map<String, Value>) -> Option<Data> {
    let fields = object.get("data")?.as_object()?;
    let mut out = Data::new();
    for (key, value) in fields {
        let item = match value {
            Value::Bool(flag) => DataValue::Flag(*flag),
            Value::Number(number) => DataValue::Number(plain_zero(number.as_f64()?)),
            Value::String(text) => DataValue::Text(text.clone()),
            Value::Null | Value::Array(_) | Value::Object(_) => return None,
        };
        out.insert(key.clone(), item);
    }
    Some(out)
}

struct Shared {
    actor: String,
    at: i64,
    data: Data,
    id: String,
    kind: String,
    reference: String,
    seq: i64,
    session: String,
    unit: String,
}

fn shared(object: &Map<String, Value>) -> Option<Shared> {
    Some(Shared {
        actor: text(object, "actor")?,
        at: moment(object, "at")?,
        data: data(object)?,
        id: name(object, "id")?,
        kind: name(object, "kind")?,
        reference: text(object, "ref")?,
        seq: moment(object, "seq")?,
        session: text(object, "session")?,
        unit: text(object, "unit")?,
    })
}

fn labelled(fields: Shared, content: String, site: String) -> Event {
    Event {
        actor: fields.actor,
        at: fields.at,
        content,
        data: fields.data,
        id: fields.id,
        kind: fields.kind,
        reference: fields.reference,
        seq: fields.seq,
        session: fields.session,
        site,
        unit: fields.unit,
        version: CURRENT_VERSION,
    }
}

fn current(object: &Map<String, Value>) -> Option<Event> {
    if !version(object, CURRENT_VERSION) {
        return None;
    }
    let fields = shared(object)?;
    Some(labelled(
        fields,
        text(object, "content")?,
        text(object, "site")?,
    ))
}

fn unlabelled(object: &Map<String, Value>) -> Option<Event> {
    if !version(object, UNLABELLED_VERSION) {
        return None;
    }
    Some(labelled(
        shared(object)?,
        UNKNOWN_CONTENT.to_owned(),
        UNKNOWN_CONTENT.to_owned(),
    ))
}

fn legacy(object: &Map<String, Value>) -> Option<Event> {
    if !version(object, LEGACY_VERSION) {
        return None;
    }
    let fields = Shared {
        actor: text(object, "device")?,
        at: moment(object, "at")?,
        data: Data::new(),
        id: name(object, "id")?,
        kind: name(object, "kind")?,
        reference: text(object, "ref")?,
        seq: 0,
        session: LEGACY_SESSION.to_owned(),
        unit: text(object, "unit")?,
    };
    Some(labelled(
        fields,
        UNKNOWN_CONTENT.to_owned(),
        UNKNOWN_CONTENT.to_owned(),
    ))
}

#[must_use]
pub fn read_event(value: &Value) -> Option<Event> {
    let object = value.as_object()?;
    current(object)
        .or_else(|| unlabelled(object))
        .or_else(|| legacy(object))
}

#[must_use]
pub fn read_counted(values: &[Value]) -> Reading {
    let mut events = Vec::new();
    let mut unreadable = 0;
    for value in values {
        match read_event(value) {
            Some(event) => events.push(event),
            None => unreadable += 1,
        }
    }
    let unlabelled = events
        .iter()
        .filter(|event| event.content == UNKNOWN_CONTENT)
        .count();
    Reading {
        events,
        unlabelled: i64::try_from(unlabelled).unwrap_or(i64::MAX),
        unreadable,
    }
}

#[must_use]
pub fn read_events(values: &[Value]) -> Vec<Event> {
    read_counted(values).events
}

fn by_time(left: &Event, right: &Event) -> Ordering {
    left.at
        .cmp(&right.at)
        .then_with(|| left.session.as_bytes().cmp(right.session.as_bytes()))
        .then_with(|| left.seq.cmp(&right.seq))
        .then_with(|| left.id.as_bytes().cmp(right.id.as_bytes()))
}

const fn value_rank(value: &DataValue) -> u8 {
    match value {
        DataValue::Flag(_) => 0,
        DataValue::Number(_) => 1,
        DataValue::Text(_) => 2,
    }
}

fn value_order(left: &DataValue, right: &DataValue) -> Ordering {
    match (left, right) {
        (DataValue::Flag(one), DataValue::Flag(other)) => one.cmp(other),
        (DataValue::Number(one), DataValue::Number(other)) => one.total_cmp(other),
        (DataValue::Text(one), DataValue::Text(other)) => one.as_bytes().cmp(other.as_bytes()),
        _ => value_rank(left).cmp(&value_rank(right)),
    }
}

fn data_order(left: &Data, right: &Data) -> Ordering {
    for ((left_key, left_value), (right_key, right_value)) in left.iter().zip(right) {
        let found = left_key
            .as_bytes()
            .cmp(right_key.as_bytes())
            .then_with(|| value_order(left_value, right_value));
        if found != Ordering::Equal {
            return found;
        }
    }
    left.len().cmp(&right.len())
}

fn rank(left: &Event, right: &Event) -> Ordering {
    by_time(left, right)
        .then_with(|| left.kind.as_bytes().cmp(right.kind.as_bytes()))
        .then_with(|| left.unit.as_bytes().cmp(right.unit.as_bytes()))
        .then_with(|| left.reference.as_bytes().cmp(right.reference.as_bytes()))
        .then_with(|| left.content.as_bytes().cmp(right.content.as_bytes()))
        .then_with(|| left.site.as_bytes().cmp(right.site.as_bytes()))
        .then_with(|| left.actor.as_bytes().cmp(right.actor.as_bytes()))
        .then_with(|| left.version.cmp(&right.version))
        .then_with(|| data_order(&left.data, &right.data))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ordered {
    pub conflicts: Vec<String>,
    pub duplicates: i64,
    pub events: Vec<Event>,
}

#[must_use]
pub fn order(events: &[Event]) -> Ordered {
    let mut kept: BTreeMap<&str, &Event> = BTreeMap::new();
    let mut conflicts = BTreeSet::new();
    let mut duplicates = 0;
    for event in events {
        match kept.entry(event.id.as_str()) {
            Slot::Vacant(slot) => {
                slot.insert(event);
            }
            Slot::Occupied(mut slot) => {
                duplicates += 1;
                let found = rank(event, slot.get());
                if found != Ordering::Equal {
                    conflicts.insert(event.id.clone());
                }
                if found == Ordering::Less {
                    slot.insert(event);
                }
            }
        }
    }
    let mut events: Vec<Event> = kept.into_values().cloned().collect();
    events.sort_by(by_time);
    Ordered {
        conflicts: conflicts.into_iter().collect(),
        duplicates,
        events,
    }
}

#[must_use]
pub fn ordered(events: &[Event]) -> Vec<Event> {
    order(events).events
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appended {
    Inside,
    Kept,
    Last,
    Replaced,
    Same,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Log {
    conflicts: BTreeSet<String>,
    events: Vec<Event>,
    ids: BTreeSet<String>,
}

impl Log {
    #[must_use]
    pub fn new(events: &[Event]) -> Self {
        let Ordered {
            conflicts, events, ..
        } = order(events);
        let ids = events.iter().map(|event| event.id.clone()).collect();
        Self {
            conflicts: conflicts.into_iter().collect(),
            events,
            ids,
        }
    }

    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    #[must_use]
    pub fn conflicts(&self) -> Vec<String> {
        self.conflicts.iter().cloned().collect()
    }

    pub fn append(&mut self, event: Event) -> Appended {
        if self.ids.contains(&event.id) {
            return self.replace(event);
        }
        self.ids.insert(event.id.clone());
        let last = self
            .events
            .last()
            .is_none_or(|last| by_time(last, &event) == Ordering::Less);
        if last {
            self.events.push(event);
            return Appended::Last;
        }
        let place = self
            .events
            .partition_point(|held| by_time(held, &event) == Ordering::Less);
        self.events.insert(place, event);
        Appended::Inside
    }

    fn replace(&mut self, event: Event) -> Appended {
        let Some(place) = self.events.iter().position(|held| held.id == event.id) else {
            return Appended::Same;
        };
        let found = rank(&event, &self.events[place]);
        if found == Ordering::Equal {
            return Appended::Same;
        }
        self.conflicts.insert(event.id.clone());
        if found == Ordering::Greater {
            return Appended::Kept;
        }
        self.events.remove(place);
        let spot = self
            .events
            .partition_point(|kept| by_time(kept, &event) == Ordering::Less);
        self.events.insert(spot, event);
        Appended::Replaced
    }
}

#[must_use]
pub fn latest_of(events: &[Event]) -> i64 {
    events.iter().fold(0, |latest, event| latest.max(event.at))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub actor: String,
    pub content: String,
    pub session: String,
    pub site: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Data>,
    #[serde(default, rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clock {
    pub latest: i64,
    pub seq: i64,
}

impl Clock {
    #[must_use]
    pub fn after(events: &[Event]) -> Self {
        Self {
            latest: latest_of(events),
            seq: 0,
        }
    }
}

#[must_use]
pub fn build(
    source: &Source,
    clock: &mut Clock,
    kind: &str,
    entry: &Entry,
    now: i64,
    id: &str,
) -> Event {
    clock.seq += 1;
    clock.latest = entry.at.unwrap_or(now).max(clock.latest + 1);
    Event {
        actor: source.actor.clone(),
        at: clock.latest,
        content: source.content.clone(),
        data: entry.data.clone().unwrap_or_default(),
        id: id.to_owned(),
        kind: kind.to_owned(),
        reference: entry.reference.clone().unwrap_or_default(),
        seq: clock.seq,
        session: source.session.clone(),
        site: source.site.clone(),
        unit: entry.unit.clone().unwrap_or_default(),
        version: CURRENT_VERSION,
    }
}
