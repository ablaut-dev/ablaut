//! Urdu golden-test harness: diff the engine against the agreement of
//! the two Urdu oracles — kaikki.org (Wiktextract) and apertium-urd (a
//! hand-built lttoolbox dictionary, the independent leg).
//!
//! Usage: cargo run --release --bin golden_urd [gold.tsv ...] [--check]
//!        (default: data/urd/kaikki.tsv data/urd/apertium.tsv —
//!         see scripts/urd/fetch_kaikki.sh, scripts/urd/fetch_apertium.sh)
//!
//! The `ur-conj` template kaikki draws on emits **no person tags**, so
//! kaikki resolves only the person-independent core (infinitive, oblique
//! infinitive, imperfective and perfective participles); that core is the
//! surface it shares with apertium and the two-oracle gate scored here.
//! The person-resolved finite paradigm the engine also produces
//! (subjunctive, imperatives, synthetic future, analytic layer) is
//! corroborated by apertium and UniMorph separately — see
//! docs/urd/oracles.md. Run against data/urd/unimorph.tsv alone to
//! exercise the full paradigm.

use ablaut::harness::{run, Spec};
use ablaut::urd::{Aspect, AuxMood, Gender, Number, Person, Politeness, Verb};

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "imperative",
    "subjunctive",
    "future",
    "participle",
    "analytic",
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
        _ => None,
    }
}

fn aspect(tag: &str) -> Option<Aspect> {
    match tag {
        "HAB" => Some(Aspect::Imperfective),
        "PRF" => Some(Aspect::Perfective),
        "PROG" => Some(Aspect::Progressive),
        _ => None,
    }
}

fn mood(tag: &str) -> Option<AuxMood> {
    match tag {
        "PRS" => Some(AuxMood::Present),
        "PST" => Some(AuxMood::Past),
        "SBJV" => Some(AuxMood::Subjunctive),
        "LGSPEC3" => Some(AuxMood::Presumptive),
        "LGSPEC4" => Some(AuxMood::Counterfactual),
        _ => None,
    }
}

fn one(s: String) -> Option<Vec<String>> {
    Some(vec![s])
}

fn generate(v: &Verb, features: &str) -> Option<Vec<String>> {
    let t: Vec<&str> = features.split(';').collect();
    match t.as_slice() {
        ["V", "NFIN", "LGSPEC1"] => one(v.infinitive()),
        ["V", "NFIN", "LGSPEC2"] => one(v.oblique_infinitive()),
        ["V", "2", "SG", "IMP", "INFM"] => one(v.imperative(Politeness::Intimate)),
        ["V", "2", "SG", "IMP", "FORM"] => one(v.imperative(Politeness::Familiar)),
        ["V", "2", "PL", "IMP", "FORM"] => one(v.imperative(Politeness::Polite)),
        ["V", "V.PTCP", g, n, "IPFV"] => one(v.imperfective(gender(g)?, number(n)?)),
        ["V", "V.PTCP", g, n, "PFV"] => one(v.perfective(gender(g)?, number(n)?)),
        ["V", p, n, "SBJV"] => one(v.subjunctive(person(p)?, number(n)?)),
        ["V", p, n, "SBJV", g] => one(v.future(person(p)?, number(n)?, gender(g)?)),
        ["V", p, n, asp, m, g] => {
            one(v.analytic(aspect(asp)?, mood(m)?, person(p)?, number(n)?, gender(g)?))
        }
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.contains("IMP") {
        "imperative"
    } else if features.contains("V.PTCP") {
        "participle"
    } else if features.ends_with("SBJV") {
        "subjunctive"
    } else if features.ends_with("MASC") || features.ends_with("FEM") {
        // A five-field bundle ending in a gender is the synthetic future;
        // a six-field one is analytic.
        if features.split(';').count() == 5 {
            "future"
        } else {
            "analytic"
        }
    } else {
        "analytic"
    }
}

fn main() {
    run(Spec {
        lang: "urd",
        default_paths: ["data/urd/kaikki.tsv", "data/urd/apertium.tsv"],
        adjudications: "docs/urd/adjudications.tsv",
        mismatches: "target/golden_urd_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
