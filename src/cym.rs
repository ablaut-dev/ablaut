//! Welsh conjugation. Welsh (Cymraeg) is a Celtic language cited by its
//! verbal noun (berfenw), e.g. `canu` "to sing". The scored scope is the
//! synthetic (single-word) *literary* paradigm: the present(-future),
//! imperfect, preterite and pluperfect indicatives, the present subjunctive
//! and the imperative — each in six personal forms (1/2/3 × SG/PL) plus the
//! impersonal/autonomous form (UniMorph person `4`) — together with the
//! verbal noun (`V;V.MSDR`) and the verbal adjective (`V;V.PTCP`).
//!
//! Productive by rule (mirrored from `scripts/cym/mine.py`): a fixed table of
//! personal endings attached to a STEM, where the stem is the verbal noun
//! minus a final vowel (`canu`→`can`, `codi`→`cod`, `gweithio`→`gweithi`;
//! consonant-final `agor`→`agor`). Everything the rule cannot predict is
//! lexical and carries a mined row in `data/cym/parts.tsv`: irregular stems
//! (`addo`→`addaw-`), the vowel *affection* that fronts a stem vowel before
//! `-i`/`-ir` endings (`talu`→`teli`, `telir`, `telais`) and the suppletive
//! verbs (`bod`, `mynd`, `cael`…). The colloquial paradigm, which carries the
//! subject pronoun as a second word, is out of scope. Verified against two
//! oracles (UniMorph ∩ kaikki.org).

use std::collections::HashMap;
use std::sync::OnceLock;

/// The stored cells, in the column order of `data/cym/parts.tsv`.
pub const CELLS: [&str; 43] = [
    "V;LIT;1;SG;IND;PRS",
    "V;LIT;2;SG;IND;PRS",
    "V;LIT;3;SG;IND;PRS",
    "V;LIT;1;PL;IND;PRS",
    "V;LIT;2;PL;IND;PRS",
    "V;LIT;3;PL;IND;PRS",
    "V;LIT;4;IND;PRS",
    "V;LIT;1;SG;IND;IPFV",
    "V;LIT;2;SG;IND;IPFV",
    "V;LIT;3;SG;IND;IPFV",
    "V;LIT;1;PL;IND;IPFV",
    "V;LIT;2;PL;IND;IPFV",
    "V;LIT;3;PL;IND;IPFV",
    "V;LIT;4;IND;IPFV",
    "V;LIT;1;SG;IND;PST",
    "V;LIT;2;SG;IND;PST",
    "V;LIT;3;SG;IND;PST",
    "V;LIT;1;PL;IND;PST",
    "V;LIT;2;PL;IND;PST",
    "V;LIT;3;PL;IND;PST",
    "V;LIT;4;IND;PST",
    "V;LIT;1;SG;IND;PST;PFV",
    "V;LIT;2;SG;IND;PST;PFV",
    "V;LIT;3;SG;IND;PST;PFV",
    "V;LIT;1;PL;IND;PST;PFV",
    "V;LIT;2;PL;IND;PST;PFV",
    "V;LIT;3;PL;IND;PST;PFV",
    "V;LIT;4;IND;PST;PFV",
    "V;LIT;1;SG;SBJV",
    "V;LIT;2;SG;SBJV",
    "V;LIT;3;SG;SBJV",
    "V;LIT;1;PL;SBJV",
    "V;LIT;2;PL;SBJV",
    "V;LIT;3;PL;SBJV",
    "V;LIT;4;SBJV",
    "V;LIT;2;SG;IMP",
    "V;LIT;3;SG;IMP",
    "V;LIT;1;PL;IMP",
    "V;LIT;2;PL;IMP",
    "V;LIT;3;PL;IMP",
    "V;LIT;4;IMP",
    "V;V.MSDR",
    "V;V.PTCP",
];

/// Why a form cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Welsh verbal noun.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Welsh verb")
    }
}

static PARTS_TSV: &str = include_str!("../data/cym/parts.tsv");

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

/// The vowels that may end a verbal noun (and so be dropped to form the stem).
const VOWELS: &str = "aeiouwyâêîôûŵŷàèìòùáéíóúäëïöü";

/// Productive stem: the verbal noun minus a final vowel.
fn stem(cit: &str) -> String {
    match cit.chars().next_back() {
        Some(c) if VOWELS.contains(c) => {
            let mut s = cit.to_string();
            s.pop();
            s
        }
        _ => cit.to_string(),
    }
}

/// The tense/mood key of a feature bundle, or None if it is not one we model.
fn tensemood(feat: &str) -> Option<&'static str> {
    if feat.ends_with("IMP") {
        Some("imp")
    } else if feat.ends_with("SBJV") {
        Some("sbjv")
    } else if feat.contains("IND;PST;PFV") {
        Some("ppf")
    } else if feat.ends_with("IND;PST") {
        Some("pst")
    } else if feat.ends_with("IND;IPFV") {
        Some("ipf")
    } else if feat.ends_with("IND;PRS") {
        Some("prs")
    } else {
        None
    }
}

