//! Golden-test harness: diff ablaut's output against the `UniMorph` `deu`
//! dataset (<https://github.com/unimorph/deu>).
//!
//! Usage: cargo run --release --bin golden [path-to-gold-tsv] [--check]
//!        (default: data/unimorph/deu — see `scripts/fetch_unimorph_deu.sh`)
//!
//! A prediction counts as a match if it is among the gold variants for that
//! (lemma, feature bundle). Mismatches are written to
//! `target/golden_mismatches.tsv` for adjudication. Accuracy is reported
//! separately for lexicon-covered lemmas (where we claim correctness) and
//! weak-fallback lemmas (where unlisted strong verbs are expected failures).

use ablaut::{AnalyticTense, Auxiliary, Mood, Number, Person, Tense, Verb};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;

/// CI regression gates (percent). Current `UniMorph` baseline is
/// 97.7 / 99.55; the gates leave a small margin so noise doesn't flake,
/// but any real regression fails the build. Raise them as accuracy grows.
const MIN_COVERED_PCT: f64 = 97.4;
const MIN_FALLBACK_PCT: f64 = 99.3;

const CATEGORIES: [&str; 10] = [
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
];

/// Accepted gold variants per feature bundle, per lemma.
type Gold<'a> = HashMap<&'a str, HashMap<&'a str, HashSet<&'a str>>>;

enum Prediction {
    /// The harness does not model this feature bundle.
    Unsupported,
    /// The library says the form does not exist (modal imperatives).
    NoForm,
    Form(String),
}

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

/// Map a `UniMorph` feature bundle to our API.
fn generate(verb: &Verb, features: &str) -> Prediction {
    let f: Vec<&str> = features.split(';').collect();
    let form = match f.as_slice() {
        ["V", "NFIN"] => Some(verb.infinitive().to_string()),
        ["V", "NFIN", "LGSPEC01"] => Some(verb.zu_infinitive()),
        ["V", "AUX"] => Some(
            match verb.auxiliary() {
                Auxiliary::Haben => "haben",
                Auxiliary::Sein => "sein",
            }
            .to_string(),
        ),
        ["V.PTCP", "PRS"] => Some(verb.present_participle()),
        ["V.PTCP", "PST"] => Some(verb.past_participle()),
        ["V", "IMP", n, "2"] => match number(n) {
            Some(n) => {
                return verb
                    .imperative(n)
                    .map_or(Prediction::NoForm, Prediction::Form)
            }
            None => None,
        },
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
            match (person(p), number(n)) {
                (Some(p), Some(n)) => Some(verb.analytic(tense, mood, p, n)),
                _ => None,
            }
        }
        ["V", mood @ ("IND" | "SBJV"), n, p, tense @ ("PRS" | "PST")] => {
            let (t, m) = match (*mood, *tense) {
                ("IND", "PRS") => (Tense::Present, Mood::Indicative),
                ("IND", "PST") => (Tense::Preterite, Mood::Indicative),
                ("SBJV", "PRS") => (Tense::Present, Mood::KonjunktivI),
                _ => (Tense::Present, Mood::KonjunktivII),
            };
            match (person(p), number(n)) {
                (Some(p), Some(n)) => Some(verb.conjugate(t, m, p, n)),
                _ => None,
            }
        }
        _ => None,
    };
    form.map_or(Prediction::Unsupported, Prediction::Form)
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

/// Parse the gold TSV. Forms are trimmed (`UniMorph` deu has trailing
/// spaces); "—" placeholders mean the form does not exist and leave the
/// variant set empty.
fn parse_gold(data: &str) -> Gold<'_> {
    let mut gold: Gold = HashMap::new();
    for line in data.lines() {
        let mut f = line.split('\t');
        let (Some(lemma), Some(form), Some(features)) = (f.next(), f.next(), f.next()) else {
            continue;
        };
        if !features.starts_with('V') {
            continue;
        }
        let entry = gold
            .entry(lemma.trim())
            .or_default()
            .entry(features.trim())
            .or_default();
        let form = form.trim();
        if !form.starts_with('—') {
            entry.insert(form);
        }
    }
    gold
}

/// Mismatches ruled "ours"/"both" in the adjudication log count as correct.
/// A "*" in the features column applies to the lemma's whole paradigm.
fn load_adjudications() -> HashSet<(String, String)> {
    fs::read_to_string("docs/adjudications-deu.tsv")
        .expect("docs/adjudications-deu.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            matches!(f[2], "ours" | "both").then(|| (f[0].to_string(), f[1].to_string()))
        })
        .collect()
}

/// Lemmas whose own V;NFIN row disagrees with the lemma (einknicken →
/// "knicken"): the whole paradigm is untrustworthy and is excluded.
fn corrupt_lemmas<'a>(gold: &Gold<'a>) -> HashSet<&'a str> {
    gold.iter()
        .filter_map(|(lemma, feats)| {
            let nfin = feats.get("V;NFIN")?;
            (!nfin.is_empty() && !nfin.contains(lemma)).then_some(*lemma)
        })
        .collect()
}

