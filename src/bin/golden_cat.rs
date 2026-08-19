//! Catalan golden-test harness: diff the engine against the agreement
//! of the two Catalan oracles (FreeLing ca and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_cat [gold.tsv ...] [--check]
//!        (default: data/cat/freeling.tsv data/cat/kaikki.tsv —
//!         see scripts/cat/fetch_freeling.sh, scripts/cat/fetch_kaikki.sh)
//!
//! With two gold files only slots the oracles agree on are scored; slots
//! where they contradict each other are the adjudication corpus. With one
//! file that file is scored directly.

use ablaut::cat::{Number, Person, SimpleTense, Verb};
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 10] = [
    "infinitive",
    "present",
    "imperfect",
    "preterite",
    "future",
    "conditional",
    "subj-present",
    "subj-imperfect",
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
        ["V", "NFIN"] => Some(vec![verb.infinitive()]),
        ["V", "GER"] => Some(vec![verb.gerund()]),
        ["V.PTCP", "PST", g @ ("MASC" | "FEM"), n @ ("SG" | "PL")] => {
            Some(vec![verb.past_participle_inflected(*g == "FEM", *n == "PL")])
        }
        ["V", "IMP", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => verb.imperative(p, n).map(|f| vec![f]),
            _ => None,
        },
        ["V", "IND", "PRS", n, p] => tense(SimpleTense::Present, n, p),
        ["V", "IND", "PST", "IPFV", n, p] => tense(SimpleTense::Imperfect, n, p),
        ["V", "IND", "PST", "PFV", n, p] => tense(SimpleTense::Preterite, n, p),
        ["V", "IND", "FUT", n, p] => tense(SimpleTense::Future, n, p),
        ["V", "COND", n, p] => tense(SimpleTense::Conditional, n, p),
        ["V", "SBJV", "PRS", n, p] => tense(SimpleTense::SubjunctivePresent, n, p),
        ["V", "SBJV", "PST", n, p] => tense(SimpleTense::SubjunctiveImperfect, n, p),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.starts_with("V.PTCP") || features == "V;GER" {
        "participle"
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
    } else if features.starts_with("V;IND;FUT") {
        "future"
    } else if features.starts_with("V;COND") {
        "conditional"
    } else if features.starts_with("V;SBJV;PRS") {
        "subj-present"
    } else {
        "subj-imperfect"
    }
}

fn main() {
    run(Spec {
        lang: "cat",
        default_paths: ["data/cat/freeling.tsv", "data/cat/kaikki.tsv"],
        adjudications: "docs/cat/adjudications.tsv",
        mismatches: "target/golden_cat_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
