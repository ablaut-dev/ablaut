//! Kannada golden-test harness: diff the engine against the gold data.
//!
//! Usage: cargo run --release --bin golden_kan [gold.tsv ...] [--check]
//!        (default: data/kan/unimorph.tsv data/kan/kaikki.tsv —
//!         see scripts/kan/fetch_unimorph.sh, scripts/kan/fetch_kaikki.sh)
//!
//! With two files they are intersected into a two-oracle agreement gold;
//! with one file that file is scored directly. Unlike Telugu, the two
//! Kannada oracles overlap fully at the lemma level (all 41 UniMorph
//! verbs are tabulated by kaikki too) and agree on 411 person/tense
//! slots, so this is a genuine two-oracle gate — see docs/kan/oracles.md.

use ablaut::harness::{run, Spec};
use ablaut::kan::{Gender, Number, Person, Tense, Verb};

const CATEGORIES: [&str; 4] = ["past", "present", "future", "imperative"];

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let tokens: Vec<&str> = features.split(';').collect();
    if tokens.first() != Some(&"V") {
        return None;
    }
    let mut person = None;
    let mut number = None;
    let mut gender = Gender::Neuter; // default; overridden for 3rd person
    let mut tense = None;
    let mut imperative = false;
    for t in &tokens[1..] {
        match *t {
            "1" => person = Some(Person::First),
            "2" => person = Some(Person::Second),
            "3" => person = Some(Person::Third),
            "SG" => number = Some(Number::Singular),
            "PL" => number = Some(Number::Plural),
            "MASC" => gender = Gender::Masculine,
            "FEM" => gender = Gender::Feminine,
            "NEUT" => gender = Gender::Neuter,
            "PST" => tense = Some(Tense::Past),
            "PRS" => tense = Some(Tense::Present),
            "FUT" => tense = Some(Tense::Future),
            "IMP" => imperative = true,
            _ => return None,
        }
    }
    let (p, n) = (person?, number?);
    if imperative {
        // The imperative is scored for the second person (2sg = root,
        // 2pl = the -ಇರಿ form); the 1sg/1pl cohortatives are outside
        // the agreement gold.
        return match p {
            Person::Second => Some(vec![verb.imperative(n)]),
            _ => None,
        };
    }
    Some(vec![verb.form(tense?, p, n, gender)])
}

fn category(features: &str) -> &'static str {
    if features.ends_with("PST") {
        "past"
    } else if features.ends_with("PRS") {
        "present"
    } else if features.ends_with("FUT") {
        "future"
    } else {
        "imperative"
    }
}

fn main() {
    run(Spec {
        lang: "kan",
        default_paths: ["data/kan/unimorph.tsv", "data/kan/kaikki.tsv"],
        adjudications: "docs/kan/adjudications.tsv",
        mismatches: "target/golden_kan_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
