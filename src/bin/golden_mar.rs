//! Marathi golden-test harness: diff the engine against the gold data.
//!
//! Usage: cargo run --release --bin golden_mar [gold.tsv ...] [--check]
//!        (default: data/mar/apertium.tsv data/mar/kaikki.tsv —
//!         see scripts/mar/fetch_apertium.sh, scripts/mar/fetch_kaikki.sh)
//!
//! apertium-mar is the primary oracle: a hand-built lttoolbox dictionary
//! (no Wiktionary lineage) with the full person × gender × number finite
//! paradigm. It is scored per cell. kaikki.org Marathi is the independent
//! second oracle, but Wiktextract lost the person/number on its finite
//! `mr-conj` cells, so its clean per-cell contribution is the non-finite
//! forms (infinitive, converb, prospective, purposive); it corroborates
//! the finite paradigm at the set level. See docs/mar/oracles.md.
//!
//! With one file that file is scored directly; two files are intersected
//! into an agreement gold (the non-finite two-oracle gate).

use ablaut::mar::{Gender, Number, Person, Verb};

const CATEGORIES: [&str; 6] = [
    "present",
    "past",
    "subjunctive",
    "future",
    "imperative",
    "nonfinite",
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

fn gender(tag: &str) -> Option<Gender> {
    match tag {
        "MASC" => Some(Gender::Masculine),
        "FEM" => Some(Gender::Feminine),
        "NEUT" => Some(Gender::Neuter),
        _ => None,
    }
}

/// The present habitual does not distinguish gender in the plural, so
/// apertium's combined MF/MFN tag maps onto the masculine cell.
fn present_gender(tag: &str) -> Option<Gender> {
    match tag {
        "MF" | "MFN" => Some(Gender::Masculine),
        other => gender(other),
    }
}

fn one(s: String) -> Option<Vec<String>> {
    Some(vec![s])
}

fn generate(v: &Verb, features: &str) -> Option<Vec<String>> {
    let t: Vec<&str> = features.split(';').collect();
    match t.as_slice() {
        ["V", "NFIN"] => one(v.infinitive()),
        ["V", "CVB", "PFV"] => one(v.completive()),
        ["V", "PROSP"] => one(v.prospective()),
        ["V", "PURP"] => one(v.purposive()),
        ["V", "IND", "PRS", "HAB", p, g, n] => {
            // The present-habitual plural collapses gender, so apertium's
            // combined MF/MFN plural equals the per-gender plural.
            one(v.present(person(p)?, present_gender(g)?, number(n)?))
        }
        ["V", "IND", "PST", "PFV", p, g, n] => {
            one(v.perfective(person(p)?, gender(g)?, number(n)?))
        }
        // The subjunctive does not distinguish person.
        ["V", "SBJV", g, n] => one(v.subjunctive(gender(g)?, number(n)?)),
        ["V", "IND", "FUT", p, n] => one(v.future(person(p)?, number(n)?)),
        ["V", "IMP", p, n] => one(v.imperative(person(p)?, number(n)?)),
        // The combined-gender (MF/MFN) plural cells apertium also lists
        // are the same surface as the specific-gender plural the engine
        // produces; they are covered by the per-gender cells above, so
        // anything left here is outside the bounded cell set.
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.contains("PST") {
        "past"
    } else if features.contains("SBJV") {
        "subjunctive"
    } else if features.contains("FUT") {
        "future"
    } else if features.contains("IMP") {
        "imperative"
    } else if features.contains("HAB") {
        "present"
    } else {
        "nonfinite"
    }
}

fn main() {
    ablaut::harness::run(ablaut::harness::Spec {
        lang: "mar",
        default_paths: ["data/mar/apertium.tsv", "data/mar/kaikki.tsv"],
        adjudications: "docs/mar/adjudications.tsv",
        mismatches: "target/golden_mar_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
