//! Golden-test harness: diff ablaut's output against the UniMorph `deu`
//! dataset (https://github.com/unimorph/deu).
//!
//! Usage: cargo run --release --bin golden [path-to-unimorph-file]
//!        (default: data/unimorph/deu — see scripts/fetch_unimorph.sh)
//!
//! A prediction counts as a match if it is among the gold variants for that
//! (lemma, feature bundle). Mismatches are written to
//! target/golden_mismatches.tsv for adjudication. Accuracy is reported
//! separately for lexicon-covered lemmas (where we claim correctness) and
//! weak-fallback lemmas (where unlisted strong verbs are expected failures).

use ablaut::{AnalyticTense, Auxiliary, Mood, Number, Person, Tense, Verb};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;

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

/// Map a UniMorph feature bundle to our API; None = unsupported bundle.
fn generate(verb: &Verb, features: &str) -> Option<Option<String>> {
    let f: Vec<&str> = features.split(';').collect();
    match f.as_slice() {
        ["V", "NFIN"] => Some(Some(verb.infinitive().to_string())),
        ["V", "NFIN", "LGSPEC01"] => Some(Some(verb.zu_infinitive())),
        ["V", "AUX"] => Some(Some(
            match verb.auxiliary() {
                Auxiliary::Haben => "haben",
                Auxiliary::Sein => "sein",
            }
            .to_string(),
        )),
        ["V.PTCP", "PRS"] => Some(Some(verb.present_participle())),
        ["V.PTCP", "PST"] => Some(Some(verb.past_participle())),
        ["V", "IMP", n, "2"] => Some(verb.imperative(number(n)?)),
        ["V", t @ ("PRF" | "PLPRF" | "FUT1" | "FUT2"), m, n, p] => {
            let tense = match *t {
                "PRF" => AnalyticTense::Perfect,
                "PLPRF" => AnalyticTense::Pluperfect,
                "FUT1" => AnalyticTense::FutureI,
                _ => AnalyticTense::FutureII,
            };
            // Subjunctive analytic forms: kaikki uses KonjI auxiliaries for
            // Perfekt/Futur (habe gekauft, werde kaufen) and KonjII for
            // Plusquamperfekt (hätte gekauft).
            let mood = match (*m, *t) {
                ("IND", _) => Mood::Indicative,
                (_, "PLPRF") => Mood::KonjunktivII,
                _ => Mood::KonjunktivI,
            };
            Some(Some(verb.analytic(tense, mood, person(p)?, number(n)?)))
        }
        ["V", mood @ ("IND" | "SBJV"), n, p, tense @ ("PRS" | "PST")] => {
            let (t, m) = match (*mood, *tense) {
                ("IND", "PRS") => (Tense::Present, Mood::Indicative),
                ("IND", "PST") => (Tense::Preterite, Mood::Indicative),
                ("SBJV", "PRS") => (Tense::Present, Mood::KonjunktivI),
                ("SBJV", "PST") => (Tense::Present, Mood::KonjunktivII),
                _ => unreachable!(),
            };
            Some(Some(verb.conjugate(t, m, person(p)?, number(n)?)))
        }
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
fn category(features: &str) -> &'static str {
    if features.starts_with("V.PTCP") {
        "participle"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features == "V;AUX" {
        "auxiliary"
    } else if features.starts_with("V;PRF") || features.starts_with("V;PLPRF") {
        "perfect"
    } else if features.starts_with("V;FUT") {
        "future"
    } else if features.starts_with("V;IND") && features.ends_with("PRS") {
        "present"
    } else if features.starts_with("V;IND") && features.ends_with("PST") {
        "preterite"
    } else if features.starts_with("V;SBJV") && features.ends_with("PRS") {
        "konjunktiv1"
    } else {
        "konjunktiv2"
    }
}

#[derive(Default)]
struct Tally {
    total: usize,
    matched: usize,
}

/// CI regression gates (percent). Current UniMorph baseline is
/// 97.5 / 98.9; the gates leave a small margin so noise doesn't flake,
/// but any real regression fails the build. Raise them as accuracy grows.
const MIN_COVERED_PCT: f64 = 97.2;
const MIN_FALLBACK_PCT: f64 = 98.5;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|a| a == "--check");
    let path = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| "data/unimorph/deu".to_string());
    let data = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {path}: {e}. Run scripts/fetch_unimorph.sh first"));

    // (lemma, features) -> accepted gold variants. Forms are trimmed
    // (UniMorph deu has trailing spaces); "—" placeholders mean the form
    // does not exist and leave the variant set empty.
    let mut gold: HashMap<(&str, &str), HashSet<&str>> = HashMap::new();
    for line in data.lines() {
        let mut f = line.split('\t');
        let (Some(lemma), Some(form), Some(features)) = (f.next(), f.next(), f.next()) else {
            continue;
        };
        if !features.starts_with('V') {
            continue;
        }
        let entry = gold.entry((lemma.trim(), features.trim())).or_default();
        let form = form.trim();
        if !form.starts_with('—') {
            entry.insert(form);
        }
    }

    // Adjudication log: mismatches ruled "ours"/"both" are adjudicated-correct.
    let adjudicated: HashSet<(String, String)> = fs::read_to_string("docs/adjudications.tsv")
        .expect("docs/adjudications.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            matches!(f[2], "ours" | "both").then(|| (f[0].to_string(), f[1].to_string()))
        })
        .collect();

    // Corrupt gold entries: if the dataset's own V;NFIN form disagrees with
    // the lemma (einknicken → "knicken"), the whole paradigm is untrustworthy.
    let corrupt: HashSet<&str> = gold
        .iter()
        .filter(|((_, feat), forms)| *feat == "V;NFIN" && !forms.is_empty())
        .filter(|((lemma, _), forms)| !forms.contains(lemma))
        .map(|((lemma, _), _)| *lemma)
        .collect();

    let lemmas: HashSet<&str> = gold
        .keys()
        .map(|(l, _)| *l)
        .filter(|l| !corrupt.contains(l))
        .collect();
    let mut by_category: BTreeMap<(&str, bool), Tally> = BTreeMap::new();
    let mut lemma_errors: HashMap<&str, usize> = HashMap::new();
    let mut mismatches = String::new();
    let mut skipped_bundles: HashSet<&str> = HashSet::new();
    let mut covered_lemmas = 0usize;
    let mut adjudicated_hits = 0usize;

    for lemma in &lemmas {
        let Ok(verb) = Verb::from_infinitive(lemma) else {
            continue;
        };
        let covered = verb.is_lexical();
        if covered {
            covered_lemmas += 1;
        }
        for ((l, features), variants) in gold.iter().filter(|((l, _), _)| l == lemma) {
            let Some(prediction) = generate(&verb, features) else {
                skipped_bundles.insert(features);
                continue;
            };
            let tally = by_category
                .entry((category(features), covered))
                .or_default();
            tally.total += 1;
            let ok = match &prediction {
                Some(p) => variants.contains(p.as_str()),
                // An empty variant set means gold says the form doesn't exist.
                None => variants.is_empty(),
            };
            let ok = if ok {
                true
            } else if adjudicated.contains(&(l.to_string(), features.to_string()))
                || adjudicated.contains(&(l.to_string(), "*".to_string()))
            {
                adjudicated_hits += 1;
                true
            } else {
                false
            };
            if ok {
                tally.matched += 1;
            } else {
                *lemma_errors.entry(l).or_default() += 1;
                let mut sorted: Vec<&&str> = variants.iter().collect();
                sorted.sort();
                let _ = writeln!(
                    mismatches,
                    "{l}\t{features}\t{}\t{}",
                    prediction.as_deref().unwrap_or("<none>"),
                    sorted
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join("|")
                );
            }
        }
    }

    let sum = |covered: bool| {
        by_category
            .iter()
            .filter(|((_, c), _)| *c == covered)
            .fold((0, 0), |(t, m), (_, tally)| {
                (t + tally.total, m + tally.matched)
            })
    };
    let (cov_total, cov_matched) = sum(true);
    let (fb_total, fb_matched) = sum(false);
    let pct = |m: usize, t: usize| {
        if t == 0 {
            100.0
        } else {
            100.0 * m as f64 / t as f64
        }
    };

    println!("== ablaut vs gold: {path} ==");
    println!(
        "lemmas: {} total, {} lexicon-covered",
        lemmas.len(),
        covered_lemmas
    );
    println!("adjudicated forms counted as correct: {adjudicated_hits}");
    println!(
        "corrupt-gold lemmas excluded (NFIN ≠ lemma): {}",
        corrupt.len()
    );
    println!();
    println!(
        "lexicon-covered forms: {cov_matched}/{cov_total} ({:.2}%)",
        pct(cov_matched, cov_total)
    );
    println!(
        "weak-fallback forms:   {fb_matched}/{fb_total} ({:.2}%)",
        pct(fb_matched, fb_total)
    );
    println!();
    println!("{:<14}{:>24}{:>24}", "category", "covered", "fallback");
    for cat in [
        "infinitive",
        "auxiliary",
        "present",
        "preterite",
        "konjunktiv1",
        "konjunktiv2",
        "imperative",
        "participle",
        "perfect",
        "future",
    ] {
        let c = by_category
            .get(&(cat, true))
            .map_or((0, 0), |t| (t.matched, t.total));
        let f = by_category
            .get(&(cat, false))
            .map_or((0, 0), |t| (t.matched, t.total));
        println!(
            "{cat:<14}{:>15}/{:<3}{:>6.2}%{:>13}/{:<6}{:>6.2}%",
            c.0,
            c.1,
            pct(c.0, c.1),
            f.0,
            f.1,
            pct(f.0, f.1)
        );
    }
    if !skipped_bundles.is_empty() {
        let mut s: Vec<&&str> = skipped_bundles.iter().collect();
        s.sort();
        println!("\nskipped feature bundles: {s:?}");
    }

    let mut worst: Vec<(&&str, &usize)> = lemma_errors.iter().collect();
    worst.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("\nworst lemmas (errors):");
    for (lemma, n) in worst.iter().take(15) {
        println!("  {lemma}: {n}");
    }

    fs::write("target/golden_mismatches.tsv", &mismatches).unwrap();
    println!(
        "\n{} mismatching forms written to target/golden_mismatches.tsv",
        mismatches.lines().count()
    );

    if check {
        let covered_pct = pct(cov_matched, cov_total);
        let fallback_pct = pct(fb_matched, fb_total);
        if covered_pct < MIN_COVERED_PCT || fallback_pct < MIN_FALLBACK_PCT {
            eprintln!(
                "REGRESSION: covered {covered_pct:.2}% (min {MIN_COVERED_PCT}) / \
                 fallback {fallback_pct:.2}% (min {MIN_FALLBACK_PCT})"
            );
            std::process::exit(1);
        }
        println!(
            "check passed: covered {covered_pct:.2}% >= {MIN_COVERED_PCT}, \
             fallback {fallback_pct:.2}% >= {MIN_FALLBACK_PCT}"
        );
    }
}
