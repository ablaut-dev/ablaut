//! Estonian golden-test harness: diff the engine against the
//! agreement of the two Estonian oracles (Vabamorf and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_est [gold.tsv ...] [--check]
//!        (default: data/est/vabamorf.tsv data/est/kaikki.tsv —
//!         see `scripts/est/fetch_vabamorf.sh`, `scripts/est/fetch_kaikki.sh`)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); disagreements are the adjudication corpus.
//! With one file, that file is scored directly (the CI path).

use ablaut::est::{ImpersonalSlot, NonfiniteSlot, Number, Person, Verb};
use ablaut::harness::{run, Spec};

const CATEGORIES: [&str; 7] = [
    "nonfinite",
    "present",
    "past",
    "conditional",
    "imperative",
    "impersonal",
    "participle",
];

fn person(tag: &str) -> Option<Person> {
    match tag {
        "1" => Some(Person::First),
        "2" => Some(Person::Second),
        "3" => Some(Person::Third),
        _ => None,
    }
}

fn number(tag: &str) -> Option<Number> {
    match tag {
        "SG" => Some(Number::Singular),
        "PL" => Some(Number::Plural),
        _ => None,
    }
}

/// Map a feature bundle from the oracle TSVs to the engine's output
/// (None means unsupported bundle).
fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let f: Vec<&str> = features.split(';').collect();
    let pn = |n: &str, p: &str| match (person(p), number(n)) {
        (Some(p), Some(n)) => Some((p, n)),
        _ => None,
    };
    let one = |s: String| Some(vec![s]);
    match f.as_slice() {
        ["V", "NFIN", "MA"] => one(verb.nonfinite(NonfiniteSlot::Ma)),
        ["V", "NFIN", "DA"] => one(verb.nonfinite(NonfiniteSlot::Da)),
        ["V", "NFIN", "MA", "INE"] => one(verb.nonfinite(NonfiniteSlot::MaInessive)),
        ["V", "NFIN", "MA", "ELA"] => one(verb.nonfinite(NonfiniteSlot::MaElative)),
        ["V", "NFIN", "MA", "TRANSL"] => one(verb.nonfinite(NonfiniteSlot::MaTranslative)),
        ["V", "NFIN", "MA", "ABE"] => one(verb.nonfinite(NonfiniteSlot::MaAbessive)),
        ["V", "NFIN", "MA", "PASS"] => one(verb.nonfinite(NonfiniteSlot::MaPassive)),
        ["V", "QUOT"] => one(verb.nonfinite(NonfiniteSlot::Quotative)),
        ["V.PTCP", "PRS", "ACT"] => one(verb.nonfinite(NonfiniteSlot::PresentParticiple)),
        ["V.PTCP", "PRS", "PASS"] => one(verb.nonfinite(NonfiniteSlot::PassivePresentParticiple)),
        ["V.PTCP", "PST", "ACT"] => one(verb.nonfinite(NonfiniteSlot::NudParticiple)),
        ["V.PTCP", "PST", "PASS"] => one(verb.nonfinite(NonfiniteSlot::TudParticiple)),
        ["V", "IND", "PRS", "IMPRS"] => one(verb.impersonal(ImpersonalSlot::Present)),
        ["V", "IND", "PST", "IMPRS"] => one(verb.impersonal(ImpersonalSlot::Past)),
        ["V", "COND", "PRS", "IMPRS"] => one(verb.impersonal(ImpersonalSlot::Conditional)),
        ["V", "IMP", "IMPRS"] => one(verb.impersonal(ImpersonalSlot::Imperative)),
        ["V", "IND", "PRS", n, p] => pn(n, p).map(|(p, n)| vec![verb.present(p, n)]),
        ["V", "IND", "PST", n, p] => pn(n, p).map(|(p, n)| vec![verb.past(p, n)]),
        ["V", "COND", "PRS", n, p] => pn(n, p).map(|(p, n)| vec![verb.conditional(p, n)]),
        ["V", "COND", "PRF", n, p] => pn(n, p).map(|(p, n)| vec![verb.conditional_perfect(p, n)]),
        ["V", "IMP", n, p] => {
            let (p, n) = pn(n, p)?;
            verb.imperative(p, n).map(|f| vec![f])
        }
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
fn category(features: &str) -> &'static str {
    if features.starts_with("V;NFIN") || features == "V;QUOT" {
        "nonfinite"
    } else if features.starts_with("V.PTCP") {
        "participle"
    } else if features.ends_with("IMPRS") {
        "impersonal"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IND;PST") {
        "past"
    } else if features.starts_with("V;COND") {
        "conditional"
    } else {
        "imperative"
    }
}

fn main() {
    run(Spec {
        lang: "est",
        default_paths: ["data/est/vabamorf.tsv", "data/est/kaikki.tsv"],
        adjudications: "docs/est/adjudications.tsv",
        mismatches: "target/golden_est_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.5,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
