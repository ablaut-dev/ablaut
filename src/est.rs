//! Estonian conjugation. The lemma is the ma-infinitive (rääkima);
//! consonant gradation makes the present stem underivable (räägin),
//! so gradating verbs carry mined rows in `data/est/verbs.tsv` while
//! everything else runs on rules: past from the ma-stem (rääkisin),
//! conditional off the present stem (räägiksin), the t-impersonals
//! off the tud-participle base (räägiti, räägitagu), imperatives off
//! the ma-stem (rääkigu → rääkigem/rääkige).

use std::collections::HashMap;
use std::sync::OnceLock;

/// Grammatical person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Person {
    First,
    Second,
    Third,
}

/// Grammatical number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Singular,
    Plural,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like an Estonian ma-infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Estonian ma-infinitive")
    }
}

/// Mined row: lemma, present 1sg, present 3sg, da-infinitive, past
/// 1sg, nud-participle, tud-participle, imperative 2sg, imperative
/// 3sg (-gu), present impersonal (-takse) ("-" = derive).
static VERBS_TSV: &str = include_str!("../data/est/verbs.tsv");

#[derive(Debug, Clone, Default)]
struct Row {
    p1sg: Option<String>,
    p3sg: Option<String>,
    da: Option<String>,
    pst1sg: Option<String>,
    nud: Option<String>,
    tud: Option<String>,
    imp2: Option<String>,
    gu: Option<String>,
    takse: Option<String>,
    pst3sg: Option<String>,
    v_ptcp: Option<String>,
    p3pl: Option<String>,
}

fn rows() -> &'static HashMap<&'static str, Row> {
    static MAP: OnceLock<HashMap<&'static str, Row>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in VERBS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            let opt = |i: usize| {
                cols.get(i)
                    .filter(|c| **c != "-" && !c.is_empty())
                    .map(|c| (*c).to_string())
            };
            m.insert(
                cols[0],
                Row {
                    p1sg: opt(1),
                    p3sg: opt(2),
                    da: opt(3),
                    pst1sg: opt(4),
                    nud: opt(5),
                    tud: opt(6),
                    imp2: opt(7),
                    gu: opt(8),
                    takse: opt(9),
                    pst3sg: opt(10),
                    v_ptcp: opt(11),
                    p3pl: opt(12),
                },
            );
        }
        m
    })
}

/// A conjugatable Estonian verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    row: Row,
}

