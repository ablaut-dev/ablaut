//! Yiddish golden-test harness: diff the engine against the single Yiddish
//! oracle (kaikki.org) — Beta tier, no independent second oracle overlaps
//! enough to form the agreement loop.
//!
//! Usage: cargo run --release --bin golden_ydd [gold.tsv ...] [--check]
//!        (default: data/ydd/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::ydd::Verb;

const CATEGORIES: [&str; 5] = [
    "infinitive",
    "present",
    "imperative",
    "present-participle",
    "past-participle",
];

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features == "V.PTCP;PRS" {
        "present-participle"
    } else if features == "V.PTCP;PST" {
        "past-participle"
    } else if features.starts_with("V;PRS") {
        "present"
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
        lang: "ydd",
        // Single oracle: the second path does not exist, so kaikki is scored
        // directly (Beta).
        default_paths: ["data/ydd/kaikki.tsv", "data/ydd/_no_second_oracle.tsv"],
        adjudications: "docs/ydd/adjudications.tsv",
        mismatches: "target/golden_ydd_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
