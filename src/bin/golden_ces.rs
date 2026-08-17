//! Czech golden-test harness: diff the engine against the
//! agreement of the two Czech oracles (MorfFlex CZ and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_ces [gold.tsv ...] [--check]
//!        (default: data/ces/morfflex.tsv data/ces/kaikki.tsv —
//!         see `scripts/ces/fetch_morfflex.sh`, `scripts/ces/fetch_kaikki.sh`)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); disagreements are the adjudication corpus.
//! With one file, that file is scored directly (the CI path).

use ablaut::ces::{Error, Gender, Number, Person, TransgressiveSlot, Verb};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;

/// CI regression gates (percent), measured against dexonline alone.
const MIN_FORM_PCT: f64 = 99.9;
const MIN_LEMMA_COVERAGE_PCT: f64 = 99.5;

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "present",
    "imperative",
    "l-participle",
    "passive-participle",
    "transgressive",
];

/// Accepted gold variants per feature bundle, per lemma.
type Gold = HashMap<String, HashMap<String, HashSet<String>>>;

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
    let gender = |g: &str| match g {
        "MA" => Some(Gender::MasculineAnimate),
        "MI" => Some(Gender::MasculineInanimate),
        "F" => Some(Gender::Feminine),
        "N" => Some(Gender::Neuter),
        _ => None,
    };
    match f.as_slice() {
        ["V", "NFIN"] => Some(vec![verb.infinitive().to_string()]),
        ["V", "IND", "PRS", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => Some(vec![verb.present(p, n)]),
            _ => None,
        },
        ["V", "IMP", n, p] => match (person(p), number(n)) {
            (Some(p), Some(n)) => verb.imperative(p, n).map(|f| vec![f]),
            _ => None,
        },
        ["V.PTCP", "PST", g, n] => match (gender(g), number(n)) {
            (Some(g), Some(n)) => Some(vec![verb.past_participle(g, n)]),
            _ => None,
        },
        ["V.PTCP", "PASS", g, n] => match (gender(g), number(n)) {
            (Some(g), Some(n)) => verb.passive_participle(g, n).map(|f| vec![f]),
            _ => None,
        },
        ["V", "CVB", "PRS", s] => {
            let slot = match *s {
                "M" => TransgressiveSlot::Masculine,
                "FN" => TransgressiveSlot::FeminineNeuter,
                "PL" => TransgressiveSlot::Plural,
                _ => return None,
            };
            verb.transgressive(slot).map(|f| vec![f])
        }
        _ => None,
    }
}

/// Coarse category for the per-slot breakdown.
fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "infinitive"
    } else if features.starts_with("V;IND;PRS") {
        "present"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else if features.starts_with("V.PTCP;PST") {
        "l-participle"
    } else if features.starts_with("V.PTCP;PASS") {
        "passive-participle"
    } else {
        "transgressive"
    }
}

/// Mismatches ruled "ours"/"both" in the adjudication log count as
/// correct. A "*" in the features column covers the whole paradigm.
fn load_adjudications() -> HashSet<(String, String)> {
    fs::read_to_string("docs/ces/adjudications.tsv")
        .expect("docs/ces/adjudications.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            matches!(f[2], "ours" | "both").then(|| (f[0].to_string(), f[1].to_string()))
        })
        .collect()
}

fn parse_gold(data: &str) -> Gold {
    let mut gold: Gold = HashMap::new();
    for line in data.lines() {
        let mut f = line.split('\t');
        let (Some(lemma), Some(form), Some(features)) = (f.next(), f.next(), f.next()) else {
            continue;
        };
        if !features.starts_with('V') {
            continue;
        }
        gold.entry(lemma.trim().to_string())
            .or_default()
            .entry(features.trim().to_string())
            .or_default()
            .insert(form.trim().to_string());
    }
    gold
}

/// Intersect two oracles into agreement gold: slots both cover with
/// overlapping variant sets, unioned so either oracle's form counts.
fn agree(a: Gold, b: &Gold) -> (Gold, usize) {
    let mut dropped = 0;
    let mut out: Gold = HashMap::new();
    for (lemma, feats) in a {
        let Some(bfeats) = b.get(&lemma) else {
            continue;
        };
        for (features, mut variants) in feats {
            let Some(bvariants) = bfeats.get(&features) else {
                continue;
            };
            if variants.is_disjoint(bvariants) {
                dropped += 1;
                continue;
            }
            variants.extend(bvariants.iter().cloned());
            out.entry(lemma.clone())
                .or_default()
                .insert(features, variants);
        }
    }
    (out, dropped)
}

#[derive(Default)]
struct Tally {
    total: usize,
    matched: usize,
}

#[derive(Default)]
struct Scores {
    by_category: BTreeMap<&'static str, Tally>,
    lemma_errors: HashMap<String, usize>,
    mismatches: String,
    supported_lemmas: usize,
    total_lemmas: usize,
    adjudicated_hits: usize,
}

