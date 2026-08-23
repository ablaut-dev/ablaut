//! Macedonian conjugation. South Slavic, and the outlier of the family
//! here: it has no infinitive, so the lemma is the 3sg present, and the
//! conjugation class is read off its final thematic vowel — а (игра →
//! играм), и (носи → носам/носиш) or е (пише → пишам/пишеш). From the
//! stem the productive rules generate the imperfective synthetic system —
//! present, imperfect (минато определено несвршено), its l-participle,
//! the imperative — plus the passive participle and the non-finite
//! converb (глаголски прилог) and verbal noun (глаголска именка).
//! Irregular verbs carry explicit forms in `data/mkd/verbs.tsv`, mined
//! from the agreement of the two oracles. The aorist and its participle
//! are out of scope (see docs/mkd/oracles.md).

use std::collections::HashMap;
use std::sync::OnceLock;

/// Why a lemma cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Macedonian verb (3sg present).
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Macedonian verb")
    }
}

/// Conjugation class, read from the lemma's final thematic vowel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// -а stem (игра): present игра-м/-ш/-∅…
    A,
    /// -и stem (носи): present нос-ам, нос-иш…
    I,
    /// -е stem (пише): present пиш-ам, пиш-еш…
    E,
}

/// The 21 covered feature bundles, in the column order of
/// `data/mkd/verbs.tsv`.
const SLOTS: [&str; 21] = [
    "V;PRS;1;SG",
    "V;PRS;2;SG",
    "V;PRS;3;SG",
    "V;PRS;1;PL",
    "V;PRS;2;PL",
    "V;PRS;3;PL",
    "V;PROG;PST;1;SG",
    "V;PROG;PST;2;SG",
    "V;PROG;PST;3;SG",
    "V;PROG;PST;1;PL",
    "V;PROG;PST;2;PL",
    "V;PROG;PST;3;PL",
    "V.PTCP;PROG;PST;MASC;SG",
    "V.PTCP;PROG;PST;FEM;SG",
    "V.PTCP;PROG;PST;NEUT;SG",
    "V.PTCP;PROG;PST;PL",
    "V;IMP;2;SG",
    "V;IMP;2;PL",
    "V.PTCP;PST;PASS",
    "V.CVB",
    "V.MSDR",
];

static VERBS_TSV: &str = include_str!("../data/mkd/verbs.tsv");

/// Mined overrides: lemma → (feature bundle → variants). Absent cells
/// (`-`) fall through to the productive rule.
fn overrides() -> &'static HashMap<&'static str, HashMap<&'static str, Vec<String>>> {
    static MAP: OnceLock<HashMap<&'static str, HashMap<&'static str, Vec<String>>>> =
        OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in VERBS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < SLOTS.len() + 1 {
                continue;
            }
            let mut row = HashMap::new();
            for (i, slot) in SLOTS.iter().enumerate() {
                let cell = cols[i + 1];
                if cell != "-" && !cell.is_empty() {
                    row.insert(*slot, cell.split('|').map(str::to_string).collect());
                }
            }
            m.insert(cols[0], row);
        }
        m
    })
}

/// Strip the final character of a Cyrillic string, returning (stem, last).
fn split_last(s: &str) -> (String, Option<char>) {
    let mut chars: Vec<char> = s.chars().collect();
    let last = chars.pop();
    (chars.into_iter().collect(), last)
}

/// A conjugatable Macedonian verb.
#[derive(Debug, Clone)]
pub struct Verb {
    lemma: String,
    stem: String,
    class: Class,
}

impl Verb {
    /// Build a verb from its lemma (the 3sg present; the particle "да"
    /// is accepted and stripped).
    pub fn from_infinitive(lemma: &str) -> Result<Self, Error> {
        let lowered = lemma.trim().to_lowercase();
        let mut l = lowered.as_str();
        if let Some(rest) = l.strip_prefix("да ") {
            l = rest.trim();
        }
        if l.is_empty()
            || l.contains(char::is_whitespace)
            || !l.chars().all(|c| c.is_alphabetic() || c == '-')
        {
            return Err(Error::NotAVerb);
        }
        let (stem, last) = split_last(l);
        let class = match last {
            Some('а') => Class::A,
            Some('и') => Class::I,
            Some('е') => Class::E,
            // Athematic/irregular (сум, е-less lemmas): no productive class;
            // it conjugates only through mined rows.
            _ => return Err(Error::NotAVerb),
        };
        Ok(Self {
            lemma: l.to_string(),
            stem,
            class,
        })
    }

