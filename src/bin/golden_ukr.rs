//! Ukrainian golden-test harness: diff the engine against the agreement
//! of the two Ukrainian oracles (kaikki.org and VESUM / brown-uk dict_uk).
//!
//! Usage: cargo run --release --bin golden_ukr [gold.tsv ...] [--check]
//!        (default: data/ukr/vesum.tsv data/ukr/kaikki.tsv —
//!         see scripts/ukr/fetch_vesum.sh, scripts/ukr/fetch_kaikki.sh)
//!
//! With two gold files only slots the oracles agree on are scored; slots
//! where they contradict each other are the adjudication corpus. With one
//! file that file is scored directly.

use ablaut::harness::{run, Spec};
use ablaut::ukr::{Gender, Number, Person, Verb};

const CATEGORIES: [&str; 4] = ["infinitive", "present", "past", "imperative"];

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
    match f.as_slice() {
        ["V", "NFIN"] => Some(vec![verb.infinitive().to_string()]),
        ["V", "IND", "PRS", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => Some(vec![verb.present(p, n)]),
            _ => None,
        },
        ["V", "IMP", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => verb.imperative(p, n).map(|f| vec![f]),
            _ => None,
        },
        ["V", "IND", "PST", "PL"] => Some(vec![verb.past(Gender::Masculine, Number::Plural)]),
        ["V", "IND", "PST", g, "SG"] => {
            let g = match *g {
                "MASC" => Gender::Masculine,
                "FEM" => Gender::Feminine,
                "NEUT" => Gender::Neuter,
                _ => return None,
            };
            Some(vec![verb.past(g, Number::Singular)])
        }
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IND;PST") {
        "past"
    } else {
        "imperative"
    }
}

fn main() {
    run(Spec {
        lang: "ukr",
        default_paths: ["data/ukr/vesum.tsv", "data/ukr/kaikki.tsv"],
        adjudications: "docs/ukr/adjudications.tsv",
        mismatches: "target/golden_ukr_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
