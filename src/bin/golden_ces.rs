//! Czech golden-test harness: diff the engine against the
//! agreement of the two Czech oracles (MorfFlex CZ and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_ces [gold.tsv ...] [--check]
//!        (default: data/ces/morfflex.tsv data/ces/kaikki.tsv —
//!         see `scripts/ces/fetch_morfflex.sh`, `scripts/ces/fetch_kaikki.sh`)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); disagreements are the adjudication corpus.
//! With one file, that file is scored directly (the CI path).

use ablaut::ces::{Gender, Number, Person, TransgressiveSlot, Verb};
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "present",
    "imperative",
    "l-participle",
    "passive-participle",
    "transgressive",
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
/// (None means unsupported bundle).
fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    let gender = |g: &str| match g {
        "MA" => Some(Gender::MasculineAnimate),
        "MI" => Some(Gender::MasculineInanimate),
        "F" => Some(Gender::Feminine),
        "N" => Some(Gender::Neuter),
        _ => None,
    };
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
        ["V.PTCP", "PST", g, n] => match (gender(g), number(n)) {
            (Some(g), Some(n)) => Some(vec![verb.past_participle(g, n)]),
            _ => None,
        },
        ["V.PTCP", "PASS", g, n] => match (gender(g), number(n)) {
            (Some(g), Some(n)) => verb.passive_participle(g, n).map(|f| vec![f]),
            _ => None,
        },
        ["V", "CVB", "PRS", s] => {
            let slot = match *s {
                "M" => TransgressiveSlot::Masculine,
                "FN" => TransgressiveSlot::FeminineNeuter,
                "PL" => TransgressiveSlot::Plural,
                _ => return None,
            };
            verb.transgressive(slot).map(|f| vec![f])
        }
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V.PTCP;PST") {
        "l-participle"
    } else if features.starts_with("V.PTCP;PASS") {
        "passive-participle"
    } else {
        "transgressive"
    }
}

fn main() {
    run(Spec {
        lang: "ces",
        default_paths: ["data/ces/morfflex.tsv", "data/ces/kaikki.tsv"],
        adjudications: "docs/ces/adjudications.tsv",
        mismatches: "target/golden_ces_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.9,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
