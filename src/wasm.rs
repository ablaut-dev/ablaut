//! WebAssembly bindings (feature `wasm`): a JS-friendly API returning the
//! full conjugation table of a verb as one structured object.
//!
//! Build with: `wasm-pack build --target web -- --features wasm`

use crate::table::Table;
use crate::Verb;
use wasm_bindgen::prelude::*;

/// The full conjugation table for an infinitive:
/// `conjugate("aufstehen").present[0]` → "stehe auf",
/// `conjugate("parler", "fra").present[0]` → "parle". The language
/// defaults to German; throws for unknown languages and for strings that
/// are not infinitives of the requested language.
#[wasm_bindgen]
pub fn conjugate(infinitive: &str, lang: Option<String>) -> Result<JsValue, JsError> {
    let lang = lang.as_deref().unwrap_or("deu");
    match crate::Lang::from_code(lang) {
        Some(crate::Lang::Deu) => {
            let v = Verb::from_infinitive(infinitive).map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&Table::build(&v))?)
        }
        Some(crate::Lang::Fra) => {
            let v = crate::fra::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::fra::Table::build(&v))?)
        }
        Some(crate::Lang::Swe) => {
            let v = crate::swe::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::swe::Table::build(&v))?)
        }
        Some(crate::Lang::Ita) => {
            let v = crate::ita::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ita::Table::build(&v))?)
        }
        Some(crate::Lang::Por) => {
            let v = crate::por::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::por::Table::build(&v))?)
        }
        Some(crate::Lang::Spa) => {
            let v = crate::spa::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::spa::Table::build(&v))?)
        }
        None => Err(JsError::new(&format!("unknown language: {lang}"))),
    }
}
