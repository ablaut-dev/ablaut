//! Esperanto golden-test harness: diff the engine against the kaikki.org
//! (Wiktextract) Esperanto verb extraction.
//!
//! Usage: cargo run --release --bin golden_epo [gold.tsv ...] [--check]
//!        (default: data/epo/kaikki.tsv — see scripts/epo/fetch_kaikki.sh)
//!
//! Esperanto is perfectly regular, so a single clean oracle (kaikki)
//! suffices (Beta tier). The engine and the oracle both build every cell
//! by pure suffixation, so the gate is a full 100.00%.

use ablaut::epo::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "present",
    "past",
    "future",
    "conditional",
    "participle",
];

fn category(features: &str) -> &'static str {
    if features.starts_with("V.PTCP") {
        "participle"
    } else if features == "V;NFIN" {
        "infinitive"
    } else if features == "V;PST" {
        "past"
    } else if features == "V;FUT" {
        "future"
    } else if features == "V;COND" {
        "conditional"
    } else {
        // V;PRS and V;VOL
        "present"
    }
}

fn main() {
    run(Spec {
        lang: "epo",
        // Single oracle (Beta): the second path is an empty placeholder,
        // so kaikki is scored directly.
        default_paths: ["data/epo/kaikki.tsv", "data/epo/_no_second_oracle.tsv"],
        adjudications: "docs/epo/adjudications.tsv",
        mismatches: "target/golden_epo_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate: |verb, features| Some(vec![verb.form(features)?]),
        category,
    });
}