/// The person/number key (`1sg`, `2pl`, … or `4` for the impersonal).
fn personkey(feat: &str) -> Option<String> {
    let parts: Vec<&str> = feat.split(';').collect();
    let person = parts.get(2)?;
    if *person == "4" {
        return Some("4".to_string());
    }
    let number = parts.get(3)?;
    Some(format!(
        "{person}{}",
        if *number == "SG" { "sg" } else { "pl" }
    ))
}

/// The ending for a (tense/mood, person) slot, or None where the paradigm has
/// no such cell (e.g. a 1sg imperative).
fn ending(tm: &str, pk: &str) -> Option<&'static str> {
    let table: &[(&str, &str)] = match tm {
        "prs" => &[
            ("1sg", "af"),
            ("2sg", "i"),
            ("3sg", ""),
            ("1pl", "wn"),
            ("2pl", "wch"),
            ("3pl", "ant"),
            ("4", "ir"),
        ],
        "ipf" => &[
            ("1sg", "wn"),
            ("2sg", "it"),
            ("3sg", "ai"),
            ("1pl", "em"),
            ("2pl", "ech"),
            ("3pl", "ent"),
            ("4", "id"),
        ],
        "pst" => &[
            ("1sg", "ais"),
            ("2sg", "aist"),
            ("3sg", "odd"),
            ("1pl", "asom"),
            ("2pl", "asoch"),
            ("3pl", "asant"),
            ("4", "wyd"),
        ],
        "ppf" => &[
            ("1sg", "aswn"),
            ("2sg", "asit"),
            ("3sg", "asai"),
            ("1pl", "asem"),
            ("2pl", "asech"),
            ("3pl", "asent"),
            ("4", "asid"),
        ],
        "sbjv" => &[
            ("1sg", "wyf"),
            ("2sg", "ych"),
            ("3sg", "o"),
            ("1pl", "om"),
            ("2pl", "och"),
            ("3pl", "ont"),
            ("4", "er"),
        ],
        "imp" => &[
            ("2sg", ""),
            ("3sg", "ed"),
            ("1pl", "wn"),
            ("2pl", "wch"),
            ("3pl", "ent"),
            ("4", "er"),
        ],
        _ => return None,
    };
    table.iter().find(|(k, _)| *k == pk).map(|(_, v)| *v)
}

/// The productive rule-generated form, or None if the paradigm has no such
/// slot. Mirrors `productive()` in `scripts/cym/mine.py`.
fn productive(cit: &str, feat: &str) -> Option<String> {
    if feat == "V;V.MSDR" {
        return Some(cit.to_string());
    }
    let st = stem(cit);
    if feat == "V;V.PTCP" {
        return Some(format!("{st}edig"));
    }
    if !feat.starts_with("V;LIT;") {
        return None;
    }
    let tm = tensemood(feat)?;
    let pk = personkey(feat)?;
    let end = ending(tm, &pk)?;
    Some(format!("{st}{end}"))
}

/// A conjugatable Welsh verb, cited by its verbal noun.
#[derive(Debug, Clone)]
pub struct Verb {
    citation: String,
    overrides: HashMap<&'static str, &'static str>,
}

impl Verb {
    /// Build a verb from its verbal-noun citation.
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
        let over = overrides(&cit).unwrap_or_default();
        Ok(Self {
            citation: cit,
            overrides: over,
        })
    }

    /// The verbal noun (citation).
    pub fn citation(&self) -> &str {
        &self.citation
    }

    /// The form for a UniMorph feature bundle: a mined override if one exists,
    /// else the productive rule.
    pub fn form(&self, feature: &str) -> Option<String> {
        if let Some(f) = self.overrides.get(feature) {
            return Some((*f).to_string());
        }
        productive(&self.citation, feature)
    }
}

