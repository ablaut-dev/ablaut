//! Tatar conjugation. Kazan Tatar (Cyrillic) is an agglutinative Turkic
//! (Kipchak) language with two-way backness vowel harmony. It is cited
//! here by its verbal noun in -у / -ү; the stem is that citation minus its
//! final vowel. The synthetic core — the present (-а / -ә, contracting to
//! -ый / -и after an а/ә stem), the definite past (-ды / -де, devoicing to
//! -ты / -те after a voiceless consonant), the aorist / indefinite future
//! (bare -р after an а/ә stem, else high -ыр / -ер for a polysyllabic stem
//! and low -ар / -әр for a monosyllabic one), the conditional (-са / -сә)
//! and the two imperatives — is generated productively by harmony, with
//! two harmonic vowels A ∈ {а, ә} and H ∈ {ы, е} and the agreement
//! suffixes. Stems whose aorist is lexically irregular (бар → барыр,
//! бул → булыр), or that otherwise deviate (тиү past тите, йөрү present
//! йөри), carry a mined row in `data/tat/parts.tsv`. Verified against one
//! oracle (kaikki.org — the UniMorph Tatar table is Latin-script and does
//! not overlap the Cyrillic paradigm): Beta. The perfect (-ган), the
//! definite future (-ачак), the optative and the negative are a later pass.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The stored cells, in the column order of `data/tat/parts.tsv`.
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
    "V;COND;1;SG",
    "V;COND;2;SG",
    "V;COND;3;SG",
    "V;COND;1;PL",
    "V;COND;2;PL",
    "V;COND;3;PL",
    "V;IMP;2;SG",
    "V;IMP;2;PL",
    "V;NFIN",
];

/// Why a form cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Tatar verbal-noun citation.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Tatar verbal noun")
    }
}

static PARTS_TSV: &str = include_str!("../data/tat/parts.tsv");

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

const VOWELS: &str = "аоуыяюәөүеэи";
const BACK: &str = "аоуыяю";
const VOICELESS: &str = "кпстфхцчшщ";

/// Backness harmony from a stem: (low vowel A, high vowel H).
fn harmony(stem: &str) -> (char, char) {
    let v = stem
        .chars()
        .rev()
        .find(|c| VOWELS.contains(*c))
        .unwrap_or('а');
    if BACK.contains(v) {
        ('а', 'ы')
    } else {
        ('ә', 'е')
    }
}

/// The productive harmony-generated form, or None for a non-verb citation.
fn productive(cit: &str, feat: &str) -> Option<String> {
    if !(cit.ends_with('у') || cit.ends_with('ү')) {
        return None;
    }
    if feat == "V;NFIN" {
        return Some(cit.to_string());
    }
    // Stem = verbal noun minus its final -у / -ү.
    let n = cit.chars().count();
    let stem: String = cit.chars().take(n - 1).collect();
    if stem.is_empty() {
        return None;
    }
    let (a, h) = harmony(&stem);
    let last = stem.chars().next_back().unwrap();
    // Only the low vowels а/ә contract before the present and take the bare
    // -р aorist; и/у/ү-final stems behave like consonant-final ones.
    let vowel_final = last == 'а' || last == 'ә';

    // Present-tense agreement (the marker -а/-ә is already attached).
    let prs = |pn: &str| -> String {
        match pn {
            "1;SG" => "м".into(),
            "2;SG" => format!("с{h}ң"),
            "3;SG" => String::new(),
            "1;PL" => format!("б{h}з"),
            "2;PL" => format!("с{h}з"),
            "3;PL" => format!("л{a}р"),
            _ => String::new(),
        }
    };
    // Aorist / future agreement (copular: 1sg -мын / -мен).
    let fut = |pn: &str| -> String {
        if pn == "1;SG" {
            format!("м{h}н")
        } else {
            prs(pn)
        }
    };
    // Past / conditional agreement.
    let past = |pn: &str| -> String {
        match pn {
            "1;SG" => "м".into(),
            "2;SG" => "ң".into(),
            "3;SG" => String::new(),
            "1;PL" => "к".into(),
            "2;PL" => format!("г{h}з"),
            "3;PL" => format!("л{a}р"),
            _ => String::new(),
        }
    };

    if feat == "V;IMP;2;SG" {
        return Some(stem);
    }
    if feat == "V;IMP;2;PL" {
        let g = if a == 'а' { "гыз" } else { "гез" };
        return Some(if vowel_final {
            format!("{stem}{g}")
        } else {
            format!("{stem}{h}{g}")
        });
    }

    let parts: Vec<&str> = feat.split(';').collect();
    let tense = parts[1];
    let pn = format!("{};{}", parts[2], parts[3]);
    Some(match tense {
        "PRS" => {
            let base = if vowel_final {
                let cut: String = stem.chars().take(stem.chars().count() - 1).collect();
                let hi = if a == 'а' { "ый" } else { "и" };
                format!("{cut}{hi}")
            } else {
                format!("{stem}{a}")
            };
            format!("{base}{}", prs(&pn))
        }
        "PST" => {
            let d = if VOICELESS.contains(last) { 'т' } else { 'д' };
            format!("{stem}{d}{h}{}", past(&pn))
        }
        "COND" => format!("{stem}с{a}{}", past(&pn)),
        "FUT" => {
            // Aorist vowel: bare -р after an а/ә stem, else high -ыр/-ер for
            // a polysyllabic stem and low -ар/-әр for a monosyllabic one.
            let marker = if vowel_final {
                "р".to_string()
            } else {
                let nvowels = stem.chars().filter(|c| VOWELS.contains(*c)).count();
                let v = if nvowels >= 2 { h } else { a };
                format!("{v}р")
            };
            format!("{stem}{marker}{}", fut(&pn))
        }
        _ => return None,
    })
}

