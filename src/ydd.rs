//! Yiddish conjugation. Yiddish is a Germanic language written right-to-left
//! in the Hebrew script and cited by its infinitive, which ends in ־ן (or
//! syllabic ־ען). The synthetic core generated here is the present indicative
//! (person marked by the ending: 1sg ־∅, 2sg ־סט, 3sg/2pl ־ט, 1pl/3pl = the
//! infinitive), the two imperatives, the present participle (־נדיק) and the
//! past participle (weak גע־…־ט, with ge- suppressed after an unstressed
//! prefix and infixed after a separable one). Strong verbs — whose past
//! participle carries lexical ablaut (זינגען → געזונגען) — and other deviations
//! carry a mined row in `data/ydd/parts.tsv`. The past, pluperfect, future and
//! future-perfect are periphrastic (auxiliary + participle) and left out.
//! Verified against one oracle (kaikki.org): Beta.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The stored cells, in the column order of `data/ydd/parts.tsv`.
pub const CELLS: [&str; 11] = [
    "V;PRS;1;SG",
    "V;PRS;2;SG",
    "V;PRS;3;SG",
    "V;PRS;1;PL",
    "V;PRS;2;PL",
    "V;PRS;3;PL",
    "V;IMP;2;SG",
    "V;IMP;2;PL",
    "V.PTCP;PRS",
    "V.PTCP;PST",
    "V;NFIN",
];

/// Why a form cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Yiddish infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Yiddish infinitive")
    }
}

static PARTS_TSV: &str = include_str!("../data/ydd/parts.tsv");

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

const RAFE: char = '\u{05BF}';

/// Combining Hebrew point (niqqud / rafe), skipped when finding the base letter.
fn is_point(c: char) -> bool {
    ('\u{0591}'..='\u{05C7}').contains(&c)
}

/// Non-final → final form of the five Yiddish consonants that have one.
fn to_final(c: char) -> Option<char> {
    match c {
        'כ' => Some('ך'),
        'מ' => Some('ם'),
        'נ' => Some('ן'),
        'פ' => Some('ף'),
        'צ' => Some('ץ'),
        _ => None,
    }
}

/// Final → non-final form.
fn to_nonfinal(c: char) -> Option<char> {
    match c {
        'ך' => Some('כ'),
        'ם' => Some('מ'),
        'ן' => Some('נ'),
        'ף' => Some('פ'),
        'ץ' => Some('צ'),
        _ => None,
    }
}

/// The last non-point (base) letter of a string.
fn last_base(s: &str) -> Option<char> {
    s.chars().rev().find(|c| !is_point(*c))
}

/// Sonorants after which infinitival ־ען is a syllabic ending the stem drops.
fn is_syll(c: char) -> bool {
    matches!(c, 'ל' | 'מ' | 'נ' | 'ג' | 'ײ')
}

/// Unstressed prefixes: their verbs take no ge- in the past participle.
/// Iteration order mirrors mine.py's UNSTR list.
const UNSTR: [&str; 9] = ["באַ", "גע", "דער", "פֿאַר", "פֿער", "צע", "אַנט", "ער", "מיס"];

/// Separable (stressed) prefixes: ge- is infixed (prefix-ge-stem-t). Tried
/// longest-first (see `separable_prefixes`).
const SEP: [&str; 25] = [
    "אַרויס",
    "אַרײַן",
    "אַראָפּ",
    "אַרום",
    "אַנידער",
    "אַוועק",
    "אונטער",
    "איבער",
    "אויס",
    "אויפֿ",
    "אויף",
    "אָפּ",
    "אָן",
    "אײַן",
    "צונויפֿ",
    "צוזאַמען",
    "צוריק",
    "צו",
    "מיט",
    "נאָך",
    "פֿאָר",
    "פֿונאַנדער",
    "ווײַטער",
    "דורך",
    "אַהיים",
];

/// SEP sorted by character length descending (stable), matching mine.py's
/// `sorted(SEP, key=len, reverse=True)`.
fn separable_prefixes() -> &'static [&'static str] {
    static SORTED: OnceLock<Vec<&'static str>> = OnceLock::new();
    SORTED.get_or_init(|| {
        let mut v: Vec<&'static str> = SEP.to_vec();
        v.sort_by_key(|b| std::cmp::Reverse(b.chars().count()));
        v
    })
}

/// Trim the last `n` characters from `s`.
fn drop_last(s: &str, n: usize) -> String {
    let count = s.chars().count();
    s.chars().take(count.saturating_sub(n)).collect()
}

/// Word-final spelling: convert a trailing non-final consonant to its final
/// form (פֿ → ף drops the rafe).
fn finalize(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let Some(&last) = chars.last() else {
        return String::new();
    };
    if last == RAFE {
        if chars.len() >= 2 && chars[chars.len() - 2] == 'פ' {
            let mut out: String = chars[..chars.len() - 2].iter().collect();
            out.push('ף');
            return out;
        }
        return s.to_string();
    }
    if let Some(f) = to_final(last) {
        let mut out: String = chars[..chars.len() - 1].iter().collect();
        out.push(f);
        return out;
    }
    s.to_string()
}

/// Word-internal spelling: undo a final consonant so a suffix can follow.
fn definalize(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if let Some(&last) = chars.last() {
        if let Some(nf) = to_nonfinal(last) {
            let mut out: String = chars[..chars.len() - 1].iter().collect();
            out.push(nf);
            return out;
        }
    }
    s.to_string()
}

