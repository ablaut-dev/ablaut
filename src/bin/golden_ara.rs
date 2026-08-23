//! Arabic golden-test harness: diff the engine against the agreement of the
//! two Arabic oracles (UniMorph and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_ara [gold.tsv ...] [--check]
//!        (default: data/ara/unimorph.tsv data/ara/kaikki.tsv)

use ablaut::ara::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 5] = [
    "perfect",
    "imperfect",
    "subjunctive",
    "jussive",
    "imperative",
];

fn category(features: &str) -> &'static str {
    let has = |t: &str| features.split(';').any(|x| x == t);
    if has("IMP") {
        "imperative"
    } else if has("JUS") {
        "jussive"
    } else if has("SBJV") {
        "subjunctive"
    } else if has("PRF") {
        "perfect"
    } else {
        "imperfect"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    Some(vec![verb.form(features)?])
}

fn main() {
    run(Spec {
        lang: "ara",
        default_paths: ["data/ara/unimorph.tsv", "data/ara/kaikki.tsv"],
        adjudications: "docs/ara/adjudications.tsv",
        mismatches: "target/golden_ara_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_lemma(lemma).ok(),
        generate,
        category,
    });
}
