//! Dutch golden-test harness: diff the engine against the agreement of
//! the two Dutch oracles (kaikki.org and Apertium-nld).
//!
//! Usage: cargo run --release --bin golden_nld [gold.tsv ...] [--check]
//!        (default: data/nld/apertium.tsv data/nld/kaikki.tsv —
//!         see scripts/nld/fetch_apertium.sh, scripts/nld/fetch_kaikki.sh)
//!
//! With two gold files only slots the oracles agree on are scored; slots
//! where they contradict each other are the adjudication corpus. With one
//! file that file is scored directly.

use ablaut::harness::{run, Spec};
use ablaut::nld::{Number, Person, Verb};

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "present",
    "past",
    "imperative",
    "participle",
    "present-participle",
];

fn person(tag: &str) -> Option<Person> {
    match tag {
        "1" => Some(Person::First),
        "2" => Some(Person::Second),
        "3" => Some(Person::Third),
        _ => None,
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    match f.as_slice() {
        ["V", "NFIN"] => Some(vec![verb.infinitive().to_string()]),
        ["V", "IMP"] => Some(vec![verb.imperative()]),
        ["V.PTCP", "PRS"] => Some(vec![verb.present_participle()]),
        ["V.PTCP", "PST"] => Some(vec![verb.past_participle()]),
        ["V", "IND", "PRS", "SG", p] => person(p).map(|p| vec![verb.present(p, Number::Singular)]),
        ["V", "IND", "PRS", "PL", p] => person(p).map(|p| vec![verb.present(p, Number::Plural)]),
        ["V", "IND", "PST", "SG"] => Some(vec![verb.past(Number::Singular)]),
        ["V", "IND", "PST", "PL"] => Some(vec![verb.past(Number::Plural)]),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features == "V.PTCP;PRS" {
        "present-participle"
    } else if features.starts_with("V.PTCP") {
        "participle"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else {
        "past"
    }
}

fn main() {
    run(Spec {
        lang: "nld",
        default_paths: ["data/nld/apertium.tsv", "data/nld/kaikki.tsv"],
        adjudications: "docs/nld/adjudications.tsv",
        mismatches: "target/golden_nld_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
