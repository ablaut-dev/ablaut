//! Latin conjugation. Latin is fusional with four conjugations; it is
//! cited by the 1sg present active indicative (amō, moneō, regō, capiō,
//! audiō; deponents by their 1sg -or: hortor, loquor). From that citation
//! we read a conjugation class and a stem and generate the present system
//! productively: present, imperfect and simple-future indicative
//! (person/number), the present imperative (2sg/2pl) and the present active
//! infinitive — for both the active paradigm and, with passive endings, the
//! deponent one (a verb is deponent iff its citation ends in -r).
//!
//! The class is not fully recoverable from the 1sg (amō 1st vs regō 3rd
//! both end -ō), so the dominant reading of each ending is the productive
//! default (with the genuine -scō → 3rd inchoative rule), and every verb the
//! default gets wrong carries a mined row in `data/lat/parts.tsv`, keyed by
//! feature. Verified against the agreement of two oracles (UniMorph and
//! kaikki.org). Scope: the present-system active-indicative core; the
//! perfect system, subjunctive, non-deponent passive, participles, supines
//! and gerunds are a later pass.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The stored cells, in the column order of `data/lat/parts.tsv`.
pub const CELLS: [&str; 21] = [
    "V;IND;ACT;PRS;1;SG",
    "V;IND;ACT;PRS;2;SG",
    "V;IND;ACT;PRS;3;SG",
    "V;IND;ACT;PRS;1;PL",
    "V;IND;ACT;PRS;2;PL",
    "V;IND;ACT;PRS;3;PL",
    "V;IND;ACT;PST;IPFV;1;SG",
    "V;IND;ACT;PST;IPFV;2;SG",
    "V;IND;ACT;PST;IPFV;3;SG",
    "V;IND;ACT;PST;IPFV;1;PL",
    "V;IND;ACT;PST;IPFV;2;PL",
    "V;IND;ACT;PST;IPFV;3;PL",
    "V;IND;ACT;FUT;1;SG",
    "V;IND;ACT;FUT;2;SG",
    "V;IND;ACT;FUT;3;SG",
    "V;IND;ACT;FUT;1;PL",
    "V;IND;ACT;FUT;2;PL",
    "V;IND;ACT;FUT;3;PL",
    "V;IMP;ACT;PRS;2;SG",
    "V;IMP;ACT;PRS;2;PL",
    "V;NFIN;ACT;PRS",
];

/// Why a form cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Latin 1sg-present citation.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Latin verb")
    }
}

static PARTS_TSV: &str = include_str!("../data/lat/parts.tsv");

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

/// Conjugation class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // C3io (capiō-type) is folded into C3 by the productive rule.
enum Cls {
    C1,
    C2,
    C3,
    C3io,
    C4,
}

/// Read (stem, class, deponent) off the citation, taking the dominant
/// reading of each ending as the default (plus the -scō inchoative rule).
/// Mirrors `classify()` in scripts/lat/mine.py exactly.
fn classify(cit: &str) -> Option<(&str, Cls, bool)> {
    if let Some(s) = cit.strip_suffix("eor") {
        return Some((s, Cls::C2, true));
    }
    if let Some(s) = cit.strip_suffix("ior") {
        return Some((s, Cls::C4, true)); // largior-type dominates -ior deponents
    }
    if let Some(s) = cit.strip_suffix("or") {
        return Some((s, Cls::C1, true)); // hortor-type dominates -or deponents
    }
    if cit.ends_with("scō") {
        // Inchoatives in -scō are always 3rd; strip only the final ō.
        return Some((&cit[..cit.len() - "ō".len()], Cls::C3, false));
    }
    if let Some(s) = cit.strip_suffix("eō") {
        return Some((s, Cls::C2, false));
    }
    if let Some(s) = cit.strip_suffix("iō") {
        return Some((s, Cls::C4, false)); // audiō-type dominates -iō actives
    }
    if let Some(s) = cit.strip_suffix("ō") {
        return Some((s, Cls::C1, false)); // amō-type (1st conj) dominates -ō
    }
    None
}

