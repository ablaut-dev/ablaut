//! French golden-test harness: diff the first-group engine against the
//! agreement of the two French oracles (Lefff 3.4 and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_fra [gold.tsv ...] [--check]
//!        (default: data/fra/lefff.tsv data/fra/kaikki.tsv —
//!         see `scripts/fra/fetch_lefff.sh`, `scripts/fra/fetch_kaikki.sh`)
//!
//! With two gold files, only slots the oracles agree on are scored (the
//! variant sets are unioned, so either standard spelling counts); slots
//! where they contradict each other are the adjudication corpus, not gold.
//! With one file, that file is scored directly (the CI path: Lefff alone,
//! kaikki being a 300 MB download).
//!
//! Lemmas the engine does not support yet (non-first-group verbs, *aller*,
//! the *envoyer* family) are skipped and reported as lemma coverage.
//! Mismatches go to `target/golden_fra_mismatches.tsv`.

use ablaut::fra::{Auxiliary, Number, Person, SimpleTense, Verb};
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 11] = [
    "infinitive",
    "auxiliary",
    "present",
    "imperfect",
    "past-historic",
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
        ["V", "AUX"] => Some(vec![match verb.auxiliary() {
            Auxiliary::Avoir => "avoir".to_string(),
            Auxiliary::Etre => "être".to_string(),
        }]),
        ["V.PTCP", "PRS"] => Some(verb.present_participle_variants()),
        ["V.PTCP", "PST", "MASC", "SG"] => Some(vec![verb.past_participle()]),
        ["V", "IMP", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => Some(verb.imperative_variants(p, n)).filter(|v| !v.is_empty()),
            _ => None,
        },
        ["V", "IND", "PRS", n, p] => tense(SimpleTense::Present, n, p),
        ["V", "IND", "PST", "IPFV", n, p] => tense(SimpleTense::Imperfect, n, p),
        ["V", "IND", "PST", "PFV", n, p] => tense(SimpleTense::PastHistoric, n, p),
        ["V", "IND", "FUT", n, p] => tense(SimpleTense::Future, n, p),
        ["V", "COND", n, p] => tense(SimpleTense::Conditional, n, p),
        ["V", "SBJV", "PRS", n, p] => tense(SimpleTense::SubjunctivePresent, n, p),
        ["V", "SBJV", "PST", n, p] => tense(SimpleTense::SubjunctiveImperfect, n, p),
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
fn category(features: &str) -> &'static str {
    if features == "V;AUX" {
        "auxiliary"
    } else if features.starts_with("V.PTCP") {
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
        "past-historic"
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
        lang: "fra",
        default_paths: ["data/fra/lefff.tsv", "data/fra/kaikki.tsv"],
        adjudications: "docs/fra/adjudications.tsv",
        mismatches: "target/golden_fra_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.95,
        min_lemma_coverage_pct: 99.5,
        carry_features: &["V;AUX"],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
