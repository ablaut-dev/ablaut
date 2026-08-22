//! Tamil golden-test harness: diff the engine against the agreement of
//! the two Tamil oracles (kaikki.org and the ThamizhiMorph FST).
//!
//! Usage: cargo run --release --bin golden_tam [gold.tsv ...] [--check]
//!        (default: data/tam/thamizhi.tsv data/tam/kaikki.tsv —
//!         see scripts/tam/fetch_thamizhi.sh, scripts/tam/fetch_kaikki.sh)
//!
//! With two gold files only slots the oracles agree on are scored; slots
//! where they contradict each other are the disagreement corpus. With one
//! file that file is scored directly.

use ablaut::harness::{run, Spec};
use ablaut::tam::{Png, Relative, Tense, Verb};

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "present",
    "past",
    "future",
    "participle",
    "imperative",
];

fn png(tag: &str) -> Option<Png> {
    Some(match tag {
        "1SG" => Png::P1Sg,
        "1PL" => Png::P1Pl,
        "2SG" => Png::P2Sg,
        "2PL" => Png::P2Pl,
        "3SGM" => Png::P3SgM,
        "3SGF" => Png::P3SgF,
        "3SGH" => Png::P3SgH,
        "3SGN" => Png::P3SgN,
        "3PLE" => Png::P3PlE,
        "3PLN" => Png::P3PlN,
        _ => return None,
    })
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let t: Vec<&str> = features.split(';').collect();
    let one = |s: String| Some(vec![s]);
    match t.as_slice() {
        ["V", "NFIN"] => one(verb.root().to_string()),
        ["V", "INF"] => one(verb.infinitive().to_string()),
        ["V", "CVB"] => one(verb.adverbial()),
        ["V", "COND"] => one(verb.conditional()),
        ["V", "PTCP", "PST"] => one(verb.relative(Relative::Past)),
        ["V", "PTCP", "PRS"] => one(verb.relative(Relative::Present)),
        ["V", "PTCP", "FUT"] => one(verb.relative(Relative::Future)),
        ["V", "IMP", "SG"] => one(verb.imperative(ablaut::tam::Number::Singular)),
        ["V", "IMP", "PL"] => one(verb.imperative(ablaut::tam::Number::Plural)),
        ["V", "PST", p] => one(verb.finite(Tense::Past, png(p)?)),
        ["V", "PRS", p] => one(verb.finite(Tense::Present, png(p)?)),
        ["V", "FUT", p] => one(verb.finite(Tense::Future, png(p)?)),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.starts_with("V;INF") || features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V;PTCP") || features.starts_with("V;CVB") {
        "participle"
    } else if features.starts_with("V;PRS") {
        "present"
    } else if features.starts_with("V;PST") {
        "past"
    } else if features.starts_with("V;FUT") || features.starts_with("V;COND") {
        "future"
    } else {
        "past"
    }
}

fn main() {
    run(Spec {
        lang: "tam",
        default_paths: ["data/tam/thamizhi.tsv", "data/tam/kaikki.tsv"],
        adjudications: "docs/tam/adjudications.tsv",
        mismatches: "target/golden_tam_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_root(lemma).ok(),
        generate,
        category,
    });
}
