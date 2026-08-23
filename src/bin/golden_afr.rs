//! Afrikaans golden-test harness: diff the engine against the agreement
//! of the two Afrikaans oracles (UniMorph and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_afr [gold.tsv ...] [--check]
//!        (default: data/afr/unimorph.tsv data/afr/kaikki.tsv)

use ablaut::afr::{Slot, Verb};
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 5] = ["infinitive", "present", "past", "participle", "imperative"];

fn slot(features: &str) -> Option<Slot> {
    match features {
        "V;INF" | "V;NFIN" | "V;NFIN;ACT" => Some(Slot::Infinitive),
        "V;PRS" | "V;PRS;ACT" => Some(Slot::Present),
        "V;PST" | "V;PST;ACT" => Some(Slot::Past),
        "V;IMP" => Some(Slot::Imperative),
        "V.PTCP;PRS" => Some(Slot::PresentParticiple),
        "V.PTCP;PST" => Some(Slot::PastParticiple),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.starts_with("V.PTCP") {
        "participle"
    } else if features.starts_with("V;INF") || features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.starts_with("V;PRS") {
        "present"
    } else if features.starts_with("V;PST") {
        "past"
    } else {
        "imperative"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let sl = slot(features)?;
    Some(vec![verb.form(sl)?])
}

fn main() {
    run(Spec {
        lang: "afr",
        default_paths: ["data/afr/unimorph.tsv", "data/afr/kaikki.tsv"],
        adjudications: "docs/afr/adjudications.tsv",
        mismatches: "target/golden_afr_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
