use std::collections::BTreeSet;
use std::io::{self, BufRead, Write};

use konekode_complete::{Engine, PairItem, auto_closed, catalog, is_include_line, match_pairs};
use konekode_edit::{
    Buffer, Hint, Rules, Session, Slip, between, cause_of, create_buffer, effect_of, expand,
    expected_now, final_text, fulfil, goal_of, indent_below, indent_split, keys_of, known_keys,
    press, run, step,
};
use konekode_record::{
    Clock, Entry, Event, Source, build, latest_of, ordered, read_counted, read_event,
};
use konekode_typing::{
    Drill, Feedback, KeyInput, Next, attempt_of, create_drill, escape, expected_char, find_key,
    has_started, is_done, is_typed, judge, needs_telling, remind, rows, type_into,
};
use konekode_unit::{Part, Task, Text, flatten, path_of, plan_for, program_of, tasks_of};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

type Reply = Option<Result<Value, String>>;

#[derive(Deserialize)]
struct Request {
    op: String,
    #[serde(default)]
    args: Value,
}

#[derive(Deserialize)]
struct Call {
    entry: Entry,
    id: String,
    kind: String,
    now: i64,
}

fn arg<T: DeserializeOwned>(args: &Value, name: &str) -> Result<T, String> {
    let value = args.get(name).cloned().unwrap_or(Value::Null);
    serde_json::from_value(value).map_err(|error| format!("{name}: {error}"))
}

fn encode<T: serde::Serialize>(value: &T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|error| error.to_string())
}

fn engine_view(args: &Value) -> Result<Value, String> {
    let unlocked = arg::<BTreeSet<String>>(args, "unlocked")?;
    let engine = Engine::new(catalog());
    Ok(json!({
        "brackets": engine.brackets(&unlocked),
        "keys": engine.keys(&unlocked),
        "pairs": encode(&engine.pairs(&unlocked))?,
        "linePair": encode(&engine.line_pair(&unlocked, "<"))?,
    }))
}

fn match_pairs_op(args: &Value) -> Result<Value, String> {
    encode(&match_pairs(
        &arg::<String>(args, "text")?,
        &arg::<Vec<PairItem>>(args, "pairs")?,
    ))
}

fn auto_closed_op(args: &Value) -> Result<Value, String> {
    encode(&auto_closed(
        &arg::<String>(args, "text")?,
        arg(args, "position")?,
        &arg::<Vec<PairItem>>(args, "pairs")?,
    ))
}

fn complete_op(op: &str, args: &Value) -> Reply {
    Some(match op {
        "matchPairs" => match_pairs_op(args),
        "autoClosed" => auto_closed_op(args),
        "isIncludeLine" => {
            arg::<String>(args, "line").and_then(|line| encode(&is_include_line(&line)))
        }
        "catalog" => encode(&catalog()),
        "engine" => engine_view(args),
        _ => return None,
    })
}

fn step_op(args: &Value) -> Result<Value, String> {
    encode(&step(
        &arg::<Buffer>(args, "buffer")?,
        &arg::<String>(args, "key")?,
        &arg::<Rules>(args, "rules")?,
    ))
}

fn run_op(args: &Value) -> Result<Value, String> {
    encode(&run(
        &arg::<Buffer>(args, "buffer")?,
        &arg::<Vec<String>>(args, "keys")?,
        &arg::<Rules>(args, "rules")?,
    ))
}

fn effect_op(args: &Value) -> Result<Value, String> {
    encode(&effect_of(
        &arg::<Buffer>(args, "before")?,
        &arg::<Buffer>(args, "after")?,
        &arg::<String>(args, "key")?,
    ))
}

fn buffer_op(op: &str, args: &Value) -> Reply {
    Some(match op {
        "createBuffer" => arg::<String>(args, "text")
            .and_then(|text| encode(&create_buffer(&text, arg(args, "cursor")?))),
        "step" => step_op(args),
        "run" => run_op(args),
        "effectOf" => effect_op(args),
        "indentBelow" => arg::<String>(args, "text")
            .and_then(|text| encode(&indent_below(&text, arg(args, "at")?))),
        "indentSplit" => arg::<String>(args, "head")
            .and_then(|head| encode(&indent_split(&head, &arg::<String>(args, "tail")?))),
        "knownKeys" => encode(&known_keys()),
        _ => return None,
    })
}

fn guided(args: &Value) -> Result<(Session, Rules), String> {
    Ok((
        arg::<Session>(args, "session")?,
        arg::<Rules>(args, "rules")?,
    ))
}

fn session_op(op: &str, args: &Value) -> Reply {
    Some(match op {
        "press" => guided(args).and_then(|(session, rules)| {
            encode(&press(&session, &arg::<String>(args, "key")?, &rules))
        }),
        "goalOf" => guided(args).and_then(|(session, rules)| encode(&goal_of(&session, &rules))),
        "finalText" => {
            guided(args).and_then(|(session, rules)| encode(&final_text(&session, &rules)))
        }
        "between" => arg::<String>(args, "text")
            .and_then(|text| encode(&between(&text, &arg::<String>(args, "final")?))),
        "fulfil" => arg::<String>(args, "text")
            .and_then(|text| encode(&fulfil(&text, &arg::<Vec<Hint>>(args, "hints")?))),
        "expand" => arg::<Vec<String>>(args, "path").and_then(|path| encode(&expand(&path))),
        "keysOf" => arg::<Vec<String>>(args, "path").and_then(|path| encode(&keys_of(&path))),
        "causeOf" => arg::<Slip>(args, "slip").and_then(|slip| encode(&cause_of(&slip))),
        "expectedNow" => {
            arg::<Session>(args, "session").and_then(|session| encode(&expected_now(&session)))
        }
        _ => return None,
    })
}

