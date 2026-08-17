//! Danish golden-test harness: diff the danine against the agreement
//! of the two Danish oracles (COR and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_dan [gold.tsv ...] [--check]
//!        (default: data/dan/cor-verbs.tsv data/dan/kaikki.tsv)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); with one file, that file is scored directly
//! (the CI path: AGID alone).

use ablaut::dan::{Slot, Verb};
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "present",
    "past",
    "imperative",
    "participle",
    "passive",
];

/// (slot, passive?) for a feature bundle.
fn slot(features: &str) -> Option<(Slot, bool)> {
    match features {
        "V;NFIN;ACT" => Some((Slot::Infinitive, false)),
        "V;NFIN;PASS" => Some((Slot::Infinitive, true)),
        "V;PRS;ACT" => Some((Slot::Present, false)),
        "V;PRS;PASS" => Some((Slot::Present, true)),
        "V;PST;ACT" => Some((Slot::Past, false)),
        "V;PST;PASS" => Some((Slot::Past, true)),
        "V;IMP" => Some((Slot::Imperative, false)),
        "V.PTCP;PRS" => Some((Slot::PresentParticiple, false)),
        "V.PTCP;PST" => Some((Slot::PastParticiple, false)),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.ends_with("PASS") {
        "passive"
    } else if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.starts_with("V;PRS") {
        "present"
    } else if features.starts_with("V;PST") {
        "past"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else {
        "participle"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let (sl, passive) = slot(features)?;
    let form = if passive {
        verb.passive(sl)
    } else {
        verb.active(sl)
    }?;
    Some(vec![form])
}

fn main() {
    run(Spec {
        lang: "dan",
        default_paths: ["data/dan/cor-verbs.tsv", "data/dan/kaikki.tsv"],
        adjudications: "docs/dan/adjudications.tsv",
        mismatches: "target/golden_dan_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
