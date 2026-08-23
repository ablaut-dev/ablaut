//! Faroese golden-test harness: diff the engine against the agreement of
//! the two Faroese oracles (UniMorph and kaikki.org). kaikki's Faroese
//! tables are sparse, so their overlap with UniMorph is chiefly the supine
//! and the past indicative — a narrow but genuine second attestation.
//!
//! Usage: cargo run --release --bin golden_fao [gold.tsv ...] [--check]
//!        (default: data/fao/unimorph.tsv data/fao/kaikki.tsv)

use ablaut::fao::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "present",
    "past",
    "imperative",
    "supine",
    "participle",
];

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V.PTCP") {
        "participle"
    } else if features == "V.CVB" {
        "supine"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IND;PST") {
        "past"
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
        lang: "fao",
        default_paths: ["data/fao/unimorph.tsv", "data/fao/kaikki.tsv"],
        adjudications: "docs/fao/adjudications.tsv",
        mismatches: "target/golden_fao_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
