use std::collections::BTreeMap;
use std::fs;
use std::process::ExitCode;

use konekode_flow::{Content, Engine, Report, Row, State, collect};
use konekode_record::{Event, read_events};
use serde::Serialize;
use serde_json::Value;

const USAGE: &str = "konekode flow <content.json> <rows.json>
konekode collect <rows.json>
konekode state <content.json> <events.json>";

fn read(path: &str) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    serde_json::from_str(&text).map_err(|error| format!("{path}: {error}"))
}

fn content(path: &str) -> Result<Content, String> {
    serde_json::from_value(read(path)?).map_err(|error| format!("{path}: {error}"))
}

fn rows(path: &str) -> Result<Vec<Row>, String> {
    serde_json::from_value(read(path)?).map_err(|error| format!("{path}: {error}"))
}

fn events(path: &str) -> Result<Vec<Event>, String> {
    let value = read(path)?;
    let list = value
        .as_array()
        .ok_or_else(|| format!("{path}: not a list of events"))?;
    Ok(read_events(list))
}

fn encode(value: &impl serde::Serialize) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| error.to_string())
}

#[derive(Serialize)]
struct Flowed<'a> {
    actors: BTreeMap<&'a String, State>,
    #[serde(rename = "catalogMatches")]
    catalog_matches: bool,
    content: &'a str,
    report: &'a Report,
}

fn flow(content_path: &str, rows_path: &str) -> Result<String, String> {
    let content = content(content_path)?;
    let collected = collect(&rows(rows_path)?);
    encode(&Flowed {
        actors: collected
            .actors
            .iter()
            .map(|(actor, events)| (actor, Engine::new(&content, events).state()))
            .collect(),
        catalog_matches: content.catalog_matches(),
        content: &content.version,
        report: &collected.report,
    })
}

fn run(args: &[String]) -> Result<String, String> {
    match args {
        [command, content_path, rows_path] if command == "flow" => flow(content_path, rows_path),
        [command, rows_path] if command == "collect" => encode(&collect(&rows(rows_path)?)),
        [command, content_path, events_path] if command == "state" => {
            encode(&Engine::new(&content(content_path)?, &events(events_path)?).state())
        }
        _ => Err(USAGE.to_owned()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
