//! Macedonian golden-test harness: diff the engine against the agreement
//! of the two Macedonian oracles (apertium-mkd and UniMorph mkd).
//!
//! Usage: cargo run --release --bin golden_mkd [gold.tsv ...] [--check]
//!        (default: data/mkd/apertium.tsv data/mkd/unimorph.tsv)
//!
//! The engine covers the imperfective synthetic system (present, imperfect,
//! its l-participle, imperative), the passive participle and the non-finite
//! converb/verbal noun. The aorist and aorist participle the oracles also
//! carry are out of scope and show up as uncovered slots — see
//! docs/mkd/oracles.md.

use ablaut::harness::{run, Spec};
use ablaut::mkd::Verb;

const CATEGORIES: [&str; 6] = [
    "present",
    "imperfect",
    "participle",
    "imperative",
    "passive",
    "nonfinite",
];

fn category(features: &str) -> &'static str {
    if features == "V.PTCP;PST;PASS" {
        "passive"
    } else if features == "V.CVB" || features == "V.MSDR" {
        "nonfinite"
    } else if features.starts_with("V;PRS") {
        "present"
    } else if features.starts_with("V;PROG;PST") {
        "imperfect"
    } else if features.starts_with("V.PTCP") {
        "participle"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else {
        // aorist (V;PST…) and anything else the engine does not cover
        "other"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let forms = verb.forms(features);
    if forms.is_empty() {
        None
    } else {
        Some(forms)
    }
}

fn main() {
    run(Spec {
        lang: "mkd",
        default_paths: ["data/mkd/apertium.tsv", "data/mkd/unimorph.tsv"],
        adjudications: "docs/mkd/adjudications.tsv",
        mismatches: "target/golden_mkd_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
