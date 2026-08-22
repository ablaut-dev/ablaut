//! Turkish conjugation. Turkish is agglutinative and near-exclusively
//! suffixing, so the verb is built productively: a stem, a tense/aspect
//! suffix, and a personal ending, with vowel harmony and a few
//! consonant rules deciding the surface shape. Almost nothing is stored.
//!
//! The engine covers the single-word synthetic paradigm — the bound
//! shared with the two oracles (see `docs/tur/oracles.md`): six base TAM
//! categories (aorist, present progressive, future, definite past,
//! evidential, necessitative), the seven single-word copular stacks that
//! carry the past or evidential copula on top of them, the imperative
//! and the infinitive, each in both polarities. The periphrastic tenses
//! (`gelecek olacak`) and the interrogative particle (`gelir mi`) are
//! syntax and out of scope.
//!
//! ## Productive rules
//!
//! * **Vowel harmony.** Suffix vowels written `I` here are the fourfold
//!   set i/ı/ü/u and `A` the twofold set e/a, both fixed by the last
//!   vowel of what precedes them (frontness for both, rounding for the
//!   fourfold set).
//! * **Consonant rules.** A buffer `y` separates two vowels
//!   (gelmeli → gelmeli**y**im); stem-final `k` softens to `ğ` before a
//!   vowel (gelecek → gelece**ğ**im); the past/─di copula assimilates its
//!   d→t after a voiceless consonant (gelecek → gelecek**t**i).
//!
//! ## The exception table
//!
//! `data/tur/verbs.tsv` stores the handful of stems the rules cannot
//! predict: the closed class of monosyllables whose aorist is `-Ir`
//! rather than the default `-Ar` (al → alır, gel → gelir), the verbs
//! that voice a final t→d before a vowel (git → gider, et → eder), and
//! the suppletive future of demek/yemek (diyecek, yiyecek). Everything
//! else — including the productive progressive of vowel-final stems
//! (oku → okuyor, de → diyor) — derives.

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

/// Affirmative or negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Polarity {
    Positive,
    Negative,
}

/// A finite tense: the six base TAM categories and the seven single-word
/// copular stacks (base + past or evidential copula).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    /// `gelir` — aorist / habitual present.
    Aorist,
    /// `geliyor` — present progressive.
    Progressive,
    /// `gelecek` — future.
    Future,
    /// `geldi` — definite past.
    Past,
    /// `gelmiş` — evidential (inferential) past.
    Evidential,
    /// `gelmeli` — necessitative.
    Necessitative,
    /// `gelirdi` — aorist + past copula.
    AoristPast,
    /// `gelirmiş` — aorist + evidential copula.
    AoristEvidential,
    /// `geliyordu` — progressive + past copula.
    ProgressivePast,
    /// `geliyormuş` — progressive + evidential copula.
    ProgressiveEvidential,
    /// `gelecekti` — future + past copula.
    FuturePast,
    /// `gelecekmiş` — future + evidential copula.
    FutureEvidential,
    /// `gelmişti` — evidential + past copula.
    EvidentialPast,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not end in `-mek`/`-mak`.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Turkish infinitive")
    }
}

/// The compiled-in table of the stems the productive rules cannot derive.
static VERBS_TSV: &str = include_str!("../data/tur/verbs.tsv");

/// A stored row. Every column is optional; `-` falls through to the rule.
/// The stored bases are 3rd-singular predicative forms — the personal
/// endings still derive.
#[derive(Debug, Clone, Default)]
struct Row {
    aorist: Option<String>,
    progressive: Option<String>,
    future: Option<String>,
    past: Option<String>,
    evidential: Option<String>,
    necessitative: Option<String>,
    imperative_sg: Option<String>,
    imperative_pl: Option<String>,
}

fn opt(s: &str) -> Option<String> {
    (s != "-" && !s.is_empty()).then(|| s.to_string())
}

fn rows() -> &'static HashMap<String, Row> {
    static MAP: OnceLock<HashMap<String, Row>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in VERBS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            let g = |i: usize| c.get(i).copied().unwrap_or("-");
            m.insert(
                c[0].to_string(),
                Row {
                    aorist: opt(g(1)),
                    progressive: opt(g(2)),
                    future: opt(g(3)),
                    past: opt(g(4)),
                    evidential: opt(g(5)),
                    necessitative: opt(g(6)),
                    imperative_sg: opt(g(7)),
                    imperative_pl: opt(g(8)),
                },
            );
        }
        m
    })
}

