//! Reverse-lookup gate: for the four languages with productive
//! de-inflection (fra, spa, deu, eng), sample the gold lexicon and
//! check that every sampled form reverses to a set containing its own
//! lemma. The irregular quartet (war→sein, fue→ser/ir, chuaigh→téigh,
//! mindes→minnas) is asserted for the index-only path.
//!
//! Usage: cargo run --release --bin reverse_gate [--check] [--sample N]
//!        [--data-root DIR] [--langs fra,spa,eng]
//!        (default gold: data/fra/lefff.tsv, data/spa/freeling.tsv,
//!         data/deu/kaikki.tsv, data/eng/agid.tsv — the fetch scripts
//!         download them; every Nth distinct (lemma, form) pair is
//!         scored, N = 50 by default.)
//!
//! A (lemma, form) pair is scored only when the engine supports the
//! lemma at all (mirroring the golden harness' lemma-coverage split),
//! the form is a single word, and the slot is not the auxiliary
//! listing. Two rates are reported: the inversion rate over forms the
//! forward engine itself generates (gated at >= 99%) and the raw rate
//! over all sampled gold rows, which additionally counts oracle-only
//! variant spellings the engine never produces and therefore cannot
//! reverse.

use ablaut::reverse::engine_generates;
use ablaut::{conjugate, reverse, Lang};
use std::collections::HashSet;
use std::fs;

const MIN_PCT: f64 = 99.0;

struct GateLang {
    code: &'static str,
    lang: Lang,
    gold: &'static str,
}

const LANGS: [GateLang; 4] = [
    GateLang {
        code: "fra",
        lang: Lang::Fra,
        gold: "data/fra/lefff.tsv",
    },
    GateLang {
        code: "spa",
        lang: Lang::Spa,
        gold: "data/spa/freeling.tsv",
    },
    GateLang {
        code: "deu",
        lang: Lang::Deu,
        gold: "data/deu/kaikki.tsv",
    },
    GateLang {
        code: "eng",
        lang: Lang::Eng,
        gold: "data/eng/agid.tsv",
    },
];

fn irregular_quartet() {
    let cases: [(&str, Lang, &[&str]); 4] = [
        ("war", Lang::Deu, &["sein"]),
        ("fue", Lang::Spa, &["ser", "ir"]),
        ("chuaigh", Lang::Gle, &["téigh"]),
        ("mindes", Lang::Swe, &["minnas"]),
    ];
    for (form, lang, lemmas) in cases {
        let got: Vec<String> = reverse(form, lang)
            .into_iter()
            .map(|m| m.infinitive)
            .collect();
        for lemma in lemmas {
            assert!(
                got.iter().any(|g| g == lemma),
                "index-only regression: {form} did not reverse to {lemma} (got {got:?})"
            );
        }
    }
    println!("irregular quartet (index-only languages): ok\n");
}

#[allow(clippy::cast_precision_loss)]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|a| a == "--check");
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let sample: usize = flag("--sample").map_or(50, |v| v.parse().expect("--sample N"));
    let root = flag("--data-root").unwrap_or_default();
    let langs = flag("--langs");

    irregular_quartet();

    let mut failed = false;
    for gl in LANGS {
        if let Some(l) = &langs {
            if !l.split(',').any(|c| c == gl.code) {
                continue;
            }
        }
        let path = if root.is_empty() {
            gl.gold.to_string()
        } else {
            format!("{root}/{}", gl.gold)
        };
        let data = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {path}: {e}. Run the fetch scripts first"));
        let mut seen: HashSet<(String, String)> = HashSet::new();
        let mut supported: HashSet<String> = HashSet::new();
        let mut unsupported: HashSet<String> = HashSet::new();
        let (mut total, mut hit) = (0usize, 0usize);
        let (mut raw_total, mut raw_hit) = (0usize, 0usize);
        let mut failures: Vec<(String, String)> = Vec::new();
        for line in data.lines() {
            let mut f = line.split('\t');
            let (Some(lemma), Some(form), Some(features)) = (f.next(), f.next(), f.next()) else {
                continue;
            };
            let (lemma, form, features) = (lemma.trim(), form.trim(), features.trim());
            if !features.starts_with('V') || features == "V;AUX" {
                continue;
            }
            if lemma.contains(char::is_whitespace)
                || lemma.contains(['\'', '\u{2019}'])
                || form.contains(char::is_whitespace)
                || form.contains(['\'', '\u{2019}'])
                || form.is_empty()
            {
                continue;
            }
            if !seen.insert((lemma.to_string(), form.to_string())) {
                continue;
            }
            if seen.len() % sample != 0 {
                continue;
            }
            if unsupported.contains(lemma) {
                continue;
            }
            if !supported.contains(lemma) {
                if conjugate(lemma, gl.lang).is_ok() {
                    supported.insert(lemma.to_string());
                } else {
                    unsupported.insert(lemma.to_string());
                    continue;
                }
            }
            let reachable = engine_generates(lemma, form, gl.lang);
            raw_total += 1;
            if reachable {
                total += 1;
            }
            let want = lemma.to_lowercase();
            if reverse(form, gl.lang)
                .iter()
                .any(|m| m.infinitive.to_lowercase() == want)
            {
                raw_hit += 1;
                if reachable {
                    hit += 1;
                }
            } else if reachable {
                failures.push((lemma.to_string(), form.to_string()));
            }
        }
        let pct = if total == 0 {
            100.0
        } else {
            100.0 * hit as f64 / total as f64
        };
        let raw_pct = if raw_total == 0 {
            100.0
        } else {
            100.0 * raw_hit as f64 / raw_total as f64
        };
        println!(
            "{}: {hit}/{total} engine-generatable sampled forms reverse to their lemma \
             ({pct:.2}%); raw incl. oracle-only variant spellings: {raw_hit}/{raw_total} \
             ({raw_pct:.2}%); {} unsupported lemmas skipped",
            gl.code,
            unsupported.len()
        );
        for (lemma, form) in failures.iter().take(30) {
            println!("  MISS {form} !-> {lemma}");
        }
        if failures.len() > 30 {
            println!("  ... and {} more", failures.len() - 30);
        }
        if pct < MIN_PCT {
            eprintln!(
                "REGRESSION: {} reverse rate {pct:.2}% < {MIN_PCT}%",
                gl.code
            );
            failed = true;
        }
    }
    if check && failed {
        std::process::exit(1);
    }
    if check {
        println!("\ncheck passed: all languages >= {MIN_PCT}%");
    }
}
