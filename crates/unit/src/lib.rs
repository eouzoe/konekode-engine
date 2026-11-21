use std::collections::BTreeMap;
use std::fmt;

pub const RESET_TOKEN: &str = "reset";
pub const RUN_TOKEN: &str = "run";
const INDENT: &str = "    ";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Stage {
    Code,
    Drill,
    Empty,
    Run,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum Wait {
    Drill,
    Run,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum When {
    Fullscreen,
    Windowed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub enum Needs {
    FullscreenKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum After {
    Next,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Line {
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub after: Option<After>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub append: Option<bool>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub auto: Option<f64>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub demo: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub expect: Option<String>,
    pub id: String,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub input: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub keys: Option<Vec<String>>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub lines: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub needs: Option<Needs>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub snippet: Option<String>,
    pub stage: Stage,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub unlock: Option<Vec<String>>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub wait: Option<Wait>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub when: Option<When>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Beat {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub line: Line,
    pub say: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Shown {
    pub id: String,
    pub keys: Vec<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub teaches: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Task {
    pub demo: Vec<Shown>,
    pub expect: String,
    pub goal: String,
    pub id: String,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub input: Option<String>,
    pub show: String,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum Part {
    Task { task: Task },
    Line(Line),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Plan {
    pub history: Vec<String>,
    pub path: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    NoSentence(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSentence(id) => write!(formatter, "no sentence for {id}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Text = BTreeMap<String, String>;

fn said(text: &Text, id: &str) -> Result<String, Error> {
    text.get(id)
        .cloned()
        .ok_or_else(|| Error::NoSentence(id.to_owned()))
}

fn keys_of(shown: &Shown, first: bool) -> Vec<String> {
    if first {
        return std::iter::once(RESET_TOKEN.to_owned())
            .chain(shown.keys.iter().cloned())
            .collect();
    }
    shown.keys.clone()
}

fn bare(id: &str) -> Line {
    Line {
        after: None,
        append: None,
        auto: None,
        demo: None,
        expect: None,
        id: id.to_owned(),
        input: None,
        keys: None,
        lines: None,
        needs: None,
        snippet: None,
        stage: Stage::Run,
        unlock: None,
        wait: None,
        when: None,
    }
}

fn shown_beat(task: &Task, shown: &Shown, first: bool, say: String) -> Beat {
    Beat {
        line: Line {
            keys: Some(keys_of(shown, first)),
            snippet: Some(task.show.clone()),
            unlock: shown.teaches.clone(),
            ..bare(&shown.id)
        },
        say,
    }
}

fn gate_beat(task: &Task, say: String) -> Beat {
    Beat {
        line: Line {
            expect: Some(task.expect.clone()),
            input: task.input.clone(),
            snippet: Some(task.show.clone()),
            wait: Some(Wait::Run),
            ..bare(&task.id)
        },
        say,
    }
}

fn task_beats(task: &Task, text: &Text) -> Result<Vec<Beat>, Error> {
    let mut beats = Vec::new();
    for (at, shown) in task.demo.iter().enumerate() {
        beats.push(shown_beat(task, shown, at == 0, said(text, &shown.id)?));
    }
    beats.push(gate_beat(task, said(text, &task.id)?));
    Ok(beats)
}

pub fn flatten(parts: &[Part], text: &Text) -> Result<Vec<Beat>, Error> {
    let mut beats = Vec::new();
    for part in parts {
        match part {
            Part::Task { task } => beats.extend(task_beats(task, text)?),
            Part::Line(line) => beats.push(Beat {
                line: line.clone(),
                say: said(text, &line.id)?,
            }),
        }
    }
    Ok(beats)
}

#[must_use]
pub fn tasks_of(parts: &[Part]) -> Vec<Task> {
    parts
        .iter()
        .filter_map(|part| match part {
            Part::Task { task } => Some(task.clone()),
            Part::Line(_) => None,
        })
        .collect()
}

#[must_use]
pub fn path_of(task: &Task) -> Vec<String> {
    let mut keys: Vec<String> = task
        .demo
        .iter()
        .flat_map(|shown| shown.keys.iter().cloned())
        .collect();
    if keys.is_empty() || keys.last().is_some_and(|last| last == RUN_TOKEN) {
        return keys;
    }
    keys.push(RUN_TOKEN.to_owned());
    keys
}

#[must_use]
pub fn program_of(starter: &str, goal: &str) -> String {
    let lines: Vec<&str> = starter.split('\n').collect();
    let (last, head) = lines
        .split_last()
        .map_or((None, &[][..]), |(last, head)| (Some(*last), head));
    let body = goal.split('\n').map(|line| format!("{INDENT}{line}"));
    head.iter()
        .map(|line| (*line).to_owned())
        .chain(body)
        .chain(last.map(str::to_owned))
        .collect::<Vec<_>>()
        .join("\n")
}

fn holds(task: &Task, beat: &str) -> bool {
    task.id == beat || task.demo.iter().any(|shown| shown.id == beat)
}

#[must_use]
pub fn plan_for(tasks: &[Task], beat: &str) -> Option<Plan> {
    let at = tasks.iter().position(|task| holds(task, beat))?;
    let task = tasks.get(at)?;
    let history = tasks
        .iter()
        .take(at)
        .flat_map(path_of)
        .filter(|token| token != RUN_TOKEN)
        .collect();
    Some(Plan {
        history,
        path: path_of(task),
    })
}
