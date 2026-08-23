//! Kazakh conjugation. Kazakh is an agglutinative Kipchak Turkic language
//! cited here by its -у / -ю verbal-noun infinitive (алу, беру, сөйлеу,
//! тою). The synthetic core — the aorist / present-future (-а/-е/-й), the
//! definite past / preterite (-ды/-ті) and the presumptive future
//! (-ар/-ер/-р), each across eight person/number cells (the 2nd person
//! splitting informal / formal) — is generated productively from the stem
//! (the infinitive minus its final -у, with -ю → -й) by two-way vowel
//! harmony: the low vowel A ∈ {а, е}, the high vowel Y ∈ {ы, і}, the
//! plural K ∈ {қ, к} and the 2pl D-A-r ∈ {дар, дер}, chosen by the
//! backness of the stem's last vowel. The preterite additionally devoices
//! a stem-final voiced stop before its consonant-initial suffix (б→п, г→к,
//! ғ→қ: табу → таптым). The imperative and the infinitive are generated
//! too, though kaikki attests no gold for them.
//!
//! Verbs whose infinitive drops a stem vowel (оқу ← оқы, суу ← суы, баю ←
//! байы) and other lexical irregulars carry a mined row in
//! `data/kaz/parts.tsv`. Verified against one oracle (kaikki.org) — the
//! UniMorph kaz table has no verb rows — so: Beta. The perfect,
//! renarrative, habitual-past, intentive, conditional and optative
//! (partly periphrastic) tenses are a later pass.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The stored cells, in the column order of `data/kaz/parts.tsv`.
pub const CELLS: [&str; 28] = [
    "V;AOR;1;SG",
    "V;AOR;2;SG;INFM",
    "V;AOR;2;SG;FORM",
    "V;AOR;3;SG",
    "V;AOR;1;PL",
    "V;AOR;2;PL;INFM",
    "V;AOR;2;PL;FORM",
    "V;AOR;3;PL",
    "V;PST;1;SG",
    "V;PST;2;SG;INFM",
    "V;PST;2;SG;FORM",
    "V;PST;3;SG",
    "V;PST;1;PL",
    "V;PST;2;PL;INFM",
    "V;PST;2;PL;FORM",
    "V;PST;3;PL",
    "V;FUT;1;SG",
    "V;FUT;2;SG;INFM",
    "V;FUT;2;SG;FORM",
    "V;FUT;3;SG",
    "V;FUT;1;PL",
    "V;FUT;2;PL;INFM",
    "V;FUT;2;PL;FORM",
    "V;FUT;3;PL",
    "V;IMP;2;SG",
    "V;IMP;2;SG;FORM",
    "V;IMP;2;PL",
    "V;NFIN",
];

/// Why a form cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Kazakh infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Kazakh infinitive")
    }
}

static PARTS_TSV: &str = include_str!("../data/kaz/parts.tsv");

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

const VOWELS: &str = "аәеоөұүыіэиуёюя";
const BACK: &str = "аоұыуёя";
/// Open vowels that take the -й aorist connective (not у, и, й).
const CONN: &str = "аәеоөұүыіэ";
const VOICELESS: &str = "кқптсшчцфхһщ";

/// All chars of `s` but the last.
fn drop_last(s: &str) -> String {
    let mut c = s.chars();
    c.next_back();
    c.as_str().to_string()
}

/// The last char of `s`.
fn last_char(s: &str) -> Option<char> {
    s.chars().next_back()
}

/// The stem: the infinitive minus its final -у, with -ю → -й.
fn stem_of(cit: &str) -> Option<String> {
    match last_char(cit)? {
        'ю' => Some(format!("{}й", drop_last(cit))),
        'у' => Some(drop_last(cit)),
        _ => None,
    }
}

/// Devoice a stem-final voiced stop before the preterite's consonant.
fn devoice(c: char) -> Option<char> {
    match c {
        'б' => Some('п'),
        'г' => Some('к'),
        'ғ' => Some('қ'),
        _ => None,
    }
}

/// Two-way vowel harmony from a stem: (low A, high Y, plural K, 2pl DAr).
fn harmony(stem: &str) -> (char, char, char, &'static str) {
    let v = stem
        .chars()
        .rev()
        .find(|c| VOWELS.contains(*c))
        .unwrap_or('а');
    if BACK.contains(v) {
        ('а', 'ы', 'қ', "дар")
    } else {
        ('е', 'і', 'к', "дер")
    }
}