/// A compact conjugation table — the synthetic literary core, shared by the
/// WebAssembly and Python bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// The verbal noun (the citation).
    pub verbal_noun: String,
    /// Present, 1sg/2sg/3sg/1pl/2pl/3pl/impersonal.
    pub present: Vec<Option<String>>,
    /// Imperfect, same order.
    pub imperfect: Vec<Option<String>>,
    /// Preterite, same order.
    pub preterite: Vec<Option<String>>,
    /// Pluperfect, same order.
    pub pluperfect: Vec<Option<String>>,
    /// Present subjunctive, same order.
    pub subjunctive: Vec<Option<String>>,
    /// Imperative 2sg/3sg/1pl/2pl/3pl/impersonal.
    pub imperative: Vec<Option<String>>,
    /// Verbal adjective / participle.
    pub participle: Option<String>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let many = |feats: &[&str]| feats.iter().map(|f| v.form(f)).collect();
        Self {
            verbal_noun: v.citation().to_string(),
            present: many(&[
                "V;LIT;1;SG;IND;PRS",
                "V;LIT;2;SG;IND;PRS",
                "V;LIT;3;SG;IND;PRS",
                "V;LIT;1;PL;IND;PRS",
                "V;LIT;2;PL;IND;PRS",
                "V;LIT;3;PL;IND;PRS",
                "V;LIT;4;IND;PRS",
            ]),
            imperfect: many(&[
                "V;LIT;1;SG;IND;IPFV",
                "V;LIT;2;SG;IND;IPFV",
                "V;LIT;3;SG;IND;IPFV",
                "V;LIT;1;PL;IND;IPFV",
                "V;LIT;2;PL;IND;IPFV",
                "V;LIT;3;PL;IND;IPFV",
                "V;LIT;4;IND;IPFV",
            ]),
            preterite: many(&[
                "V;LIT;1;SG;IND;PST",
                "V;LIT;2;SG;IND;PST",
                "V;LIT;3;SG;IND;PST",
                "V;LIT;1;PL;IND;PST",
                "V;LIT;2;PL;IND;PST",
                "V;LIT;3;PL;IND;PST",
                "V;LIT;4;IND;PST",
            ]),
            pluperfect: many(&[
                "V;LIT;1;SG;IND;PST;PFV",
                "V;LIT;2;SG;IND;PST;PFV",
                "V;LIT;3;SG;IND;PST;PFV",
                "V;LIT;1;PL;IND;PST;PFV",
                "V;LIT;2;PL;IND;PST;PFV",
                "V;LIT;3;PL;IND;PST;PFV",
                "V;LIT;4;IND;PST;PFV",
            ]),
            subjunctive: many(&[
                "V;LIT;1;SG;SBJV",
                "V;LIT;2;SG;SBJV",
                "V;LIT;3;SG;SBJV",
                "V;LIT;1;PL;SBJV",
                "V;LIT;2;PL;SBJV",
                "V;LIT;3;PL;SBJV",
                "V;LIT;4;SBJV",
            ]),
            imperative: many(&[
                "V;LIT;2;SG;IMP",
                "V;LIT;3;SG;IMP",
                "V;LIT;1;PL;IMP",
                "V;LIT;2;PL;IMP",
                "V;LIT;3;PL;IMP",
                "V;LIT;4;IMP",
            ]),
            participle: v.form("V;V.PTCP"),
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
    fn regular_paradigm_productive() {
        // dysgu "to learn/teach": the perfectly regular verb, stem dysg-.
        let d = v("dysgu");
        assert_eq!(d.form("V;LIT;1;SG;IND;PRS").unwrap(), "dysgaf");
        assert_eq!(d.form("V;LIT;2;SG;IND;PRS").unwrap(), "dysgi");
        assert_eq!(d.form("V;LIT;3;SG;IND;PRS").unwrap(), "dysg");
        assert_eq!(d.form("V;LIT;3;PL;IND;PRS").unwrap(), "dysgant");
        assert_eq!(d.form("V;LIT;4;IND;PRS").unwrap(), "dysgir");
        assert_eq!(d.form("V;LIT;3;SG;IND;PST").unwrap(), "dysgodd");
        assert_eq!(d.form("V;LIT;1;PL;IND;PST").unwrap(), "dysgasom");
        assert_eq!(d.form("V;LIT;3;SG;IND;IPFV").unwrap(), "dysgai");
        assert_eq!(d.form("V;LIT;1;SG;IND;PST;PFV").unwrap(), "dysgaswn");
        assert_eq!(d.form("V;LIT;3;SG;SBJV").unwrap(), "dysgo");
        assert_eq!(d.form("V;LIT;2;SG;IMP").unwrap(), "dysg");
        assert_eq!(d.form("V;V.MSDR").unwrap(), "dysgu");
        assert_eq!(d.form("V;V.PTCP").unwrap(), "dysgedig");
        // A -io loan verb: stem is the verbal noun minus final -o.
        let g = v("gweithio");
        assert_eq!(g.form("V;LIT;1;SG;IND;PST").unwrap(), "gweithiais");
    }

    #[test]
    fn affection_is_mined() {
        // talu "to pay" fronts a→e before -i/-ir endings; those cells are
        // stored, the rest stay productive.
        let t = v("talu");
        assert_eq!(t.form("V;LIT;2;SG;IND;PRS").unwrap(), "teli");
        assert_eq!(t.form("V;LIT;4;IND;PRS").unwrap(), "telir");
        assert_eq!(t.form("V;LIT;1;SG;IND;PST").unwrap(), "telais");
        // productive elsewhere
        assert_eq!(t.form("V;LIT;1;SG;IND;PRS").unwrap(), "talaf");
        assert_eq!(t.form("V;LIT;3;SG;IND;PST").unwrap(), "talodd");
    }
}
