//! Marathi conjugation: a productive rule engine over one open verb class
//! plus a small compiled-in table of the verbs whose perfective or
//! subjunctive base, future or converb is not derivable.
//!
//! Every Marathi verb is cited by its infinitive, which ends in `-णे`
//! (बसणे "sit", करणे "do", लिहिणे "write"). Dropping the `-णे` leaves the
//! **stem** (बस, कर, लिहि), and the whole finite system is built on it by
//! suffixation — Marathi is the most agglutinative of the Indo-Aryan set,
//! so almost everything is a matter of stacking a suffix on the stem:
//!
//! - the **present habitual** (the imperfective, बसतो/बसते/बसतं…) agrees
//!   with the subject in person, gender and number, on the base `stem +
//!   त`;
//! - the **perfective** (the simple past, बसला/बसली/बसलं…) agrees in
//!   person, gender and number, on the base `stem + ल` — Marathi's third
//!   gender, the neuter, is live here (बसलं) exactly as in Gujarati
//!   (`src/guj.rs`);
//! - the **subjunctive** (बसावा/बसावी/बसावं…) agrees in gender and number
//!   only (it does not distinguish person), on the base `stem + ाव`;
//! - the **future** (बसेन, बसशील, बसेल…) agrees in person and number, no
//!   gender;
//! - the **imperative** by person and number (बस, बसा, बसू, बसो, बसोत);
//! - three non-finite forms: the **completive converb** (बसून, "having
//!   sat"), the **purposive** (बसायला, "in order to sit") and the
//!   **prospective** (बसणार, "about to sit").
//!
//! Compound and light-verb lemmas (`अभ्यास करणे`) conjugate only their
//! last word; the rest is carried along unchanged. What the rules cannot
//! predict — the irregular perfective of करणे (केल-), the contracted
//! stems of देणे/घेणे/जाणे/येणे, and the suppletive copulas होणे/असणे —
//! lives in `data/mar/verbs.tsv`.

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

/// Grammatical gender: Marathi distinguishes three, all live on the verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input's last word does not end in `-णे`.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Marathi infinitive")
    }
}

/// The infinitive suffix, `-णे` (NNA + vowel-sign E).
const INF_SUFFIX: &str = "णे";

/// The compiled-in table of verbs whose forms are not derivable.
static LEXICON_TSV: &str = include_str!("../data/mar/verbs.tsv");

/// A stored paradigm. Every group is optional: a `-` cell falls through
/// to the productive rule.
#[derive(Debug, Clone, Default)]
struct LexEntry {
    /// Present-habitual base, the part before the agreement ending
    /// (regular `stem + त`).
    present: Option<String>,
    /// Perfective base, before the agreement ending (regular `stem + ल`;
    /// करणे → केल, देणे → दिल).
    perfective: Option<String>,
    /// Subjunctive base, before the agreement ending (regular `stem + ाव`;
    /// देणे → द्याव).
    subjunctive: Option<String>,
    /// Future, all six person/number forms
    /// [1sg, 2sg, 3sg, 1pl, 2pl, 3pl].
    future: Option<[String; 6]>,
    /// Imperative [2sg, 2pl, 1, 3sg, 3pl].
    imperative: Option<[String; 5]>,
    /// Completive converb (बसून; देणे → देऊन).
    completive: Option<String>,
    /// Purposive (बसायला; देणे → द्यायला).
    purposive: Option<String>,
    /// Prospective (बसणार).
    prospective: Option<String>,
}

/// The independent/dependent long-*a* sign `ा` (U+093E).
const AA: char = 'ा';

/// Concatenate a stem and a suffix, coalescing a doubled long-*a*: an
/// `आ`-final stem (जा-, गा-) plus an `ा`-initial ending (`ाव`, `ायला`)
/// keeps a single `ा` (गा + ाव → गाव, never गाााव).
fn join(stem: &str, suffix: &str) -> String {
    if stem.ends_with(AA) && suffix.starts_with(AA) {
        format!("{stem}{}", &suffix[AA.len_utf8()..])
    } else {
        format!("{stem}{suffix}")
    }
}

