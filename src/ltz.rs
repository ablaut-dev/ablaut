//! Luxembourgish conjugation. Luxembourgish is a West-Germanic language
//! cited by its `-en` infinitive (schaffen). The synthetic (one-word) core
//! — the present indicative (six person/number cells), the two imperatives
//! (2sg / 2pl) and the past participle — is generated productively from a
//! stem = infinitive minus `-en` (schaffen → schaff-):
//!
//! ```text
//!   V;NFIN         schaffen     V;IND;PRS;PL;1   schaffen
//!   V;IND;PRS;SG;1 schaffen     V;IND;PRS;PL;2   schafft
//!   V;IND;PRS;SG;2 schaffs      V;IND;PRS;PL;3   schaffen
//!   V;IND;PRS;SG;3 schafft      V;IMP;2;SG       schaff
//!   V;IMP;2;PL     schafft      V.PTCP;PST       geschafft
//! ```
//!
//! A stem-final `t` swallows the dental suffix (retten → rett) and a
//! stem-final sibilant swallows the 2sg `s` (weisen → weis, setzen → setz).
//! The participle drops `ge-` on the inseparable prefixes (be-, ver-, er-,
//! zer-, ent-, emp-, miss-, ge-) and the `-éieren` loan class. Strong
//! participles, ablaut/umlaut present stems, separable-prefix `ge-` infixes
//! and the suppletive auxiliaries carry a mined row in `data/ltz/parts.tsv`.
//! Verified against one oracle (kaikki.org): Beta. The periphrastic past,
//! perfect, conditional, subjunctive and future are a later pass.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The stored cells, in the column order of `data/ltz/parts.tsv`.
pub const CELLS: [&str; 10] = [
    "V;NFIN",
    "V;IND;PRS;SG;1",
    "V;IND;PRS;SG;2",
    "V;IND;PRS;SG;3",
    "V;IND;PRS;PL;1",
    "V;IND;PRS;PL;2",
    "V;IND;PRS;PL;3",
    "V;IMP;2;SG",
    "V;IMP;2;PL",
    "V.PTCP;PST",
];

/// Unstressed inseparable prefixes: the past participle takes no `ge-`.
const INSEP: [&str; 8] = ["be", "ver", "er", "zer", "ent", "emp", "miss", "ge"];

/// Why a form cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Luxembourgish infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Luxembourgish infinitive")
    }
}

static PARTS_TSV: &str = include_str!("../data/ltz/parts.tsv");

fn overrides(cit: &str) -> Option<HashMap<&'static str, &'static str>> {
    static MAP: OnceLock<HashMap<&'static str, HashMap<&'static str, &'static str>>> =
        OnceLock::new();
    let map = MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in PARTS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let mut cols = line.split('\t');
            let Some(lemma) = cols.next() else { continue };
            let mut row = HashMap::new();
            for (i, val) in cols.enumerate() {
                if val != "-" && !val.is_empty() {
                    if let Some(feat) = CELLS.get(i) {
                        row.insert(*feat, val);
                    }
                }
            }
            m.insert(lemma, row);
        }
        m
    });
    map.get(cit).cloned()
}

/// Append the dental suffix, collapsing a doubled `t` (rett + t → rett).
fn plus_t(stem: &str) -> String {
    if stem.ends_with('t') {
        stem.to_string()
    } else {
        format!("{stem}t")
    }
}

/// The productive form, or None for a non-`-en`/`-ën` citation.
fn productive(cit: &str, feat: &str) -> Option<String> {
    // The infinitive ends in -en, or -ën in the vowel-final `-eën` class
    // (dreeën, beleeën); wholly irregular verbs (sinn, hunn …) never reach
    // here — they are served entirely from the override table.
    if !(cit.ends_with("en") || cit.ends_with("ën")) {
        return None;
    }
    // Stem = infinitive minus the 2-char -en / -ën ending.
    let n = cit.chars().count();
    let stem: String = cit.chars().take(n - 2).collect();
    Some(match feat {
        "V;NFIN" | "V;IND;PRS;SG;1" | "V;IND;PRS;PL;1" | "V;IND;PRS;PL;3" => cit.to_string(),
        "V;IND;PRS;SG;2" => {
            if stem.ends_with('s') || stem.ends_with('z') {
                stem
            } else {
                format!("{stem}s")
            }
        }
        "V;IND;PRS;SG;3" | "V;IND;PRS;PL;2" | "V;IMP;2;PL" => plus_t(&stem),
        "V;IMP;2;SG" => stem,
        "V.PTCP;PST" => {
            let noge = cit.ends_with("éieren") || INSEP.iter().any(|p| cit.starts_with(p));
            if noge {
                plus_t(&stem)
            } else {
                plus_t(&format!("ge{stem}"))
            }
        }
        _ => return None,
    })
}

