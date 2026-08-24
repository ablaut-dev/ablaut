//! Indonesian golden-test harness: diff the engine against the single
//! Indonesian oracle (UniMorph `ind`, ~15k verb rows) — Beta tier. kaikki's
//! Indonesian verb dump lists many headwords but exposes almost no
//! UniMorph-aligned voice/derivation inflection cells, so the two cannot
//! form an agreement loop; UniMorph is scored directly.
//!
//! Usage: cargo run --release --bin golden_ind [gold.tsv ...] [--check]
//!        (default: data/ind/unimorph.tsv)

use ablaut::harness::{run, Spec};
use ablaut::ind::Verb;

const CATEGORIES: [&str; 5] = ["active", "passive", "derived", "enclitic", "other"];

fn category(features: &str) -> &'static str {
    let has = |t: &str| features.split(';').any(|x| x == t);
    if has("PSS1S") || has("PSS2S") || has("1") || has("2") || has("FOC") {
        "enclitic"
    } else if has("PASS") {
        "passive"
    } else if has("ACT") {
        "active"
    } else if has("DEF") {
        "derived"
    } else {
        "other"
    }
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let forms = verb.forms(features);
    if forms.is_empty() {
        None
    } else {
        Some(forms)
    }
}

fn main() {
    run(Spec {
        lang: "ind",
        // Single oracle (Beta): the second path is an empty placeholder, so
        // UniMorph is scored directly.
        default_paths: ["data/ind/unimorph.tsv", "data/ind/_no_second_oracle.tsv"],
        adjudications: "docs/ind/adjudications.tsv",
        mismatches: "target/golden_ind_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_lemma(lemma).ok(),
        generate,
        category,
    });
}
