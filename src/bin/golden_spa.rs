//! Spanish golden-test harness: diff the engine against the agreement
//! of the two Spanish oracles (FreeLing and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_spa [gold.tsv ...] [--check]
//!        (default: data/spa/freeling.tsv data/spa/kaikki.tsv —
//!         see `scripts/spa/fetch_freeling.sh`, `scripts/spa/fetch_kaikki.sh`)
//!
//! With two gold files, only slots the oracles agree on are scored (the
//! variant sets are unioned, so either standard spelling counts); slots
//! where they contradict each other are the adjudication corpus, not gold.
//! With one file, that file is scored directly (the CI path: FreeLing
//! alone, kaikki being a 700 MB download).
//!
//! Lemmas the engine does not support yet are skipped and reported as
//! lemma coverage. Mismatches go to `target/golden_spa_mismatches.tsv`.

use ablaut::harness::{run, Spec};
use ablaut::spa::{Number, Person, SimpleTense, Verb};

const CATEGORIES: [&str; 11] = [
    "infinitive",
    "present",
    "imperfect",
    "preterite",
    "future",
    "conditional",
    "subj-present",
    "subj-imperfect",
    "subj-future",
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

/// Map a feature bundle from the oracle TSVs to the engine's variant set
/// (canonical form first; empty means unsupported bundle).
fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    let tense = |t: SimpleTense, n: &str, p: &str| match (person(p), number(n)) {
        (Some(p), Some(n)) => Some(verb.variants(t, p, n)),
        _ => None,
    };
    match f.as_slice() {
        ["V", "NFIN"] => Some(vec![verb.infinitive().to_string()]),
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
        ["V", "SBJV", "FUT", n, p] => tense(SimpleTense::SubjunctiveFuture, n, p),
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
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
    } else if features.starts_with("V;SBJV;FUT") {
        "subj-future"
    } else {
        "subj-imperfect"
    }
}

fn main() {
    run(Spec {
        lang: "spa",
        default_paths: ["data/spa/freeling.tsv", "data/spa/kaikki.tsv"],
        adjudications: "docs/spa/adjudications.tsv",
        mismatches: "target/golden_spa_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.9,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
