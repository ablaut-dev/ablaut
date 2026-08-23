//! Bulgarian golden-test harness: diff the engine against the agreement
//! of the two Bulgarian oracles (UniMorph and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_bul [gold.tsv ...] [--check]
//!        (default: data/bul/unimorph.tsv data/bul/kaikki.tsv)

use ablaut::bul::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 7] = [
    "present",
    "aorist",
    "imperfect",
    "imperative",
    "participle",
    "verbal-noun",
    "converb",
];

fn category(features: &str) -> &'static str {
    if features.starts_with("V.PTCP") {
        "participle"
    } else if features.starts_with("V.MSDR") {
        "verbal-noun"
    } else if features == "V.CVB" {
        "converb"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IND;PST") {
        "aorist"
    } else if features.starts_with("V;IND;PROG;PST") {
        "imperfect"
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
        lang: "bul",
        default_paths: ["data/bul/unimorph.tsv", "data/bul/kaikki.tsv"],
        adjudications: "docs/bul/adjudications.tsv",
        mismatches: "target/golden_bul_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
