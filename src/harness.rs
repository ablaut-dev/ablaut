//! The shared golden-test harness. Every language's `golden_*`
//! binary is a thin adapter: it supplies the verb parser, the
//! feature-bundle → forms generator, the category mapping and the
//! gates; everything else — gold parsing, the two-oracle agreement
//! intersection, scoring, reporting, regression gates — lives here
//! once instead of thirteen times.
//!
//! Not part of the public API.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;

/// Accepted gold variants per feature bundle, per lemma.
pub type Gold = HashMap<String, HashMap<String, HashSet<String>>>;

/// A language's wiring into the shared harness.
pub struct Spec<'a, V> {
    /// ISO code, for the report header (`ablaut::fra`).
    pub lang: &'a str,
    /// Default gold paths: [primary/CI oracle, kaikki]. The second is
    /// optional at runtime — with one file it is scored directly.
    pub default_paths: [&'a str; 2],
    /// Adjudication log (rulings ours/both count as correct).
    pub adjudications: &'a str,
    /// Where mismatches are written.
    pub mismatches: &'a str,
    /// Coarse categories for the breakdown, in display order.
    pub categories: &'a [&'a str],
    /// CI regression gates (percent).
    pub min_form_pct: f64,
    pub min_lemma_coverage_pct: f64,
    /// Features imported into the gold from the second oracle even
    /// without agreement (French V;AUX: Lefff has no auxiliary
    /// column).
    pub carry_features: &'a [&'a str],
    /// Parse a lemma; None skips it (counted against coverage).
    pub parse: fn(&str) -> Option<V>,
    /// Engine variants for a feature bundle; None = unsupported slot.
    pub generate: fn(&V, &str) -> Option<Vec<String>>,
    /// Coarse category of a feature bundle.
    pub category: fn(&str) -> &'static str,
}

pub fn parse_gold(data: &str) -> Gold {
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
/// overlapping variant sets, unioned so either oracle's spelling
/// counts; disjoint slots are the adjudication corpus and dropped.
fn agree(a: Gold, b: &Gold, carry: &[&str]) -> (Gold, usize) {
    let mut dropped = 0;
    let mut out: Gold = HashMap::new();
    for feature in carry {
        for (lemma, feats) in b {
            if let Some(v) = feats.get(*feature) {
                out.entry(lemma.clone())
                    .or_default()
                    .insert((*feature).to_string(), v.clone());
            }
        }
    }
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

fn load_adjudications(path: &str) -> HashSet<(String, String)> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            matches!(f[2], "ours" | "both").then(|| (f[0].to_string(), f[1].to_string()))
        })
        .collect()
}

fn score<V>(spec: &Spec<V>, gold: &Gold, adjudicated: &HashSet<(String, String)>) -> Scores {
    let mut s = Scores::default();
    for (lemma, feats) in gold {
        s.total_lemmas += 1;
        let Some(verb) = (spec.parse)(lemma) else {
            continue;
        };
        s.supported_lemmas += 1;
        for (features, variants) in feats {
            let Some(forms) = (spec.generate)(&verb, features) else {
                continue;
            };
            let tally = s.by_category.entry((spec.category)(features)).or_default();
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

fn report<V>(spec: &Spec<V>, paths: &[String], s: &Scores, dropped: usize) {
    let (total, matched) = s.totals();
    println!("== ablaut::{} vs gold: {} ==", spec.lang, paths.join(" ∩ "));
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
    for cat in spec.categories {
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

    fs::write(spec.mismatches, &s.mismatches).unwrap();
    println!(
        "\n{} mismatching forms written to {}",
        s.mismatches.lines().count(),
        spec.mismatches
    );
}

fn check_gates<V>(spec: &Spec<V>, s: &Scores) {
    let (total, matched) = s.totals();
    let form_pct = pct(matched, total);
    let coverage_pct = pct(s.supported_lemmas, s.total_lemmas);
    if form_pct < spec.min_form_pct || coverage_pct < spec.min_lemma_coverage_pct {
        eprintln!(
            "REGRESSION: forms {form_pct:.2}% (min {}) / lemma coverage {coverage_pct:.2}% (min {})",
            spec.min_form_pct, spec.min_lemma_coverage_pct
        );
        std::process::exit(1);
    }
    println!(
        "check passed: forms {form_pct:.2}% >= {}, lemma coverage {coverage_pct:.2}% >= {}",
        spec.min_form_pct, spec.min_lemma_coverage_pct
    );
}

/// The whole harness: parse args, load golds, score, report, gate.
pub fn run<V>(spec: Spec<V>) {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|a| a == "--check");
    let mut paths: Vec<String> = args.into_iter().filter(|a| !a.starts_with("--")).collect();
    if paths.is_empty() {
        paths.push(spec.default_paths[0].to_string());
        if fs::metadata(spec.default_paths[1]).is_ok() {
            paths.push(spec.default_paths[1].to_string());
        }
    }
    let mut golds: Vec<Gold> = paths
        .iter()
        .map(|p| {
            let data = fs::read_to_string(p)
                .unwrap_or_else(|e| panic!("cannot read {p}: {e}. Run the fetch scripts first"));
            parse_gold(&data)
        })
        .collect();
    let first = golds.remove(0);
    let (gold, dropped) = match golds.pop() {
        Some(second) => agree(first, &second, spec.carry_features),
        None => (first, 0),
    };
    let scores = score(&spec, &gold, &load_adjudications(spec.adjudications));
    report(&spec, &paths, &scores, dropped);
    if check {
        check_gates(&spec, &scores);
    }
}