/// The Turkish vowels, back and front.
fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'ı' | 'i' | 'o' | 'ö' | 'u' | 'ü')
}

fn is_back(v: char) -> bool {
    matches!(v, 'a' | 'ı' | 'o' | 'u')
}

fn is_round(v: char) -> bool {
    matches!(v, 'o' | 'ö' | 'u' | 'ü')
}

/// Voiceless consonants — the past/─di copula assimilates d→t after one.
fn is_voiceless(c: char) -> bool {
    matches!(c, 'p' | 'ç' | 't' | 'k' | 'f' | 's' | 'ş' | 'h')
}

fn last_vowel(s: &str) -> char {
    s.chars().rev().find(|c| is_vowel(*c)).unwrap_or('e')
}

/// The fourfold suffix vowel (i/ı/ü/u) harmonizing with `base`.
fn four(base: &str) -> char {
    let v = last_vowel(base);
    match (is_back(v), is_round(v)) {
        (false, false) => 'i',
        (false, true) => 'ü',
        (true, false) => 'ı',
        (true, true) => 'u',
    }
}

/// The twofold suffix vowel (e/a) harmonizing with `base`.
fn two(base: &str) -> char {
    if is_back(last_vowel(base)) {
        'a'
    } else {
        'e'
    }
}

fn last_char(s: &str) -> char {
    s.chars().last().unwrap_or(' ')
}

/// Attach a vowel-initial suffix to a *predicative base*, applying the two
/// consonant rules that meet a vowel: a buffer `y` after a vowel-final
/// base, and k→ğ softening. The softening is right for the suffixal `-cAk`
/// of the future (gelecek → geleceğim) but must not touch a bare
/// verb-stem's final k (bak → bakacak, not *bağacak*), so stem-level
/// attachment uses [`add_vowel_stem`] instead.
fn add_vowel(base: &str, vowel: char, tail: &str) -> String {
    let last = last_char(base);
    if is_vowel(last) {
        format!("{base}y{vowel}{tail}")
    } else if last == 'k' {
        let keep: String = base.chars().take(base.chars().count() - 1).collect();
        format!("{keep}ğ{vowel}{tail}")
    } else {
        format!("{base}{vowel}{tail}")
    }
}

/// Attach a vowel-initial suffix to a bare stem: a buffer `y` after a
/// vowel, but no k→ğ softening (a verb stem's final k stays: bak → bakın,
/// çık → çıkacak).
fn add_vowel_stem(base: &str, vowel: char, tail: &str) -> String {
    if is_vowel(last_char(base)) {
        format!("{base}y{vowel}{tail}")
    } else {
        format!("{base}{vowel}{tail}")
    }
}

/// The pronominal (z-type) endings, on a predicative base: aorist,
/// progressive, future, evidential, necessitative and the evidential
/// copula all take these.
fn set1(base: &str) -> [String; 6] {
    let i = four(base);
    let a = two(base);
    [
        add_vowel(base, i, "m"),
        format!("{base}s{i}n"),
        base.to_string(),
        add_vowel(base, i, "z"),
        format!("{base}s{i}n{i}z"),
        format!("{base}l{a}r"),
    ]
}

/// The possessive (k-type) endings, on a predicative base: the definite
/// past and the past copula take these.
fn set2(base: &str) -> [String; 6] {
    let i = four(base);
    let a = two(base);
    [
        format!("{base}m"),
        format!("{base}n"),
        base.to_string(),
        format!("{base}k"),
        format!("{base}n{i}z"),
        format!("{base}l{a}r"),
    ]
}

const SLOTS: [(Person, Number); 6] = [
    (Person::First, Number::Singular),
    (Person::Second, Number::Singular),
    (Person::Third, Number::Singular),
    (Person::First, Number::Plural),
    (Person::Second, Number::Plural),
    (Person::Third, Number::Plural),
];

impl Person {
    const fn index(self, number: Number) -> usize {
        let p = match self {
            Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
        };
        match number {
            Number::Singular => p,
            Number::Plural => p + 3,
        }
    }
}

/// A conjugatable Turkish verb: the stem plus any stored overrides.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    stem: String,
    row: Row,
}

