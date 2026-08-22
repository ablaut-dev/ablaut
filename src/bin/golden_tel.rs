//! Telugu golden-test harness: diff the engine against the gold data.
//!
//! Usage: cargo run --release --bin golden_tel [gold.tsv ...] [--check]
//!        (default: data/tel/unimorph.tsv data/tel/kaikki.tsv —
//!         see scripts/tel/fetch_unimorph.sh, scripts/tel/fetch_kaikki.sh)
//!
//! With one file that file is scored directly. Two files would be
//! intersected into an agreement gold, but the two Telugu oracles
//! barely overlap (see docs/tel/oracles.md): kaikki has full
//! inflection tables for only ~11 verbs, one of which UniMorph also
//! lists, so there is no agreement surface to speak of. The score is
//! therefore taken against UniMorph alone, with kaikki run separately
//! as an independent spot check on the handful of verbs it tabulates.

use ablaut::harness::{run, Spec};
use ablaut::tel::{Gender, Number, Person, Tense, Verb};

const CATEGORIES: [&str; 3] = ["past", "present", "future"];

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let tokens: Vec<&str> = features.split(';').collect();
    if tokens.first() != Some(&"V") {
        return None;
    }
    let mut person = None;
    let mut number = None;
    let mut gender = Gender::Masculine; // default; overridden for 3rd
    let mut tense = None;
    let mut dur = false;
    for t in &tokens[1..] {
        match *t {
            "1" => person = Some(Person::First),
            "2" => person = Some(Person::Second),
            "3" => person = Some(Person::Third),
            "SG" => number = Some(Number::Singular),
            "PL" => number = Some(Number::Plural),
            "MASC" => gender = Gender::Masculine,
            "FEM" => gender = Gender::NonMasculine,
            "PST" => tense = Some(Tense::Past),
            "FUT" => tense = Some(Tense::Future),
            "PRS" => tense = Some(Tense::PresentDurative),
            "DUR" => dur = true,
            _ => return None,
        }
    }
    // The only present tense modelled is the durative (PRS;DUR).
    if tense == Some(Tense::PresentDurative) && !dur {
        return None;
    }
    let (p, n, t) = (person?, number?, tense?);
    Some(vec![verb.form(t, p, n, gender)])
}

fn category(features: &str) -> &'static str {
    if features.ends_with("PST") {
        "past"
    } else if features.ends_with("FUT") {
        "future"
    } else {
        "present"
    }
}

fn main() {
    run(Spec {
        lang: "tel",
        default_paths: ["data/tel/unimorph.tsv", "data/tel/kaikki.tsv"],
        adjudications: "docs/tel/adjudications.tsv",
        mismatches: "target/golden_tel_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
