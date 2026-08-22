//! Swahili golden-test harness: diff the engine against the agreement of
//! the two Swahili oracles (UniMorph swc and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_swa [gold.tsv ...] [--check]
//!        (default: data/swa/swc.tsv data/swa/kaikki.tsv —
//!         see scripts/swa/fetch_unimorph.sh, scripts/swa/fetch_kaikki.sh)
//!
//! Both adapters emit the same canonical bundle `V;TAM[;SUBJ][;NEG]`, so
//! the shared harness intersects them: only slots the two oracles agree
//! on are scored, the rest are the adjudication corpus. The scored core
//! is the productive paradigm both spell out as whole words — the
//! infinitive, habitual, present and subjunctive over the person and
//! class-1/2 subjects, and the a-tense (gnomic) over every noun class.

use ablaut::harness::{run, Spec};
use ablaut::swa::{Number, Polarity, Subject, Tense, Verb};

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "imperative",
    "present",
    "subjunctive",
    "gnomic",
    "other",
];

/// Parse a canonical subject token (1SG, 2PL, CL7, …).
fn subject(tag: &str) -> Option<Subject> {
    match tag {
        "1SG" => Some(Subject::First(Number::Singular)),
        "2SG" => Some(Subject::Second(Number::Singular)),
        "1PL" => Some(Subject::First(Number::Plural)),
        "2PL" => Some(Subject::Second(Number::Plural)),
        _ => tag
            .strip_prefix("CL")
            .and_then(|n| n.parse::<u8>().ok())
            .map(Subject::Class),
    }
}

fn tense(tag: &str) -> Option<Tense> {
    Some(match tag {
        "PRS" => Tense::Present,
        "PST" => Tense::Past,
        "FUT" => Tense::Future,
        "PRF" => Tense::Perfect,
        "GNOM" => Tense::Gnomic,
        "SBJV" => Tense::Subjunctive,
        "SEQ" => Tense::Consecutive,
        "SIT" => Tense::Situative,
        "CONDP" => Tense::ConditionalPresent,
        "CONDPST" => Tense::ConditionalPast,
        _ => return None,
    })
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let neg = features.ends_with(";NEG");
    let polarity = if neg {
        Polarity::Negative
    } else {
        Polarity::Positive
    };
    let bare = features.strip_suffix(";NEG").unwrap_or(features);
    let f: Vec<&str> = bare.split(';').collect();
    let one = |s: String| Some(vec![s]);
    match f.as_slice() {
        ["V", "NFIN"] => one(verb.infinitive(polarity)),
        ["V", "IMP", "SG"] => one(verb.imperative(Number::Singular, polarity)),
        ["V", "IMP", "PL"] => one(verb.imperative(Number::Plural, polarity)),
        ["V", "HAB"] => one(verb.form(Tense::Habitual, Subject::Class(1), Polarity::Positive)),
        ["V", tam, subj] => {
            let t = tense(tam)?;
            let s = subject(subj)?;
            one(verb.form(t, s, polarity))
        }
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V;PRS") {
        "present"
    } else if features.starts_with("V;SBJV") {
        "subjunctive"
    } else if features.starts_with("V;GNOM") {
        "gnomic"
    } else {
        "other"
    }
}

fn main() {
    run(Spec {
        lang: "swa",
        default_paths: ["data/swa/swc.tsv", "data/swa/kaikki.tsv"],
        adjudications: "docs/swa/adjudications.tsv",
        mismatches: "target/golden_swa_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