#[derive(Default)]
struct Tally {
    total: usize,
    matched: usize,
}

#[derive(Default)]
struct Scores<'a> {
    by_category: BTreeMap<(&'static str, bool), Tally>,
    lemma_errors: HashMap<&'a str, usize>,
    mismatches: String,
    skipped_bundles: HashSet<&'a str>,
    covered_lemmas: usize,
    scored_lemmas: usize,
    adjudicated_hits: usize,
}

impl Scores<'_> {
    fn totals(&self, covered: bool) -> (usize, usize) {
        self.by_category
            .iter()
            .filter(|((_, c), _)| *c == covered)
            .fold((0, 0), |(t, m), (_, tally)| {
                (t + tally.total, m + tally.matched)
            })
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

fn score<'a>(
    gold: &'a Gold<'a>,
    corrupt: &HashSet<&str>,
    adjudicated: &HashSet<(String, String)>,
) -> Scores<'a> {
    let mut s = Scores::default();
    for (lemma, feats) in gold {
        if corrupt.contains(lemma) {
            continue;
        }
        let Ok(verb) = Verb::from_infinitive(lemma) else {
            continue;
        };
        s.scored_lemmas += 1;
        let covered = verb.is_lexical();
        if covered {
            s.covered_lemmas += 1;
        }
        for (features, variants) in feats {
            let prediction = generate(&verb, features);
            if matches!(prediction, Prediction::Unsupported) {
                s.skipped_bundles.insert(features);
                continue;
            }
            let tally = s
                .by_category
                .entry((category(features), covered))
                .or_default();
            tally.total += 1;
            let ok = match &prediction {
                Prediction::Form(p) => variants.contains(p.as_str()),
                // An empty variant set means gold says the form doesn't exist.
                Prediction::NoForm => variants.is_empty(),
                Prediction::Unsupported => unreachable!(),
            };
            let ok = ok
                || if adjudicated.contains(&((*lemma).to_string(), (*features).to_string()))
                    || adjudicated.contains(&((*lemma).to_string(), "*".to_string()))
                {
                    s.adjudicated_hits += 1;
                    true
                } else {
                    false
                };
            if ok {
                tally.matched += 1;
            } else {
                *s.lemma_errors.entry(lemma).or_default() += 1;
                let mut sorted: Vec<&&str> = variants.iter().collect();
                sorted.sort();
                let shown = match &prediction {
                    Prediction::Form(p) => p.as_str(),
                    _ => "<none>",
                };
                let _ = writeln!(
                    s.mismatches,
                    "{lemma}\t{features}\t{shown}\t{}",
                    sorted
                        .iter()
                        .map(|v| (**v).to_string())
                        .collect::<Vec<_>>()
                        .join("|")
                );
            }
        }
    }
    s
}

fn report(path: &str, s: &Scores, corrupt_count: usize) {
    let (cov_total, cov_matched) = s.totals(true);
    let (fb_total, fb_matched) = s.totals(false);

    println!("== ablaut vs gold: {path} ==");
    println!(
        "lemmas: {} total, {} lexicon-covered",
        s.scored_lemmas, s.covered_lemmas
    );
    println!(
        "adjudicated forms counted as correct: {}",
        s.adjudicated_hits
    );
    println!("corrupt-gold lemmas excluded (NFIN ≠ lemma): {corrupt_count}");
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
    for cat in CATEGORIES {
        let c = s
            .by_category
            .get(&(cat, true))
            .map_or((0, 0), |t| (t.matched, t.total));
        let f = s
            .by_category
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
    if !s.skipped_bundles.is_empty() {
        let mut b: Vec<&&str> = s.skipped_bundles.iter().collect();
        b.sort();
        println!("\nskipped feature bundles: {b:?}");
    }

    let mut worst: Vec<(&&str, &usize)> = s.lemma_errors.iter().collect();
    worst.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("\nworst lemmas (errors):");
    for (lemma, n) in worst.iter().take(15) {
        println!("  {lemma}: {n}");
    }

    fs::write("target/golden_mismatches.tsv", &s.mismatches).unwrap();
    println!(
        "\n{} mismatching forms written to target/golden_mismatches.tsv",
        s.mismatches.lines().count()
    );
}

fn check_gates(s: &Scores) {
    let (cov_total, cov_matched) = s.totals(true);
    let (fb_total, fb_matched) = s.totals(false);
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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|a| a == "--check");
    let path = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| "data/unimorph/deu".to_string());
    let data = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {path}: {e}. Run scripts/fetch_unimorph_deu.sh first"));

    let gold = parse_gold(&data);
    let adjudicated = load_adjudications();
    let corrupt = corrupt_lemmas(&gold);
    let scores = score(&gold, &corrupt, &adjudicated);
    report(&path, &scores, corrupt.len());
    if check {
        check_gates(&scores);
    }
}
