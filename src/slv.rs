//! Slovenian conjugation. Three number values — the dual lives —
//! with class-driven presents (delam, govorim, kupujem, dvignem),
//! mined class assignments in `data/slv/classes.tsv` and explicit
//! rows in `data/slv/verbs.tsv`. The l-participle inflects off one
//! base (delal → delala/delali/delale), the supine is the infinitive
//! minus -i (delat), and imperative duals/plurals ride the 2sg
//! (delaj → delajva/delajmo).

use std::collections::HashMap;
use std::sync::OnceLock;

/// Grammatical person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Person {
    First,
    Second,
    Third,
}

/// Grammatical number — Slovenian keeps the dual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Singular,
    Dual,
    Plural,
}

/// Participle gender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Slovenian infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Slovenian infinitive")
    }
}

/// Present-tense class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// -ati → delam.
    A,
    /// -iti/-eti → govorim.
    I,
    /// -ovati/-evati → kupujem.
    Uje,
    /// -niti → dvignem.
    Ne,
}

static CLASSES_TSV: &str = include_str!("../data/slv/classes.tsv");
static VERBS_TSV: &str = include_str!("../data/slv/verbs.tsv");

fn classes() -> &'static HashMap<&'static str, Class> {
    static MAP: OnceLock<HashMap<&'static str, Class>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in CLASSES_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let mut f = line.split('\t');
            let (Some(lemma), Some(class)) = (f.next(), f.next()) else {
                continue;
            };
            let class = match class {
                "a" => Class::A,
                "i" => Class::I,
                "uje" => Class::Uje,
                "ne" => Class::Ne,
                _ => continue,
            };
            m.insert(lemma, class);
        }
        m
    })
}

/// Explicit row: lemma, present ×9 (sg/du/pl × 1/2/3), imperative
/// 2sg, l-participle masc sg, supine ("-" = derive).
#[derive(Debug, Clone, Default)]
struct Row {
    present: [Option<String>; 9],
    imp2: Option<String>,
    lptcp: Option<String>,
    supine: Option<String>,
    lptcp_f: Option<String>,
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
            let mut present: [Option<String>; 9] = Default::default();
            for (j, slot) in present.iter_mut().enumerate() {
                *slot = opt(1 + j);
            }
            m.insert(
                cols[0],
                Row {
                    present,
                    imp2: opt(10),
                    lptcp: opt(11),
                    supine: opt(12),
                    lptcp_f: opt(13),
                },
            );
        }
        m
    })
}

/// A conjugatable Slovenian verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    class: Class,
    row: Row,
}

impl Verb {
    /// Build a verb from its infinitive.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim();
        if inf.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf.chars().all(|c| c.is_alphabetic() || c == '-')
        {
            return Err(Error::NotAVerb);
        }
        let class = classes().get(inf).copied().unwrap_or({
            if inf.ends_with("ovati") || inf.ends_with("evati") {
                Class::Uje
            } else if inf.ends_with("ati") {
                Class::A
            } else {
                Class::I
            }
        });
        Ok(Self {
            infinitive: inf.to_string(),
            class,
            row: rows().get(inf).cloned().unwrap_or_default(),
        })
    }

    /// The infinitive as normalized.
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// The present stem per class.
    fn stem(&self) -> String {
        let inf = &self.infinitive;
        match self.class {
            Class::Uje => {
                let base = inf
                    .strip_suffix("ovati")
                    .or_else(|| inf.strip_suffix("evati"))
                    .unwrap_or(inf);
                format!("{base}uj")
            }
            Class::Ne => inf.strip_suffix("iti").unwrap_or(inf).to_string(),
            Class::A => inf.strip_suffix("ti").unwrap_or(inf).to_string(),
            Class::I => {
                let base = inf
                    .strip_suffix("iti")
                    .or_else(|| inf.strip_suffix("eti"))
                    .unwrap_or(inf);
                base.to_string()
            }
        }
    }

    fn idx(person: Person, number: Number) -> usize {
        let p = match person {
            Person::First => 0,
            Person::Second => 1,
            Person::Third => 2,
        };
        let n = match number {
            Number::Singular => 0,
            Number::Dual => 1,
            Number::Plural => 2,
        };
        n * 3 + p
    }

    /// The present indicative.
    pub fn present(&self, person: Person, number: Number) -> String {
        if let Some(f) = &self.row.present[Self::idx(person, number)] {
            return f.clone();
        }
        let s = self.stem();
        let endings: [&str; 9] = match self.class {
            Class::A => ["am", "aš", "a", "ava", "ata", "ata", "amo", "ate", "ajo"],
            Class::I => ["im", "iš", "i", "iva", "ita", "ita", "imo", "ite", "ijo"],
            Class::Uje | Class::Ne => ["em", "eš", "e", "eva", "eta", "eta", "emo", "ete", "ejo"],
        };
        format!("{s}{}", endings[Self::idx(person, number)])
    }

    /// The imperative 2sg (delaj, govori, kupuj, dvigni).
    fn imp2(&self) -> String {
        if let Some(f) = &self.row.imp2 {
            return f.clone();
        }
        let s = self.stem();
        match self.class {
            Class::A => format!("{s}j"),
            Class::I | Class::Ne => format!("{s}i"),
            Class::Uje => s,
        }
    }

    /// An imperative form: 2sg plus the du/pl set off it (delajva,
    /// delajta, delajmo, delajte).
    pub fn imperative(&self, person: Person, number: Number) -> Option<String> {
        let suffix = match (number, person) {
            (Number::Singular, Person::Second) => "",
            (Number::Dual, Person::First) => "va",
            (Number::Dual, Person::Second) => "ta",
            (Number::Plural, Person::First) => "mo",
            (Number::Plural, Person::Second) => "te",
            _ => return None,
        };
        Some(format!("{}{suffix}", self.imp2()))
    }

    /// The l-participle masc sg base (delal): infinitive minus -ti,
    /// plus l.
    fn lptcp_base(&self) -> String {
        if let Some(f) = &self.row.lptcp {
            return f.clone();
        }
        let b = self
            .infinitive
            .strip_suffix("ti")
            .unwrap_or(&self.infinitive);
        format!("{b}l")
    }

    /// The l-participle across gender and number. Verbs with a
    /// fleeting -e- in the masculine (rekel → rekla) carry a mined
    /// feminine base for the rest of the paradigm.
    pub fn participle(&self, gender: Gender, number: Number) -> String {
        let base = if gender == Gender::Masculine && number == Number::Singular {
            self.lptcp_base()
        } else if let Some(fem) = &self.row.lptcp_f {
            fem.strip_suffix('a').unwrap_or(fem).to_string()
        } else {
            self.lptcp_base()
        };
        let suffix = match (gender, number) {
            (Gender::Masculine, Number::Singular) => "",
            (Gender::Masculine, Number::Dual) => "a",
            (Gender::Masculine, Number::Plural) => "i",
            (Gender::Feminine, Number::Singular) => "a",
            (Gender::Feminine, Number::Dual) => "i",
            (Gender::Feminine, Number::Plural) => "e",
            (Gender::Neuter, Number::Singular) => "o",
            (Gender::Neuter, Number::Dual) => "i",
            (Gender::Neuter, Number::Plural) => "a",
        };
        format!("{base}{suffix}")
    }

    /// The supine (delat): infinitive minus -i.
    pub fn supine(&self) -> String {
        if let Some(f) = &self.row.supine {
            return f.clone();
        }
        self.infinitive
            .strip_suffix('i')
            .unwrap_or(&self.infinitive)
            .to_string()
    }
}

