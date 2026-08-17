//! Swedish golden-test harness: diff the engine against the agreement
//! of the two Swedish oracles (SALDO and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_swe [gold.tsv ...] [--check]
//!        (default: data/swe/saldo.tsv data/swe/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::swe::{Slot, Verb};

const CATEGORIES: [&str; 7] = [
    "infinitive",
    "present",
    "past",
    "supine",
    "imperative",
    "subj-present",
    "subj-past",
];

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let (slot, passive) = match features {
        "V;NFIN;ACT" => (Slot::Infinitive, false),
        "V;NFIN;PASS" => (Slot::Infinitive, true),
        "V;IND;PRS;ACT" => (Slot::Present, false),
        "V;IND;PRS;PASS" => (Slot::Present, true),
        "V;IND;PST;ACT" => (Slot::Past, false),
        "V;IND;PST;PASS" => (Slot::Past, true),
        "V;SUP;ACT" => (Slot::Supine, false),
        "V;SUP;PASS" => (Slot::Supine, true),
        "V;IMP" => (Slot::Imperative, false),
        "V;SBJV;PRS;ACT" => (Slot::SubjunctivePresent, false),
        "V;SBJV;PST;ACT" => (Slot::SubjunctivePast, false),
        _ => return None,
    };
    if passive {
        let v = verb.passive_variants(slot);
        return if v.is_empty() { None } else { Some(v) };
    }
    verb.active_voice(slot).map(|f| vec![f])
}

fn category(features: &str) -> &'static str {
    if features.contains("NFIN") {
        "infinitive"
    } else if features.contains("PRS") && features.contains("SBJV") {
        "subj-present"
    } else if features.contains("SBJV") {
        "subj-past"
    } else if features.contains("PRS") {
        "present"
    } else if features.contains("SUP") {
        "supine"
    } else if features.contains("IMP") {
        "imperative"
    } else {
        "past"
    }
}

fn main() {
    run(Spec {
        lang: "swe",
        default_paths: ["data/swe/saldo.tsv", "data/swe/kaikki.tsv"],
        adjudications: "docs/swe/adjudications.tsv",
        mismatches: "target/golden_swe_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
