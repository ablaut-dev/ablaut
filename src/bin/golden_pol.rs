//! Polish golden-test harness: diff the engine against the agreement of
//! the two Polish oracles (UniMorph and SGJP).
//!
//! Usage: cargo run --release --bin golden_pol [gold.tsv ...] [--check]
//!        (default: data/pol/unimorph.tsv data/pol/sgjp.tsv)

use ablaut::harness::{run, Spec};
use ablaut::pol::Verb;

const CATEGORIES: [&str; 5] = ["infinitive", "present", "future", "past", "imperative"];

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V;PRS") {
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
        lang: "pol",
        default_paths: ["data/pol/unimorph.tsv", "data/pol/sgjp.tsv"],
        adjudications: "docs/pol/adjudications.tsv",
        mismatches: "target/golden_pol_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
