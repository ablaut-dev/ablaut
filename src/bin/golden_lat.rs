//! Latin golden-test harness: diff the engine against the agreement of the
//! two Latin oracles (UniMorph and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_lat [gold.tsv ...] [--check]
//!        (default: data/lat/unimorph.tsv data/lat/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::lat::Verb;

const CATEGORIES: [&str; 5] = ["present", "imperfect", "future", "imperative", "infinitive"];

fn category(features: &str) -> &'static str {
    if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V;IND;ACT;PRS") {
        "present"
    } else if features.starts_with("V;IND;ACT;PST;IPFV") {
        "imperfect"
    } else if features.starts_with("V;IND;ACT;FUT") {
        "future"
    } else {
        "present"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    Some(vec![verb.form(features)?])
}

fn main() {
    run(Spec {
        lang: "lat",
        default_paths: ["data/lat/unimorph.tsv", "data/lat/kaikki.tsv"],
        adjudications: "docs/lat/adjudications.tsv",
        mismatches: "target/golden_lat_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