/// The full conjugation table of a Slovenian verb — shared by the
/// WebAssembly and Python bindings. Nine-slot rows run sg1…pl3.
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub supine: String,
    pub present: [String; 9],
    /// [2sg, 1du, 2du, 1pl, 2pl].
    pub imperative: [Option<String>; 5],
    /// [m, f, n] × [sg, du, pl].
    pub participle: [String; 9],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let ps = [
            (Person::First, Number::Singular),
            (Person::Second, Number::Singular),
            (Person::Third, Number::Singular),
            (Person::First, Number::Dual),
            (Person::Second, Number::Dual),
            (Person::Third, Number::Dual),
            (Person::First, Number::Plural),
            (Person::Second, Number::Plural),
            (Person::Third, Number::Plural),
        ];
        Self {
            infinitive: v.infinitive().to_string(),
            supine: v.supine(),
            present: ps.map(|(p, n)| v.present(p, n)),
            imperative: [
                v.imperative(Person::Second, Number::Singular),
                v.imperative(Person::First, Number::Dual),
                v.imperative(Person::Second, Number::Dual),
                v.imperative(Person::First, Number::Plural),
                v.imperative(Person::Second, Number::Plural),
            ],
            participle: [
                (Gender::Masculine, Number::Singular),
                (Gender::Masculine, Number::Dual),
                (Gender::Masculine, Number::Plural),
                (Gender::Feminine, Number::Singular),
                (Gender::Feminine, Number::Dual),
                (Gender::Feminine, Number::Plural),
                (Gender::Neuter, Number::Singular),
                (Gender::Neuter, Number::Dual),
                (Gender::Neuter, Number::Plural),
            ]
            .map(|(g, n)| v.participle(g, n)),
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
    fn class_defaults() {
        let d = v("delati");
        assert_eq!(d.present(Person::First, Number::Singular), "delam");
        assert_eq!(d.present(Person::First, Number::Dual), "delava");
        assert_eq!(d.present(Person::Third, Number::Plural), "delajo");
        assert_eq!(
            d.imperative(Person::Second, Number::Singular).unwrap(),
            "delaj"
        );
        assert_eq!(
            d.imperative(Person::First, Number::Dual).unwrap(),
            "delajva"
        );
        assert_eq!(d.participle(Gender::Feminine, Number::Plural), "delale");
        assert_eq!(d.supine(), "delat");
        let g = v("govoriti");
        assert_eq!(g.present(Person::First, Number::Singular), "govorim");
        assert_eq!(g.participle(Gender::Masculine, Number::Singular), "govoril");
        let k = v("kupovati");
        assert_eq!(k.present(Person::First, Number::Singular), "kupujem");
        assert_eq!(
            k.imperative(Person::Second, Number::Singular).unwrap(),
            "kupuj"
        );
    }
}
