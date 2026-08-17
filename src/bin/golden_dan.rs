//! Danish golden-test harness: diff the danine against the agreement
//! of the two Danish oracles (COR and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_dan [gold.tsv ...] [--check]
//!        (default: data/dan/cor-verbs.tsv data/dan/kaikki.tsv)
//!
//! With two gold files, only slots the oracles agree on are scored
//! (variant sets unioned); with one file, that file is scored directly
//! (the CI path: AGID alone).

use ablaut::dan::{Error, Slot, Verb};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;

/// CI regression gates (percent), measured against AGID alone.
const MIN_FORM_PCT: f64 = 99.8;
const MIN_LEMMA_COVERAGE_PCT: f64 = 99.5;

const CATEGORIES: [&str; 6] = [
    "infinitive",
    "present",
    "past",
    "imperative",
    "participle",
    "passive",
];

type Gold = HashMap<String, HashMap<String, HashSet<String>>>;

/// (slot, passive?) for a feature bundle.
fn slot(features: &str) -> Option<(Slot, bool)> {
    match features {
        "V;NFIN;ACT" => Some((Slot::Infinitive, false)),
        "V;NFIN;PASS" => Some((Slot::Infinitive, true)),
        "V;PRS;ACT" => Some((Slot::Present, false)),
        "V;PRS;PASS" => Some((Slot::Present, true)),
        "V;PST;ACT" => Some((Slot::Past, false)),
        "V;PST;PASS" => Some((Slot::Past, true)),
        "V;IMP" => Some((Slot::Imperative, false)),
        "V.PTCP;PRS" => Some((Slot::PresentParticiple, false)),
        "V.PTCP;PST" => Some((Slot::PastParticiple, false)),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features.ends_with("PASS") {
        "passive"
    } else if features.starts_with("V;NFIN") {
        "infinitive"
    } else if features.starts_with("V;PRS") {
        "present"
    } else if features.starts_with("V;PST") {
        "past"
    } else if features.starts_with("V;IMP") {
        "imperative"
    } else {
        "participle"
    }
}

fn load_adjudications() -> HashSet<(String, String)> {
    fs::read_to_string("docs/dan/adjudications.tsv")
        .expect("docs/dan/adjudications.tsv")
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
            let Some((sl, pass)) = slot(features) else {
                continue;
            };
            let Some(form) = (if pass {
                verb.passive(sl)
            } else {
                verb.active(sl)
            }) else {
                continue;
            };
            let tally = s.by_category.entry(category(features)).or_default();
            tally.total += 1;
            let ok = variants.contains(&form)
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
                    "{lemma}\t{features}\t{form}\t{}",
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
    println!("== ablaut::dan vs gold: {} ==", paths.join(" ∩ "));
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
    println!("{:<22}{:>20}", "category", "matched");
    for cat in CATEGORIES {
        let t = s
            .by_category
            .get(cat)
            .map_or((0, 0), |t| (t.matched, t.total));
        println!("{cat:<22}{:>12}/{:<8}{:>7.2}%", t.0, t.1, pct(t.0, t.1));
    }

    let mut worst: Vec<(&String, &usize)> = s.lemma_errors.iter().collect();
    worst.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("\nworst lemmas (errors):");
    for (lemma, n) in worst.iter().take(15) {
        println!("  {lemma}: {n}");
    }

    fs::write("target/golden_dan_mismatches.tsv", &s.mismatches).unwrap();
    println!(
        "\n{} mismatching forms written to target/golden_dan_mismatches.tsv",
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
        paths.push("data/dan/cor-verbs.tsv".to_string());
        if fs::metadata("data/dan/kaikki.tsv").is_ok() {
            paths.push("data/dan/kaikki.tsv".to_string());
        }
    }
    let mut golds: Vec<Gold> = paths
        .iter()
        .map(|p| {
            let data = fs::read_to_string(p).unwrap_or_else(|e| {
                panic!("cannot read {p}: {e}. Run the scripts/dan/fetch_*.sh scripts first")
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
