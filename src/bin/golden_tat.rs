//! Tatar golden-test harness: diff the engine against the single Tatar
//! oracle (kaikki.org) — Beta tier. The UniMorph Tatar table is
//! Latin-script (Common Turkic orthography) while kaikki is Cyrillic Kazan
//! Tatar, so the two do not overlap and no agreement loop can form.
//!
//! Usage: cargo run --release --bin golden_tat [gold.tsv ...] [--check]
//!        (default: data/tat/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::tat::Verb;

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "present",
    "past",
    "future",
    "conditional",
    "imperative",
];

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V;PRS") {
        "present"
    } else if features.starts_with("V;PST") {
        "past"
    } else if features.starts_with("V;FUT") {
        "future"
    } else if features.starts_with("V;COND") {
        "conditional"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else {
        "present"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    Some(vec![verb.form(features)?])
}

fn main() {
    run(Spec {
        lang: "tat",
        // Single oracle: the second path does not exist, so kaikki is
        // scored directly (Beta).
        default_paths: ["data/tat/kaikki.tsv", "data/tat/_no_second_oracle.tsv"],
        adjudications: "docs/tat/adjudications.tsv",
        mismatches: "target/golden_tat_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
