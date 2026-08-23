//! Azerbaijani conjugation. Azerbaijani is an agglutinative Turkic
//! language cited by its infinitive (-maq / -mək). The synthetic core —
//! present (-ir), past definite (-di), future definite (-acaq), aorist /
//! future indefinite (-ar), the two imperatives and the infinitive — is
//! generated productively by four-way vowel harmony from the stem: the low
//! vowel A ∈ {a, ə}, the high vowel I ∈ {ı, i, u, ü} and the final K ∈
//! {q, k}. Verbs whose aorist takes the -ir class, or that show consonant
//! voicing (et- → ed-) or other irregularity, carry a mined row in
//! `data/aze/parts.tsv`. Verified against one oracle (kaikki.org): Beta.
//! The perfect, continuative, evidential, conditional and negative forms
//! are a later pass.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The stored cells, in the column order of `data/aze/parts.tsv`.
pub const CELLS: [&str; 27] = [
    "V;PRS;1;SG",
    "V;PRS;2;SG",
    "V;PRS;3;SG",
    "V;PRS;1;PL",
    "V;PRS;2;PL",
    "V;PRS;3;PL",
    "V;PST;1;SG",
    "V;PST;2;SG",
    "V;PST;3;SG",
    "V;PST;1;PL",
    "V;PST;2;PL",
    "V;PST;3;PL",
    "V;FUT;1;SG",
    "V;FUT;2;SG",
    "V;FUT;3;SG",
    "V;FUT;1;PL",
    "V;FUT;2;PL",
    "V;FUT;3;PL",
    "V;AOR;1;SG",
    "V;AOR;2;SG",
    "V;AOR;3;SG",
    "V;AOR;1;PL",
    "V;AOR;2;PL",
    "V;AOR;3;PL",
    "V;IMP;2;SG",
    "V;IMP;2;PL",
    "V;NFIN",
];

/// Why a form cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like an Azerbaijani infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Azerbaijani infinitive")
    }
}

static PARTS_TSV: &str = include_str!("../data/aze/parts.tsv");

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

const VOWELS: &str = "aıoueəiöü";

/// Four-way vowel harmony from a stem: (low A, high I, final K).
fn harmony(stem: &str) -> (char, char, char) {
    let v = stem
        .chars()
        .rev()
        .find(|c| VOWELS.contains(*c))
        .unwrap_or('ə');
    let back = "aıou".contains(v);
    let round = "ouöü".contains(v);
    let a = if back { 'a' } else { 'ə' };
    let i = match (back, round) {
        (true, false) => 'ı',
        (true, true) => 'u',
        (false, false) => 'i',
        (false, true) => 'ü',
    };
    let k = if back { 'q' } else { 'k' };
    (a, i, k)
}

/// The productive harmony-generated form, or None for a non-verb citation.
fn productive(cit: &str, feat: &str) -> Option<String> {
    if !(cit.ends_with("maq") || cit.ends_with("mək")) {
        return None;
    }
    if feat == "V;NFIN" {
        return Some(cit.to_string());
    }
    // Stem = infinitive minus the 3-char -maq/-mək.
    let n = cit.chars().count();
    let stem: String = cit.chars().take(n - 3).collect();
    let (a, i, k) = harmony(&stem);
    let vowel_final = stem.chars().next_back().is_some_and(|c| VOWELS.contains(c));
    let y = if vowel_final { "y" } else { "" };

    // Copular person set (present / aorist / future).
    let cop = |pn: &str| -> String {
        match pn {
            "1;SG" => format!("{a}m"),
            "2;SG" => format!("s{a}n"),
            "3;SG" => String::new(),
            "1;PL" => format!("{i}{k}"),
            "2;PL" => format!("s{i}n{i}z"),
            "3;PL" => format!("l{a}r"),
            _ => String::new(),
        }
    };
    // Past person set.
    let past = |pn: &str| -> String {
        match pn {
            "1;SG" => "m".into(),
            "2;SG" => "n".into(),
            "3;SG" => String::new(),
            "1;PL" => k.to_string(),
            "2;PL" => format!("n{i}z"),
            "3;PL" => format!("l{a}r"),
            _ => String::new(),
        }
    };

    if feat == "V;IMP;2;SG" {
        return Some(stem);
    }
    if feat == "V;IMP;2;PL" {
        return Some(format!("{stem}{y}{i}n"));
    }
    let parts: Vec<&str> = feat.split(';').collect();
    let tense = parts[1];
    let pn = format!("{};{}", parts[2], parts[3]);
    Some(match tense {
        "PRS" => format!("{stem}{y}{i}r{}", cop(&pn)),
        "PST" => format!("{stem}d{i}{}", past(&pn)),
        "AOR" => format!("{stem}{y}{a}r{}", cop(&pn)),
        "FUT" => {
            let base = format!("{stem}{y}{a}c{a}");
            let p = cop(&pn);
            if p.starts_with(|c| VOWELS.contains(c)) {
                // Intervocalic softening of the future's K.
                let soft = if k == 'q' { 'ğ' } else { 'y' };
                format!("{base}{soft}{p}")
            } else {
                format!("{base}{k}{p}")
            }
        }
        _ => return None,
    })
}

