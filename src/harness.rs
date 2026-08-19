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

/// One oracle-vs-oracle disagreement: a slot both oracles cover but with
/// disjoint variant sets. The two variant sets are recorded so the
/// resolution log can show what is being ruled between.
pub struct Split {
    pub lemma: String,
    pub features: String,
    /// The first oracle's forms (`|`-joined, sorted).
    pub o1: String,
    /// The second oracle's forms.
    pub o2: String,
}

/// Intersect two oracles into agreement gold: slots both cover with
/// overlapping variant sets, unioned so either oracle's spelling counts.
/// Disjoint slots are returned separately as the disagreement corpus.
fn agree(a: Gold, b: &Gold, carry: &[&str]) -> (Gold, Vec<Split>) {
    let mut splits: Vec<Split> = Vec::new();
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
    let join = |set: &HashSet<String>| {
        let mut v: Vec<&String> = set.iter().collect();
        v.sort();
        v.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("|")
    };
    for (lemma, feats) in a {
        let Some(bfeats) = b.get(&lemma) else {
            continue;
        };
        for (features, mut variants) in feats {
            let Some(bvariants) = bfeats.get(&features) else {
                continue;
            };
            if variants.is_disjoint(bvariants) {
                splits.push(Split {
                    lemma: lemma.clone(),
                    features: features.clone(),
                    o1: join(&variants),
                    o2: join(bvariants),
                });
                continue;
            }
            variants.extend(bvariants.iter().cloned());
            out.entry(lemma.clone())
                .or_default()
                .insert(features, variants);
        }
    }
    splits.sort_by(|x, y| x.lemma.cmp(&y.lemma).then(x.features.cmp(&y.features)));
    (out, splits)
}

/// The resolution log for oracle-vs-oracle disagreements
/// (`docs/<lang>/disagreements.tsv`): a `(lemma, features) -> resolution`
/// map. Resolutions: `variant` (both standard), `o1`/`o2` (one oracle is
/// right), `neither` (both wrong, the engine's own form stands). A slot
/// absent from the map is unresolved — the work left to do.
fn load_disagreements(path: &str) -> HashMap<(String, String), String> {
    let Ok(data) = fs::read_to_string(path) else {
        return HashMap::new();
    };
    data.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f.len() >= 3).then(|| ((f[0].to_string(), f[1].to_string()), f[2].to_string()))
        })
        .collect()
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
    /// Distinct feature bundles present in the agreed gold.
    gold_slots: HashSet<String>,
    /// Feature bundles the engine produces a form for (generate → Some).
    /// The gold slots not in here are the paradigm's uncovered tenses.
    covered_slots: HashSet<String>,
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
            s.gold_slots.insert(features.clone());
            let Some(forms) = (spec.generate)(&verb, features) else {
                continue;
            };
            s.covered_slots.insert(features.clone());
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

/// Bucket the disagreement corpus by its recorded resolution, returning
/// `(variant, o1, o2, neither, unresolved)`.
fn resolution_counts(
    splits: &[Split],
    resolutions: &HashMap<(String, String), String>,
) -> [usize; 5] {
    let mut c = [0usize; 5]; // variant, o1, o2, neither, unresolved
    for sp in splits {
        match resolutions
            .get(&(sp.lemma.clone(), sp.features.clone()))
            .map(String::as_str)
        {
            Some("variant") => c[0] += 1,
            Some("o1") => c[1] += 1,
            Some("o2") => c[2] += 1,
            Some("neither") => c[3] += 1,
            _ => c[4] += 1,
        }
    }
    c
}