impl Verb {
    /// Build a verb from its infinitive (`gelmek`, `almak`).
    ///
    /// ```
    /// use ablaut::tur::{Number, Person, Polarity, Tense, Verb};
    /// let v = Verb::from_infinitive("gelmek").unwrap();
    /// let (p, n, pos) = (Person::Third, Number::Singular, Polarity::Positive);
    /// assert_eq!(v.form(Tense::Aorist, p, n, pos), "gelir");
    /// assert_eq!(v.form(Tense::Progressive, p, n, pos), "geliyor");
    /// ```
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim().to_lowercase();
        if inf.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        if !(inf.ends_with("mek") || inf.ends_with("mak")) {
            return Err(Error::NotAVerb);
        }
        let stem: String = inf.chars().take(inf.chars().count() - 3).collect();
        if stem.is_empty() {
            return Err(Error::NotAVerb);
        }
        let row = rows().get(&inf).cloned().unwrap_or_default();
        Ok(Self {
            infinitive: inf,
            stem,
            row,
        })
    }

    /// The normalized infinitive (`gelmek`).
    #[must_use]
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// The negative stem: stem + the negative suffix -mA (gelme, yazma).
    fn neg_stem(&self) -> String {
        format!("{}m{}", self.stem, two(&self.stem))
    }

    /// Number of vowels in the stem (its syllable count).
    fn syllables(&self) -> usize {
        self.stem.chars().filter(|c| is_vowel(*c)).count()
    }

    /// The aorist 3sg base: `-r` after a vowel, `-Ir` for polysyllables,
    /// the default `-Ar` for monosyllables (the `-Ir` monosyllables are
    /// stored). Negative is the invariant `-mAz` predicative base.
    fn aorist_base(&self, polarity: Polarity) -> String {
        if polarity == Polarity::Negative {
            let neg = self.neg_stem();
            return format!("{neg}z");
        }
        if let Some(a) = &self.row.aorist {
            return a.clone();
        }
        let stem = &self.stem;
        if is_vowel(last_char(stem)) {
            format!("{stem}r")
        } else if self.syllables() <= 1 {
            format!("{stem}{}r", two(stem))
        } else {
            format!("{stem}{}r", four(stem))
        }
    }

    /// The present-progressive 3sg base (`geliyor`, `okuyor`). A
    /// vowel-final stem drops its vowel before `-Iyor` (oku → okuyor,
    /// de → diyor); the negative is `-mIyor` (gelmiyor).
    fn progressive_base(&self, polarity: Polarity) -> String {
        if polarity == Polarity::Positive {
            if let Some(p) = &self.row.progressive {
                return p.clone();
            }
        }
        let base: String = if polarity == Polarity::Negative {
            format!("{}m", self.stem)
        } else if is_vowel(last_char(&self.stem)) {
            self.stem.chars().take(self.stem.chars().count() - 1).collect()
        } else {
            self.stem.clone()
        };
        format!("{base}{}yor", four(&base))
    }

    /// The future 3sg base (`gelecek`, `okuyacak`), negative `-mAyAcAk`.
    fn future_base(&self, polarity: Polarity) -> String {
        if polarity == Polarity::Positive {
            if let Some(f) = &self.row.future {
                return f.clone();
            }
        }
        let base = if polarity == Polarity::Negative {
            self.neg_stem()
        } else {
            self.stem.clone()
        };
        let a = two(&base);
        add_vowel_stem(&base, a, &format!("c{a}k"))
    }

    /// The definite-past 3sg base (`geldi`, `gitti`, `okudu`); negative
    /// `gelmedi`. The past suffix assimilates d→t after a voiceless stem.
    fn past_base(&self, polarity: Polarity) -> String {
        if polarity == Polarity::Positive {
            if let Some(p) = &self.row.past {
                return p.clone();
            }
        }
        let base = if polarity == Polarity::Negative {
            self.neg_stem()
        } else {
            self.stem.clone()
        };
        let d = if is_voiceless(last_char(&base)) { 't' } else { 'd' };
        format!("{base}{d}{}", four(&base))
    }

    /// The evidential 3sg base (`gelmiş`), negative `gelmemiş`.
    fn evidential_base(&self, polarity: Polarity) -> String {
        if polarity == Polarity::Positive {
            if let Some(e) = &self.row.evidential {
                return e.clone();
            }
        }
        let base = if polarity == Polarity::Negative {
            self.neg_stem()
        } else {
            self.stem.clone()
        };
        format!("{base}m{}ş", four(&base))
    }

    /// The necessitative 3sg base (`gelmeli`), negative `gelmemeli`.
    fn necessitative_base(&self, polarity: Polarity) -> String {
        if polarity == Polarity::Positive {
            if let Some(n) = &self.row.necessitative {
                return n.clone();
            }
        }
        let base = if polarity == Polarity::Negative {
            self.neg_stem()
        } else {
            self.stem.clone()
        };
        let a = two(&base);
        format!("{base}m{a}l{}", four(&format!("{base}{a}")))
    }

    /// The special aorist-negative row (gelmem, gelmezsin, gelmez,
    /// gelmeyiz, gelmezsiniz, gelmezler).
    fn aorist_negative(&self) -> [String; 6] {
        let neg = self.neg_stem();
        let mez = format!("{neg}z");
        let i = four(&mez);
        let a = two(&mez);
        [
            format!("{neg}m"),
            format!("{mez}s{i}n"),
            mez.clone(),
            add_vowel(&neg, i, "z"),
            format!("{mez}s{i}n{i}z"),
            format!("{mez}l{a}r"),
        ]
    }

    /// The six person forms of a base tense.
    fn base_row(&self, tense: Tense, polarity: Polarity) -> [String; 6] {
        match tense {
            Tense::Aorist => {
                if polarity == Polarity::Negative {
                    self.aorist_negative()
                } else {
                    set1(&self.aorist_base(polarity))
                }
            }
            Tense::Progressive => set1(&self.progressive_base(polarity)),
            Tense::Future => set1(&self.future_base(polarity)),
            Tense::Evidential => set1(&self.evidential_base(polarity)),
            Tense::Necessitative => set1(&self.necessitative_base(polarity)),
            Tense::Past => set2(&self.past_base(polarity)),
            _ => unreachable!("base_row called on a stacked tense"),
        }
    }

    /// The predicative base a copula stacks onto (the 3sg of the
    /// underlying tense; aorist-negative uses its invariant `-mAz`).
    fn predicative(&self, tense: Tense, polarity: Polarity) -> String {
        match tense {
            Tense::Aorist => self.aorist_base(polarity),
            Tense::Progressive => self.progressive_base(polarity),
            Tense::Future => self.future_base(polarity),
            Tense::Evidential => self.evidential_base(polarity),
            _ => unreachable!("predicative called on a non-stackable tense"),
        }
    }

    /// A copular stack: the base tense's predicative form carrying the
    /// past copula (`-DI`, k-type endings) or the evidential copula
    /// (`-mIş`, z-type endings). The 3pl marker `-lAr` precedes the
    /// copula (gelirlerdi, geliyorlarmış).
    fn stacked_row(&self, base_tense: Tense, past_copula: bool, polarity: Polarity) -> [String; 6] {
        // The past copula is -DI (d→t after a voiceless base), the
        // evidential -mIş; each harmonizes with whatever it attaches to.
        let copula = |base: &str| {
            if past_copula {
                let d = if is_voiceless(last_char(base)) { 't' } else { 'd' };
                format!("{d}{}", four(base))
            } else {
                format!("m{}ş", four(base))
            }
        };
        let pred = self.predicative(base_tense, polarity);
        let stacked = format!("{pred}{}", copula(&pred));
        let mut row = if past_copula { set2(&stacked) } else { set1(&stacked) };
        // 3pl: -lAr goes on the predicative base, before the copula, and
        // the copula's vowel harmonizes with that -lAr.
        let plural = format!("{pred}l{}r", two(&pred));
        row[5] = format!("{plural}{}", copula(&plural));
        row
    }

    /// A finite form.
    ///
    /// ```
    /// use ablaut::tur::{Number, Person, Polarity, Tense, Verb};
    /// let v = Verb::from_infinitive("gelmek").unwrap();
    /// let (p, n) = (Person::First, Number::Singular);
    /// assert_eq!(v.form(Tense::Aorist, p, n, Polarity::Negative), "gelmem");
    /// assert_eq!(v.form(Tense::Future, p, n, Polarity::Positive), "geleceğim");
    /// ```
    #[must_use]
    pub fn form(&self, tense: Tense, person: Person, number: Number, polarity: Polarity) -> String {
        let i = person.index(number);
        let row = match tense {
            Tense::Aorist
            | Tense::Progressive
            | Tense::Future
            | Tense::Past
            | Tense::Evidential
            | Tense::Necessitative => self.base_row(tense, polarity),
            Tense::AoristPast => self.stacked_row(Tense::Aorist, true, polarity),
            Tense::AoristEvidential => self.stacked_row(Tense::Aorist, false, polarity),
            Tense::ProgressivePast => self.stacked_row(Tense::Progressive, true, polarity),
            Tense::ProgressiveEvidential => self.stacked_row(Tense::Progressive, false, polarity),
            Tense::FuturePast => self.stacked_row(Tense::Future, true, polarity),
            Tense::FutureEvidential => self.stacked_row(Tense::Future, false, polarity),
            Tense::EvidentialPast => self.stacked_row(Tense::Evidential, true, polarity),
        };
        row[i].clone()
    }

    /// The second-person imperative. Singular is the bare stem (gel,
    /// git → stored gid- verbs excepted); plural takes `-In`, and the
    /// formal plural `-InIz`. Negative prefixes the negative stem.
    #[must_use]
    pub fn imperative(&self, number: Number, polarity: Polarity, formal: bool) -> String {
        match (number, polarity) {
            (Number::Singular, Polarity::Positive) => self
                .row
                .imperative_sg
                .clone()
                .unwrap_or_else(|| self.stem.clone()),
            (Number::Singular, Polarity::Negative) => self.neg_stem(),
            (Number::Plural, polarity) => {
                let base = if polarity == Polarity::Negative {
                    self.neg_stem()
                } else if let Some(p) = &self.row.imperative_pl {
                    // A stored plural already carries -In; strip it back
                    // to the (possibly voiced) stem so -InIz can attach.
                    if formal {
                        let i = four(p);
                        return format!("{p}{i}z");
                    }
                    return p.clone();
                } else {
                    self.stem.clone()
                };
                let i = four(&base);
                if formal {
                    add_vowel_stem(&base, i, &format!("n{i}z"))
                } else {
                    add_vowel_stem(&base, i, "n")
                }
            }
        }
    }
}

