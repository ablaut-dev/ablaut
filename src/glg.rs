//! Galician conjugation. Galician is a Romance language cited by its
//! infinitive, with a clean three-class paradigm keyed by the infinitive
//! ending — -ar (falar), -er (abater) or -ir (partir). From the stem (the
//! infinitive minus its two-letter ending) a fixed table of endings
//! produces the whole synthetic core: the ten person/number tense blocks
//! — present, imperfect, preterite, synthetic pluperfect, future and
//! conditional (indicative); present, imperfect and future subjunctive;
//! and the inflected (personal) infinitive — plus the 2sg/2pl imperative,
//! the gerund and the four past-participle forms. Irregular verbs (ser,
//! ir, ter, facer, dicir, …), orthographic stem changes (-car/-gar/-cer,
//! …) and verbs attested only in reintegrationist orthography carry a
//! mined row in `data/glg/parts.tsv`. Verified against one oracle
//! (kaikki.org): Beta. Compound (periphrastic) tenses are a later pass.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The stored cells, in the column order of `data/glg/parts.tsv`.
pub const CELLS: [&str; 68] = [
    "V;NFIN",
    "V;GER",
    "V;PTCP;MASC;SG",
    "V;PTCP;FEM;SG",
    "V;PTCP;MASC;PL",
    "V;PTCP;FEM;PL",
    "V;IND;PRS;1;SG",
    "V;IND;PRS;1;PL",
    "V;IND;PRS;2;SG",
    "V;IND;PRS;2;PL",
    "V;IND;PRS;3;SG",
    "V;IND;PRS;3;PL",
    "V;IND;IPFV;1;SG",
    "V;IND;IPFV;1;PL",
    "V;IND;IPFV;2;SG",
    "V;IND;IPFV;2;PL",
    "V;IND;IPFV;3;SG",
    "V;IND;IPFV;3;PL",
    "V;IND;PRET;1;SG",
    "V;IND;PRET;1;PL",
    "V;IND;PRET;2;SG",
    "V;IND;PRET;2;PL",
    "V;IND;PRET;3;SG",
    "V;IND;PRET;3;PL",
    "V;IND;PLUP;1;SG",
    "V;IND;PLUP;1;PL",
    "V;IND;PLUP;2;SG",
    "V;IND;PLUP;2;PL",
    "V;IND;PLUP;3;SG",
    "V;IND;PLUP;3;PL",
    "V;IND;FUT;1;SG",
    "V;IND;FUT;1;PL",
    "V;IND;FUT;2;SG",
    "V;IND;FUT;2;PL",
    "V;IND;FUT;3;SG",
    "V;IND;FUT;3;PL",
    "V;COND;1;SG",
    "V;COND;1;PL",
    "V;COND;2;SG",
    "V;COND;2;PL",
    "V;COND;3;SG",
    "V;COND;3;PL",
    "V;SBJV;PRS;1;SG",
    "V;SBJV;PRS;1;PL",
    "V;SBJV;PRS;2;SG",
    "V;SBJV;PRS;2;PL",
    "V;SBJV;PRS;3;SG",
    "V;SBJV;PRS;3;PL",
    "V;SBJV;PST;1;SG",
    "V;SBJV;PST;1;PL",
    "V;SBJV;PST;2;SG",
    "V;SBJV;PST;2;PL",
    "V;SBJV;PST;3;SG",
    "V;SBJV;PST;3;PL",
    "V;SBJV;FUT;1;SG",
    "V;SBJV;FUT;1;PL",
    "V;SBJV;FUT;2;SG",
    "V;SBJV;FUT;2;PL",
    "V;SBJV;FUT;3;SG",
    "V;SBJV;FUT;3;PL",
    "V;INF;1;SG",
    "V;INF;1;PL",
    "V;INF;2;SG",
    "V;INF;2;PL",
    "V;INF;3;SG",
    "V;INF;3;PL",
    "V;IMP;2;SG",
    "V;IMP;2;PL",
];

/// Why a lemma cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Galician infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Galician infinitive")
    }
}

static PARTS_TSV: &str = include_str!("../data/glg/parts.tsv");

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