/// A conjugatable Azerbaijani verb, cited by its infinitive.
#[derive(Debug, Clone)]
pub struct Verb {
    citation: String,
    overrides: HashMap<&'static str, &'static str>,
}

impl Verb {
    /// Build a verb from its infinitive (-maq / -mək).
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
        if !cit.ends_with("maq") && !cit.ends_with("mək") && overrides(&cit).is_none() {
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
    /// the productive harmony rule.
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
    /// Present, 1sg/2sg/3sg/1pl/2pl/3pl.
    pub present: Vec<Option<String>>,
    /// Past definite, same order.
    pub past: Vec<Option<String>>,
    /// Future definite, same order.
    pub future: Vec<Option<String>>,
    /// Aorist (future indefinite), same order.
    pub aorist: Vec<Option<String>>,
    /// Imperative 2sg/2pl.
    pub imperative: Vec<Option<String>>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let many = |feats: &[&str]| feats.iter().map(|f| v.form(f)).collect();
        Self {
            infinitive: v.citation().to_string(),
            present: many(&[
                "V;PRS;1;SG",
                "V;PRS;2;SG",
                "V;PRS;3;SG",
                "V;PRS;1;PL",
                "V;PRS;2;PL",
                "V;PRS;3;PL",
            ]),
            past: many(&[
                "V;PST;1;SG",
                "V;PST;2;SG",
                "V;PST;3;SG",
                "V;PST;1;PL",
                "V;PST;2;PL",
                "V;PST;3;PL",
            ]),
            future: many(&[
                "V;FUT;1;SG",
                "V;FUT;2;SG",
                "V;FUT;3;SG",
                "V;FUT;1;PL",
                "V;FUT;2;PL",
                "V;FUT;3;PL",
            ]),
            aorist: many(&[
                "V;AOR;1;SG",
                "V;AOR;2;SG",
                "V;AOR;3;SG",
                "V;AOR;1;PL",
                "V;AOR;2;PL",
                "V;AOR;3;PL",
            ]),
            imperative: many(&["V;IMP;2;SG", "V;IMP;2;PL"]),
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
    fn front_harmony() {
        let s = v("sevmək"); // stem sev-, front unrounded: A=ə, I=i, K=k
        assert_eq!(s.form("V;PRS;1;SG").unwrap(), "sevirəm");
        assert_eq!(s.form("V;PRS;1;PL").unwrap(), "sevirik");
        assert_eq!(s.form("V;PST;3;SG").unwrap(), "sevdi");
        assert_eq!(s.form("V;AOR;3;SG").unwrap(), "sevər");
        assert_eq!(s.form("V;FUT;3;SG").unwrap(), "sevəcək");
        assert_eq!(s.form("V;FUT;1;SG").unwrap(), "sevəcəyəm");
    }

    #[test]
    fn back_harmony() {
        let q = v("qadamaq"); // stem qada-, back unrounded: A=a, I=ı, K=q
        assert_eq!(q.form("V;PRS;1;PL").unwrap(), "qadayırıq");
        assert_eq!(q.form("V;PST;1;PL").unwrap(), "qadadıq");
        assert_eq!(q.form("V;FUT;1;PL").unwrap(), "qadayacağıq");
        assert_eq!(q.form("V;NFIN").unwrap(), "qadamaq");
    }
}