impl Verb {
    /// Build a verb from its ma-infinitive.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        // Case-fold: lemmas are lowercase in every oracle of this
        // language (the Abbrechen lesson, generalized).
        let lowered = infinitive.to_lowercase();
        let infinitive = lowered.as_str();
        let inf = infinitive.trim();
        if inf.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf.chars().all(|c| c.is_alphabetic() || c == '-')
            || !inf.ends_with("ma")
        {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf.to_string(),
            row: rows().get(inf).cloned().unwrap_or_default(),
        })
    }

    /// The ma-infinitive as normalized.
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// The ma-stem (rääki-).
    fn ma_stem(&self) -> &str {
        &self.infinitive[..self.infinitive.len() - "ma".len()]
    }

    /// The present (weak-grade) stem: mined 1sg minus -n, else the
    /// ma-stem.
    fn weak(&self) -> String {
        match &self.row.p1sg {
            Some(f) => f.strip_suffix('n').unwrap_or(f).to_string(),
            None => self.ma_stem().to_string(),
        }
    }

    /// The tud-participle (räägitud) and its t-base for the
    /// impersonals.
    fn tud(&self) -> String {
        self.row
            .tud
            .clone()
            .unwrap_or_else(|| format!("{}tud", self.weak()))
    }

    fn t_base(&self) -> String {
        let tud = self.tud();
        tud.strip_suffix("ud").unwrap_or(&tud).to_string()
    }

    /// The nud-participle (rääkinud).
    pub fn nud(&self) -> String {
        self.row
            .nud
            .clone()
            .unwrap_or_else(|| format!("{}nud", self.ma_stem()))
    }

    /// The present indicative.
    pub fn present(&self, person: Person, number: Number) -> String {
        let w = self.weak();
        match (number, person) {
            (Number::Singular, Person::First) => format!("{w}n"),
            (Number::Singular, Person::Second) => format!("{w}d"),
            (Number::Singular, Person::Third) => {
                self.row.p3sg.clone().unwrap_or_else(|| format!("{w}b"))
            }
            (Number::Plural, Person::First) => format!("{w}me"),
            (Number::Plural, Person::Second) => format!("{w}te"),
            (Number::Plural, Person::Third) => {
                self.row.p3pl.clone().unwrap_or_else(|| format!("{w}vad"))
            }
        }
    }

    /// The past indicative (rääkisin; the 3sg drops the -i of si-
    /// pasts: rääkis, but i-pasts keep it: tuli).
    pub fn past(&self, person: Person, number: Number) -> String {
        let base = match &self.row.pst1sg {
            Some(f) => f.strip_suffix('n').unwrap_or(f).to_string(),
            None => format!("{}si", self.ma_stem()),
        };
        match (number, person) {
            (Number::Singular, Person::First) => format!("{base}n"),
            (Number::Singular, Person::Second) | (Number::Plural, Person::Third) => {
                format!("{base}d")
            }
            (Number::Singular, Person::Third) => {
                if let Some(f) = &self.row.pst3sg {
                    return f.clone();
                }
                if base.ends_with("si") {
                    base[..base.len() - 1].to_string()
                } else {
                    base
                }
            }
            (Number::Plural, Person::First) => format!("{base}me"),
            (Number::Plural, Person::Second) => format!("{base}te"),
        }
    }

    /// The conditional present (räägiksin).
    pub fn conditional(&self, person: Person, number: Number) -> String {
        let w = self.weak();
        let ending = match (number, person) {
            (Number::Singular, Person::First) => "ksin",
            (Number::Singular, Person::Second) | (Number::Plural, Person::Third) => "ksid",
            (Number::Singular, Person::Third) => "ks",
            (Number::Plural, Person::First) => "ksime",
            (Number::Plural, Person::Second) => "ksite",
        };
        format!("{w}{ending}")
    }

    /// The conditional perfect (rääkinuksin), off the nud-participle.
    pub fn conditional_perfect(&self, person: Person, number: Number) -> String {
        let nud = self.nud();
        let base = nud.strip_suffix('d').unwrap_or(&nud);
        let ending = match (number, person) {
            (Number::Singular, Person::First) => "ksin",
            (Number::Singular, Person::Second) | (Number::Plural, Person::Third) => "ksid",
            (Number::Singular, Person::Third) => "ks",
            (Number::Plural, Person::First) => "ksime",
            (Number::Plural, Person::Second) => "ksite",
        };
        format!("{base}{ending}")
    }

    /// An imperative form (2sg räägi, 3sg/3pl rääkigu, 1pl rääkigem,
    /// 2pl rääkige).
    pub fn imperative(&self, person: Person, number: Number) -> Option<String> {
        let gu = self
            .row
            .gu
            .clone()
            .unwrap_or_else(|| format!("{}gu", self.ma_stem()));
        Some(match (number, person) {
            (Number::Singular, Person::Second) => {
                self.row.imp2.clone().unwrap_or_else(|| self.weak())
            }
            (_, Person::Third) => gu,
            (Number::Plural, Person::First) => {
                format!("{}em", &gu[..gu.len() - 1])
            }
            (Number::Plural, Person::Second) => {
                format!("{}e", &gu[..gu.len() - 1])
            }
            _ => return None,
        })
    }

    /// An impersonal form off the t-base (räägitakse, räägiti,
    /// räägitaks, räägitagu).
    pub fn impersonal(&self, slot: ImpersonalSlot) -> String {
        let t = self.t_base();
        match slot {
            ImpersonalSlot::Present => self.row.takse.clone().unwrap_or_else(|| format!("{t}akse")),
            ImpersonalSlot::Past => format!("{t}i"),
            ImpersonalSlot::Conditional => format!("{t}aks"),
            ImpersonalSlot::Imperative => format!("{t}agu"),
        }
    }

    /// A non-finite form.
    pub fn nonfinite(&self, slot: NonfiniteSlot) -> String {
        let ma = self.ma_stem();
        match slot {
            NonfiniteSlot::Ma => self.infinitive.clone(),
            NonfiniteSlot::Da => self.row.da.clone().unwrap_or_else(|| format!("{ma}da")),
            NonfiniteSlot::MaInessive => format!("{}s", self.infinitive),
            NonfiniteSlot::MaElative => format!("{}st", self.infinitive),
            NonfiniteSlot::MaTranslative => format!("{}ks", self.infinitive),
            NonfiniteSlot::MaAbessive => format!("{}ta", self.infinitive),
            NonfiniteSlot::MaPassive => format!("{}ama", self.t_base()),
            NonfiniteSlot::Quotative => format!("{ma}vat"),
            NonfiniteSlot::PresentParticiple => {
                self.row.v_ptcp.clone().unwrap_or_else(|| format!("{ma}v"))
            }
            NonfiniteSlot::PassivePresentParticiple => format!("{}av", self.t_base()),
            NonfiniteSlot::NudParticiple => self.nud(),
            NonfiniteSlot::TudParticiple => self.tud(),
        }
    }
}

