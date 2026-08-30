//! Guaraní golden-test harness: diff the engine against the kaikki.org
//! Paraguayan Guaraní conjugation tables.
//!
//! Usage: cargo run --release --bin golden_grn [gold.tsv] [--check]
//!        (default: data/grn/kaikki.tsv — see scripts/grn/fetch_kaikki.sh
//!         and scripts/grn/build.py)
//!
//! Single-oracle Beta gate. The gold is every cleanly-tagged cell of the
//! 68 verbs carrying a `gug-conj-*` template — active plus the passive,
//! reciprocal, coactive and objective voices, across the indicative,
//! hortative and imperative, stripped of the optional subject pronoun.
//! Excluded and documented: the `error-unrecognized-form` cells kaikki
//! itself flags (poro-/mba'e- incorporations) and the single stative
//! (chendal) verb, whose possessive-marked paradigm is unverifiable from
//! one attestation.

use ablaut::grn::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 6] = [
    "active",
    "passive",
    "reciprocal",
    "coactive",
    "objective",
    "other",
];

fn category(features: &str) -> &'static str {
    if features.starts_with("V;ACT;") {
        "active"
    } else if features.starts_with("V;PASSIVE;") {
        "passive"
    } else if features.starts_with("V;RECIPROCAL;") {
        "reciprocal"
    } else if features.starts_with("V;COACTIVE;") {
        "coactive"
    } else if features.starts_with("V;OBJECTIVE;") {
        "objective"
    } else {
        "other"
    }
}

fn main() {
    run(Spec {
        lang: "grn",
        default_paths: ["data/grn/kaikki.tsv", ""],
        adjudications: "docs/grn/adjudications.tsv",
        mismatches: "target/golden_grn_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_lemma(lemma).ok(),
        generate: |verb, features| verb.generate(features),
        category,
    });
}
