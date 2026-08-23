//! Norwegian Bokmål golden-test harness: diff the engine against the
//! agreement of the two Bokmål oracles (apertium-nob and UniMorph nob).
//!
//! Usage: cargo run --release --bin golden_nob [gold.tsv ...] [--check]
//!        (default: data/nob/apertium.tsv data/nob/unimorph.tsv)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); with one file, that file is scored directly
//! (the local smoke-test path: UniMorph alone).

use ablaut::harness::{run, Spec};
use ablaut::nob::{Slot, Verb};

const CATEGORIES: [&str; 5] = ["present", "past", "imperative", "participle", "passive"];

/// (slot, passive?) for a UniMorph feature bundle.
fn slot(features: &str) -> Option<(Slot, bool)> {
    match features {
        "V;IND;PRS" => Some((Slot::Present, false)),
        "V;IND;PST" => Some((Slot::Past, false)),
        "V;IND;PASS" => Some((Slot::Present, true)),
        "V;IMP" => Some((Slot::Imperative, false)),
        "V.PTCP;PRS" => Some((Slot::PresentParticiple, false)),
        "V.PTCP;PST" => Some((Slot::PastParticiple, false)),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.ends_with("PASS") {
        "passive"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IND;PST") {
        "past"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else {
        "participle"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let (sl, passive) = slot(features)?;
    let forms = if passive {
        verb.passive_forms(sl)
    } else {
        verb.active_forms(sl)
    };
    if forms.is_empty() {
        None
    } else {
        Some(forms)
    }
}

fn main() {
    run(Spec {
        lang: "nob",
        default_paths: ["data/nob/apertium.tsv", "data/nob/unimorph.tsv"],
        adjudications: "docs/nob/adjudications.tsv",
        mismatches: "target/golden_nob_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
