//! Russian golden-test harness: diff the engine against the agreement
//! of the two Russian oracles (OpenCorpora and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_rus [gold.tsv ...] [--check]
//!        (default: data/rus/opencorpora.tsv data/rus/kaikki.tsv —
//!         see scripts/rus/fetch_opencorpora.sh, scripts/rus/fetch_kaikki.sh)
//!
//! With two gold files only slots the oracles agree on are scored; slots
//! where they contradict are the adjudication corpus. With one file that
//! file is scored directly (the CI path gates on OpenCorpora alone).

use ablaut::harness::{run, Spec};
use ablaut::rus::{Gender, Number, Person, Verb};

const CATEGORIES: [&str; 5] = ["infinitive", "non-past", "past", "imperative", "other"];

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

fn gender(tag: &str) -> Option<Gender> {
    match tag {
        "MASC" => Some(Gender::Masculine),
        "FEM" => Some(Gender::Feminine),
        "NEUT" => Some(Gender::Neuter),
        _ => None,
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    match f.as_slice() {
        ["V", "NFIN"] => Some(vec![verb.infinitive()]),
        // Present (imperfective) and future (perfective) share morphology.
        ["V", "IND", "PRS" | "FUT", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => Some(verb.non_past_variants(p, n)),
            _ => None,
        },
        ["V", "IND", "PST", "PL"] => Some(vec![verb.past(Gender::Masculine, Number::Plural)]),
        ["V", "IND", "PST", g, "SG"] => gender(g).map(|g| vec![verb.past(g, Number::Singular)]),
        ["V", "IMP", n, "2"] => number(n).map(|n| verb.imperative_variants(n)),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V;IND;PRS") || features.starts_with("V;IND;FUT") {
        "non-past"
    } else if features.starts_with("V;IND;PST") {
        "past"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else {
        "other"
    }
}

fn main() {
    run(Spec {
        lang: "rus",
        default_paths: ["data/rus/opencorpora.tsv", "data/rus/kaikki.tsv"],
        adjudications: "docs/rus/adjudications.tsv",
        mismatches: "target/golden_rus_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
