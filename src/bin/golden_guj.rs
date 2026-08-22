//! Gujarati golden-test harness: diff the engine against the gold data.
//!
//! Usage: cargo run --release --bin golden_guj [gold.tsv ...] [--check]
//!        (default: data/guj/unimorph.tsv data/guj/kaikki.tsv —
//!         see scripts/guj/fetch_unimorph.sh, scripts/guj/fetch_kaikki.sh)
//!
//! Gujarati is scored against UniMorph guj alone. The intended second
//! oracle, kaikki.org, is the *same* English-Wiktionary lineage as
//! UniMorph guj (both descend from the `gu-conj` template), so their
//! agreement is not an independent cross-check — see docs/guj/oracles.md.
//! kaikki is kept as an independent spot check, run separately.
//!
//! With one file that file is scored directly; two files would be
//! intersected into an agreement gold.

use ablaut::guj::{Aspect, AuxMood, Gender, Number, Person, Politeness, Verb};

const CATEGORIES: [&str; 7] = [
    "present",
    "future",
    "progressive",
    "past",
    "imperative",
    "nonfinite",
    "conditional",
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
        "SG" | "SG+PL" => Some(Number::Singular),
        "PL" => Some(Number::Plural),
        _ => None,
    }
}

fn one(s: String) -> Option<Vec<String>> {
    Some(vec![s])
}

fn generate(v: &Verb, features: &str) -> Option<Vec<String>> {
    use Gender::Neuter as N;
    use Number::Singular as SG;
    use Person::Third as P3;
    let t: Vec<&str> = features.split(';').collect();
    match t.as_slice() {
        ["V", "V.MSDR"] => one(v.verbal_noun()),
        ["V", "LGSPEC1"] => one(v.conjunctive()),
        ["V", "LGSPEC2"] => one(v.consecutive()),
        ["V", "LGSPEC3"] => one(v.conditional()),
        ["V", "LGSPEC4"] => {
            one(v.analytic(Aspect::Imperfective, AuxMood::Counterfactual, P3, SG, N))
        }
        ["V", "IND", "PST", "POS"] => one(v.perfective(N, SG)),
        ["V", "IND", "PST", "PROG", "POS"] => {
            one(v.analytic(Aspect::Imperfective, AuxMood::Past, P3, SG, N))
        }
        ["V", "IMP", "PRS", "POS", "2", "SG"] => one(v.imperative(Politeness::Intimate)),
        ["V", "IMP", "PRS", "POS", "2", "PL"] => one(v.imperative(Politeness::Familiar)),
        ["V", "IMP", "PRS", "POS", "1", "PL"] => one(v.present(Person::First, Number::Plural)),
        ["V", "IMP", "PRS", "POS", "POL", "2", "SG"] => {
            one(v.imperative(Politeness::PoliteSingular))
        }
        ["V", "IMP", "PRS", "POS", "POL", "2", "PL"] => one(v.imperative(Politeness::PolitePlural)),
        // The present indicative and the (homophonous) conditional.
        ["V", "IND", "PRS", "POS", p, n] | ["V", "COND", "POS", p, n] => {
            one(v.present(person(p)?, number(n)?))
        }
        ["V", "IND", "FUT", "POS", p, n] => one(v.future(person(p)?, number(n)?)),
        ["V", "IND", "PRS", "PROG", "POS", p, n] => {
            one(v.analytic(Aspect::Present, AuxMood::Present, person(p)?, number(n)?, N))
        }
        // Everything else — the negatives (particle નહીં/ન), passive,
        // potential, optative, present subjunctive and future
        // progressive analytics — is outside the bounded cell set.
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features == "V;V.MSDR" || features == "V;LGSPEC1" || features == "V;LGSPEC2" {
        "nonfinite"
    } else if features == "V;LGSPEC3" || features == "V;LGSPEC4" {
        "conditional"
    } else if features.contains("PST") {
        "past"
    } else if features.contains("IMP") {
        "imperative"
    } else if features.contains("PROG") {
        "progressive"
    } else if features.contains("FUT") {
        "future"
    } else {
        "present"
    }
}

fn main() {
    ablaut::harness::run(ablaut::harness::Spec {
        lang: "guj",
        default_paths: ["data/guj/unimorph.tsv", "data/guj/kaikki.tsv"],
        adjudications: "docs/guj/adjudications.tsv",
        mismatches: "target/golden_guj_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
