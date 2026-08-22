//! Persian golden-test harness: diff the engine against the agreement of
//! the two Persian oracles — kaikki.org (Wiktextract) and apertium-pes
//! (a hand-built lttoolbox dictionary, the independent leg).
//!
//! Usage: cargo run --release --bin golden_pes [gold.tsv ...] [--check]
//!        (default: data/pes/kaikki.tsv data/pes/apertium.tsv —
//!         see scripts/pes/fetch_kaikki.sh, scripts/pes/fetch_apertium.sh)
//!
//! With two gold files only the slots the oracles agree on are scored;
//! the slots where they contradict each other are the adjudication
//! corpus. With one file that file is scored directly (kaikki alone also
//! exercises the aorist, pluperfect, future, perfect-subjunctive and
//! progressives, which apertium-pes does not spell out).

use ablaut::harness::{run, Spec};
use ablaut::pes::{Number, Person, Polarity, Tense, Verb};

const CATEGORIES: [&str; 8] = [
    "infinitive",
    "participle",
    "imperative",
    "past",
    "present",
    "subjunctive",
    "perfect",
    "periphrastic",
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

/// A finite form, plus — for the multi-word tenses — the space-removed
/// spelling, since the oracles differ on whether the perfect/future
/// auxiliary is written joined (apertium کرده‌است → کردهاست) or apart
/// (kaikki کرده است).
fn finite(v: &Verb, t: Tense, p: &str, n: &str) -> Option<Vec<String>> {
    let (p, n) = (person(p)?, number(n)?);
    let form = v.form(t, p, n, Polarity::Positive);
    let joined = form.replace(' ', "");
    Some(if joined == form {
        vec![form]
    } else {
        vec![form, joined]
    })
}

fn generate(v: &Verb, features: &str) -> Option<Vec<String>> {
    let t: Vec<&str> = features.split(';').collect();
    let one = |s: String| Some(vec![s]);
    match t.as_slice() {
        ["V", "NFIN"] => one(v.infinitive().to_string()),
        ["V", "PTCP", "PST"] => one(v.past_participle()),
        ["V", "PTCP", "PRS"] => one(v.present_participle()),
        ["V", "IMP", "2", n] => one(v.imperative(number(n)?, Polarity::Positive)),
        ["V", "AOR", p, n] => finite(v, Tense::Aorist, p, n),
        ["V", "IND", "PRS", p, n] => finite(v, Tense::Present, p, n),
        ["V", "IND", "PST", p, n] => finite(v, Tense::Past, p, n),
        ["V", "IND", "PRF", p, n] => finite(v, Tense::Perfect, p, n),
        ["V", "IND", "PLUP", p, n] => finite(v, Tense::Pluperfect, p, n),
        ["V", "IND", "PRS", "PROG", p, n] => finite(v, Tense::PresentProgressive, p, n),
        ["V", "IND", "PST", "PROG", p, n] => finite(v, Tense::PastProgressive, p, n),
        ["V", "SBJV", p, n] => finite(v, Tense::Subjunctive, p, n),
        ["V", "SBJV", "PRF", p, n] => finite(v, Tense::PerfectSubjunctive, p, n),
        ["V", "FUT", p, n] => finite(v, Tense::Future, p, n),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V;PTCP") {
        "participle"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.contains("PROG")
        || features.contains("PLUP")
        || features.starts_with("V;FUT")
    {
        "periphrastic"
    } else if features.contains("PRF") {
        "perfect"
    } else if features.contains("SBJV") || features.contains("AOR") {
        "subjunctive"
    } else if features.contains("PRS") {
        "present"
    } else {
        "past"
    }
}

fn main() {
    run(Spec {
        lang: "pes",
        default_paths: ["data/pes/kaikki.tsv", "data/pes/apertium.tsv"],
        adjudications: "docs/pes/adjudications.tsv",
        mismatches: "target/golden_pes_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