/// Present stem: drop ־ן, or the syllabic ־ען after a sonorant.
fn stem_of(inf: &str) -> String {
    if inf.ends_with("ען") {
        let pre2 = drop_last(inf, 2);
        if last_base(&pre2).is_some_and(is_syll) {
            return pre2;
        }
        return drop_last(inf, 1);
    }
    if inf.ends_with("ן") {
        return drop_last(inf, 1);
    }
    inf.to_string()
}

/// The productive form for a feature bundle, or None for an unsupported slot.
fn productive(inf: &str, feat: &str) -> Option<String> {
    if feat == "V;NFIN" {
        return Some(inf.to_string());
    }
    if feat == "V.PTCP;PRS" {
        return Some(format!("{}דיק", definalize(inf)));
    }
    if feat == "V.PTCP;PST" {
        let st = stem_of(inf);
        for pre in UNSTR {
            if inf.starts_with(pre) {
                return Some(format!("{st}ט"));
            }
        }
        for pre in separable_prefixes() {
            if let Some(rest) = inf.strip_prefix(pre) {
                return Some(format!("{pre}גע{}ט", stem_of(rest)));
            }
        }
        return Some(format!("גע{st}ט"));
    }
    let st = stem_of(inf);
    let lb = last_base(&st);
    let tf = matches!(lb, Some('ט') | Some('ד')); // t/d-final stem: -t merges
    if feat == "V;IMP;2;SG" {
        return Some(finalize(&st));
    }
    if feat == "V;IMP;2;PL" {
        return Some(if tf { st } else { format!("{st}ט") });
    }
    let parts: Vec<&str> = feat.split(';').collect();
    if parts.len() < 4 {
        return None;
    }
    let pn = format!("{};{}", parts[2], parts[3]);
    Some(match pn.as_str() {
        "1;SG" => finalize(&st),
        "1;PL" | "3;PL" => inf.to_string(),
        "3;SG" | "2;PL" => {
            if tf {
                st
            } else {
                format!("{st}ט")
            }
        }
        "2;SG" => {
            if lb == Some('ס') {
                format!("{st}ט") // -s + -st degeminates
            } else {
                format!("{st}סט")
            }
        }
        _ => return None,
    })
}

/// A conjugatable Yiddish verb, cited by its infinitive.
#[derive(Debug, Clone)]
pub struct Verb {
    citation: String,
    overrides: HashMap<&'static str, &'static str>,
}

impl Verb {
    /// Build a verb from its infinitive (ending ־ן / ־ען).
    pub fn from_infinitive(citation: &str) -> Result<Self, Error> {
        let cit = citation.trim().to_string();
        if cit.is_empty() || cit.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        if !cit.ends_with("ן") && overrides(&cit).is_none() {
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

    /// The form for a feature bundle: a mined override if one exists, else the
    /// productive rule.
    pub fn form(&self, feature: &str) -> Option<String> {
        if let Some(f) = self.overrides.get(feature) {
            return Some((*f).to_string());
        }
        productive(&self.citation, feature)
    }
}

/// A compact conjugation table — the synthetic core, shared by the WebAssembly
/// and Python bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// Infinitive (the citation).
    pub infinitive: String,
    /// Present indicative, 1sg/2sg/3sg/1pl/2pl/3pl.
    pub present: Vec<Option<String>>,
    /// Imperative 2sg/2pl.
    pub imperative: Vec<Option<String>>,
    /// Present participle.
    pub present_participle: Option<String>,
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
                "V;PRS;1;SG",
                "V;PRS;2;SG",
                "V;PRS;3;SG",
                "V;PRS;1;PL",
                "V;PRS;2;PL",
                "V;PRS;3;PL",
            ]),
            imperative: many(&["V;IMP;2;SG", "V;IMP;2;PL"]),
            present_participle: v.form("V.PTCP;PRS"),
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
    fn regular_weak() {
        // בענטשן "to bless": stem בענטש.
        let b = v("בענטשן");
        assert_eq!(b.form("V;PRS;1;SG").unwrap(), "בענטש");
        assert_eq!(b.form("V;PRS;2;SG").unwrap(), "בענטשסט");
        assert_eq!(b.form("V;PRS;3;SG").unwrap(), "בענטשט");
        assert_eq!(b.form("V;PRS;1;PL").unwrap(), "בענטשן");
        assert_eq!(b.form("V;IMP;2;SG").unwrap(), "בענטש");
        assert_eq!(b.form("V;IMP;2;PL").unwrap(), "בענטשט");
        assert_eq!(b.form("V.PTCP;PRS").unwrap(), "בענטשנדיק");
        assert_eq!(b.form("V.PTCP;PST").unwrap(), "געבענטשט");
        assert_eq!(b.form("V;NFIN").unwrap(), "בענטשן");
    }

    #[test]
    fn final_letter_and_sibilant() {
        // קעמפֿן "to fight": stem קעמפֿ; 1sg finalizes פֿ → ף.
        let k = v("קעמפֿן");
        assert_eq!(k.form("V;PRS;1;SG").unwrap(), "קעמף");
        assert_eq!(k.form("V;PRS;2;SG").unwrap(), "קעמפֿסט");
        // ניסן "to sneeze": stem ניס; -s + -st degeminates.
        let n = v("ניסן");
        assert_eq!(n.form("V;PRS;2;SG").unwrap(), "ניסט");
    }

    #[test]
    fn strong_override() {
        // זינגען "to sing": strong past participle mined from the oracle.
        let z = v("זינגען");
        assert_eq!(z.form("V.PTCP;PST").unwrap(), "געזונגען");
    }
}