/// The productive harmony-generated form, or None for a non-verb citation.
fn productive(cit: &str, feat: &str) -> Option<String> {
    let stem = stem_of(cit)?;
    if feat == "V;NFIN" {
        return Some(cit.to_string());
    }
    let (a, y, k, dar) = harmony(&stem);
    let last = last_char(&stem)?;
    let parts: Vec<&str> = feat.split(';').collect();
    let tense = *parts.get(1)?;
    let pn = parts[2..].join(";");

    // The full personal set (aorist / presumptive), attached after a vowel
    // or р; `third` is the 3rd-person ending (present -ды/-ді vs empty).
    let full = |third: &str| -> Option<String> {
        Some(match pn.as_str() {
            "1;SG" => format!("м{y}н"),
            "2;SG;INFM" => format!("с{y}ң"),
            "2;SG;FORM" => format!("с{y}з"),
            "3;SG" => third.to_string(),
            "1;PL" => format!("м{y}з"),
            "2;PL;INFM" => format!("с{y}ң{dar}"),
            "2;PL;FORM" => format!("с{y}з{dar}"),
            "3;PL" => third.to_string(),
            _ => return None,
        })
    };

    match tense {
        "IMP" => Some(match pn.as_str() {
            "2;SG" => stem.clone(),
            "2;SG;FORM" => {
                let link = if VOWELS.contains(last) {
                    String::new()
                } else {
                    y.to_string()
                };
                format!("{stem}{link}ң{y}з")
            }
            "2;PL" => {
                let link = if VOWELS.contains(last) {
                    String::new()
                } else {
                    y.to_string()
                };
                format!("{stem}{link}ң{dar}")
            }
            _ => return None,
        }),
        "AOR" => {
            let base = if CONN.contains(last) {
                if last == 'ы' || last == 'і' {
                    format!("{}и", drop_last(&stem))
                } else {
                    format!("{stem}й")
                }
            } else if last == 'й' && a == 'а' {
                format!("{}я", drop_last(&stem))
            } else {
                format!("{stem}{a}")
            };
            Some(format!("{base}{}", full(&format!("д{y}"))?))
        }
        "PST" => {
            let st = match devoice(last) {
                Some(v) => format!("{}{v}", drop_last(&stem)),
                None => stem.clone(),
            };
            let d = if VOICELESS.contains(last_char(&st)?) {
                'т'
            } else {
                'д'
            };
            let pbase = format!("{st}{d}{y}");
            let end = match pn.as_str() {
                "1;SG" => "м".to_string(),
                "2;SG;INFM" => "ң".to_string(),
                "2;SG;FORM" => format!("ң{y}з"),
                "3;SG" => String::new(),
                "1;PL" => k.to_string(),
                "2;PL;INFM" => format!("ң{dar}"),
                "2;PL;FORM" => format!("ң{y}з{dar}"),
                "3;PL" => String::new(),
                _ => return None,
            };
            Some(format!("{pbase}{end}"))
        }
        "FUT" => {
            let base = if CONN.contains(last) {
                format!("{stem}р")
            } else if last == 'й' && a == 'а' {
                format!("{}яр", drop_last(&stem))
            } else {
                format!("{stem}{a}р")
            };
            Some(format!("{base}{}", full("")?))
        }
        _ => None,
    }
}

/// A conjugatable Kazakh verb, cited by its infinitive.
#[derive(Debug, Clone)]
pub struct Verb {
    citation: String,
    overrides: HashMap<&'static str, &'static str>,
}

