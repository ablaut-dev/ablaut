//! Greek golden-test harness: diff the engine against the agreement of
//! the two Greek oracles (UniMorph and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_ell [gold.tsv ...] [--check]
//!        (default: data/ell/unimorph.tsv data/ell/kaikki.tsv)

use ablaut::ell::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 5] = ["present", "imperfect", "aorist", "imperative", "participle"];

fn category(features: &str) -> &'static str {
    if features == "V;PTCP" {
        "participle"
    } else if features.contains("IPFV;PRS") {
        "present"
    } else if features.contains("IPFV;PST") {
        "imperfect"
    } else if features.contains("PFV;PST") {
        "aorist"
    } else if features.contains("IMP") {
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
        lang: "ell",
        default_paths: ["data/ell/unimorph.tsv", "data/ell/kaikki.tsv"],
        adjudications: "docs/ell/adjudications.tsv",
        mismatches: "target/golden_ell_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