/// A conjugatable Luxembourgish verb, cited by its `-en` infinitive.
#[derive(Debug, Clone)]
pub struct Verb {
    citation: String,
    overrides: HashMap<&'static str, &'static str>,
}

impl Verb {
    /// Build a verb from its infinitive (schaffen, kommen, studéieren …).
    pub fn from_infinitive(citation: &str) -> Result<Self, Error> {
        let lowered = citation.trim().to_lowercase();
        let cit = if overrides(citation.trim()).is_none() && overrides(&lowered).is_some() {
            lowered.clone()
        } else {
            citation.trim().to_string()
        };
        if cit.is_empty() || cit.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        if !(cit.ends_with("en") || cit.ends_with("ën")) && overrides(&cit).is_none() {
            return Err(Error::NotAVerb);
        }
        let over = overrides(&cit).unwrap_or_default();
        Ok(Self {
            citation: cit,
            overrides: over,
        })
    }

    /// The infinitive (citation).
    pub fn citation(&self) -> &str {
        &self.citation
    }

    /// The form for a feature bundle: a mined override if one exists, else
    /// the productive rule.
    pub fn form(&self, feature: &str) -> Option<String> {
        if let Some(f) = self.overrides.get(feature) {
            return Some((*f).to_string());
        }
        productive(&self.citation, feature)
    }
}

/// A compact conjugation table — the synthetic core, shared by the
/// WebAssembly and Python bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// Infinitive (the citation).
    pub infinitive: String,
    /// Present indicative, 1sg/2sg/3sg/1pl/2pl/3pl.
    pub present: Vec<Option<String>>,
    /// Imperative 2sg/2pl.
    pub imperative: Vec<Option<String>>,
    /// Past participle.
    pub past_participle: Option<String>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let many = |feats: &[&str]| feats.iter().map(|f| v.form(f)).collect();
        Self {
            infinitive: v.citation().to_string(),
            present: many(&[
                "V;IND;PRS;SG;1",
                "V;IND;PRS;SG;2",
                "V;IND;PRS;SG;3",
                "V;IND;PRS;PL;1",
                "V;IND;PRS;PL;2",
                "V;IND;PRS;PL;3",
            ]),
            imperative: many(&["V;IMP;2;SG", "V;IMP;2;PL"]),
            past_participle: v.form("V.PTCP;PST"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(cit: &str) -> Verb {
        Verb::from_infinitive(cit).unwrap()
    }

    #[test]
    fn regular() {
        let s = v("schaffen"); // stem schaff-
        assert_eq!(s.form("V;NFIN").unwrap(), "schaffen");
        assert_eq!(s.form("V;IND;PRS;SG;1").unwrap(), "schaffen");
        assert_eq!(s.form("V;IND;PRS;SG;2").unwrap(), "schaffs");
        assert_eq!(s.form("V;IND;PRS;SG;3").unwrap(), "schafft");
        assert_eq!(s.form("V;IND;PRS;PL;2").unwrap(), "schafft");
        assert_eq!(s.form("V;IND;PRS;PL;3").unwrap(), "schaffen");
        assert_eq!(s.form("V;IMP;2;SG").unwrap(), "schaff");
        assert_eq!(s.form("V;IMP;2;PL").unwrap(), "schafft");
        assert_eq!(s.form("V.PTCP;PST").unwrap(), "geschafft");
    }

    #[test]
    fn spelling_adjustments() {
        // Stem-final t swallows the dental suffix.
        let r = v("retten");
        assert_eq!(r.form("V;IND;PRS;SG;3").unwrap(), "rett");
        assert_eq!(r.form("V.PTCP;PST").unwrap(), "gerett");
        // Stem-final sibilant swallows the 2sg s.
        assert_eq!(v("weisen").form("V;IND;PRS;SG;2").unwrap(), "weis");
        assert_eq!(v("setzen").form("V;IND;PRS;SG;2").unwrap(), "setz");
        // Inseparable prefix and -éieren loan class: no ge-.
        assert_eq!(v("studéieren").form("V.PTCP;PST").unwrap(), "studéiert");
        assert_eq!(v("berouegen").form("V.PTCP;PST").unwrap(), "berouegt");
    }

    #[test]
    fn mined_override_wins() {
        // droen is strong: mined present stem dréi- and participle gedroen.
        let d = v("droen");
        assert_eq!(d.form("V;IND;PRS;SG;2").unwrap(), "dréis");
        assert_eq!(d.form("V.PTCP;PST").unwrap(), "gedroen");
    }
}
