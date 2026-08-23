//! Welsh golden-test harness: diff the engine against the agreement of the
//! two Welsh oracles (UniMorph and kaikki.org) — the literary single-word
//! paradigm. Two-oracle: Verified.
//!
//! Usage: cargo run --release --bin golden_cym [gold.tsv ...] [--check]
//!        (default: data/cym/unimorph.tsv data/cym/kaikki.tsv)

use ablaut::cym::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 7] = [
    "present",
    "imperfect",
    "preterite",
    "pluperfect",
    "subjunctive",
    "imperative",
    "nonfinite",
];

fn category(features: &str) -> &'static str {
    if features == "V;V.MSDR" || features == "V;V.PTCP" {
        "nonfinite"
    } else if features.ends_with("IMP") {
        "imperative"
    } else if features.ends_with("SBJV") {
        "subjunctive"
    } else if features.contains("IND;PST;PFV") {
        "pluperfect"
    } else if features.ends_with("IND;PST") {
        "preterite"
    } else if features.ends_with("IND;IPFV") {
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
        lang: "cym",
        default_paths: ["data/cym/unimorph.tsv", "data/cym/kaikki.tsv"],
        adjudications: "docs/cym/adjudications.tsv",
        mismatches: "target/golden_cym_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
