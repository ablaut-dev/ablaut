//! Uzbek golden-test harness: diff the engine against the single Uzbek
//! oracle (kaikki.org) — Beta tier, third-person core.
//!
//! Usage: cargo run --release --bin golden_uzb [gold.tsv ...] [--check]
//!        (default: data/uzb/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::uzb::Verb;

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "present",
    "past",
    "future",
    "aorist",
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
    } else if features.starts_with("V;AOR") {
        "aorist"
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
        lang: "uzb",
        default_paths: ["data/uzb/kaikki.tsv", "data/uzb/_no_second_oracle.tsv"],
        adjudications: "docs/uzb/adjudications.tsv",
        mismatches: "target/golden_uzb_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