    /// The lemma (3sg present) as normalized.
    pub fn infinitive(&self) -> &str {
        &self.lemma
    }

    /// The productive form for a feature bundle, before overrides.
    fn rule(&self, feat: &str) -> Option<String> {
        let s = &self.stem;
        // Imperfect / l-participle theme: а for the a-class, е otherwise.
        let th = if self.class == Class::A { "а" } else { "е" };
        let form = match feat {
            "V;PRS;1;SG" => format!("{s}ам"),
            "V;PRS;2;SG" => match self.class {
                Class::A => format!("{s}аш"),
                Class::I => format!("{s}иш"),
                Class::E => format!("{s}еш"),
            },
            "V;PRS;3;SG" => match self.class {
                Class::A => format!("{s}а"),
                Class::I => format!("{s}и"),
                Class::E => format!("{s}е"),
            },
            "V;PRS;1;PL" => match self.class {
                Class::A => format!("{s}аме"),
                Class::I => format!("{s}име"),
                Class::E => format!("{s}еме"),
            },
            "V;PRS;2;PL" => match self.class {
                Class::A => format!("{s}ате"),
                Class::I => format!("{s}ите"),
                Class::E => format!("{s}ете"),
            },
            "V;PRS;3;PL" => {
                if self.class == Class::A {
                    format!("{s}аат")
                } else {
                    format!("{s}ат")
                }
            }
            "V;PROG;PST;1;SG" => format!("{s}{th}в"),
            "V;PROG;PST;2;SG" | "V;PROG;PST;3;SG" => format!("{s}{th}ше"),
            "V;PROG;PST;1;PL" => format!("{s}{th}вме"),
            "V;PROG;PST;2;PL" => format!("{s}{th}вте"),
            "V;PROG;PST;3;PL" => format!("{s}{th}а"),
            "V.PTCP;PROG;PST;MASC;SG" => format!("{s}{th}л"),
            "V.PTCP;PROG;PST;FEM;SG" => format!("{s}{th}ла"),
            "V.PTCP;PROG;PST;NEUT;SG" => format!("{s}{th}ло"),
            "V.PTCP;PROG;PST;PL" => format!("{s}{th}ле"),
            "V;IMP;2;SG" => {
                if self.class == Class::A {
                    format!("{s}ај")
                } else {
                    format!("{s}и")
                }
            }
            "V;IMP;2;PL" => {
                if self.class == Class::A {
                    format!("{s}ајте")
                } else {
                    format!("{s}ете")
                }
            }
            "V.PTCP;PST;PASS" => {
                if self.class == Class::A {
                    format!("{s}ан")
                } else {
                    format!("{s}ен")
                }
            }
            "V.CVB" => {
                if self.class == Class::A {
                    format!("{s}ајќи")
                } else {
                    format!("{s}ејќи")
                }
            }
            "V.MSDR" => {
                if self.class == Class::A {
                    format!("{s}ање")
                } else {
                    format!("{s}ење")
                }
            }
            _ => return None,
        };
        Some(form)
    }

    /// Every accepted form for a feature bundle: a mined override if one
    /// exists, otherwise the productive rule. Empty for unsupported slots
    /// (e.g. the aorist).
    pub fn forms(&self, feat: &str) -> Vec<String> {
        if let Some(row) = overrides().get(self.lemma.as_str()) {
            if let Some(v) = row.get(feat) {
                return v.clone();
            }
        }
        self.rule(feat).into_iter().collect()
    }

    /// The primary form for a feature bundle, if any.
    fn one(&self, feat: &str) -> Option<String> {
        self.forms(feat).into_iter().next()
    }
}

