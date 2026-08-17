//! Romanian golden-test harness: diff the engine against the
//! agreement of the two Romanian oracles (kaikki.org and dexonline).
//!
//! Usage: cargo run --release --bin golden_ron [gold.tsv ...] [--check]
//!        (default: data/ron/dexonline.tsv data/ron/kaikki.tsv —
//!         see `scripts/ron/fetch_dexonline.sh`, `scripts/ron/fetch_kaikki.sh`)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); disagreements are the adjudication corpus.
//! With one file, that file is scored directly (the CI path).

use ablaut::harness::{run, Spec};
use ablaut::ron::{Number, Person, Tense, Verb};

const CATEGORIES: [&str; 9] = [
    "infinitive",
    "present",
    "imperfect",
    "preterite",
    "pluperfect",
    "subj-present",
    "imperative",
    "participle",
    "gerund",
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
        "PL" => Some(Number::Plural),
        _ => None,
    }
}

/// Map a feature bundle from the oracle TSVs to the engine's output
/// (empty means unsupported bundle).
fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    let tense = |t: Tense, n: &str, p: &str| match (person(p), number(n)) {
        (Some(p), Some(n)) => Some(vec![verb.indicative(t, p, n)]),
        _ => None,
    };
    match f.as_slice() {
        ["V", "NFIN"] => Some(vec![verb.infinitive().to_string()]),
        ["V", "GER"] => Some(vec![verb.gerund()]),
        ["V.PTCP", "PST", "MASC", "SG"] => Some(vec![verb.participle()]),
        ["V", "IMP", n, "2"] => number(n).map(|n| vec![verb.imperative(n)]),
        ["V", "IND", "PRS", n, p] => tense(Tense::Present, n, p),
        ["V", "IND", "PST", "IPFV", n, p] => tense(Tense::Imperfect, n, p),
        ["V", "IND", "PST", "PFV", n, p] => tense(Tense::SimplePerfect, n, p),
        ["V", "IND", "PST", "PQP", n, p] => tense(Tense::Pluperfect, n, p),
        ["V", "SBJV", "PRS", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => Some(vec![verb.subjunctive(p, n)]),
            _ => None,
        },
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
fn category(features: &str) -> &'static str {
    if features.starts_with("V.PTCP") {
        "participle"
    } else if features == "V;GER" {
        "gerund"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IND;PST;IPFV") {
        "imperfect"
    } else if features.starts_with("V;IND;PST;PFV") {
        "preterite"
    } else if features.starts_with("V;IND;PST;PQP") {
        "pluperfect"
    } else {
        "subj-present"
    }
}

fn main() {
    run(Spec {
        lang: "ron",
        default_paths: ["data/ron/dexonline.tsv", "data/ron/kaikki.tsv"],
        adjudications: "docs/ron/adjudications.tsv",
        mismatches: "target/golden_ron_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
