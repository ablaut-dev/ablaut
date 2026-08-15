//! WebAssembly bindings (feature `wasm`): a JS-friendly API returning the
//! full conjugation table of a verb as one structured object.
//!
//! Build with: `wasm-pack build --target web -- --features wasm`

use crate::{AnalyticTense, Mood, Number, Person, Tense, Verb};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub infinitive: String,
    pub zu_infinitive: String,
    pub auxiliary: String,
    pub present_participle: String,
    pub past_participle: String,
    /// [2sg, 2pl]; null for verbs without an imperative (modals).
    pub imperative: [Option<String>; 2],
    /// Each row is [1sg, 2sg, 3sg, 1pl, 2pl, 3pl].
    pub present: [String; 6],
    pub preterite: [String; 6],
    pub konjunktiv1: [String; 6],
    pub konjunktiv2: [String; 6],
    pub perfect: [String; 6],
    pub pluperfect: [String; 6],
    pub future1: [String; 6],
    pub future2: [String; 6],
    /// würde-form (Futur I in Konjunktiv II).
    pub wuerde: [String; 6],
}

const SLOTS: [(Person, Number); 6] = [
    (Person::First, Number::Singular),
    (Person::Second, Number::Singular),
    (Person::Third, Number::Singular),
    (Person::First, Number::Plural),
    (Person::Second, Number::Plural),
    (Person::Third, Number::Plural),
];

fn row(f: impl Fn(Person, Number) -> String) -> [String; 6] {
    SLOTS.map(|(p, n)| f(p, n))
}

/// The full conjugation table for an infinitive.
#[wasm_bindgen]
pub fn conjugation_table(infinitive: &str) -> Result<JsValue, JsError> {
    let v = Verb::from_infinitive(infinitive).map_err(|e| JsError::new(&e.to_string()))?;
    let table = Table {
        infinitive: v.infinitive().to_string(),
        zu_infinitive: v.zu_infinitive(),
        auxiliary: match v.auxiliary() {
            crate::Auxiliary::Haben => "haben".to_string(),
            crate::Auxiliary::Sein => "sein".to_string(),
        },
        present_participle: v.present_participle(),
        past_participle: v.past_participle(),
        imperative: [
            v.imperative(Number::Singular),
            v.imperative(Number::Plural),
        ],
        present: row(|p, n| v.conjugate(Tense::Present, Mood::Indicative, p, n)),
        preterite: row(|p, n| v.conjugate(Tense::Preterite, Mood::Indicative, p, n)),
        konjunktiv1: row(|p, n| v.conjugate(Tense::Present, Mood::KonjunktivI, p, n)),
        konjunktiv2: row(|p, n| v.conjugate(Tense::Present, Mood::KonjunktivII, p, n)),
        perfect: row(|p, n| v.analytic(AnalyticTense::Perfect, Mood::Indicative, p, n)),
        pluperfect: row(|p, n| v.analytic(AnalyticTense::Pluperfect, Mood::Indicative, p, n)),
        future1: row(|p, n| v.analytic(AnalyticTense::FutureI, Mood::Indicative, p, n)),
        future2: row(|p, n| v.analytic(AnalyticTense::FutureII, Mood::Indicative, p, n)),
        wuerde: row(|p, n| v.analytic(AnalyticTense::FutureI, Mood::KonjunktivII, p, n)),
    };
    Ok(serde_wasm_bindgen::to_value(&table)?)
}
