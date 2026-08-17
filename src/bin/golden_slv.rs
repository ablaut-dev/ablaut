//! Slovenian golden-test harness: diff the engine against the
//! agreement of the two Slovenian oracles (Sloleks and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_slv [gold.tsv ...] [--check]
//!        (default: data/slv/sloleks.tsv data/slv/kaikki.tsv —
//!         see `scripts/slv/fetch_sloleks.sh`, `scripts/slv/fetch_kaikki.sh`)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); disagreements are the adjudication corpus.
//! With one file, that file is scored directly (the CI path).

use ablaut::harness::{run, Spec};
use ablaut::slv::{Gender, Number, Person, Verb};

const CATEGORIES: [&str; 5] = [
    "infinitive",
    "supine",
    "present",
    "imperative",
    "participle",
];

fn person(tag: &str) -> Option<Person> {
    match tag {
        "1" => Some(Person::First),
        "2" => Some(Person::Second),
        "3" => Some(Person::Third),
        _ => None,
    }
}

fn number(tag: &str) -> Option<Number> {
    match tag {
        "SG" => Some(Number::Singular),
        "DU" => Some(Number::Dual),
        "PL" => Some(Number::Plural),
        _ => None,
    }
}

/// Map a feature bundle from the oracle TSVs to the engine's output
/// (None means unsupported bundle).
fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    let gender = |g: &str| match g {
        "M" => Some(Gender::Masculine),
        "F" => Some(Gender::Feminine),
        "N" => Some(Gender::Neuter),
        _ => None,
    };
    match f.as_slice() {
        ["V", "NFIN"] => Some(vec![verb.infinitive().to_string()]),
        ["V", "SUP"] => Some(vec![verb.supine()]),
        ["V", "IND", "PRS", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => Some(vec![verb.present(p, n)]),
            _ => None,
        },
        ["V", "IMP", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => verb.imperative(p, n).map(|f| vec![f]),
            _ => None,
        },
        ["V.PTCP", "PST", g, n] => match (gender(g), number(n)) {
            (Some(g), Some(n)) => Some(vec![verb.participle(g, n)]),
            _ => None,
        },
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features == "V;SUP" {
        "supine"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else {
        "participle"
    }
}

fn main() {
    run(Spec {
        lang: "slv",
        default_paths: ["data/slv/sloleks.tsv", "data/slv/kaikki.tsv"],
        adjudications: "docs/slv/adjudications.tsv",
        mismatches: "target/golden_slv_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.9,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
