//! The exception lexicon (Layer A of `docs/design.md`): the finite list of
//! verbs whose forms are not fully predictable from the infinitive.
//!
//! The data lives in `data/deu/verbs.tsv` — human-readable, diffable, and the
//! unit the golden-test harness will grow against. It is embedded at compile
//! time and parsed once on first lookup; no I/O at runtime.

use crate::prefix::Separability;
use crate::Auxiliary;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Inflection class of a lexicon entry.
/// A per-lexeme prefix ruling: prefix, behavior, forced-weak base, and an
/// optional participle override for hybrids like obliegen (fused finite,
/// zu-infix, but inseparable participle: oblegen).
pub type DualRuling = (&'static str, Separability, bool, Option<&'static str>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexClass {
    /// Ablaut verbs: strong preterite endings (ich sang, zero ending).
    Strong,
    /// Changed stem with weak -te endings (denken → dachte).
    Mixed,
    /// Präteritopräsentia (modals + wissen): the present singular inflects
    /// like a strong preterite (ich kann, zero ending), the preterite is weak.
    PreteritePresent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexEntry {
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

static TSV: &str = include_str!("../data/deu/verbs.tsv");

fn opt(field: &str) -> Option<String> {
    (field != "-").then(|| field.to_string())
}

#[derive(Default)]
struct Lexicon {
    entries: HashMap<&'static str, LexEntry>,
    /// Class "w": whole-word weak verbs that must never be decomposed into
    /// prefix + base (abonnieren is not ab+onnieren).
    forced_weak: std::collections::HashSet<&'static str>,
    /// Classes "i"/"x" (+ "iw"/"xw" for a forced-weak base): per-lexeme
    /// dual-prefix rulings overriding the prefix's default behavior
    /// (umarmen is inseparable; umringen is inseparable over a *weak* base —
    /// denominal from Ring, not strong ringen). Maps lemma →
    /// (prefix, behavior, `force_weak_base`).
    dual: HashMap<&'static str, DualRuling>,
    /// Class "a": per-lexeme perfect-auxiliary overrides. Prefixed verbs
    /// inherit their base's auxiliary by default, which change-of-state
    /// derivations flip (wachen: haben → aufwachen: sein).
    aux: HashMap<&'static str, Auxiliary>,
}

fn parse(tsv: &'static str) -> Lexicon {
    let mut lex = Lexicon::default();
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
            "w" => {
                lex.forced_weak.insert(f[0]);
                continue;
            }
            "a" => {
                let aux = if f[7] == "s" {
                    Auxiliary::Sein
                } else {
                    Auxiliary::Haben
                };
                lex.aux.insert(f[0], aux);
                continue;
            }
            "i" | "x" | "iw" | "xw" | "f" => {
                let sep = match f[1] {
                    "f" => Separability::Fused,
                    s if s.starts_with('i') => Separability::Inseparable,
                    _ => Separability::Separable,
                };
                // "-" means an empty prefix: the whole word is the base.
                // With class iw this expresses "weak but no ge- participle"
                // (malochen, maledeien).
                let prefix = if f[2] == "-" { "" } else { f[2] };
                assert!(
                    f[0].starts_with(prefix),
                    "dual override prefix mismatch: {line}"
                );
                let part2 = (f[5] != "-").then_some(f[5]);
                lex.dual
                    .insert(f[0], (prefix, sep, f[1].ends_with('w'), part2));
                continue;
            }
            c => panic!("unknown class {c} in lexicon line: {line}"),
        };
        let aux = match f[7] {
            "h" => Auxiliary::Haben,
            "s" => Auxiliary::Sein,
            a => panic!("unknown auxiliary {a} in lexicon line: {line}"),
        };
        lex.entries.insert(
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
    lex
}

fn lexicon() -> &'static Lexicon {
    static LEX: OnceLock<Lexicon> = OnceLock::new();
    LEX.get_or_init(|| parse(TSV))
}

pub fn lookup(infinitive: &str) -> Option<&'static LexEntry> {
    lexicon().entries.get(infinitive)
}

pub fn is_forced_weak(infinitive: &str) -> bool {
    lexicon().forced_weak.contains(infinitive)
}

pub fn dual_override(infinitive: &str) -> Option<DualRuling> {
    lexicon().dual.get(infinitive).copied()
}

pub fn aux_override(infinitive: &str) -> Option<Auxiliary> {
    lexicon().aux.get(infinitive).copied()
}
