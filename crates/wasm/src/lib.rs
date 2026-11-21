use konekode_flow::{Content, Row, collect};
use konekode_record::{read_event, read_events};
use serde_json::Value;
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

#[wasm_bindgen]
pub struct Engine {
    inner: konekode_flow::Engine,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(content: &str, events: &str) -> Result<Engine, JsError> {
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
}

#[wasm_bindgen(js_name = collectRows)]
pub fn collect_rows(rows: &str) -> Result<String, JsError> {
    let rows: Vec<Row> = serde_json::from_str(rows)?;
    Ok(serde_json::to_string(&collect(&rows))?)
}
