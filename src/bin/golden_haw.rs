//! Hawaiian golden-test harness: diff the engine against the single
//! Hawaiian oracle (kaikki.org Hawaiian — the lemma-linked "Derived terms"
//! and passive `forms` from Wiktionary) — Beta tier. Hawaiian marks TAM with
//! free particles (periphrasis, out of scope); the bound morphology is
//! derivational, so the scored slots are the causative (hoʻo-), full
//! reduplication and the -ʻia passive. There is no second oracle (UniMorph
//! has no `haw`), so kaikki is scored directly.
//!
//! Usage: cargo run --release --bin golden_haw [gold.tsv ...] [--check]
//!        (default: data/haw/kaikki.tsv)

use ablaut::harness::{run, Spec};
use ablaut::haw::Verb;

const CATEGORIES: [&str; 3] = ["causative", "reduplicated", "passive"];

fn category(features: &str) -> &'static str {
    let has = |t: &str| features.split(';').any(|x| x == t);
    if has("CAUS") {
        "causative"
    } else if has("RDP") {
        "reduplicated"
    } else if has("PASS") {
        "passive"
    } else {
        "causative"
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
        lang: "haw",
        // Single oracle (Beta): the second path is an empty placeholder, so
        // kaikki is scored directly.
        default_paths: ["data/haw/kaikki.tsv", "data/haw/_no_second_oracle.tsv"],
        adjudications: "docs/haw/adjudications.tsv",
        mismatches: "target/golden_haw_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_lemma(lemma).ok(),
        generate,
        category,
    });
}