/// Write the machine-readable stats record (`target/stats_<lang>.json`)
/// the correctness table is compiled from, and dump the still-unresolved
/// disagreements to `target/<lang>_disagreements_todo.tsv` so they can be
/// worked through and moved into `docs/<lang>/disagreements.tsv`.
fn write_stats<V>(
    spec: &Spec<V>,
    paths: &[String],
    s: &Scores,
    splits: &[Split],
    resolutions: &HashMap<(String, String), String>,
    single_oracle: bool,
) {
    let (total, matched) = s.totals();
    let uncovered: Vec<&String> = {
        let mut u: Vec<&String> = s.gold_slots.difference(&s.covered_slots).collect();
        u.sort();
        u
    };
    let [variant, o1, o2, neither, unresolved] = resolution_counts(splits, resolutions);
    let esc = |v: &str| v.replace('\\', "\\\\").replace('"', "\\\"");
    let arr = |items: &[String]| {
        items
            .iter()
            .map(|i| format!("\"{}\"", esc(i)))
            .collect::<Vec<_>>()
            .join(",")
    };
    let oracles: Vec<String> = paths
        .iter()
        .map(|p| p.rsplit('/').next().unwrap_or(p).to_string())
        .collect();
    let cats: Vec<String> = s
        .by_category
        .iter()
        .map(|(k, t)| format!("\"{k}\":[{},{}]", t.matched, t.total))
        .collect();
    let json = format!(
        concat!(
            "{{\n",
            "  \"lang\": \"{lang}\",\n",
            "  \"oracles\": [{oracles}],\n",
            "  \"single_oracle\": {single},\n",
            "  \"lemmas_in_gold\": {lig},\n",
            "  \"lemmas_supported\": {lsup},\n",
            "  \"agreed_forms\": {total},\n",
            "  \"agreed_matched\": {matched},\n",
            "  \"agreed_pct\": {apct:.4},\n",
            "  \"adjudicated_hits\": {adj},\n",
            "  \"by_category\": {{{cats}}},\n",
            "  \"slot_types\": {slots},\n",
            "  \"slot_types_covered\": {covered},\n",
            "  \"uncovered_slots\": [{uncov}],\n",
            "  \"disagreements\": {disc},\n",
            "  \"disagreements_resolved\": {resolved},\n",
            "  \"resolution\": {{\"variant\":{v},\"o1\":{o1},\"o2\":{o2},",
            "\"neither\":{n},\"unresolved\":{unr}}}\n",
            "}}\n"
        ),
        lang = spec.lang,
        oracles = arr(&oracles),
        single = single_oracle,
        lig = s.total_lemmas,
        lsup = s.supported_lemmas,
        total = total,
        matched = matched,
        apct = pct(matched, total),
        adj = s.adjudicated_hits,
        cats = cats.join(","),
        slots = s.gold_slots.len(),
        covered = s.covered_slots.len(),
        uncov = arr(&uncovered.iter().map(|s| (*s).clone()).collect::<Vec<_>>()),
        disc = splits.len(),
        resolved = splits.len() - unresolved,
        v = variant,
        o1 = o1,
        o2 = o2,
        n = neither,
        unr = unresolved,
    );
    let stats_path = format!("target/stats_{}.json", spec.lang);
    let _ = fs::create_dir_all("target");
    fs::write(&stats_path, json).unwrap();

    let mut todo = String::from("# lemma\tfeatures\toracle1\toracle2\n");
    for sp in splits {
        if !resolutions.contains_key(&(sp.lemma.clone(), sp.features.clone())) {
            let _ = writeln!(todo, "{}\t{}\t{}\t{}", sp.lemma, sp.features, sp.o1, sp.o2);
        }
    }
    let _ = fs::write(format!("target/{}_disagreements_todo.tsv", spec.lang), todo);

    if !splits.is_empty() {
        println!(
            "\ndisagreements: {} total — {} resolved ({} variant, {} o1, {} o2, {} neither), {} unresolved",
            splits.len(),
            splits.len() - unresolved,
            variant,
            o1,
            o2,
            neither,
            unresolved
        );
    }
    if !uncovered.is_empty() {
        println!(
            "uncovered slot types: {} of {} ({})",
            uncovered.len(),
            s.gold_slots.len(),
            uncovered
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!("stats written to {stats_path}");
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
    let single_oracle = golds.len() == 1;
    let first = golds.remove(0);
    let (gold, splits) = match golds.pop() {
        Some(second) => agree(first, &second, spec.carry_features),
        None => (first, Vec::new()),
    };
    let scores = score(&spec, &gold, &load_adjudications(spec.adjudications));
    report(&spec, &paths, &scores, splits.len());
    let resolutions = load_disagreements(&format!("docs/{}/disagreements.tsv", spec.lang));
    write_stats(&spec, &paths, &scores, &splits, &resolutions, single_oracle);
    if check {
        check_gates(&spec, &scores);
    }
}