/// The full conjugation table of a Macedonian verb — shared by the
/// WebAssembly and Python bindings. No infinitive (Macedonian has none);
/// the lemma is the 3sg present.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// The lemma (3sg present).
    pub lemma: String,
    /// Present, [1sg…3pl].
    pub present: [String; 6],
    /// Imperfect (минато определено несвршено), [1sg…3pl].
    pub imperfect: [String; 6],
    /// The imperfect l-participle: [masc sg, fem sg, neut sg, plural].
    pub imperfect_participle: [String; 4],
    /// [2sg, 2pl].
    pub imperative: [Option<String>; 2],
    pub passive_participle: Option<String>,
    pub converb: Option<String>,
    pub verbal_noun: Option<String>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let g = |feat: &str| v.one(feat).unwrap_or_default();
        Self {
            lemma: v.infinitive().to_string(),
            present: [
                g("V;PRS;1;SG"),
                g("V;PRS;2;SG"),
                g("V;PRS;3;SG"),
                g("V;PRS;1;PL"),
                g("V;PRS;2;PL"),
                g("V;PRS;3;PL"),
            ],
            imperfect: [
                g("V;PROG;PST;1;SG"),
                g("V;PROG;PST;2;SG"),
                g("V;PROG;PST;3;SG"),
                g("V;PROG;PST;1;PL"),
                g("V;PROG;PST;2;PL"),
                g("V;PROG;PST;3;PL"),
            ],
            imperfect_participle: [
                g("V.PTCP;PROG;PST;MASC;SG"),
                g("V.PTCP;PROG;PST;FEM;SG"),
                g("V.PTCP;PROG;PST;NEUT;SG"),
                g("V.PTCP;PROG;PST;PL"),
            ],
            imperative: [v.one("V;IMP;2;SG"), v.one("V;IMP;2;PL")],
            passive_participle: v.one("V.PTCP;PST;PASS"),
            converb: v.one("V.CVB"),
            verbal_noun: v.one("V.MSDR"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(lemma: &str) -> Verb {
        Verb::from_infinitive(lemma).unwrap()
    }

    #[test]
    fn a_class() {
        let g = v("игра");
        assert_eq!(g.forms("V;PRS;1;SG"), ["играм"]);
        assert_eq!(g.forms("V;PRS;3;PL"), ["играат"]);
        assert_eq!(g.forms("V;PROG;PST;1;SG"), ["играв"]);
        assert_eq!(g.forms("V.PTCP;PROG;PST;MASC;SG"), ["играл"]);
        assert_eq!(g.forms("V;IMP;2;SG"), ["играј"]);
        assert_eq!(g.forms("V.PTCP;PST;PASS"), ["игран"]);
        assert_eq!(g.forms("V.CVB"), ["играјќи"]);
        assert_eq!(g.forms("V.MSDR"), ["играње"]);
    }

    #[test]
    fn i_and_e_class() {
        let n = v("носи");
        assert_eq!(n.forms("V;PRS;1;SG"), ["носам"]);
        assert_eq!(n.forms("V;PRS;2;SG"), ["носиш"]);
        assert_eq!(n.forms("V;PRS;3;PL"), ["носат"]);
        assert_eq!(n.forms("V;PROG;PST;1;SG"), ["носев"]);
        assert_eq!(n.forms("V;IMP;2;SG"), ["носи"]);
        assert_eq!(n.forms("V.PTCP;PST;PASS"), ["носен"]);
        let p = v("пише");
        assert_eq!(p.forms("V;PRS;2;SG"), ["пишеш"]);
        assert_eq!(p.forms("V;PRS;1;SG"), ["пишам"]);
    }

    #[test]
    fn j_stem_from_mined_data() {
        // Vowel-final stems take a ј before back-vowel endings; these come
        // from data/mkd/verbs.tsv (apertium over-regularizes them).
        let b = v("брои");
        assert_eq!(b.forms("V;PRS;1;SG"), ["бројам"]);
        assert_eq!(b.forms("V;PRS;3;PL"), ["бројат"]);
        assert_eq!(b.forms("V;IMP;2;SG"), ["број"]);
        // …while the converb stays on the correct rule form (-јќи).
        assert_eq!(b.forms("V.CVB"), ["броејќи"]);
    }

    #[test]
    fn aorist_is_out_of_scope() {
        assert!(v("игра").forms("V;PST;1;SG").is_empty());
    }
}