/// The full conjugation table of a Turkish verb — shared by the
/// WebAssembly and Python bindings. Every six-slot row is
/// `[1sg, 2sg, 3sg, 1pl, 2pl, 3pl]`, affirmative; negatives are a method
/// away on [`Verb`].
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// `gelir` — aorist.
    pub aorist: [String; 6],
    /// `geliyor` — present progressive.
    pub progressive: [String; 6],
    /// `gelecek` — future.
    pub future: [String; 6],
    /// `geldi` — definite past.
    pub past: [String; 6],
    /// `gelmiş` — evidential.
    pub evidential: [String; 6],
    /// `gelmeli` — necessitative.
    pub necessitative: [String; 6],
    /// `[gel, gelin]` — imperative 2sg and 2pl.
    pub imperative: [String; 2],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row = |t: Tense| SLOTS.map(|(p, n)| v.form(t, p, n, Polarity::Positive));
        Self {
            infinitive: v.infinitive().to_string(),
            aorist: row(Tense::Aorist),
            progressive: row(Tense::Progressive),
            future: row(Tense::Future),
            past: row(Tense::Past),
            evidential: row(Tense::Evidential),
            necessitative: row(Tense::Necessitative),
            imperative: [
                v.imperative(Number::Singular, Polarity::Positive, false),
                v.imperative(Number::Plural, Polarity::Positive, false),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};
    use Polarity::{Negative as NEG, Positive as POS};

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn aorist_default_ar() {
        // Monosyllabic consonant stems default to -Ar.
        assert_eq!(v("yazmak").form(Tense::Aorist, P3, SG, POS), "yazar");
        assert_eq!(v("bakmak").form(Tense::Aorist, P3, SG, POS), "bakar");
        assert_eq!(v("sevmek").form(Tense::Aorist, P3, SG, POS), "sever");
        // Polysyllables take -Ir.
        assert_eq!(v("göndermek").form(Tense::Aorist, P3, SG, POS), "gönderir");
        // Vowel-final stems take -r.
        assert_eq!(v("okumak").form(Tense::Aorist, P3, SG, POS), "okur");
    }

    #[test]
    fn person_endings() {
        let g = v("gelmek");
        // Aorist uses stored -Ir base gelir.
        assert_eq!(g.form(Tense::Aorist, P1, SG, POS), "gelirim");
        assert_eq!(g.form(Tense::Aorist, P2, SG, POS), "gelirsin");
        assert_eq!(g.form(Tense::Aorist, P3, SG, POS), "gelir");
        assert_eq!(g.form(Tense::Aorist, P1, PL, POS), "geliriz");
        assert_eq!(g.form(Tense::Aorist, P2, PL, POS), "gelirsiniz");
        assert_eq!(g.form(Tense::Aorist, P3, PL, POS), "gelirler");
    }

    #[test]
    fn harmony_and_consonants() {
        let g = v("gelmek");
        assert_eq!(g.form(Tense::Progressive, P1, SG, POS), "geliyorum");
        assert_eq!(g.form(Tense::Progressive, P3, SG, POS), "geliyor");
        assert_eq!(g.form(Tense::Future, P1, SG, POS), "geleceğim"); // k→ğ
        assert_eq!(g.form(Tense::Future, P3, SG, POS), "gelecek");
        assert_eq!(g.form(Tense::Necessitative, P1, SG, POS), "gelmeliyim"); // buffer y
        // Back-harmony verb.
        let y = v("yazmak");
        assert_eq!(y.form(Tense::Progressive, P3, SG, POS), "yazıyor");
        assert_eq!(y.form(Tense::Past, P3, SG, POS), "yazdı");
        assert_eq!(y.form(Tense::Future, P1, SG, POS), "yazacağım");
        assert_eq!(y.form(Tense::Necessitative, P3, SG, POS), "yazmalı");
    }

    #[test]
    fn past_assimilation() {
        assert_eq!(v("yapmak").form(Tense::Past, P3, SG, POS), "yaptı");
        assert_eq!(v("gitmek").form(Tense::Past, P3, SG, POS), "gitti");
        assert_eq!(v("okumak").form(Tense::Past, P3, SG, POS), "okudu");
        assert_eq!(v("gelmek").form(Tense::Past, P1, SG, POS), "geldim");
    }

    #[test]
    fn negatives() {
        let g = v("gelmek");
        assert_eq!(g.form(Tense::Aorist, P1, SG, NEG), "gelmem");
        assert_eq!(g.form(Tense::Aorist, P3, SG, NEG), "gelmez");
        assert_eq!(g.form(Tense::Aorist, P1, PL, NEG), "gelmeyiz");
        assert_eq!(g.form(Tense::Aorist, P2, SG, NEG), "gelmezsin");
        assert_eq!(g.form(Tense::Progressive, P3, SG, NEG), "gelmiyor");
        assert_eq!(g.form(Tense::Future, P3, SG, NEG), "gelmeyecek");
        assert_eq!(g.form(Tense::Past, P3, SG, NEG), "gelmedi");
        assert_eq!(g.form(Tense::Evidential, P3, SG, NEG), "gelmemiş");
    }

    #[test]
    fn stacked_tenses() {
        let g = v("gelmek");
        assert_eq!(g.form(Tense::AoristPast, P3, SG, POS), "gelirdi");
        assert_eq!(g.form(Tense::AoristPast, P1, SG, POS), "gelirdim");
        assert_eq!(g.form(Tense::AoristPast, P3, PL, POS), "gelirlerdi");
        assert_eq!(g.form(Tense::AoristEvidential, P3, SG, POS), "gelirmiş");
        assert_eq!(g.form(Tense::AoristEvidential, P3, PL, POS), "gelirlermiş");
        assert_eq!(g.form(Tense::ProgressivePast, P3, SG, POS), "geliyordu");
        assert_eq!(g.form(Tense::ProgressivePast, P3, PL, POS), "geliyorlardı");
        assert_eq!(g.form(Tense::FuturePast, P3, SG, POS), "gelecekti");
        assert_eq!(g.form(Tense::EvidentialPast, P3, SG, POS), "gelmişti");
    }

    #[test]
    fn imperative_and_infinitive() {
        let g = v("gelmek");
        assert_eq!(g.imperative(SG, POS, false), "gel");
        assert_eq!(g.imperative(PL, POS, false), "gelin");
        assert_eq!(g.imperative(PL, POS, true), "geliniz");
        assert_eq!(g.imperative(SG, NEG, false), "gelme");
        assert_eq!(g.imperative(PL, NEG, false), "gelmeyin");
        assert_eq!(g.infinitive(), "gelmek");
    }

    #[test]
    fn rejects_non_verbs() {
        for input in ["ev", "run", "gel mek", ""] {
            assert_eq!(Verb::from_infinitive(input).err(), Some(Error::NotAVerb));
        }
    }
}
