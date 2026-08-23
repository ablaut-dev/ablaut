//! Azerbaijani golden-test harness: diff the engine against the single
//! Azerbaijani oracle (kaikki.org) — Beta tier, no independent second
//! oracle overlaps enough to form the agreement loop.
//!
//! Usage: cargo run --release --bin golden_aze [gold.tsv ...] [--check]
//!        (default: data/aze/kaikki.tsv)

use ablaut::aze::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 6] = ["infinitive", "present", "past", "future", "aorist", "imperative"];

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
        lang: "aze",
        // Single oracle: the second path does not exist, so kaikki is
        // scored directly (Beta).
        default_paths: ["data/aze/kaikki.tsv", "data/aze/_no_second_oracle.tsv"],
        adjudications: "docs/aze/adjudications.tsv",
        mismatches: "target/golden_aze_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