impl Scores {
    fn totals(&self) -> (usize, usize) {
        self.by_category
            .values()
            .fold((0, 0), |(t, m), tally| (t + tally.total, m + tally.matched))
    }
}

fn pct(matched: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        #[allow(clippy::cast_precision_loss)]
        {
            100.0 * matched as f64 / total as f64
        }
    }
}

fn score(gold: &Gold, adjudicated: &HashSet<(String, String)>) -> Scores {
    let mut s = Scores::default();
    for (lemma, feats) in gold {
        s.total_lemmas += 1;
        let verb = match Verb::from_infinitive(lemma) {
            Ok(v) => v,
            Err(Error::NotAVerb) => continue,
        };
        s.supported_lemmas += 1;
        for (features, variants) in feats {
            let Some(forms) = generate(&verb, features) else {
                continue;
            };
            let tally = s.by_category.entry(category(features)).or_default();
            tally.total += 1;
            let ok = forms.iter().any(|f| variants.contains(f))
                || if adjudicated.contains(&(lemma.clone(), features.clone()))
                    || adjudicated.contains(&(lemma.clone(), "*".to_string()))
                {
                    s.adjudicated_hits += 1;
                    true
                } else {
                    false
                };
            if ok {
                tally.matched += 1;
            } else {
                *s.lemma_errors.entry(lemma.clone()).or_default() += 1;
                let mut sorted: Vec<&String> = variants.iter().collect();
                sorted.sort();
                let _ = writeln!(
                    s.mismatches,
                    "{lemma}\t{features}\t{}\t{}",
                    forms.join("|"),
                    sorted
                        .iter()
                        .map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join("|")
                );
            }
        }
    }
    s
}

fn report(paths: &[String], s: &Scores, dropped: usize) {
    let (total, matched) = s.totals();
    println!("== ablaut::ces vs gold: {} ==", paths.join(" ∩ "));
    println!(
        "lemmas: {} in gold, {} supported ({:.2}%)",
        s.total_lemmas,
        s.supported_lemmas,
        pct(s.supported_lemmas, s.total_lemmas)
    );
    if s.adjudicated_hits > 0 {
        println!(
            "adjudicated forms counted as correct: {}",
            s.adjudicated_hits
        );
    }
    if dropped > 0 {
        println!("oracle-disagreement slots excluded from gold: {dropped}");
    }
    println!();
    println!("forms: {matched}/{total} ({:.2}%)", pct(matched, total));
    println!();
    println!("{:<16}{:>20}", "category", "matched");
    for cat in CATEGORIES {
        let t = s
            .by_category
            .get(cat)
            .map_or((0, 0), |t| (t.matched, t.total));
        println!("{cat:<16}{:>12}/{:<8}{:>7.2}%", t.0, t.1, pct(t.0, t.1));
    }

    let mut worst: Vec<(&String, &usize)> = s.lemma_errors.iter().collect();
    worst.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("\nworst lemmas (errors):");
    for (lemma, n) in worst.iter().take(15) {
        println!("  {lemma}: {n}");
    }

    fs::write("target/golden_ces_mismatches.tsv", &s.mismatches).unwrap();
    println!(
        "\n{} mismatching forms written to target/golden_ces_mismatches.tsv",
        s.mismatches.lines().count()
    );
}

fn check_gates(s: &Scores) {
    let (total, matched) = s.totals();
    let form_pct = pct(matched, total);
    let coverage_pct = pct(s.supported_lemmas, s.total_lemmas);
    if form_pct < MIN_FORM_PCT || coverage_pct < MIN_LEMMA_COVERAGE_PCT {
        eprintln!(
            "REGRESSION: forms {form_pct:.2}% (min {MIN_FORM_PCT}) / \
             lemma coverage {coverage_pct:.2}% (min {MIN_LEMMA_COVERAGE_PCT})"
        );
        std::process::exit(1);
    }
    println!(
        "check passed: forms {form_pct:.2}% >= {MIN_FORM_PCT}, \
         lemma coverage {coverage_pct:.2}% >= {MIN_LEMMA_COVERAGE_PCT}"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|a| a == "--check");
    let mut paths: Vec<String> = args.into_iter().filter(|a| !a.starts_with("--")).collect();
    if paths.is_empty() {
        paths.push("data/ces/morfflex.tsv".to_string());
        if fs::metadata("data/ces/kaikki.tsv").is_ok() {
            paths.push("data/ces/kaikki.tsv".to_string());
        }
    }
    let mut golds: Vec<Gold> = paths
        .iter()
        .map(|p| {
            let data = fs::read_to_string(p).unwrap_or_else(|e| {
                panic!("cannot read {p}: {e}. Run the scripts/ces/fetch_*.sh scripts first")
            });
            parse_gold(&data)
        })
        .collect();
    let first = golds.remove(0);
    let (gold, dropped) = match golds.pop() {
        Some(second) => agree(first, &second),
        None => (first, 0),
    };

    let scores = score(&gold, &load_adjudications());
    report(&paths, &scores, dropped);
    if check {
        check_gates(&scores);
    }
}
