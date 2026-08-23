//! Luxembourgish golden-test harness: diff the engine against the single
//! Luxembourgish oracle (kaikki.org) — Beta tier, no independent second
//! oracle overlaps enough to form the agreement loop.
//!
//! Usage: cargo run --release --bin golden_ltz [gold.tsv ...] [--check]
//!        (default: data/ltz/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::ltz::Verb;

const CATEGORIES: [&str; 4] = ["infinitive", "present", "imperative", "participle"];

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V.PTCP") {
        "participle"
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
        lang: "ltz",
        // Single oracle: the second path does not exist, so kaikki is
        // scored directly (Beta).
        default_paths: ["data/ltz/kaikki.tsv", "data/ltz/_no_second_oracle.tsv"],
        adjudications: "docs/ltz/adjudications.tsv",
        mismatches: "target/golden_ltz_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
