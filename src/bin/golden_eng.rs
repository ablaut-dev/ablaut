//! English golden-test harness: diff the engine against the agreement
//! of the two English oracles (AGID and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_eng [gold.tsv ...] [--check]
//!        (default: data/eng/agid.tsv data/eng/kaikki.tsv)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); with one file, that file is scored directly
//! (the CI path: AGID alone).

use ablaut::eng::{Slot, Verb};
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 5] = [
    "infinitive",
    "past",
    "past-participle",
    "present-participle",
    "third-singular",
];

fn slot(features: &str) -> Option<Slot> {
    match features {
        "V;NFIN" => Some(Slot::Infinitive),
        "V;PST" => Some(Slot::Past),
        "V.PTCP;PST" => Some(Slot::PastParticiple),
        "V.PTCP;PRS" => Some(Slot::PresentParticiple),
        "V;PRS;3;SG" => Some(Slot::ThirdSingular),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    match features {
        "V;NFIN" => "infinitive",
        "V;PST" => "past",
        "V.PTCP;PST" => "past-participle",
        "V.PTCP;PRS" => "present-participle",
        _ => "third-singular",
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    slot(features).map(|sl| vec![verb.form(sl)])
}

fn main() {
    run(Spec {
        lang: "eng",
        default_paths: ["data/eng/agid.tsv", "data/eng/kaikki.tsv"],
        adjudications: "docs/eng/adjudications.tsv",
        mismatches: "target/golden_eng_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
