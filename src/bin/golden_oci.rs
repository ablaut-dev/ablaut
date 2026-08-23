//! Occitan golden-test harness: diff the engine against the agreement of
//! the two Occitan oracles (UniMorph and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_oci [gold.tsv ...] [--check]
//!        (default: data/oci/unimorph.tsv data/oci/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::oci::Verb;

const CATEGORIES: [&str; 11] = [
    "present",
    "imperfect",
    "preterite",
    "future",
    "conditional",
    "subjunctive-present",
    "subjunctive-imperfect",
    "imperative",
    "infinitive",
    "participle",
    "gerund",
];

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features == "V.CVB;PRS" {
        "gerund"
    } else if features.starts_with("V.PTCP") {
        "participle"
    } else if features.contains(";IMP") {
        "imperative"
    } else if features.contains("SBJV;PST") {
        "subjunctive-imperfect"
    } else if features.contains("SBJV;PRS") {
        "subjunctive-present"
    } else if features.contains(";COND") {
        "conditional"
    } else if features.contains(";FUT") {
        "future"
    } else if features.contains(";PFV") {
        "preterite"
    } else if features.contains(";IPFV") {
        "imperfect"
    } else {
        "present"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    Some(vec![verb.form(features)?])
}

fn main() {
    run(Spec {
        lang: "oci",
        default_paths: ["data/oci/unimorph.tsv", "data/oci/kaikki.tsv"],
        adjudications: "docs/oci/adjudications.tsv",
        mismatches: "target/golden_oci_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