/// The six endings of a person/number tense block, in the order
/// 1sg, 2sg, 3sg, 1pl, 2pl, 3pl.
type Block = [&'static str; 6];

/// The productive ending block for (class, tense), or None if the tense
/// is not one this engine covers. Mirrors mine.py's per-class tables.
fn block(cls: &str, tense: &str) -> Option<Block> {
    Some(match (cls, tense) {
        // -ar
        ("ar", "V;IND;PRS") => ["o", "as", "a", "amos", "ades", "an"],
        ("ar", "V;IND;IPFV") => ["aba", "abas", "aba", "abamos", "abades", "aban"],
        ("ar", "V;IND;PRET") => ["ei", "aches", "ou", "amos", "astes", "aron"],
        ("ar", "V;IND;PLUP") => ["ara", "aras", "ara", "aramos", "arades", "aran"],
        ("ar", "V;IND;FUT") => ["arei", "arás", "ará", "aremos", "aredes", "arán"],
        ("ar", "V;COND") => ["aría", "arías", "aría", "ariamos", "ariades", "arían"],
        ("ar", "V;SBJV;PRS") => ["e", "es", "e", "emos", "edes", "en"],
        ("ar", "V;SBJV;PST") => ["ase", "ases", "ase", "ásemos", "ásedes", "asen"],
        ("ar", "V;SBJV;FUT") => ["ar", "ares", "ar", "armos", "ardes", "aren"],
        ("ar", "V;INF") => ["ar", "ares", "ar", "armos", "ardes", "aren"],
        // -er
        ("er", "V;IND;PRS") => ["o", "es", "e", "emos", "edes", "en"],
        ("er", "V;IND;IPFV") => ["ía", "ías", "ía", "iamos", "iades", "ían"],
        ("er", "V;IND;PRET") => ["ín", "iches", "eu", "emos", "estes", "eron"],
        ("er", "V;IND;PLUP") => ["era", "eras", "era", "eramos", "erades", "eran"],
        ("er", "V;IND;FUT") => ["erei", "erás", "erá", "eremos", "eredes", "erán"],
        ("er", "V;COND") => ["ería", "erías", "ería", "eriamos", "eriades", "erían"],
        ("er", "V;SBJV;PRS") => ["a", "as", "a", "amos", "ades", "an"],
        ("er", "V;SBJV;PST") => ["ese", "eses", "ese", "ésemos", "ésedes", "esen"],
        ("er", "V;SBJV;FUT") => ["er", "eres", "er", "ermos", "erdes", "eren"],
        ("er", "V;INF") => ["er", "eres", "er", "ermos", "erdes", "eren"],
        // -ir
        ("ir", "V;IND;PRS") => ["o", "es", "e", "imos", "ides", "en"],
        ("ir", "V;IND;IPFV") => ["ía", "ías", "ía", "iamos", "iades", "ían"],
        ("ir", "V;IND;PRET") => ["ín", "iches", "iu", "imos", "istes", "iron"],
        ("ir", "V;IND;PLUP") => ["ira", "iras", "ira", "iramos", "irades", "iran"],
        ("ir", "V;IND;FUT") => ["irei", "irás", "irá", "iremos", "iredes", "irán"],
        ("ir", "V;COND") => ["iría", "irías", "iría", "iriamos", "iriades", "irían"],
        ("ir", "V;SBJV;PRS") => ["a", "as", "a", "amos", "ades", "an"],
        ("ir", "V;SBJV;PST") => ["ise", "ises", "ise", "ísemos", "ísedes", "isen"],
        ("ir", "V;SBJV;FUT") => ["ir", "ires", "ir", "irmos", "irdes", "iren"],
        ("ir", "V;INF") => ["ir", "ires", "ir", "irmos", "irdes", "iren"],
        _ => return None,
    })
}

/// The productive ending for a class and full feature bundle.
fn ending(cls: &str, feat: &str) -> Option<&'static str> {
    match feat {
        "V;NFIN" => {
            return Some(match cls {
                "ar" => "ar",
                "er" => "er",
                _ => "ir",
            })
        }
        "V;GER" => {
            return Some(match cls {
                "ar" => "ando",
                "er" => "endo",
                _ => "indo",
            })
        }
        "V;PTCP;MASC;SG" => return Some(if cls == "ar" { "ado" } else { "ido" }),
        "V;PTCP;FEM;SG" => return Some(if cls == "ar" { "ada" } else { "ida" }),
        "V;PTCP;MASC;PL" => return Some(if cls == "ar" { "ados" } else { "idos" }),
        "V;PTCP;FEM;PL" => return Some(if cls == "ar" { "adas" } else { "idas" }),
        "V;IMP;2;SG" => return Some(if cls == "ar" { "a" } else { "e" }),
        "V;IMP;2;PL" => {
            return Some(match cls {
                "ar" => "ade",
                "er" => "ede",
                _ => "ide",
            })
        }
        _ => {}
    }
    let parts: Vec<&str> = feat.split(';').collect();
    if parts.len() < 3 {
        return None;
    }
    let (n, p) = (parts[parts.len() - 1], parts[parts.len() - 2]);
    let tense = parts[..parts.len() - 2].join(";");
    let idx = match (p, n) {
        ("1", "SG") => 0,
        ("2", "SG") => 1,
        ("3", "SG") => 2,
        ("1", "PL") => 3,
        ("2", "PL") => 4,
        ("3", "PL") => 5,
        _ => return None,
    };
    block(cls, &tense).map(|b| b[idx])
}

/// The productive rule-generated form, or None if the class/cell is
/// outside this engine's scope. Mirrors mine.py's `produce()` exactly.
fn productive(cit: &str, feat: &str) -> Option<String> {
    let n = cit.chars().count();
    if n < 3 {
        return None;
    }
    let cls: String = cit.chars().skip(n - 2).collect();
    if !matches!(cls.as_str(), "ar" | "er" | "ir") {
        return None;
    }
    let stem: String = cit.chars().take(n - 2).collect();
    let end = ending(&cls, feat)?;
    Some(format!("{stem}{end}"))
}

