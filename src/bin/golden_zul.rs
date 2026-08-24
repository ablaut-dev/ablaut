//! Zulu golden-test harness: diff the engine against the agreement of the
//! two Zulu oracles (UniMorph `zul` and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_zul [gold.tsv ...] [--check]
//!        (default: data/zul/unimorph.tsv data/zul/kaikki.tsv —
//!         see scripts/zul/fetch_unimorph.sh, scripts/zul/fetch_kaikki.sh)
//!
//! Both adapters emit the same canonical bundle `V;<SUBJ>;<TAM>` (with
//! macrons stripped), so the shared harness intersects them: only slots
//! the two oracles agree on are scored. kaikki's cleanly-tagged slots are
//! per-person, so the scored core is the infinitive, the imperative, the
//! four person subjunctives and the four person remote pasts — the
//! productive template's backbone, independently confirmed.

use ablaut::harness::{run, Spec};
use ablaut::zul::Verb;

const CATEGORIES: [&str; 5] = [
    "infinitive",
    "imperative",
    "subjunctive",
    "remote_past",
    "other",
];

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.ends_with(";SBJV") {
        "subjunctive"
    } else if features.ends_with(";RMT_PST") {
        "remote_past"
    } else {
        "other"
    }
}

fn main() {
    run(Spec {
        lang: "zul",
        default_paths: ["data/zul/unimorph.tsv", "data/zul/kaikki.tsv"],
        adjudications: "docs/zul/adjudications.tsv",
        mismatches: "target/golden_zul_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.8,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_lemma(lemma).ok(),
        generate: |verb, features| verb.generate(features),
        category,
    });
}