fn flatten_op(args: &Value) -> Result<Value, String> {
    flatten(
        &arg::<Vec<Part>>(args, "parts")?,
        &arg::<Text>(args, "text")?,
    )
    .map_err(|error| error.to_string())
    .and_then(|beats| encode(&beats))
}

fn unit_op(op: &str, args: &Value) -> Reply {
    Some(match op {
        "flatten" => flatten_op(args),
        "tasksOf" => arg::<Vec<Part>>(args, "parts").and_then(|parts| encode(&tasks_of(&parts))),
        "pathOf" => arg::<Task>(args, "task").and_then(|task| encode(&path_of(&task))),
        "programOf" => arg::<String>(args, "starter")
            .and_then(|starter| encode(&program_of(&starter, &arg::<String>(args, "goal")?))),
        "planFor" => arg::<Vec<Task>>(args, "tasks")
            .and_then(|tasks| encode(&plan_for(&tasks, &arg::<String>(args, "beat")?))),
        _ => return None,
    })
}

fn track(args: &Value) -> Result<Value, String> {
    let loaded = read_counted(&arg::<Vec<Value>>(args, "loaded")?).events;
    let source = arg::<Source>(args, "source")?;
    let mut clock = Clock::after(&loaded);
    let made: Vec<Event> = arg::<Vec<Call>>(args, "calls")?
        .iter()
        .map(|call| {
            build(
                &source,
                &mut clock,
                &call.kind,
                &call.entry,
                call.now,
                &call.id,
            )
        })
        .collect();
    encode(&made)
}

fn record_op(op: &str, args: &Value) -> Reply {
    Some(match op {
        "readEvent" => arg::<Value>(args, "value").and_then(|value| encode(&read_event(&value))),
        "readCounted" => {
            arg::<Vec<Value>>(args, "values").and_then(|values| encode(&read_counted(&values)))
        }
        "ordered" => arg::<Vec<Event>>(args, "events").and_then(|events| encode(&ordered(&events))),
        "latestOf" => {
            arg::<Vec<Event>>(args, "events").and_then(|events| encode(&latest_of(&events)))
        }
        "track" => track(args),
        _ => return None,
    })
}

fn next_value(next: &Next) -> Result<Value, String> {
    match next {
        Next::Same => Ok(json!("same")),
        Next::New(drill) => encode(drill),
    }
}

fn judged(args: &Value) -> Result<Value, String> {
    let drill = arg::<Drill>(args, "drill")?;
    let input = arg::<KeyInput>(args, "input")?;
    let after = judge(&drill, &input);
    Ok(json!({
        "after": next_value(&after)?,
        "attempt": encode(&attempt_of(&drill, &after, &input.key))?,
    }))
}

fn drill_state(args: &Value) -> Result<Value, String> {
    let drill = arg::<Drill>(args, "drill")?;
    Ok(json!({
        "expectedChar": expected_char(&drill),
        "hasStarted": has_started(&drill),
        "isDone": is_done(&drill),
        "isTyped": is_typed(&drill),
    }))
}

fn typing_op(op: &str, args: &Value) -> Reply {
    Some(match op {
        "createDrill" => {
            arg::<String>(args, "target").and_then(|target| encode(&create_drill(&target)))
        }
        "drillPress" => arg::<Drill>(args, "drill").and_then(|drill| {
            next_value(&konekode_typing::press(
                &drill,
                &arg::<String>(args, "input")?,
            ))
        }),
        "judge" => judged(args),
        "typeInto" => arg::<Drill>(args, "drill")
            .and_then(|drill| encode(&type_into(&drill, &arg::<String>(args, "text")?))),
        "remind" => arg::<Drill>(args, "drill")
            .and_then(|drill| next_value(&remind(&drill, arg::<Feedback>(args, "feedback")?))),
        "escape" => arg::<Drill>(args, "drill").and_then(|drill| encode(&escape(&drill))),
        "drillState" => drill_state(args),
        "findKey" => arg::<String>(args, "char").and_then(|char| encode(&find_key(&char))),
        "needsTelling" => {
            arg::<String>(args, "char").and_then(|char| encode(&needs_telling(&char)))
        }
        "rows" => encode(&rows()),
        _ => return None,
    })
}

fn answer(op: &str, args: &Value) -> Result<Value, String> {
    complete_op(op, args)
        .or_else(|| buffer_op(op, args))
        .or_else(|| session_op(op, args))
        .or_else(|| unit_op(op, args))
        .or_else(|| record_op(op, args))
        .or_else(|| typing_op(op, args))
        .unwrap_or_else(|| Err(format!("unknown op {op}")))
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in stdin.lock().lines() {
        let line = line?;
        let reply = match serde_json::from_str::<Request>(&line) {
            Ok(request) => match answer(&request.op, &request.args) {
                Ok(value) => json!({ "ok": value }),
                Err(error) => json!({ "error": error }),
            },
            Err(error) => json!({ "error": error.to_string() }),
        };
        writeln!(stdout, "{reply}")?;
    }
    stdout.flush()
}
