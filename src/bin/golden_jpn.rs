//! Japanese golden-test harness: diff the engine against the agreement
//! of the two Japanese oracles (kaikki.org and mecab-ipadic).
//!
//! Usage: cargo run --release --bin golden_jpn [gold.tsv ...] [--check]
//!        (default: data/jpn/ipadic.tsv data/jpn/kaikki.tsv —
//!         see scripts/jpn/fetch_ipadic.sh, scripts/jpn/fetch_kaikki.sh)
//!
//! With two gold files only slots the oracles agree on are scored; slots
//! where they contradict (special-ラ行 なさり/なさい, class-ambiguous
//! verbs, the する/来る godan homographs) are the adjudication corpus.

use ablaut::harness::{run, Spec};
use ablaut::jpn::{Form, Verb};

const CATEGORIES: [&str; 3] = ["terminal", "continuative", "past"];

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let form = match features {
        "V;NFIN" => Form::Terminal,
        "V;CONT" => Form::Continuative,
        "V;PST" => Form::Past,
        _ => return None,
    };
    Some(vec![verb.form(form)])
}

fn category(features: &str) -> &'static str {
    match features {
        "V;CONT" => "continuative",
        "V;PST" => "past",
        _ => "terminal",
    }
}

fn main() {
    run(Spec {
        lang: "jpn",
        default_paths: ["data/jpn/ipadic.tsv", "data/jpn/kaikki.tsv"],
        adjudications: "docs/jpn/adjudications.tsv",
        mismatches: "target/golden_jpn_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
