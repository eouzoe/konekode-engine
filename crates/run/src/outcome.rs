use crate::explain::explain;
use crate::say::{Missing, Text, line, say};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "kind", rename_all = "kebab-case")
)]
pub enum Outcome {
    Finished {
        #[cfg_attr(feature = "serde", serde(rename = "exitCode"))]
        exit_code: i64,
        output: String,
    },
    CompileError {
        message: String,
    },
    Crashed {
        message: String,
    },
    Timeout,
    Unavailable {
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct View {
    pub state: String,
    pub status: String,
    pub text: String,
}

const COMPLETE: i64 = 100;

fn view(state: &str, status: String, text: String) -> View {
    View {
        state: state.to_owned(),
        status,
        text,
    }
}

pub fn describe(text: &Text, outcome: &Outcome) -> Result<View, Missing> {
    Ok(match outcome {
        Outcome::Finished { exit_code, output } if *exit_code != 0 => {
            view("error", line(text, "run.stopped")?, output.clone())
        }
        Outcome::Finished { output, .. } => view("ok", line(text, "run.finished")?, output.clone()),
        Outcome::CompileError { message } => view(
            "error",
            line(text, "run.compileError")?,
            explain(text, message)?,
        ),
        Outcome::Crashed { message } => view("error", line(text, "run.crashed")?, message.clone()),
        Outcome::Timeout => view("error", line(text, "run.timeout")?, String::new()),
        Outcome::Unavailable { .. } => view(
            "error",
            line(text, "run.unavailable")?,
            line(text, "run.retry")?,
        ),
    })
}

pub fn busy_status(text: &Text, percent: i64) -> Result<String, Missing> {
    if percent < COMPLETE {
        return say(text, "run.loading", &[("percent", &percent.to_string())]);
    }
    line(text, "run.running")
}
