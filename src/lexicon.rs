//! The exception lexicon (Layer A of `docs/ontology.md`): the finite list of
//! verbs whose forms are not fully predictable from the infinitive.
//!
//! The data lives in `data/verbs.tsv` — human-readable, diffable, and the
//! unit the golden-test harness will grow against. It is embedded at compile
//! time and parsed once on first lookup; no I/O at runtime.

use crate::Auxiliary;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Inflection class of a lexicon entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LexClass {
    /// Ablaut verbs: strong preterite endings (ich sang, zero ending).
    Strong,
    /// Changed stem with weak -te endings (denken → dachte).
    Mixed,
    /// Präteritopräsentia (modals + wissen): the present singular inflects
    /// like a strong preterite (ich kann, zero ending), the preterite is weak.
    PreteritePresent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LexEntry {
    pub class: LexClass,
    /// Strong/mixed: changed 2/3sg present stem (sprich-, fähr-, ha-).
    /// Preterite-present: the whole-singular present stem (kann, weiß).
    pub pres: Option<String>,
    /// Preterite stem (sang, dach, konn).
    pub pret: String,
    /// Konjunktiv II stem (säng, däch, könn).
    pub konj2: String,
    /// Full past participle (gesungen, gedacht, vergessen).
    pub part2: String,
    /// 2sg imperative override (sprich); None = derive by rule.
    pub imp: Option<String>,
    pub aux: Auxiliary,
}

static TSV: &str = include_str!("../data/verbs.tsv");

fn opt(field: &str) -> Option<String> {
    (field != "-").then(|| field.to_string())
}

fn parse(tsv: &'static str) -> HashMap<&'static str, LexEntry> {
    let mut map = HashMap::new();
    for line in tsv.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        assert_eq!(f.len(), 8, "malformed lexicon line: {line}");
        let class = match f[1] {
            "s" => LexClass::Strong,
            "m" => LexClass::Mixed,
            "p" => LexClass::PreteritePresent,
            c => panic!("unknown class {c} in lexicon line: {line}"),
        };
        let aux = match f[7] {
            "h" => Auxiliary::Haben,
            "s" => Auxiliary::Sein,
            a => panic!("unknown auxiliary {a} in lexicon line: {line}"),
        };
        map.insert(
            f[0],
            LexEntry {
                class,
                pres: opt(f[2]),
                pret: f[3].to_string(),
                konj2: f[4].to_string(),
                part2: f[5].to_string(),
                imp: opt(f[6]),
                aux,
            },
        );
    }
    map
}

pub(crate) fn lookup(infinitive: &str) -> Option<&'static LexEntry> {
    static MAP: OnceLock<HashMap<&'static str, LexEntry>> = OnceLock::new();
    MAP.get_or_init(|| parse(TSV)).get(infinitive)
}
