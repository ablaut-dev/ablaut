//! Scottish Gaelic golden harness: diff the engine against the
//! kaikki.org Scottish Gaelic gold (`data/gla/kaikki.tsv`).
//!
//! Usage: cargo run --release --bin golden_gla [gold.tsv ...] [--check]
//!        (default: data/gla/kaikki.tsv — see scripts/gla/kaikki_to_tsv.py)

use ablaut::gla::{Slot, Tense, Verb};
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 6] = [
    "nonfinite",
    "past",
    "future",
    "conditional",
    "imperative",
    "relative-future",
];

/// Map a feature bundle from the gold TSV to the engine's output
/// (None means unsupported bundle).
fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    let one = |t: Tense, s: Slot| verb.form(t, s).map(|f| vec![f]);
    match f.as_slice() {
        ["V", "VN"] => Some(vec![verb.verbal_noun()]),
        ["V.PTCP"] => Some(vec![verb.verbal_adjective()]),
        ["V", "PST", "IND"] => one(Tense::Past, Slot::Independent),
        ["V", "PST", "DEP"] => one(Tense::Past, Slot::Dependent),
        ["V", "PST", "IMPRS"] => one(Tense::Past, Slot::Impersonal),
        ["V", "FUT", "IND"] => one(Tense::Future, Slot::Independent),
        ["V", "FUT", "DEP"] => one(Tense::Future, Slot::Dependent),
        ["V", "FUT", "IMPRS"] => one(Tense::Future, Slot::Impersonal),
        ["V", "FUT", "REL"] => one(Tense::RelativeFuture, Slot::Independent),
        ["V", "COND", "3"] => one(Tense::Conditional, Slot::Third),
        ["V", "COND", "1SG"] => one(Tense::Conditional, Slot::FirstSingular),
        ["V", "COND", "1PL"] => one(Tense::Conditional, Slot::FirstPlural),
        ["V", "COND", "IMPRS"] => one(Tense::Conditional, Slot::Impersonal),
        ["V", "IMP", "2SG"] => one(Tense::Imperative, Slot::SecondSingular),
        ["V", "IMP", "1SG"] => one(Tense::Imperative, Slot::FirstSingular),
        ["V", "IMP", "1PL"] => one(Tense::Imperative, Slot::FirstPlural),
        ["V", "IMP", "2PL"] => one(Tense::Imperative, Slot::SecondPlural),
        ["V", "IMP", "3"] => one(Tense::Imperative, Slot::Third),
        ["V", "IMP", "IMPRS"] => one(Tense::Imperative, Slot::Impersonal),
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
fn category(features: &str) -> &'static str {
    if features == "V;VN" || features == "V.PTCP" {
        "nonfinite"
    } else if features.starts_with("V;PST") {
        "past"
    } else if features == "V;FUT;REL" {
        "relative-future"
    } else if features.starts_with("V;FUT") {
        "future"
    } else if features.starts_with("V;COND") {
        "conditional"
    } else {
        "imperative"
    }
}

fn main() {
    run(Spec {
        lang: "gla",
        default_paths: ["data/gla/kaikki.tsv", "data/gla/.no-second-oracle"],
        adjudications: "docs/gla/adjudications.tsv",
        mismatches: "target/golden_gla_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
