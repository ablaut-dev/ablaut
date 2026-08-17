//! Irish golden-tgle harness: diff the engine against the
//! agreement of the two Irish oracles (BuNaMo and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_gle [gold.tsv ...] [--check]
//!        (default: data/gle/bunamo.tsv data/gle/kaikki.tsv —
//!         see `scripts/gle/fetch_bunamo.sh`, `scripts/gle/fetch_kaikki.sh`)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); disagreements are the adjudication corpus.
//! With one file, that file is scored directly (the CI path).

use ablaut::gle::{Slot, Tense, Verb};
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 8] = [
    "nonfinite",
    "present",
    "past",
    "past-habitual",
    "future",
    "conditional",
    "imperative",
    "subjunctive",
];

/// Map a feature bundle from the oracle TSVs to the engine's output
/// (None means unsupported bundle).
fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    let slot = |s: &str| match s {
        "BASE" => Some(Slot::Base),
        "1SG" => Some(Slot::FirstSingular),
        "2SG" => Some(Slot::SecondSingular),
        "1PL" => Some(Slot::FirstPlural),
        "2PL" => Some(Slot::SecondPlural),
        "3PL" => Some(Slot::ThirdPlural),
        "AUTO" => Some(Slot::Autonomous),
        _ => None,
    };
    let tense = |t: &str| match t {
        "PRS" => Some(Tense::Present),
        "PST" => Some(Tense::Past),
        "PSTHAB" => Some(Tense::PastHabitual),
        "FUT" => Some(Tense::Future),
        "COND" => Some(Tense::Conditional),
        "IMP" => Some(Tense::Imperative),
        "SBJV" => Some(Tense::Subjunctive),
        _ => None,
    };
    match f.as_slice() {
        ["V", "VN"] => Some(vec![verb.verbal_noun()]),
        ["V.PTCP"] => Some(vec![verb.verbal_adjective()]),
        ["V", t, s] => {
            let (t, s) = (tense(t)?, slot(s)?);
            verb.form(t, s).map(|f| vec![f])
        }
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
fn category(features: &str) -> &'static str {
    if features == "V;VN" || features == "V.PTCP" {
        "nonfinite"
    } else if features.starts_with("V;PRS") {
        "present"
    } else if features.starts_with("V;PSTHAB") {
        "past-habitual"
    } else if features.starts_with("V;PST") {
        "past"
    } else if features.starts_with("V;FUT") {
        "future"
    } else if features.starts_with("V;COND") {
        "conditional"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else {
        "subjunctive"
    }
}

fn main() {
    run(Spec {
        lang: "gle",
        default_paths: ["data/gle/bunamo.tsv", "data/gle/kaikki.tsv"],
        adjudications: "docs/gle/adjudications.tsv",
        mismatches: "target/golden_gle_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.0,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
