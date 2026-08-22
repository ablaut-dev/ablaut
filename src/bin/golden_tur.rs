//! Turkish golden-test harness: diff the engine against the agreement of
//! the two Turkish oracles (kaikki.org Wiktextract and the native-verified
//! UniMorph tur).
//!
//! Usage: cargo run --release --bin golden_tur [gold.tsv ...] [--check]
//!        (default: data/tur/kaikki.tsv data/tur/unimorph.tsv —
//!         see scripts/tur/fetch_kaikki.sh, scripts/tur/fetch_unimorph.sh)
//!
//! Both oracles are aligned to the single-word synthetic paradigm (see
//! docs/tur/oracles.md). Only the slots the two agree on are scored.

use ablaut::harness::{run, Spec};
use ablaut::tur::{Number, Person, Polarity, Tense, Verb};

const CATEGORIES: [&str; 5] = ["infinitive", "aorist", "tense", "stacked", "imperative"];

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

fn tense(skeleton: &str) -> Option<Tense> {
    Some(match skeleton {
        "V;IND;PRS;HAB" => Tense::Aorist,
        "V;IND;PRS;PROG" => Tense::Progressive,
        "V;IND;FUT" => Tense::Future,
        "V;IND;PST" => Tense::Past,
        "V;INFR;PST" => Tense::Evidential,
        "V;OBLIG;PRS" => Tense::Necessitative,
        "V;IND;PST;HAB" => Tense::AoristPast,
        "V;INFR;PRS;HAB" => Tense::AoristEvidential,
        "V;IND;PST;PROG" => Tense::ProgressivePast,
        "V;INFR;PRS;PROG" => Tense::ProgressiveEvidential,
        "V;IND;PST;PROSP" => Tense::FuturePast,
        "V;INFR;FUT" => Tense::FutureEvidential,
        "V;INFR;PST;PFV" => Tense::EvidentialPast,
        _ => return None,
    })
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let one = |s: String| Some(vec![s]);
    if features == "V;NFIN" {
        return one(verb.infinitive().to_string());
    }
    let t: Vec<&str> = features.split(';').collect();
    // Imperative: V;IMP;2;{SG,PL}[;LGSPEC2];{POS,NEG}
    if t.len() >= 5 && t[1] == "IMP" {
        let polarity = polarity(features);
        let n = number(t[3])?;
        let formal = features.contains(";LGSPEC2");
        return one(verb.imperative(n, polarity, formal));
    }
    // Finite: <skeleton>;{p};{n};{pol}
    let pol = *t.last()?;
    let n = number(t[t.len() - 2])?;
    let p = person(t[t.len() - 3])?;
    let skeleton = t[..t.len() - 3].join(";");
    let tense = tense(&skeleton)?;
    let polarity = if pol == "NEG" {
        Polarity::Negative
    } else {
        Polarity::Positive
    };
    one(verb.form(tense, p, n, polarity))
}

fn polarity(features: &str) -> Polarity {
    if features.split(';').any(|t| t == "NEG") {
        Polarity::Negative
    } else {
        Polarity::Positive
    }
}

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.contains(";IMP;") {
        "imperative"
    } else if features.starts_with("V;IND;PRS;HAB") {
        "aorist"
    } else {
        // Single-token TAM = base tense; two+ = copular stack.
        let skeleton: Vec<&str> = features
            .split(';')
            .filter(|s| !matches!(*s, "1" | "2" | "3" | "SG" | "PL" | "POS" | "NEG"))
            .collect();
        match tense(&skeleton.join(";")) {
            Some(
                Tense::Aorist
                | Tense::Progressive
                | Tense::Future
                | Tense::Past
                | Tense::Evidential
                | Tense::Necessitative,
            ) => "tense",
            Some(_) => "stacked",
            None => "tense",
        }
    }
}

fn main() {
    run(Spec {
        lang: "tur",
        default_paths: ["data/tur/kaikki.tsv", "data/tur/unimorph.tsv"],
        adjudications: "docs/tur/adjudications.tsv",
        mismatches: "target/golden_tur_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.0,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
