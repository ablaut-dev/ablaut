//! Belarusian golden-test harness: diff the engine against the agreement
//! of the two Belarusian oracles (UniMorph and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_bel [gold.tsv ...] [--check]
//!        (default: data/bel/unimorph.tsv data/bel/kaikki.tsv)

use ablaut::bel::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 4] = ["present", "future", "past", "imperative"];

fn category(features: &str) -> &'static str {
    if features.starts_with("V;PRS") {
        "present"
    } else if features.starts_with("V;FUT") {
        "future"
    } else if features.starts_with("V;PST") {
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
        lang: "bel",
        default_paths: ["data/bel/unimorph.tsv", "data/bel/kaikki.tsv"],
        adjudications: "docs/bel/adjudications.tsv",
        mismatches: "target/golden_bel_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
