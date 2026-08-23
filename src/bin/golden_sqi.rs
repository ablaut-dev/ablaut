//! Albanian golden-test harness: diff the engine against the agreement of
//! the two Albanian oracles (UniMorph and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_sqi [gold.tsv ...] [--check]
//!        (default: data/sqi/unimorph.tsv data/sqi/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::sqi::Verb;

const CATEGORIES: [&str; 6] = [
    "present",
    "imperfect",
    "aorist",
    "admirative",
    "imperative",
    "participle",
];

fn category(features: &str) -> &'static str {
    if features == "V;V.PTCP" {
        "participle"
    } else if features.contains("ADM") {
        "admirative"
    } else if features.contains("IND;PRS") {
        "present"
    } else if features.contains("IND;IPFV") {
        "imperfect"
    } else if features.contains("IND;PST") {
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
        lang: "sqi",
        default_paths: ["data/sqi/unimorph.tsv", "data/sqi/kaikki.tsv"],
        adjudications: "docs/sqi/adjudications.tsv",
        mismatches: "target/golden_sqi_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