fn one(s: &str) -> Option<String> {
    (s != "-" && !s.is_empty()).then(|| s.to_string())
}

fn six(s: &str) -> Option<[String; 6]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
}

fn five(s: &str) -> Option<[String; 5]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
}

fn lexicon() -> &'static HashMap<String, LexEntry> {
    static MAP: OnceLock<HashMap<String, LexEntry>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in LEXICON_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            let g = |i: usize| c.get(i).copied().unwrap_or("-");
            m.insert(
                c[0].to_string(),
                LexEntry {
                    present: one(g(1)),
                    perfective: one(g(2)),
                    subjunctive: one(g(3)),
                    future: six(g(4)),
                    imperative: five(g(5)),
                    completive: one(g(6)),
                    purposive: one(g(7)),
                    prospective: one(g(8)),
                },
            );
        }
        m
    })
}

/// A conjugatable Marathi verb.
#[derive(Debug, Clone)]
pub struct Verb {
    /// The invariable material before the verb word (`अभ्यास ` in
    /// अभ्यास करणे), empty for a simple verb. Re-attached to every form.
    prefix: String,
    /// The verb word's infinitive (करणे), without the prefix.
    infinitive: String,
    /// The stem (बस, कर).
    stem: String,
    lex: Option<&'static LexEntry>,
}

/// The six person/number slots, in array order: 1sg, 2sg, 3sg, 1pl, 2pl,
/// 3pl.
const SLOTS6: [(Person, Number); 6] = [
    (Person::First, Number::Singular),
    (Person::Second, Number::Singular),
    (Person::Third, Number::Singular),
    (Person::First, Number::Plural),
    (Person::Second, Number::Plural),
    (Person::Third, Number::Plural),
];

/// The five imperative slots, in array order: 2sg, 2pl, 1, 3sg, 3pl.
const IMP5: [(Person, Number); 5] = [
    (Person::Second, Number::Singular),
    (Person::Second, Number::Plural),
    (Person::First, Number::Singular),
    (Person::Third, Number::Singular),
    (Person::Third, Number::Plural),
];

/// The six gender/number slots (subjunctive), in array order: masc sg,
/// fem sg, neut sg, masc pl, fem pl, neut pl.
const GN6: [(Gender, Number); 6] = [
    (Gender::Masculine, Number::Singular),
    (Gender::Feminine, Number::Singular),
    (Gender::Neuter, Number::Singular),
    (Gender::Masculine, Number::Plural),
    (Gender::Feminine, Number::Plural),
    (Gender::Neuter, Number::Plural),
];

