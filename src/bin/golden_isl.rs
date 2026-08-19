//! Icelandic golden-test harness: diff the engine against the agreement
//! of the two Icelandic oracles (BÍN and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_isl [gold.tsv ...] [--check]
//!        (default: data/isl/bin.tsv data/isl/kaikki.tsv —
//!         see scripts/isl/fetch_bin.sh, scripts/isl/fetch_kaikki.sh)
//!
//! With two gold files only slots the oracles agree on are scored. In CI
//! only BÍN is fetched, so the harness is run with that single file,
//! which is scored directly (BÍN is the authoritative national lexicon;
//! kaikki establishes its independent-provenance agreement offline).

use ablaut::harness::{run, Spec};
use ablaut::isl::{Number, Person, SimpleTense, Verb};

const CATEGORIES: [&str; 7] = [
    "infinitive",
    "present",
    "past",
    "subj-present",
    "subj-past",
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
        "PL" => Some(Number::Plural),
        _ => None,
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    let tense = |t: SimpleTense, n: &str, p: &str| match (person(p), number(n)) {
        (Some(p), Some(n)) => Some(verb.variants(t, p, n)),
        _ => None,
    };
    match f.as_slice() {
        ["V", "NFIN"] => Some(vec![verb.infinitive().to_string()]),
        ["V.PTCP", "PST"] => Some(vec![verb.supine()]),
        ["V", "V.PTCP", "PRS"] => Some(vec![verb.present_participle()]),
        ["V", "IMP", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => verb.imperative(p, n).map(|f| vec![f]),
            _ => None,
        },
        ["V", "IND", "PRS", n, p] => tense(SimpleTense::Present, n, p),
        ["V", "IND", "PST", n, p] => tense(SimpleTense::Past, n, p),
        ["V", "SBJV", "PRS", n, p] => tense(SimpleTense::SubjunctivePresent, n, p),
        ["V", "SBJV", "PST", n, p] => tense(SimpleTense::SubjunctivePast, n, p),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.starts_with("V.PTCP") || features.starts_with("V;V.PTCP") {
        "participle"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IND;PST") {
        "past"
    } else if features.starts_with("V;SBJV;PRS") {
        "subj-present"
    } else {
        "subj-past"
    }
}

fn main() {
    run(Spec {
        lang: "isl",
        default_paths: ["data/isl/bin.tsv", "data/isl/kaikki.tsv"],
        adjudications: "docs/isl/adjudications.tsv",
        mismatches: "target/golden_isl_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
