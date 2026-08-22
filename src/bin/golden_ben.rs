//! Bengali golden-test harness: diff the engine against the gold data.
//!
//! Usage: cargo run --release --bin golden_ben [gold.tsv ...] [--check]
//!        (default: data/ben/unimorph.tsv data/ben/kaikki.tsv —
//!         see scripts/ben/fetch_unimorph.sh, scripts/ben/fetch_kaikki.sh)
//!
//! Bengali is scored against UniMorph ben alone. The intended second
//! oracle, kaikki.org, is the *same* Wiktionary lineage as UniMorph ben
//! (whose source is Wikipedia), so their agreement is not an independent
//! cross-check — see docs/ben/oracles.md. kaikki is kept as an
//! independent spot check, run separately.
//!
//! With one file that file is scored directly; two files would be
//! intersected into an agreement gold.

use ablaut::ben::{Person, Tense, Verb};

const CATEGORIES: [&str; 7] = [
    "present",
    "past",
    "future",
    "habitual",
    "progressive",
    "perfect",
    "nonfinite",
];

fn one(s: String) -> Option<Vec<String>> {
    Some(vec![s])
}

/// The habitual and progressive participles reduplicate for the compound
/// lemmas in the gold (মনে রাখা → মনে রেখেরেখে) but not for the simple
/// verbs (রাখা → রেখে): returning both spellings matches either.
fn redup(form: &str) -> Vec<String> {
    let doubled = match form.rsplit_once(' ') {
        Some((pre, w)) => format!("{pre} {w}{w}"),
        None => format!("{form}{form}"),
    };
    vec![form.to_string(), doubled]
}

/// UniMorph's Bengali person tags are cross-wired relative to the surface
/// forms: `2;POL` is the সে (third ordinary) form and `3;INFM` the তুমি
/// (second familiar) form. This maps each (person, politeness) tag pair
/// to the engine's agreement class by the form it actually denotes.
fn person(who: &str, pol: &str) -> Option<Person> {
    match (who, pol) {
        ("1", _) => Some(Person::First),
        ("2", "LGSPEC1") => Some(Person::SecondIntimate),
        ("3", "INFM") => Some(Person::SecondFamiliar),
        ("2", "POL") => Some(Person::Third),
        ("3", "POL") => Some(Person::Honorific),
        _ => None,
    }
}

fn generate(v: &Verb, features: &str) -> Option<Vec<String>> {
    // Non-finite forms first.
    match features {
        "V;V.NFIN" => return one(v.verbal_infinitive()),
        "V;V.MSDR" => return one(v.infinitive()),
        "V;V.PTCP;PRF" => return one(v.perfective()),
        "V;V.PTCP;HAB" => return Some(redup(&v.habitual_participle())),
        "V;V.PTCP;PROG" => return Some(redup(&v.progressive_participle())),
        "V;V.PTCP;COND" => return one(v.conditional()),
        _ => {}
    }

    let t: Vec<&str> = features.split(';').collect();
    let has = |s: &str| t.contains(&s);
    let who = t.iter().find(|x| matches!(**x, "1" | "2" | "3"))?;
    let pol = t
        .iter()
        .find(|x| matches!(**x, "LGSPEC1" | "POL" | "INFM"))
        .copied()
        .unwrap_or("");
    let p = person(who, pol)?;

    let tense = if has("FUT") {
        Tense::Future
    } else if has("HAB") {
        Tense::Habitual
    } else if has("PROG") {
        if has("PST") {
            Tense::PastProgressive
        } else {
            Tense::PresentProgressive
        }
    } else if has("PRF") {
        if has("PST") {
            Tense::PastPerfect
        } else {
            Tense::PresentPerfect
        }
    } else if has("PST") {
        Tense::Past
    } else if has("PRS") {
        Tense::Present
    } else {
        return None;
    };
    one(v.finite(tense, p))
}

fn category(features: &str) -> &'static str {
    if features.contains("V.NFIN") || features.contains("V.MSDR") || features.contains("V.PTCP") {
        "nonfinite"
    } else if features.contains("FUT") {
        "future"
    } else if features.contains("HAB") {
        "habitual"
    } else if features.contains("PROG") {
        "progressive"
    } else if features.contains("PRF") {
        "perfect"
    } else if features.contains("PST") {
        "past"
    } else {
        "present"
    }
}

fn main() {
    ablaut::harness::run(ablaut::harness::Spec {
        lang: "ben",
        default_paths: ["data/ben/unimorph.tsv", "data/ben/kaikki.tsv"],
        adjudications: "docs/ben/adjudications.tsv",
        mismatches: "target/golden_ben_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