impl Verb {
    /// Build a verb from its infinitive.
    ///
    /// ```
    /// use ablaut::mar::{Gender, Number, Person, Verb};
    /// let v = Verb::from_infinitive("बसणे").unwrap();
    /// assert_eq!(
    ///     v.present(Person::First, Gender::Masculine, Number::Singular),
    ///     "बसतो"
    /// );
    /// assert_eq!(
    ///     v.perfective(Person::Third, Gender::Feminine, Number::Singular),
    ///     "बसली"
    /// );
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::NotAVerb`] when the last word does not end in
    /// `-णे`.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim();
        if inf.is_empty() {
            return Err(Error::NotAVerb);
        }
        // A compound lemma conjugates only its last word.
        let (prefix, word) = match inf.rfind(' ') {
            Some(i) => (inf[..=i].to_string(), inf[i + 1..].to_string()),
            None => (String::new(), inf.to_string()),
        };
        if !word.ends_with(INF_SUFFIX) {
            return Err(Error::NotAVerb);
        }
        let stem: String = {
            let drop = INF_SUFFIX.chars().count();
            let n = word.chars().count();
            word.chars().take(n.saturating_sub(drop)).collect()
        };
        if stem.is_empty() {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            lex: lexicon().get(&word),
            prefix,
            infinitive: word,
            stem,
        })
    }

    /// The normalized infinitive.
    #[must_use]
    pub fn infinitive(&self) -> String {
        format!("{}{}", self.prefix, self.infinitive)
    }

    /// Re-attach the invariable prefix.
    fn out(&self, form: &str) -> String {
        format!("{}{}", self.prefix, form)
    }

    /// The present-habitual base (`stem + त`, or the lexical override).
    fn present_base(&self) -> String {
        self.lex
            .and_then(|l| l.present.clone())
            .unwrap_or_else(|| format!("{}त", self.stem))
    }

    /// The perfective base (`stem + ल`, or the lexical override).
    fn perfective_base(&self) -> String {
        self.lex
            .and_then(|l| l.perfective.clone())
            .unwrap_or_else(|| format!("{}ल", self.stem))
    }

    /// The subjunctive base (`stem + ाव`, or the lexical override).
    fn subjunctive_base(&self) -> String {
        self.lex
            .and_then(|l| l.subjunctive.clone())
            .unwrap_or_else(|| join(&self.stem, "ाव"))
    }

    /// The present habitual (the imperfective): base + a person/gender/
    /// number ending. Plural collapses gender.
    #[must_use]
    pub fn present(&self, person: Person, gender: Gender, number: Number) -> String {
        let end = match (person, number) {
            (Person::First, Number::Singular) => {
                if gender == Gender::Feminine {
                    "े"
                } else {
                    "ो"
                }
            }
            (Person::First, Number::Plural) => "ो",
            (Person::Second, Number::Singular) => {
                if gender == Gender::Feminine {
                    "ेस"
                } else {
                    "ोस"
                }
            }
            (Person::Second, Number::Plural) => "ा",
            (Person::Third, Number::Singular) => match gender {
                Gender::Masculine => "ो",
                Gender::Feminine => "े",
                Gender::Neuter => "ं",
            },
            (Person::Third, Number::Plural) => "ात",
        };
        self.out(&format!("{}{end}", self.present_base()))
    }

    /// The gender/number agreement ending shared by the perfective (2nd
    /// and 3rd person) and the subjunctive.
    fn gn_ending(gender: Gender, number: Number) -> &'static str {
        match (gender, number) {
            (Gender::Masculine, Number::Singular) => "ा",
            (Gender::Feminine, Number::Singular) => "ी",
            (Gender::Neuter, Number::Singular) => "ं",
            (Gender::Masculine, Number::Plural) => "े",
            (Gender::Feminine, Number::Plural) => "्या",
            (Gender::Neuter, Number::Plural) => "ी",
        }
    }

    /// The perfective (the simple past): base + person/gender/number
    /// ending. The first person takes `ो`/`े`; the rest agree in gender
    /// and number.
    #[must_use]
    pub fn perfective(&self, person: Person, gender: Gender, number: Number) -> String {
        let end = if person == Person::First {
            match (gender, number) {
                (Gender::Feminine, Number::Singular) => "े",
                _ => "ो",
            }
        } else {
            Self::gn_ending(gender, number)
        };
        self.out(&format!("{}{end}", self.perfective_base()))
    }

    /// The subjunctive (बसावा/बसावी/बसावं…): base + gender/number ending.
    /// It does not distinguish person.
    #[must_use]
    pub fn subjunctive(&self, gender: Gender, number: Number) -> String {
        self.out(&format!(
            "{}{}",
            self.subjunctive_base(),
            Self::gn_ending(gender, number)
        ))
    }

    /// The future (बसेन, बसशील…): person and number, no gender.
    #[must_use]
    pub fn future(&self, person: Person, number: Number) -> String {
        let i = SLOTS6.iter().position(|&s| s == (person, number)).unwrap();
        if let Some(f) = self.lex.and_then(|l| l.future.as_ref()) {
            return self.out(&f[i]);
        }
        let end = match (person, number) {
            (Person::First, Number::Singular) => "ेन",
            (Person::First, Number::Plural) => "ू",
            (Person::Second, Number::Singular) => "शील",
            (Person::Second, Number::Plural) => "ाल",
            (Person::Third, Number::Singular) => "ेल",
            (Person::Third, Number::Plural) => "तील",
        };
        self.out(&join(&self.stem, end))
    }

    /// The imperative by person and number (बस, बसा, बसू, बसो, बसोत).
    #[must_use]
    pub fn imperative(&self, person: Person, number: Number) -> String {
        // The stored array is [2sg, 2pl, 1, 3sg, 3pl]: the first person
        // does not split singular from plural (बसू for both).
        let i = match (person, number) {
            (Person::Second, Number::Singular) => 0,
            (Person::Second, Number::Plural) => 1,
            (Person::First, _) => 2,
            (Person::Third, Number::Singular) => 3,
            (Person::Third, Number::Plural) => 4,
        };
        if let Some(imp) = self.lex.and_then(|l| l.imperative.as_ref()) {
            return self.out(&imp[i]);
        }
        let form = match (person, number) {
            (Person::Second, Number::Singular) => self.stem.clone(),
            (Person::Second, Number::Plural) => join(&self.stem, "ा"),
            (Person::First, _) => format!("{}ू", self.stem),
            (Person::Third, Number::Singular) => format!("{}ो", self.stem),
            (Person::Third, Number::Plural) => format!("{}ोत", self.stem),
        };
        self.out(&form)
    }

    /// The completive converb (बसून, "having sat").
    #[must_use]
    pub fn completive(&self) -> String {
        if let Some(c) = self.lex.and_then(|l| l.completive.as_ref()) {
            return self.out(c);
        }
        self.out(&format!("{}ून", self.stem))
    }

    /// The purposive (बसायला, "in order to sit").
    #[must_use]
    pub fn purposive(&self) -> String {
        if let Some(p) = self.lex.and_then(|l| l.purposive.as_ref()) {
            return self.out(p);
        }
        self.out(&join(&self.stem, "ायला"))
    }

    /// The prospective (बसणार, "about to sit").
    #[must_use]
    pub fn prospective(&self) -> String {
        if let Some(p) = self.lex.and_then(|l| l.prospective.as_ref()) {
            return self.out(p);
        }
        self.out(&format!("{}णार", self.stem))
    }
}

/// The conjugation table of a Marathi verb — shared by the WebAssembly
/// and Python bindings. Person rows are [1sg, 2sg, 3sg, 1pl, 2pl, 3pl];
/// gender/number rows are [masc sg, fem sg, neut sg, masc pl, fem pl,
/// neut pl].
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// बसून — the completive converb.
    pub completive: String,
    /// बसायला — the purposive.
    pub purposive: String,
    /// बसणार — the prospective.
    pub prospective: String,
    /// बसतो… — the present habitual, masculine.
    pub present_masculine: [String; 6],
    /// बसते… — the present habitual, feminine.
    pub present_feminine: [String; 6],
    /// बसलो… — the perfective (simple past), masculine.
    pub perfective_masculine: [String; 6],
    /// बसले… — the perfective (simple past), feminine.
    pub perfective_feminine: [String; 6],
    /// [बसावा, बसावी, बसावं, बसावे, बसाव्या, बसावी] — the subjunctive.
    pub subjunctive: [String; 6],
    /// बसेन… — the future.
    pub future: [String; 6],
    /// [बस, बसा, बसू, बसो, बसोत] — the imperative.
    pub imperative: [String; 5],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        Self {
            infinitive: v.infinitive(),
            completive: v.completive(),
            purposive: v.purposive(),
            prospective: v.prospective(),
            present_masculine: SLOTS6.map(|(p, n)| v.present(p, Gender::Masculine, n)),
            present_feminine: SLOTS6.map(|(p, n)| v.present(p, Gender::Feminine, n)),
            perfective_masculine: SLOTS6.map(|(p, n)| v.perfective(p, Gender::Masculine, n)),
            perfective_feminine: SLOTS6.map(|(p, n)| v.perfective(p, Gender::Feminine, n)),
            subjunctive: GN6.map(|(g, n)| v.subjunctive(g, n)),
            future: SLOTS6.map(|(p, n)| v.future(p, n)),
            imperative: IMP5.map(|(p, n)| v.imperative(p, n)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn regular_consonant_stem() {
        let b = v("बसणे");
        assert_eq!(b.infinitive(), "बसणे");
        // present habitual
        assert_eq!(b.present(P1, M, SG), "बसतो");
        assert_eq!(b.present(P1, F, SG), "बसते");
        assert_eq!(b.present(P2, M, SG), "बसतोस");
        assert_eq!(b.present(P2, F, SG), "बसतेस");
        assert_eq!(b.present(P2, M, PL), "बसता");
        assert_eq!(b.present(P3, M, SG), "बसतो");
        assert_eq!(b.present(P3, F, SG), "बसते");
        assert_eq!(b.present(P3, N, SG), "बसतं");
        assert_eq!(b.present(P3, M, PL), "बसतात");
        // perfective (simple past)
        assert_eq!(b.perfective(P1, M, SG), "बसलो");
        assert_eq!(b.perfective(P1, F, SG), "बसले");
        assert_eq!(b.perfective(P3, M, SG), "बसला");
        assert_eq!(b.perfective(P3, F, SG), "बसली");
        assert_eq!(b.perfective(P3, N, SG), "बसलं");
        assert_eq!(b.perfective(P3, M, PL), "बसले");
        assert_eq!(b.perfective(P3, F, PL), "बसल्या");
        // subjunctive
        assert_eq!(b.subjunctive(M, SG), "बसावा");
        assert_eq!(b.subjunctive(F, SG), "बसावी");
        assert_eq!(b.subjunctive(N, SG), "बसावं");
        assert_eq!(b.subjunctive(M, PL), "बसावे");
        assert_eq!(b.subjunctive(F, PL), "बसाव्या");
        // future
        assert_eq!(b.future(P1, SG), "बसेन");
        assert_eq!(b.future(P1, PL), "बसू");
        assert_eq!(b.future(P2, SG), "बसशील");
        assert_eq!(b.future(P2, PL), "बसाल");
        assert_eq!(b.future(P3, SG), "बसेल");
        assert_eq!(b.future(P3, PL), "बसतील");
        // imperative
        assert_eq!(b.imperative(P2, SG), "बस");
        assert_eq!(b.imperative(P2, PL), "बसा");
        assert_eq!(b.imperative(P1, SG), "बसू");
        assert_eq!(b.imperative(P3, SG), "बसो");
        assert_eq!(b.imperative(P3, PL), "बसोत");
        // non-finite
        assert_eq!(b.completive(), "बसून");
        assert_eq!(b.purposive(), "बसायला");
        assert_eq!(b.prospective(), "बसणार");
    }

    #[test]
    fn irregular_perfective() {
        // करणे: irregular perfective base केल-, regular elsewhere.
        let k = v("करणे");
        assert_eq!(k.present(P1, M, SG), "करतो");
        assert_eq!(k.perfective(P3, M, SG), "केला");
        assert_eq!(k.perfective(P3, F, SG), "केली");
        assert_eq!(k.perfective(P3, N, SG), "केलं");
        assert_eq!(k.perfective(P1, M, SG), "केलो");
        assert_eq!(k.subjunctive(M, SG), "करावा");
        assert_eq!(k.completive(), "करून");
    }

    #[test]
    fn contracted_stem() {
        // देणे: perfective दिल-, subjunctive द्याव-, converb देऊन.
        let d = v("देणे");
        assert_eq!(d.present(P1, M, SG), "देतो");
        assert_eq!(d.perfective(P3, M, SG), "दिला");
        assert_eq!(d.perfective(P3, F, SG), "दिली");
        assert_eq!(d.subjunctive(M, SG), "द्यावा");
        assert_eq!(d.completive(), "देऊन");
        assert_eq!(d.purposive(), "द्यायला");
        assert_eq!(d.future(P1, SG), "देईन");
    }

    #[test]
    fn compound_lemma() {
        let a = v("अभ्यास करणे");
        assert_eq!(a.infinitive(), "अभ्यास करणे");
        assert_eq!(a.present(P1, M, SG), "अभ्यास करतो");
        assert_eq!(a.perfective(P3, M, SG), "अभ्यास केला");
    }

    #[test]
    fn rejects_non_verbs() {
        for input in ["", "घर", "run", "बस"] {
            assert_eq!(Verb::from_infinitive(input).err(), Some(Error::NotAVerb));
        }
    }
}
