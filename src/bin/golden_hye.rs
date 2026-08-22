//! Armenian golden-test harness: diff the engine against the agreement
//! of the two Armenian oracles (UniMorph hye and the uniparser Eastern
//! Armenian grammar).
//!
//! Usage: cargo run --release --bin golden_hye [gold.tsv ...] [--check]
//!        (default: data/hye/uniparser.tsv data/hye/unimorph.tsv —
//!         see scripts/hye/fetch_uniparser.sh, scripts/hye/fetch_unimorph.sh)
//!
//! With two gold files only slots the oracles agree on are scored; slots
//! where they contradict each other are the adjudication corpus. With one
//! file that file is scored directly — running it against
//! `data/hye/unimorph.tsv` alone also exercises the analytic tenses, which
//! the grammar-derived oracle does not spell out.

use ablaut::harness::{run, Spec};
use ablaut::hye::{Converb, Number, Participle, Person, Polarity, Tense, Verb};

const CATEGORIES: [&str; 7] = [
    "infinitive",
    "converb",
    "participle",
    "aorist",
    "subjunctive",
    "imperative",
    "analytic",
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

fn finite(verb: &Verb, tense: Tense, n: &str, p: &str, polarity: Polarity) -> Option<Vec<String>> {
    let (n, p) = (number(n)?, person(p)?);
    Some(vec![verb.form(tense, p, n, polarity)])
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    // UniMorph writes the polarity inside the bundle at a
    // position that varies by tense; pull it out first.
    let polarity = if features.split(';').any(|t| t == "NEG") {
        Polarity::Negative
    } else {
        Polarity::Positive
    };
    let bare: Vec<&str> = features.split(';').filter(|t| *t != "NEG").collect();
    let one = |s: String| Some(vec![s]);
    match bare.as_slice() {
        ["V", "NFIN"] => one(verb.infinitive(polarity)),
        ["V.CVB", "IPFV"] => one(verb.converb(Converb::Imperfective, polarity)),
        ["V.CVB", "PFV"] => one(verb.converb(Converb::Perfective, polarity)),
        ["V.CVB", "FUT", "LGSPEC02"] => one(verb.converb(Converb::Future, polarity)),
        ["V.CVB", "FUT", "LGSPEC03"] => one(verb.converb(Converb::Prospective, polarity)),
        ["V.CVB", "SIM"] => one(verb.converb(Converb::Simultaneous, polarity)),
        ["V.CVB", "LGSPEC04"] => one(verb.converb(Converb::Connegative, polarity)),
        ["V", "V.PTCP", "SUB"] => one(verb.participle(Participle::Subject, polarity)),
        ["V", "V.PTCP", "LGSPEC01"] | ["V.PTCP", "LGSPEC01"] => {
            one(verb.participle(Participle::Resultative, polarity))
        }
        ["V", "PASS"] => one(verb.passive().to_string()),
        ["V", "CAUS"] => one(verb.causative().to_string()),
        ["V", "IMP", n, "2"] => one(verb.imperative(number(n)?, polarity)),
        ["V", "IND", n, p, "PST"] => finite(verb, Tense::Aorist, n, p, polarity),
        ["V", "IND", n, p, "PRS"] => finite(verb, Tense::Present, n, p, polarity),
        ["V", "IND", n, p, "FUT"] => finite(verb, Tense::Future, n, p, polarity),
        ["V", "IPFV", "IND", n, p] => finite(verb, Tense::Imperfect, n, p, polarity),
        ["V", "PRF", "IND", n, p, "PRS"] => finite(verb, Tense::Perfect, n, p, polarity),
        ["V", "PRF", "IND", n, p, "PST"] => finite(verb, Tense::Pluperfect, n, p, polarity),
        ["V", "PRF", "IND", n, p, "FUT"] => finite(verb, Tense::FutureInPast, n, p, polarity),
        ["V", "SBJV", n, p, "FUT"] => finite(verb, Tense::SubjunctiveFuture, n, p, polarity),
        ["V", "PRF", "SBJV", n, p, "FUT"] => finite(verb, Tense::SubjunctivePast, n, p, polarity),
        ["V", "COND", n, p, "FUT"] => finite(verb, Tense::Conditional, n, p, polarity),
        ["V", "PRF", "COND", n, p, "FUT"] => finite(verb, Tense::ConditionalPast, n, p, polarity),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.starts_with("V.CVB") {
        "converb"
    } else if features.contains("PTCP") || features.ends_with("PASS") || features.ends_with("CAUS")
    {
        "participle"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V;IND") && features.ends_with("PST") {
        "aorist"
    } else if features.contains("SBJV") || features.contains("COND") {
        "subjunctive"
    } else {
        "analytic"
    }
}

fn main() {
    run(Spec {
        lang: "hye",
        default_paths: ["data/hye/uniparser.tsv", "data/hye/unimorph.tsv"],
        adjudications: "docs/hye/adjudications.tsv",
        mismatches: "target/golden_hye_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 99.5,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
