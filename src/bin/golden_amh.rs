//! Amharic golden-test harness (single-oracle, Beta: UniMorph only).
//! Usage: cargo run --release --bin golden_amh [gold.tsv ...] [--check]

use ablaut::amh::Verb;
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 5] = [
    "perfective",
    "imperfective",
    "perfect",
    "imperative",
    "present",
];

fn category(features: &str) -> &'static str {
    let has = |t: &str| features.split(';').any(|x| x == t);
    if has("IMP") {
        "imperative"
    } else if has("PFV") {
        "perfective"
    } else if has("PRF") {
        "perfect"
    } else if has("PRS") {
        "present"
    } else {
        "imperfective"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    Some(vec![verb.form(features)?])
}

fn main() {
    run(Spec {
        lang: "amh",
        default_paths: ["data/amh/unimorph.tsv", "data/amh/_no_second_oracle.tsv"],
        adjudications: "docs/amh/adjudications.tsv",
        mismatches: "target/golden_amh_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_lemma(lemma).ok(),
        generate,
        category,
    });
}
