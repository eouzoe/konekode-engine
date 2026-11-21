use std::collections::BTreeSet;
use std::io::{self, BufRead, Write};

use konekode_complete::{Engine, PairItem, auto_closed, catalog, is_include_line, match_pairs};
use konekode_edit::{
    Buffer, Hint, Rules, Session, Slip, between, cause_of, create_buffer, effect_of, expand,
    expected_now, final_text, fulfil, goal_of, indent_below, indent_split, keys_of, known_keys,
    press, run, step,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Request {
    op: String,
    #[serde(default)]
    args: Value,
}

fn arg<T: DeserializeOwned>(args: &Value, name: &str) -> Result<T, String> {
    let value = args.get(name).cloned().unwrap_or(Value::Null);
    serde_json::from_value(value).map_err(|error| format!("{name}: {error}"))
}

fn encode<T: serde::Serialize>(value: &T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|error| error.to_string())
}

fn engine_view(unlocked: &BTreeSet<String>) -> Result<Value, String> {
    let engine = Engine::new(catalog());
    Ok(json!({
        "brackets": engine.brackets(unlocked),
        "keys": engine.keys(unlocked),
        "pairs": encode(&engine.pairs(unlocked))?,
        "linePair": encode(&engine.line_pair(unlocked, "<"))?,
    }))
}

fn answer(op: &str, args: &Value) -> Result<Value, String> {
    match op {
        "createBuffer" => encode(&create_buffer(
            &arg::<String>(args, "text")?,
            arg(args, "cursor")?,
        )),
        "step" => encode(&step(
            &arg::<Buffer>(args, "buffer")?,
            &arg::<String>(args, "key")?,
            &arg::<Rules>(args, "rules")?,
        )),
        "run" => encode(&run(
            &arg::<Buffer>(args, "buffer")?,
            &arg::<Vec<String>>(args, "keys")?,
            &arg::<Rules>(args, "rules")?,
        )),
        "press" => encode(&press(
            &arg::<Session>(args, "session")?,
            &arg::<String>(args, "key")?,
            &arg::<Rules>(args, "rules")?,
        )),
        "effectOf" => encode(&effect_of(
            &arg::<Buffer>(args, "before")?,
            &arg::<Buffer>(args, "after")?,
            &arg::<String>(args, "key")?,
        )),
        "goalOf" => encode(&goal_of(
            &arg::<Session>(args, "session")?,
            &arg::<Rules>(args, "rules")?,
        )),
        "finalText" => encode(&final_text(
            &arg::<Session>(args, "session")?,
            &arg::<Rules>(args, "rules")?,
        )),
        "between" => encode(&between(
            &arg::<String>(args, "text")?,
            &arg::<String>(args, "final")?,
        )),
        "fulfil" => encode(&fulfil(
            &arg::<String>(args, "text")?,
            &arg::<Vec<Hint>>(args, "hints")?,
        )),
        "expand" => encode(&expand(&arg::<Vec<String>>(args, "path")?)),
        "keysOf" => encode(&keys_of(&arg::<Vec<String>>(args, "path")?)),
        "causeOf" => encode(&cause_of(&arg::<Slip>(args, "slip")?)),
        "expectedNow" => encode(&expected_now(&arg::<Session>(args, "session")?)),
        "indentBelow" => encode(&indent_below(
            &arg::<String>(args, "text")?,
            arg(args, "at")?,
        )),
        "indentSplit" => encode(&indent_split(
            &arg::<String>(args, "head")?,
            &arg::<String>(args, "tail")?,
        )),
        "matchPairs" => encode(&match_pairs(
            &arg::<String>(args, "text")?,
            &arg::<Vec<PairItem>>(args, "pairs")?,
        )),
        "autoClosed" => encode(&auto_closed(
            &arg::<String>(args, "text")?,
            arg(args, "position")?,
            &arg::<Vec<PairItem>>(args, "pairs")?,
        )),
        "isIncludeLine" => encode(&is_include_line(&arg::<String>(args, "line")?)),
        "knownKeys" => encode(&known_keys()),
        "catalog" => encode(&catalog()),
        "engine" => engine_view(&arg::<BTreeSet<String>>(args, "unlocked")?),
        _ => Err(format!("unknown op {op}")),
    }
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
