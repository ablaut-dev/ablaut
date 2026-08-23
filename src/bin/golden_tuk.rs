//! Turkmen golden-test harness: diff the engine against the single
//! Turkmen oracle (kaikki.org) — Beta tier.
//!
//! Usage: cargo run --release --bin golden_tuk [gold.tsv ...] [--check]
//!        (default: data/tuk/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::tuk::Verb;

const CATEGORIES: [&str; 5] = ["infinitive", "present", "past", "aorist", "imperative"];

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V;PRS") {
        "present"
    } else if features.starts_with("V;PST") {
        "past"
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
        lang: "tuk",
        default_paths: ["data/tuk/kaikki.tsv", "data/tuk/_no_second_oracle.tsv"],
        adjudications: "docs/tuk/adjudications.tsv",
        mismatches: "target/golden_tuk_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