/// A conjugatable Tatar verb, cited by its verbal noun (-у / -ү).
#[derive(Debug, Clone)]
pub struct Verb {
    citation: String,
    overrides: HashMap<&'static str, &'static str>,
}

impl Verb {
    /// Build a verb from its verbal-noun citation (-у / -ү).
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
        if !cit.ends_with('у') && !cit.ends_with('ү') && overrides(&cit).is_none() {
            return Err(Error::NotAVerb);
        }
        let over = overrides(&cit).unwrap_or_default();
        Ok(Self {
            citation: cit,
            overrides: over,
        })
    }

    /// The citation form (the verbal noun).
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
    /// Verbal noun (the citation).
    pub infinitive: String,
    /// Present, 1sg/2sg/3sg/1pl/2pl/3pl.
    pub present: Vec<Option<String>>,
    /// Definite past, same order.
    pub past: Vec<Option<String>>,
    /// Aorist / indefinite future, same order.
    pub future: Vec<Option<String>>,
    /// Conditional, same order.
    pub conditional: Vec<Option<String>>,
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
            conditional: many(&[
                "V;COND;1;SG",
                "V;COND;2;SG",
                "V;COND;3;SG",
                "V;COND;1;PL",
                "V;COND;2;PL",
                "V;COND;3;PL",
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
    fn back_consonant_stem() {
        // яз- (write): back, consonant-final, voiced final → -ды past.
        let z = v("язу");
        assert_eq!(z.form("V;PRS;1;SG").unwrap(), "язам");
        assert_eq!(z.form("V;PRS;3;SG").unwrap(), "яза");
        assert_eq!(z.form("V;PRS;3;PL").unwrap(), "язалар");
        assert_eq!(z.form("V;PST;1;SG").unwrap(), "яздым");
        assert_eq!(z.form("V;FUT;3;SG").unwrap(), "язар"); // monosyllabic → low
        assert_eq!(z.form("V;COND;1;SG").unwrap(), "язсам");
        assert_eq!(z.form("V;NFIN").unwrap(), "язу");
    }

    #[test]
    fn front_and_devoicing() {
        // үс- (grow): front, voiceless final с → -те past; monosyllabic → -әр.
        let u = v("үсү");
        assert_eq!(u.form("V;PRS;1;SG").unwrap(), "үсәм");
        assert_eq!(u.form("V;PST;3;SG").unwrap(), "үсте");
        assert_eq!(u.form("V;FUT;3;SG").unwrap(), "үсәр");
        assert_eq!(u.form("V;COND;3;SG").unwrap(), "үссә");
        // ук- (read): back, voiceless к → -ты past.
        let k = v("уку");
        assert_eq!(k.form("V;PST;1;SG").unwrap(), "уктым");
        assert_eq!(k.form("V;FUT;3;SG").unwrap(), "укар");
    }

    #[test]
    fn vowel_final_and_polysyllabic() {
        // тыңла- (listen): back а-final → present contracts to -ый, aorist -р.
        let t = v("тыңлау");
        assert_eq!(t.form("V;PRS;3;SG").unwrap(), "тыңлый");
        assert_eq!(t.form("V;PST;3;SG").unwrap(), "тыңлады");
        assert_eq!(t.form("V;FUT;3;SG").unwrap(), "тыңлар");
        // адаш- (get lost): polysyllabic consonant stem → high aorist -ыр.
        let a = v("адашу");
        assert_eq!(a.form("V;FUT;3;SG").unwrap(), "адашыр");
        assert_eq!(a.form("V;PST;3;SG").unwrap(), "адашты");
    }

    #[test]
    fn mined_override() {
        // бар- (go): monosyllabic but lexically high aorist → mined.
        let b = v("бару");
        assert_eq!(b.form("V;FUT;3;SG").unwrap(), "барыр");
        assert_eq!(b.form("V;PRS;1;SG").unwrap(), "барам"); // present still productive
    }
}
