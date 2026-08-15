//! WebAssembly bindings (feature `wasm`): a JS-friendly API returning the
//! full conjugation table of a verb as one structured object.
//!
//! Build with: `wasm-pack build --target web -- --features wasm`

use crate::table::Table;
use crate::Verb;
use wasm_bindgen::prelude::*;

/// The full conjugation table for an infinitive:
/// `conjugate("aufstehen").present[0]` → "stehe auf". Throws for strings
/// that are not German infinitives.
#[wasm_bindgen]
pub fn conjugate(infinitive: &str) -> Result<JsValue, JsError> {
    let v = Verb::from_infinitive(infinitive).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(serde_wasm_bindgen::to_value(&Table::build(&v))?)
}