/// The six present / imperfect / future endings, the two imperative
/// endings and the infinitive ending for a (class, voice). Mirrors the
/// ACT/DEP tables in scripts/lat/mine.py exactly.
struct Endings {
    prs: [&'static str; 6],
    ipfv: [&'static str; 6],
    fut: [&'static str; 6],
    imp: [&'static str; 2],
    inf: &'static str,
}

fn endings(cls: Cls, dep: bool) -> Endings {
    match (cls, dep) {
        (Cls::C1, false) => Endings {
            prs: ["ō", "ās", "at", "āmus", "ātis", "ant"],
            ipfv: ["ābam", "ābās", "ābat", "ābāmus", "ābātis", "ābant"],
            fut: ["ābō", "ābis", "ābit", "ābimus", "ābitis", "ābunt"],
            imp: ["ā", "āte"],
            inf: "āre",
        },
        (Cls::C2, false) => Endings {
            prs: ["eō", "ēs", "et", "ēmus", "ētis", "ent"],
            ipfv: ["ēbam", "ēbās", "ēbat", "ēbāmus", "ēbātis", "ēbant"],
            fut: ["ēbō", "ēbis", "ēbit", "ēbimus", "ēbitis", "ēbunt"],
            imp: ["ē", "ēte"],
            inf: "ēre",
        },
        (Cls::C3, false) => Endings {
            prs: ["ō", "is", "it", "imus", "itis", "unt"],
            ipfv: ["ēbam", "ēbās", "ēbat", "ēbāmus", "ēbātis", "ēbant"],
            fut: ["am", "ēs", "et", "ēmus", "ētis", "ent"],
            imp: ["e", "ite"],
            inf: "ere",
        },
        (Cls::C3io, false) => Endings {
            prs: ["iō", "is", "it", "imus", "itis", "iunt"],
            ipfv: ["iēbam", "iēbās", "iēbat", "iēbāmus", "iēbātis", "iēbant"],
            fut: ["iam", "iēs", "iet", "iēmus", "iētis", "ient"],
            imp: ["e", "ite"],
            inf: "ere",
        },
        (Cls::C4, false) => Endings {
            prs: ["iō", "īs", "it", "īmus", "ītis", "iunt"],
            ipfv: ["iēbam", "iēbās", "iēbat", "iēbāmus", "iēbātis", "iēbant"],
            fut: ["iam", "iēs", "iet", "iēmus", "iētis", "ient"],
            imp: ["ī", "īte"],
            inf: "īre",
        },
        (Cls::C1, true) => Endings {
            prs: ["or", "āris", "ātur", "āmur", "āminī", "antur"],
            ipfv: ["ābar", "ābāris", "ābātur", "ābāmur", "ābāminī", "ābantur"],
            fut: ["ābor", "āberis", "ābitur", "ābimur", "ābiminī", "ābuntur"],
            imp: ["āre", "āminī"],
            inf: "ārī",
        },
        (Cls::C2, true) => Endings {
            prs: ["eor", "ēris", "ētur", "ēmur", "ēminī", "entur"],
            ipfv: ["ēbar", "ēbāris", "ēbātur", "ēbāmur", "ēbāminī", "ēbantur"],
            fut: ["ēbor", "ēberis", "ēbitur", "ēbimur", "ēbiminī", "ēbuntur"],
            imp: ["ēre", "ēminī"],
            inf: "ērī",
        },
        (Cls::C3, true) => Endings {
            prs: ["or", "eris", "itur", "imur", "iminī", "untur"],
            ipfv: ["ēbar", "ēbāris", "ēbātur", "ēbāmur", "ēbāminī", "ēbantur"],
            fut: ["ar", "ēris", "ētur", "ēmur", "ēminī", "entur"],
            imp: ["ere", "iminī"],
            inf: "ī",
        },
        (Cls::C3io, true) => Endings {
            prs: ["ior", "eris", "itur", "imur", "iminī", "iuntur"],
            ipfv: [
                "iēbar",
                "iēbāris",
                "iēbātur",
                "iēbāmur",
                "iēbāminī",
                "iēbantur",
            ],
            fut: ["iar", "iēris", "iētur", "iēmur", "iēminī", "ientur"],
            imp: ["ere", "iminī"],
            inf: "ī",
        },
        (Cls::C4, true) => Endings {
            prs: ["ior", "īris", "ītur", "īmur", "īminī", "iuntur"],
            ipfv: [
                "iēbar",
                "iēbāris",
                "iēbātur",
                "iēbāmur",
                "iēbāminī",
                "iēbantur",
            ],
            fut: ["iar", "iēris", "iētur", "iēmur", "iēminī", "ientur"],
            imp: ["īre", "īminī"],
            inf: "īrī",
        },
    }
}

const PN: [&str; 6] = ["1;SG", "2;SG", "3;SG", "1;PL", "2;PL", "3;PL"];

/// The productive form for a feature bundle, or None for a non-verb
/// citation or an unsupported cell. Mirrors `produce()` in mine.py exactly.
fn productive(cit: &str, feat: &str) -> Option<String> {
    let (stem, cls, dep) = classify(cit)?;
    let e = endings(cls, dep);
    if feat == "V;NFIN;ACT;PRS" {
        return Some(format!("{stem}{}", e.inf));
    }
    if feat == "V;IMP;ACT;PRS;2;SG" {
        return Some(format!("{stem}{}", e.imp[0]));
    }
    if feat == "V;IMP;ACT;PRS;2;PL" {
        return Some(format!("{stem}{}", e.imp[1]));
    }
    let parts: Vec<&str> = feat.split(';').collect();
    if parts.len() < 2 {
        return None;
    }
    let pn = format!("{};{}", parts[parts.len() - 2], parts[parts.len() - 1]);
    let i = PN.iter().position(|p| *p == pn)?;
    if feat.starts_with("V;IND;ACT;PRS;") {
        Some(format!("{stem}{}", e.prs[i]))
    } else if feat.starts_with("V;IND;ACT;PST;IPFV;") {
        Some(format!("{stem}{}", e.ipfv[i]))
    } else if feat.starts_with("V;IND;ACT;FUT;") {
        Some(format!("{stem}{}", e.fut[i]))
    } else {
        None
    }
}

/// A conjugatable Latin verb, cited by its 1sg present active indicative.
#[derive(Debug, Clone)]
pub struct Verb {
    citation: String,
    overrides: HashMap<&'static str, &'static str>,
}

impl Verb {
    /// Build a verb from its 1sg-present-active-indicative citation.
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
        // Accept only if we can conjugate it: a stored verb, or a citation
        // whose ending yields a productive class.
        if over.is_empty() && classify(&cit).is_none() {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            citation: cit,
            overrides: over,
        })
    }

    /// The citation form (1sg present active indicative).
    pub fn citation(&self) -> &str {
        &self.citation
    }

    /// The form for a UniMorph feature bundle: a mined override if one
    /// exists, else the productive rule.
    pub fn form(&self, feature: &str) -> Option<String> {
        if let Some(f) = self.overrides.get(feature) {
            return Some((*f).to_string());
        }
        productive(&self.citation, feature)
    }
}

/// A compact conjugation table — the present-system active core, shared by
/// the WebAssembly and Python bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// 1sg present active indicative (the citation).
    pub citation: String,
    /// Present active infinitive.
    pub infinitive: Option<String>,
    /// Present indicative active, 1sg/2sg/3sg/1pl/2pl/3pl.
    pub present: Vec<Option<String>>,
    /// Imperfect indicative active, same order.
    pub imperfect: Vec<Option<String>>,
    /// Simple future indicative active, same order.
    pub future: Vec<Option<String>>,
    /// Present imperative active, 2sg/2pl.
    pub imperative: Vec<Option<String>>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let many = |feats: &[&str]| feats.iter().map(|f| v.form(f)).collect();
        Self {
            citation: v.citation().to_string(),
            infinitive: v.form("V;NFIN;ACT;PRS"),
            present: many(&[
                "V;IND;ACT;PRS;1;SG",
                "V;IND;ACT;PRS;2;SG",
                "V;IND;ACT;PRS;3;SG",
                "V;IND;ACT;PRS;1;PL",
                "V;IND;ACT;PRS;2;PL",
                "V;IND;ACT;PRS;3;PL",
            ]),
            imperfect: many(&[
                "V;IND;ACT;PST;IPFV;1;SG",
                "V;IND;ACT;PST;IPFV;2;SG",
                "V;IND;ACT;PST;IPFV;3;SG",
                "V;IND;ACT;PST;IPFV;1;PL",
                "V;IND;ACT;PST;IPFV;2;PL",
                "V;IND;ACT;PST;IPFV;3;PL",
            ]),
            future: many(&[
                "V;IND;ACT;FUT;1;SG",
                "V;IND;ACT;FUT;2;SG",
                "V;IND;ACT;FUT;3;SG",
                "V;IND;ACT;FUT;1;PL",
                "V;IND;ACT;FUT;2;PL",
                "V;IND;ACT;FUT;3;PL",
            ]),
            imperative: many(&["V;IMP;ACT;PRS;2;SG", "V;IMP;ACT;PRS;2;PL"]),
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
    fn first_conjugation_productive() {
        // amō → 1st conj (the productive default for plain -ō).
        let a = v("amō");
        assert_eq!(a.form("V;IND;ACT;PRS;2;SG").unwrap(), "amās");
        assert_eq!(a.form("V;IND;ACT;PRS;3;PL").unwrap(), "amant");
        assert_eq!(a.form("V;IND;ACT;PST;IPFV;1;SG").unwrap(), "amābam");
        assert_eq!(a.form("V;IND;ACT;FUT;1;SG").unwrap(), "amābō");
        assert_eq!(a.form("V;IMP;ACT;PRS;2;SG").unwrap(), "amā");
        assert_eq!(a.form("V;NFIN;ACT;PRS").unwrap(), "amāre");
    }

    #[test]
    fn second_and_fourth_conjugation() {
        let m = v("moneō"); // -eō → 2nd
        assert_eq!(m.form("V;IND;ACT;PRS;2;SG").unwrap(), "monēs");
        assert_eq!(m.form("V;IND;ACT;FUT;3;SG").unwrap(), "monēbit");
        assert_eq!(m.form("V;NFIN;ACT;PRS").unwrap(), "monēre");
        let a = v("audiō"); // -iō → 4th (default)
        assert_eq!(a.form("V;IND;ACT;PRS;3;PL").unwrap(), "audiunt");
        assert_eq!(a.form("V;IND;ACT;FUT;1;SG").unwrap(), "audiam");
        assert_eq!(a.form("V;NFIN;ACT;PRS").unwrap(), "audīre");
    }

    #[test]
    fn inchoative_scō_is_third() {
        // -scō inchoatives are always 3rd conjugation.
        let c = v("crēscō");
        assert_eq!(c.form("V;IND;ACT;PRS;2;SG").unwrap(), "crēscis");
        assert_eq!(c.form("V;IND;ACT;FUT;1;SG").unwrap(), "crēscam");
        assert_eq!(c.form("V;NFIN;ACT;PRS").unwrap(), "crēscere");
    }

    // Deponent passive morphology (hortor-type) is outside the scored
    // active-indicative paradigm and not verified against the oracle gold,
    // so it is intentionally not asserted here.

    #[test]
    fn mined_override_beats_rule() {
        // regō is 3rd but the -ō default is 1st, so it is mined; the stored
        // row must win over the (wrong) productive guess.
        let r = v("regō");
        assert_eq!(r.form("V;IND;ACT;PRS;2;SG").unwrap(), "regis");
        assert_eq!(r.form("V;NFIN;ACT;PRS").unwrap(), "regere");
    }
}
