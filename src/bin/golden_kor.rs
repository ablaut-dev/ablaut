//! Korean golden-test harness: diff the engine against the agreement
//! of the two Korean oracles (the NIKL Basic Dictionary and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_kor [gold.tsv ...] [--check]
//!        (default: data/kor/krdict.tsv data/kor/kaikki.tsv —
//!         see scripts/kor/fetch_krdict.sh, scripts/kor/fetch_kaikki.sh)
//!
//! With two gold files only slots the oracles agree on are scored; slots
//! where they contradict each other are the adjudication corpus. With one
//! file that file is scored directly.
//!
//! Feature schema — the four whole-word forms both oracles attest:
//!   V;INTIMATE       the intimate `-아/어` form (해체, 먹어)
//!   V;CONN;NI        the `-(으)니` connective (먹으니)
//!   V;DET;PRS        the present adnominal `-는` (먹는)
//!   V;FORM;PRS       the formal-polite `-(스)ㅂ니다` (먹습니다)

use ablaut::harness::{run, Spec};
use ablaut::kor::Verb;

const CATEGORIES: [&str; 4] = ["intimate", "connective", "adnominal", "formal"];

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    match features {
        "V;INTIMATE" => Some(vec![verb.intimate()]),
        "V;CONN;NI" => Some(vec![verb.connective_ni()]),
        "V;DET;PRS" => Some(vec![verb.adnominal_present()]),
        "V;FORM;PRS" => Some(vec![verb.formal_present()]),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    match features {
        "V;INTIMATE" => "intimate",
        "V;CONN;NI" => "connective",
        "V;DET;PRS" => "adnominal",
        _ => "formal",
    }
}

fn main() {
    run(Spec {
        lang: "kor",
        default_paths: ["data/kor/krdict.tsv", "data/kor/kaikki.tsv"],
        adjudications: "docs/kor/adjudications.tsv",
        mismatches: "target/golden_kor_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
