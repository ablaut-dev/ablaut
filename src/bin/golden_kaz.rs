//! Kazakh golden-test harness: diff the engine against the single Kazakh
//! oracle (kaikki.org) — Beta tier. The UniMorph kaz table has no verb
//! rows, so no independent second oracle overlaps to form the agreement
//! loop.
//!
//! Usage: cargo run --release --bin golden_kaz [gold.tsv ...] [--check]
//!        (default: data/kaz/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::kaz::Verb;

const CATEGORIES: [&str; 5] = ["infinitive", "aorist", "past", "future", "imperative"];

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V;AOR") {
        "aorist"
    } else if features.starts_with("V;PST") {
        "past"
    } else if features.starts_with("V;FUT") {
        "future"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else {
        "aorist"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    Some(vec![verb.form(features)?])
}

fn main() {
    run(Spec {
        lang: "kaz",
        // Single oracle: the second path does not exist, so kaikki is
        // scored directly (Beta).
        default_paths: ["data/kaz/kaikki.tsv", "data/kaz/_no_second_oracle.tsv"],
        adjudications: "docs/kaz/adjudications.tsv",
        mismatches: "target/golden_kaz_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