impl Verb {
    /// Build a verb from its infinitive (-у / -ю).
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
        if stem_of(&cit).is_none() && overrides(&cit).is_none() {
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
    /// Aorist / present-future: 1sg/2sg.infm/2sg.form/3sg/1pl/2pl.infm/
    /// 2pl.form/3pl.
    pub aorist: Vec<Option<String>>,
    /// Definite past / preterite, same order.
    pub past: Vec<Option<String>>,
    /// Presumptive future, same order.
    pub future: Vec<Option<String>>,
    /// Imperative 2sg/2sg.form/2pl.
    pub imperative: Vec<Option<String>>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let many = |feats: &[&str]| feats.iter().map(|f| v.form(f)).collect();
        Self {
            infinitive: v.citation().to_string(),
            aorist: many(&[
                "V;AOR;1;SG",
                "V;AOR;2;SG;INFM",
                "V;AOR;2;SG;FORM",
                "V;AOR;3;SG",
                "V;AOR;1;PL",
                "V;AOR;2;PL;INFM",
                "V;AOR;2;PL;FORM",
                "V;AOR;3;PL",
            ]),
            past: many(&[
                "V;PST;1;SG",
                "V;PST;2;SG;INFM",
                "V;PST;2;SG;FORM",
                "V;PST;3;SG",
                "V;PST;1;PL",
                "V;PST;2;PL;INFM",
                "V;PST;2;PL;FORM",
                "V;PST;3;PL",
            ]),
            future: many(&[
                "V;FUT;1;SG",
                "V;FUT;2;SG;INFM",
                "V;FUT;2;SG;FORM",
                "V;FUT;3;SG",
                "V;FUT;1;PL",
                "V;FUT;2;PL;INFM",
                "V;FUT;2;PL;FORM",
                "V;FUT;3;PL",
            ]),
            imperative: many(&["V;IMP;2;SG", "V;IMP;2;SG;FORM", "V;IMP;2;PL"]),
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
        // алу, stem ал-, back unrounded: A=а, Y=ы, K=қ.
        let a = v("алу");
        assert_eq!(a.form("V;AOR;1;SG").unwrap(), "аламын");
        assert_eq!(a.form("V;AOR;3;SG").unwrap(), "алады");
        assert_eq!(a.form("V;AOR;2;SG;FORM").unwrap(), "аласыз");
        assert_eq!(a.form("V;PST;1;SG").unwrap(), "алдым");
        assert_eq!(a.form("V;PST;1;PL").unwrap(), "алдық");
        assert_eq!(a.form("V;FUT;1;SG").unwrap(), "алармын");
        assert_eq!(a.form("V;FUT;3;SG").unwrap(), "алар");
        assert_eq!(a.form("V;IMP;2;SG").unwrap(), "ал");
        assert_eq!(a.form("V;IMP;2;SG;FORM").unwrap(), "алыңыз");
        assert_eq!(a.form("V;NFIN").unwrap(), "алу");
    }

    #[test]
    fn front_stem() {
        // келу, stem кел-, front: A=е, Y=і, K=к.
        let k = v("келу");
        assert_eq!(k.form("V;AOR;1;SG").unwrap(), "келемін");
        assert_eq!(k.form("V;PST;1;PL").unwrap(), "келдік");
        assert_eq!(k.form("V;FUT;1;SG").unwrap(), "келермін");
    }

    #[test]
    fn vowel_final_and_yod_merge() {
        // сөйлеу, stem сөйле-: -й aorist connective, -р future.
        let s = v("сөйлеу");
        assert_eq!(s.form("V;AOR;1;SG").unwrap(), "сөйлеймін");
        assert_eq!(s.form("V;PST;1;SG").unwrap(), "сөйледім");
        assert_eq!(s.form("V;FUT;1;SG").unwrap(), "сөйлермін");
        // тою, stem той-: back й-stem, connective а merges йа→я.
        let t = v("тою");
        assert_eq!(t.form("V;AOR;1;SG").unwrap(), "тоямын");
        assert_eq!(t.form("V;AOR;3;SG").unwrap(), "тояды");
        assert_eq!(t.form("V;FUT;3;SG").unwrap(), "тояр");
        assert_eq!(t.form("V;PST;3;SG").unwrap(), "тойды");
    }

    #[test]
    fn preterite_devoicing() {
        // табу, stem таб- → devoices to тап- before the preterite.
        let t = v("табу");
        assert_eq!(t.form("V;PST;1;SG").unwrap(), "таптым");
        assert_eq!(t.form("V;PST;3;SG").unwrap(), "тапты");
        // but the aorist keeps the voiced allomorph.
        assert_eq!(t.form("V;AOR;3;SG").unwrap(), "табады");
    }

    #[test]
    fn mined_override_wins() {
        // оқу has a hidden stem vowel (оқы), so it is mined.
        let o = v("оқу");
        assert_eq!(o.form("V;AOR;1;SG").unwrap(), "оқимын");
        assert_eq!(o.form("V;PST;1;SG").unwrap(), "оқыдым");
    }
}
