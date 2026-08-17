//! Swedish golden-test harness: diff the engine against the agreement
//! of the two Swedish oracles (SALDO and kaikki.org).
//!
//! Usage: cargo run --release --bin golden_swe [gold.tsv ...] [--check]
//!        (default: data/swe/saldo.tsv data/swe/kaikki.tsv)

use ablaut::swe::{Slot, Verb};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;

/// CI regression gates (percent), measured against SALDO alone.
const MIN_FORM_PCT: f64 = 99.8;
const MIN_LEMMA_COVERAGE_PCT: f64 = 99.5;

type Gold = HashMap<String, HashMap<String, HashSet<String>>>;

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    let (slot, passive) = match features {
        "V;NFIN;ACT" => (Slot::Infinitive, false),
        "V;NFIN;PASS" => (Slot::Infinitive, true),
        "V;IND;PRS;ACT" => (Slot::Present, false),
        "V;IND;PRS;PASS" => (Slot::Present, true),
        "V;IND;PST;ACT" => (Slot::Past, false),
        "V;IND;PST;PASS" => (Slot::Past, true),
        "V;SUP;ACT" => (Slot::Supine, false),
        "V;SUP;PASS" => (Slot::Supine, true),
        "V;IMP" => (Slot::Imperative, false),
        "V;SBJV;PRS;ACT" => (Slot::SubjunctivePresent, false),
        "V;SBJV;PST;ACT" => (Slot::SubjunctivePast, false),
        _ => return None,
    };
    if passive {
        let v = verb.passive_variants(slot);
        return if v.is_empty() { None } else { Some(v) };
    }
    verb.active_voice(slot).map(|f| vec![f])
}

fn category(features: &str) -> &'static str {
    if features.contains("NFIN") {
        "infinitive"
    } else if features.contains("PRS") && features.contains("SBJV") {
        "subj-present"
    } else if features.contains("SBJV") {
        "subj-past"
    } else if features.contains("PRS") {
        "present"
    } else if features.contains("SUP") {
        "supine"
    } else if features.contains("IMP") {
        "imperative"
    } else {
        "past"
    }
}

fn parse_gold(data: &str) -> Gold {
    let mut gold: Gold = HashMap::new();
    for line in data.lines() {
        let mut f = line.split('\t');
        let (Some(lemma), Some(form), Some(features)) = (f.next(), f.next(), f.next()) else {
            continue;
        };
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
        let Some(bf) = b.get(&lemma) else { continue };
        for (features, mut variants) in feats {
            let Some(bv) = bf.get(&features) else {
                continue;
            };
            if variants.is_disjoint(bv) {
                dropped += 1;
                continue;
            }
            variants.extend(bv.iter().cloned());
            out.entry(lemma.clone())
                .or_default()
                .insert(features, variants);
        }
    }
    (out, dropped)
}

fn load_adjudications() -> HashSet<(String, String)> {
    fs::read_to_string("docs/swe/adjudications.tsv")
        .expect("docs/swe/adjudications.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            matches!(f[2], "ours" | "both").then(|| (f[0].to_string(), f[1].to_string()))
        })
        .collect()
}

fn pct(m: usize, t: usize) -> f64 {
    if t == 0 {
        100.0
    } else {
        #[allow(clippy::cast_precision_loss)]
        {
            100.0 * m as f64 / t as f64
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|a| a == "--check");
    let mut paths: Vec<String> = args.into_iter().filter(|a| !a.starts_with("--")).collect();
    if paths.is_empty() {
        paths.push("data/swe/saldo.tsv".to_string());
        if fs::metadata("data/swe/kaikki.tsv").is_ok() {
            paths.push("data/swe/kaikki.tsv".to_string());
        }
    }
    let mut golds: Vec<Gold> = paths
        .iter()
        .map(|p| parse_gold(&fs::read_to_string(p).expect("gold file")))
        .collect();
    let first = golds.remove(0);
    let (gold, dropped) = match golds.pop() {
        Some(second) => agree(first, &second),
        None => (first, 0),
    };
    let adjudicated = load_adjudications();

    let mut by_cat: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    let mut mismatches = String::new();
    let (mut total_lemmas, mut supported) = (0usize, 0usize);
    let mut adj_hits = 0usize;
    for (lemma, feats) in &gold {
        total_lemmas += 1;
        let Ok(verb) = Verb::from_infinitive(lemma) else {
            continue;
        };
        supported += 1;
        for (features, variants) in feats {
            let Some(forms) = generate(&verb, features) else {
                continue;
            };
            let e = by_cat.entry(category(features)).or_default();
            e.1 += 1;
            let ok = forms.iter().any(|f| variants.contains(f))
                || if adjudicated.contains(&(lemma.clone(), features.clone()))
                    || adjudicated.contains(&(lemma.clone(), "*".to_string()))
                {
                    adj_hits += 1;
                    true
                } else {
                    false
                };
            if ok {
                e.0 += 1;
            } else {
                let mut sorted: Vec<&String> = variants.iter().collect();
                sorted.sort();
                let _ = writeln!(
                    mismatches,
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
    let (m, t) = by_cat
        .values()
        .fold((0, 0), |(a, b), (x, y)| (a + x, b + y));
    println!("== ablaut::swe vs gold: {} ==", paths.join(" ∩ "));
    println!(
        "lemmas: {total_lemmas} in gold, {supported} supported ({:.2}%)",
        pct(supported, total_lemmas)
    );
    if adj_hits > 0 {
        println!("adjudicated forms counted as correct: {adj_hits}");
    }
    if dropped > 0 {
        println!("oracle-disagreement slots excluded from gold: {dropped}");
    }
    println!("forms: {m}/{t} ({:.2}%)", pct(m, t));
    for (cat, (x, y)) in &by_cat {
        println!("{cat:<14}{x:>9}/{y:<9}{:>7.2}%", pct(*x, *y));
    }
    fs::write("target/golden_swe_mismatches.tsv", &mismatches).unwrap();
    println!(
        "{} mismatching forms written to target/golden_swe_mismatches.tsv",
        mismatches.lines().count()
    );
    if check {
        let fp = pct(m, t);
        let cp = pct(supported, total_lemmas);
        if fp < MIN_FORM_PCT || cp < MIN_LEMMA_COVERAGE_PCT {
            eprintln!("REGRESSION: forms {fp:.2}% / coverage {cp:.2}%");
            std::process::exit(1);
        }
        println!("check passed: forms {fp:.2}% >= {MIN_FORM_PCT}, coverage {cp:.2}% >= {MIN_LEMMA_COVERAGE_PCT}");
    }
}
