use konekode_flow::{Content, Row, collect};
use konekode_record::{read_event, read_events};
use serde_json::Value;
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

#[wasm_bindgen]
#[derive(Debug)]
pub struct Engine {
    inner: konekode_flow::Engine,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(content: &str, events: &str) -> Result<Self, JsError> {
        let content: Content = serde_json::from_str(content)?;
        let values: Vec<Value> = serde_json::from_str(events)?;
        Ok(Self {
            inner: konekode_flow::Engine::new(&content, &read_events(&values)),
        })
    }

    pub fn push(&mut self, event: &str) -> Result<String, JsError> {
        let value: Value = serde_json::from_str(event)?;
        let event = read_event(&value).ok_or_else(|| JsError::new("unreadable event"))?;
        Ok(serde_json::to_string(&self.inner.push(event))?)
    }

    pub fn state(&self) -> Result<String, JsError> {
        Ok(serde_json::to_string(&self.inner.state())?)
    }

    pub fn estimates(&self, now: f64) -> Result<String, JsError> {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::as_conversions,
            reason = "JavaScript hands the time as a double of whole milliseconds"
        )]
        let now = now as i64;
        Ok(serde_json::to_string(&self.inner.estimates(now))?)
    }

    #[must_use]
    pub fn shows(&self, unit: &str, beat: &str) -> Vec<String> {
        self.inner.shows(unit, beat)
    }

    #[must_use]
    pub fn uses(&self, unit: &str, task: &str, step: f64) -> Vec<String> {
        self.inner.uses(unit, task, step)
    }
}

#[wasm_bindgen(js_name = keptKinds)]
#[must_use]
pub fn kept_kinds() -> Vec<String> {
    konekode_learner::kept_kinds()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[wasm_bindgen(js_name = collectRows)]
pub fn collect_rows(rows: &str) -> Result<String, JsError> {
    let rows: Vec<Row> = serde_json::from_str(rows)?;
    Ok(serde_json::to_string(&collect(&rows))?)
}
