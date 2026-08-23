//! Galician golden-test harness: diff the engine against the single
//! Galician oracle (kaikki.org) — Beta tier, no independent second oracle
//! overlaps enough to form the agreement loop.
//!
//! Usage: cargo run --release --bin golden_glg [gold.tsv ...] [--check]
//!        (default: data/glg/kaikki.tsv)

use ablaut::glg::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 12] = [
    "infinitive",
    "gerund",
    "participle",
    "present",
    "imperfect",
    "preterite",
    "pluperfect",
    "future",
    "conditional",
    "subjunctive",
    "personal-infinitive",
    "imperative",
];

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features == "V;GER" {
        "gerund"
    } else if features.starts_with("V;PTCP") {
        "participle"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IND;IPFV") {
        "imperfect"
    } else if features.starts_with("V;IND;PRET") {
        "preterite"
    } else if features.starts_with("V;IND;PLUP") {
        "pluperfect"
    } else if features.starts_with("V;IND;FUT") {
        "future"
    } else if features.starts_with("V;COND") {
        "conditional"
    } else if features.starts_with("V;SBJV") {
        "subjunctive"
    } else if features.starts_with("V;INF") {
        "personal-infinitive"
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
        lang: "glg",
        // Single oracle: the second path does not exist, so kaikki is
        // scored directly (Beta).
        default_paths: ["data/glg/kaikki.tsv", "data/glg/_no_second_oracle.tsv"],
        adjudications: "docs/glg/adjudications.tsv",
        mismatches: "target/golden_glg_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
