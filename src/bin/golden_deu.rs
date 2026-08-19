//! German correctness-stats adapter for the shared harness: scores the
//! engine on the UniMorph ∩ kaikki agreement and emits the stats the
//! correctness table reads (target/stats_deu.json).
//!
//! Usage: cargo run --release --bin golden_deu [gold.tsv ...] [--check]
//!        (default: data/deu/unimorph data/deu/kaikki.tsv)
//!
//! The battle-tested German CI gate is the original `golden` bin
//! (`src/bin/golden.rs`), which keeps German's covered/fallback split and
//! corrupt-lemma exclusion. This bin exists only to place German in
//! `docs/correctness.md` on the same two-oracle footing as every other
//! language; the feature→form mapping is the same one `golden` uses.

use ablaut::harness::{run, Spec};
use ablaut::{AnalyticTense, Auxiliary, Mood, Number, Person, Tense, Verb};

const CATEGORIES: [&str; 10] = [
    "infinitive",
    "auxiliary",
    "present",
    "preterite",
    "konjunktiv1",
    "konjunktiv2",
    "imperative",
    "participle",
    "perfect",
    "future",
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

/// Map a UniMorph feature bundle to the engine's form(s); None marks a
/// bundle the harness does not model (or a form the language lacks, which
/// the shared harness scores as an uncovered slot rather than a miss).
fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    let form = match f.as_slice() {
        ["V", "NFIN"] => Some(verb.infinitive().to_string()),
        ["V", "NFIN", "LGSPEC01"] => Some(verb.zu_infinitive()),
        ["V", "AUX"] => Some(
            match verb.auxiliary() {
                Auxiliary::Haben => "haben",
                Auxiliary::Sein => "sein",
            }
            .to_string(),
        ),
        ["V.PTCP", "PRS"] => Some(verb.present_participle()),
        ["V.PTCP", "PST"] => Some(verb.past_participle()),
        ["V", "IMP", n, "2"] => return number(n).and_then(|n| verb.imperative(n)).map(|f| vec![f]),
        ["V", t @ ("PRF" | "PLPRF" | "FUT1" | "FUT2"), m, n, p] => {
            let tense = match *t {
                "PRF" => AnalyticTense::Perfect,
                "PLPRF" => AnalyticTense::Pluperfect,
                "FUT1" => AnalyticTense::FutureI,
                _ => AnalyticTense::FutureII,
            };
            let mood = match (*m, *t) {
                ("IND", _) => Mood::Indicative,
                (_, "PLPRF") => Mood::KonjunktivII,
                _ => Mood::KonjunktivI,
            };
            match (person(p), number(n)) {
                (Some(p), Some(n)) => Some(verb.analytic(tense, mood, p, n)),
                _ => None,
            }
        }
        ["V", mood @ ("IND" | "SBJV"), n, p, tense @ ("PRS" | "PST")] => {
            let (t, m) = match (*mood, *tense) {
                ("IND", "PRS") => (Tense::Present, Mood::Indicative),
                ("IND", "PST") => (Tense::Preterite, Mood::Indicative),
                ("SBJV", "PRS") => (Tense::Present, Mood::KonjunktivI),
                _ => (Tense::Present, Mood::KonjunktivII),
            };
            match (person(p), number(n)) {
                (Some(p), Some(n)) => Some(verb.conjugate(t, m, p, n)),
                _ => None,
            }
        }
        _ => None,
    };
    form.map(|f| vec![f])
}

fn category(features: &str) -> &'static str {
    if features.starts_with("V.PTCP") {
        "participle"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features == "V;AUX" {
        "auxiliary"
    } else if features.starts_with("V;PRF") || features.starts_with("V;PLPRF") {
        "perfect"
    } else if features.starts_with("V;FUT") {
        "future"
    } else if features.starts_with("V;IND") && features.ends_with("PRS") {
        "present"
    } else if features.starts_with("V;IND") && features.ends_with("PST") {
        "preterite"
    } else if features.starts_with("V;SBJV") && features.ends_with("PRS") {
        "konjunktiv1"
    } else {
        "konjunktiv2"
    }
}

fn main() {
    run(Spec {
        lang: "deu",
        default_paths: ["data/deu/unimorph", "data/deu/kaikki.tsv"],
        adjudications: "docs/deu/adjudications.tsv",
        mismatches: "target/golden_deu_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.0,
        min_lemma_coverage_pct: 90.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
