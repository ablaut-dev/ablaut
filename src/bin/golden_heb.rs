//! Hebrew golden-test harness: diff the engine against the agreement of the
//! two Hebrew oracles (UniMorph and kaikki.org, the latter with niqqud
//! stripped). Usage: cargo run --release --bin golden_heb [gold.tsv ...] [--check]

use ablaut::harness::{run, Spec};
use ablaut::heb::Verb;

const CATEGORIES: [&str; 5] = ["past", "present", "future", "imperative", "infinitive"];

fn category(features: &str) -> &'static str {
    let has = |t: &str| features.split(';').any(|x| x == t);
    if has("IMP") {
        "imperative"
    } else if has("NFIN") {
        "infinitive"
    } else if has("FUT") {
        "future"
    } else if has("PRS") {
        "present"
    } else {
        "past"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    Some(vec![verb.form(features)?])
}

fn main() {
    run(Spec {
        lang: "heb",
        default_paths: ["data/heb/unimorph.tsv", "data/heb/kaikki.tsv"],
        adjudications: "docs/heb/adjudications.tsv",
        mismatches: "target/golden_heb_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_lemma(lemma).ok(),
        generate,
        category,
    });
}
