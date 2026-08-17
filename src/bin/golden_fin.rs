//! Finnish golden-tfin harness: diff the engine against the
//! agreement of the two Finnish oracles (Omorfi and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_fin [gold.tsv ...] [--check]
//!        (default: data/fin/omorfi.tsv data/fin/kaikki.tsv —
//!         see `scripts/fin/fetch_omorfi.sh`, `scripts/fin/fetch_kaikki.sh`)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); disagreements are the adjudication corpus.
//! With one file, that file is scored directly (the CI path).

use ablaut::fin::{Number, Person, Tense, Verb};
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 7] = [
    "infinitive",
    "present",
    "past",
    "conditional",
    "potential",
    "imperative",
    "passive-participle",
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
    let pn = |n: &str, p: &str| match (person(p), number(n)) {
        (Some(p), Some(n)) => Some((p, n)),
        _ => None,
    };
    let one = |s: String| Some(vec![s]);
    let tense = |t: &str| match t {
        "PRS" => Some(Tense::Present),
        "PST" => Some(Tense::Past),
        _ => None,
    };
    match f.as_slice() {
        ["V", "NFIN"] => one(verb.infinitive().to_string()),
        ["V", "IND", t, "PASS"] => tense(t).map(|t| vec![verb.passive(t)]),
        ["V", "COND", "PASS"] => one(verb.passive(Tense::Conditional)),
        ["V", "POT", "PASS"] => one(verb.passive(Tense::Potential)),
        ["V", "IMP", "PASS"] => one(verb.imperative_passive()),
        ["V", "IND", t, n, p] => {
            let t = tense(t)?;
            pn(n, p).map(|(p, n)| vec![verb.active(t, p, n)])
        }
        ["V", "COND", n, p] => pn(n, p).map(|(p, n)| vec![verb.active(Tense::Conditional, p, n)]),
        ["V", "POT", n, p] => pn(n, p).map(|(p, n)| vec![verb.active(Tense::Potential, p, n)]),
        ["V", "IMP", n, p] => {
            let (p, n) = pn(n, p)?;
            verb.imperative(p, n).map(|f| vec![f])
        }
        ["V.PTCP", t, v] => {
            let past = *t == "PST";
            let pass = *v == "PASS";
            one(verb.participle(past, pass))
        }
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V.PTCP") {
        "passive-participle"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IND;PST") {
        "past"
    } else if features.starts_with("V;COND") {
        "conditional"
    } else if features.starts_with("V;POT") {
        "potential"
    } else {
        "imperative"
    }
}

fn main() {
    run(Spec {
        lang: "fin",
        default_paths: ["data/fin/omorfi.tsv", "data/fin/kaikki.tsv"],
        adjudications: "docs/fin/adjudications.tsv",
        mismatches: "target/golden_fin_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 98.5,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