/// A conjugatable Galician verb, cited by its infinitive.
#[derive(Debug, Clone)]
pub struct Verb {
    citation: String,
    overrides: HashMap<&'static str, &'static str>,
}

impl Verb {
    /// Build a verb from its infinitive (-ar / -er / -ir, or a reflexive
    /// -se form).
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
        // A Galician infinitive ends in -r (falar, abater, partir, argüír,
        // compor) or, reflexive, in -se (chamarse).
        if !(cit.ends_with('r') || cit.ends_with("se")) && overrides(&cit).is_none() {
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
    /// the productive rule (None for cells outside scope).
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
    /// Gerund.
    pub gerund: Option<String>,
    /// Past participle: masc.sg / fem.sg / masc.pl / fem.pl.
    pub participle: Vec<Option<String>>,
    /// Present indicative, 1sg/2sg/3sg/1pl/2pl/3pl.
    pub present: Vec<Option<String>>,
    /// Imperfect indicative, same order.
    pub imperfect: Vec<Option<String>>,
    /// Preterite indicative, same order.
    pub preterite: Vec<Option<String>>,
    /// Synthetic pluperfect indicative, same order.
    pub pluperfect: Vec<Option<String>>,
    /// Future indicative, same order.
    pub future: Vec<Option<String>>,
    /// Conditional, same order.
    pub conditional: Vec<Option<String>>,
    /// Present subjunctive, same order.
    pub present_subjunctive: Vec<Option<String>>,
    /// Imperfect subjunctive, same order.
    pub imperfect_subjunctive: Vec<Option<String>>,
    /// Future subjunctive, same order.
    pub future_subjunctive: Vec<Option<String>>,
    /// Inflected (personal) infinitive, same order.
    pub personal_infinitive: Vec<Option<String>>,
    /// Imperative 2sg/2pl.
    pub imperative: Vec<Option<String>>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let many = |feats: &[&str]| feats.iter().map(|f| v.form(f)).collect();
        let six = |t: &str| -> Vec<Option<String>> {
            ["1;SG", "2;SG", "3;SG", "1;PL", "2;PL", "3;PL"]
                .iter()
                .map(|pn| v.form(&format!("{t};{pn}")))
                .collect()
        };
        Self {
            infinitive: v.citation().to_string(),
            gerund: v.form("V;GER"),
            participle: many(&[
                "V;PTCP;MASC;SG",
                "V;PTCP;FEM;SG",
                "V;PTCP;MASC;PL",
                "V;PTCP;FEM;PL",
            ]),
            present: six("V;IND;PRS"),
            imperfect: six("V;IND;IPFV"),
            preterite: six("V;IND;PRET"),
            pluperfect: six("V;IND;PLUP"),
            future: six("V;IND;FUT"),
            conditional: six("V;COND"),
            present_subjunctive: six("V;SBJV;PRS"),
            imperfect_subjunctive: six("V;SBJV;PST"),
            future_subjunctive: six("V;SBJV;FUT"),
            personal_infinitive: six("V;INF"),
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
    fn regular_ar() {
        let f = v("falar"); // stem fal-
        assert_eq!(f.form("V;IND;PRS;1;SG").unwrap(), "falo");
        assert_eq!(f.form("V;IND;PRS;3;PL").unwrap(), "falan");
        assert_eq!(f.form("V;IND;PRET;1;SG").unwrap(), "falei");
        assert_eq!(f.form("V;IND;FUT;2;SG").unwrap(), "falarás");
        assert_eq!(f.form("V;SBJV;PRS;1;SG").unwrap(), "fale");
        assert_eq!(f.form("V;SBJV;PST;1;PL").unwrap(), "falásemos");
        assert_eq!(f.form("V;IMP;2;PL").unwrap(), "falade");
        assert_eq!(f.form("V;GER").unwrap(), "falando");
        assert_eq!(f.form("V;PTCP;FEM;PL").unwrap(), "faladas");
        assert_eq!(f.form("V;NFIN").unwrap(), "falar");
    }

    #[test]
    fn regular_er_ir() {
        let a = v("abater"); // -er, stem abat-
        assert_eq!(a.form("V;IND;PRS;1;PL").unwrap(), "abatemos");
        assert_eq!(a.form("V;IND;PRET;3;SG").unwrap(), "abateu");
        assert_eq!(a.form("V;INF;2;SG").unwrap(), "abateres");
        assert_eq!(a.form("V;IMP;2;PL").unwrap(), "abatede");
        let p = v("partir"); // -ir, stem part-
        assert_eq!(p.form("V;IND;PRS;1;PL").unwrap(), "partimos");
        assert_eq!(p.form("V;IND;PRS;2;PL").unwrap(), "partides");
        assert_eq!(p.form("V;IND;PRET;3;SG").unwrap(), "partiu");
        assert_eq!(p.form("V;GER").unwrap(), "partindo");
    }

    #[test]
    fn irregular_uses_override() {
        // ser is heavily irregular; its forms must come from parts.tsv.
        let s = v("ser");
        assert_eq!(s.form("V;IND;PRS;1;SG").unwrap(), "son");
    }
}