/// The four synthetic impersonal slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpersonalSlot {
    Present,
    Past,
    Conditional,
    Imperative,
}

/// Non-finite forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonfiniteSlot {
    Ma,
    Da,
    MaInessive,
    MaElative,
    MaTranslative,
    MaAbessive,
    MaPassive,
    Quotative,
    PresentParticiple,
    PassivePresentParticiple,
    NudParticiple,
    TudParticiple,
}

/// The full conjugation table of an Estonian verb — shared by the
/// WebAssembly and Python bindings. Six-slot rows run sg1…pl3.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub da_infinitive: String,
    pub present: [String; 6],
    pub past: [String; 6],
    pub conditional: [String; 6],
    /// [2sg, 3sg, 1pl, 2pl].
    pub imperative: [Option<String>; 4],
    pub present_impersonal: String,
    pub past_impersonal: String,
    pub quotative: String,
    pub present_participle: String,
    pub nud_participle: String,
    pub tud_participle: String,
    /// olema + nud-participle: olen rääkinud.
    pub perfect: [String; 6],
    /// olin rääkinud.
    pub pluperfect: [String; 6],
}

/// The olema rows used by the perfect tenses.
const OLEMA_PRES: [&str; 6] = ["olen", "oled", "on", "oleme", "olete", "on"];
const OLEMA_PAST: [&str; 6] = ["olin", "olid", "oli", "olime", "olite", "olid"];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let ps = [
            (Person::First, Number::Singular),
            (Person::Second, Number::Singular),
            (Person::Third, Number::Singular),
            (Person::First, Number::Plural),
            (Person::Second, Number::Plural),
            (Person::Third, Number::Plural),
        ];
        Self {
            infinitive: v.infinitive().to_string(),
            da_infinitive: v.nonfinite(NonfiniteSlot::Da),
            present: ps.map(|(p, n)| v.present(p, n)),
            past: ps.map(|(p, n)| v.past(p, n)),
            conditional: ps.map(|(p, n)| v.conditional(p, n)),
            imperative: [
                v.imperative(Person::Second, Number::Singular),
                v.imperative(Person::Third, Number::Singular),
                v.imperative(Person::First, Number::Plural),
                v.imperative(Person::Second, Number::Plural),
            ],
            present_impersonal: v.impersonal(ImpersonalSlot::Present),
            past_impersonal: v.impersonal(ImpersonalSlot::Past),
            quotative: v.nonfinite(NonfiniteSlot::Quotative),
            present_participle: v.nonfinite(NonfiniteSlot::PresentParticiple),
            nud_participle: v.nonfinite(NonfiniteSlot::NudParticiple),
            tud_participle: v.nonfinite(NonfiniteSlot::TudParticiple),
            perfect: OLEMA_PRES
                .map(|a| format!("{a} {}", v.nonfinite(NonfiniteSlot::NudParticiple))),
            pluperfect: OLEMA_PAST
                .map(|a| format!("{a} {}", v.nonfinite(NonfiniteSlot::NudParticiple))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn perfect_tenses() {
        let t = Table::build(&v("rääkima"));
        assert_eq!(t.perfect[0], "olen rääkinud");
        assert_eq!(t.perfect[2], "on rääkinud");
        assert_eq!(t.perfect[3], "oleme rääkinud");
        assert_eq!(t.pluperfect[0], "olin rääkinud");
        assert_eq!(t.pluperfect[5], "olid rääkinud");
    }

    #[test]
    fn regular_and_gradating() {
        let e = v("elama");
        assert_eq!(e.present(Person::First, Number::Singular), "elan");
        assert_eq!(e.past(Person::Third, Number::Singular), "elas");
        assert_eq!(e.conditional(Person::First, Number::Singular), "elaksin");
        assert_eq!(e.nonfinite(NonfiniteSlot::Da), "elada");
        let r = v("rääkima");
        assert_eq!(r.past(Person::First, Number::Singular), "rääkisin");
        assert_eq!(
            r.imperative(Person::First, Number::Plural).unwrap(),
            "rääkigem"
        );
        assert!(Verb::from_infinitive("tulla").is_err());
    }
}
