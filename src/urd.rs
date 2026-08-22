//! Urdu (`urd`) conjugation. Urdu is Hindustani written in the
//! Perso-Arabic (Nastaliq) script — the *same* spoken language as Hindi
//! (`crate::hin`) — so the morphology is identical and this engine is a
//! script twin of the Hindi one: a productive rule over one open verb
//! class plus a small compiled-in table of the verbs with an
//! unpredictable perfective, subjunctive or imperative.
//!
//! Every Urdu verb is cited by its infinitive, which ends in `ـنا`
//! (اترنا "descend", کھانا "eat"). Dropping the `نا` leaves the **stem**
//! (اتر, کھا), and the whole finite system is built on it:
//!
//! - the **subjunctive** agrees in person and number (اتروں, اترے, اتریں,
//!   اترو) — the one synthetic non-participle finite form;
//! - the **synthetic future** is the subjunctive plus a gender/number
//!   agreeing particle written apart, `گا/گے/گی` (اتروں گا, اترے گی);
//! - three **imperatives** by politeness — تو اتر, تم اترو, آپ اترئیے;
//! - the **imperfective** and **perfective participles**, each agreeing
//!   in gender and number (اترتا/اترتے/اترتی, اترا/اترے/اتری).
//!
//! On top of that sits a large but regular **analytic layer**: an aspect
//! stem (imperfective participle, perfective participle, or `stem رہا`
//! for the progressive) followed by the conjugated copula ہونا. Three
//! aspects (habitual, perfect, progressive) each cross five auxiliary
//! tense-moods (present, past, subjunctive, presumptive, counterfactual)
//! and agree in gender and number.
//!
//! A stem ending in a vowel (ا/آ/و) takes the euphonic hamza-glide where
//! a consonant stem takes bare endings (کھایا vs اترا, کھاؤں vs اتروں).
//! Compound and light-verb lemmas conjugate only their last word; the
//! rest is carried along unchanged. What the rules cannot predict — the
//! suppletive and contracted stems of ہونا, جانا, کرنا, دینا, لینا,
//! پینا and their kin — lives in `data/urd/verbs.tsv`.
//!
//! All forms are produced in the normalized Perso-Arabic orthography of
//! [`crate::perso_arabic`] (short-vowel diacritics stripped, the optional
//! noon-ghunna nasal mark dropped, Arabic letters folded to their
//! Perso-Arabic shapes), matching what the oracle adapters emit.

use crate::perso_arabic;
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

/// Grammatical gender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Masculine,
    Feminine,
}

/// The politeness level of the second person, which the imperative
/// distinguishes: تو (intimate), تم (familiar), آپ (polite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Politeness {
    Intimate,
    Familiar,
    Polite,
}

/// A participle/analytic aspect, agreeing in gender and number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Aspect {
    /// اترتا — the imperfective (habitual) participle.
    Imperfective,
    /// اترا — the perfective participle.
    Perfective,
    /// اتر رہا — the progressive (stem + رہا).
    Progressive,
}

/// The tense-mood of the copula ہونا in an analytic form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuxMood {
    /// ہے — present (اترتا ہے).
    Present,
    /// تھا — past (اترتا تھا).
    Past,
    /// ہو — subjunctive (اترتا ہو).
    Subjunctive,
    /// ہو گا — presumptive / future (اترتا ہو گا).
    Presumptive,
    /// ہوتا — counterfactual (اترتا ہوتا).
    Counterfactual,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input's last word does not end in `ـنا`.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Urdu infinitive")
    }
}

/// Stem-final letters that make a stem vowel-final and so take the
/// hamza-glide: alef ا, alef-madda آ and waw و (کھا, آ, رو). A yeh-final
/// stem (پی-, دی-) contracts irregularly and lives in the lexicon.
const VOWELS: [char; 3] = ['\u{0627}', '\u{0622}', '\u{0648}'];

/// The compiled-in table of verbs whose stems are not derivable.
static LEXICON_TSV: &str = include_str!("../data/urd/verbs.tsv");

/// A stored paradigm. Every group is optional: a `-` cell falls through
/// to the productive rule.
#[derive(Debug, Clone, Default)]
struct LexEntry {
    /// Perfective participle [masc sg, masc pl, fem].
    perfective: Option<[String; 3]>,
    /// Subjunctive [1sg, 2sg, 3sg, 1pl, 2pl, 3pl]; the future is built
    /// on it.
    subjunctive: Option<[String; 6]>,
    /// Imperative [intimate, familiar, polite].
    imperative: Option<[String; 3]>,
}

fn triple(s: &str) -> Option<[String; 3]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(perso_arabic::normalize).collect();
    v.try_into().ok()
}

fn six(s: &str) -> Option<[String; 6]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(perso_arabic::normalize).collect();
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
                perso_arabic::normalize(c[0]),
                LexEntry {
                    perfective: triple(g(1)),
                    subjunctive: six(g(2)),
                    imperative: triple(g(3)),
                },
            );
        }
        m
    })
}

/// A conjugatable Urdu verb.
#[derive(Debug, Clone)]
pub struct Verb {
    /// The invariable material before the verb word, empty for a simple
    /// verb. Re-attached to every form.
    prefix: String,
    /// The verb word's infinitive (کرنا), without the prefix.
    infinitive: String,
    /// The stem (اتر, کھا, کر).
    stem: String,
    /// Whether the stem ends in a vowel (ا/آ/و).
    vowel: bool,
    lex: Option<&'static LexEntry>,
}

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

fn ends_with_vowel(s: &str) -> bool {
    s.chars().last().is_some_and(|c| VOWELS.contains(&c))
}

impl Verb {
    /// Build a verb from its infinitive.
    ///
    /// ```
    /// use ablaut::urd::{Gender, Number, Person, Verb};
    /// let v = Verb::from_infinitive("اترنا").unwrap();
    /// assert_eq!(v.subjunctive(Person::First, Number::Singular), "اتروں");
    /// assert_eq!(
    ///     v.perfective(Gender::Masculine, Number::Singular),
    ///     "اترا"
    /// );
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::NotAVerb`] when the last word does not end in
    /// `ـنا`.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = perso_arabic::normalize(infinitive.trim());
        if inf.is_empty() {
            return Err(Error::NotAVerb);
        }
        // A compound lemma conjugates only its last word.
        let (prefix, word) = match inf.rfind(' ') {
            Some(i) => (inf[..=i].to_string(), inf[i + 1..].to_string()),
            None => (String::new(), inf.clone()),
        };
        if !word.ends_with("نا") {
            return Err(Error::NotAVerb);
        }
        let stem: String = {
            let n = word.chars().count();
            word.chars().take(n.saturating_sub(2)).collect()
        };
        if stem.is_empty() {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            vowel: ends_with_vowel(&stem),
            lex: lexicon().get(&word),
            prefix,
            infinitive: word,
            stem,
        })
    }

    /// The normalized infinitive.
    #[must_use]
    pub fn infinitive(&self) -> String {
        self.out(&self.infinitive)
    }

    /// The oblique infinitive (اترنے).
    #[must_use]
    pub fn oblique_infinitive(&self) -> String {
        self.out(&format!("{}نے", self.stem))
    }

    /// Re-attach the invariable prefix and re-normalize.
    fn out(&self, form: &str) -> String {
        perso_arabic::normalize(&format!("{}{}", self.prefix, form))
    }

    /// One of a pair of endings, choosing the vowel-stem variant.
    fn end(&self, consonant: &'static str, vowel: &'static str) -> &'static str {
        if self.vowel {
            vowel
        } else {
            consonant
        }
    }

    /// The imperfective participle stem+تا/تے/تی.
    #[must_use]
    pub fn imperfective(&self, gender: Gender, number: Number) -> String {
        let end = match (gender, number) {
            (Gender::Masculine, Number::Singular) => "تا",
            (Gender::Masculine, Number::Plural) => "تے",
            (Gender::Feminine, _) => "تی",
        };
        self.out(&format!("{}{end}", self.stem))
    }

    /// The perfective participle (اترا/اترے/اتری), from the lexicon or
    /// the rule.
    #[must_use]
    pub fn perfective(&self, gender: Gender, number: Number) -> String {
        let i = match (gender, number) {
            (Gender::Masculine, Number::Singular) => 0,
            (Gender::Masculine, Number::Plural) => 1,
            (Gender::Feminine, _) => 2,
        };
        if let Some(p) = self.lex.and_then(|l| l.perfective.as_ref()) {
            return self.out(&p[i]);
        }
        let end = match i {
            0 => self.end("ا", "یا"),
            1 => self.end("ے", "ئے"),
            _ => self.end("ی", "ئی"),
        };
        self.out(&format!("{}{end}", self.stem))
    }

    /// The `stem رہا` progressive base, agreeing in gender and number.
    fn progressive(&self, gender: Gender, number: Number) -> String {
        let raha = match (gender, number) {
            (Gender::Masculine, Number::Singular) => "رہا",
            (Gender::Masculine, Number::Plural) => "رہے",
            (Gender::Feminine, _) => "رہی",
        };
        format!("{} {raha}", self.stem)
    }

    /// The aspect stem an analytic form is built on (without the prefix).
    fn aspect_stem(&self, aspect: Aspect, gender: Gender, number: Number) -> String {
        match aspect {
            Aspect::Imperfective => {
                let end = match (gender, number) {
                    (Gender::Masculine, Number::Singular) => "تا",
                    (Gender::Masculine, Number::Plural) => "تے",
                    (Gender::Feminine, _) => "تی",
                };
                format!("{}{end}", self.stem)
            }
            Aspect::Perfective => {
                // Reuse the (possibly stored) perfective participle,
                // stripped of the prefix which `out` adds back later.
                let full = self.perfective(gender, number);
                full[self.prefix.len()..].to_string()
            }
            Aspect::Progressive => self.progressive(gender, number),
        }
    }

    /// The subjunctive (اتروں, اترے…), from the lexicon or the rule.
    #[must_use]
    pub fn subjunctive(&self, person: Person, number: Number) -> String {
        let i = person.index(number);
        if let Some(s) = self.lex.and_then(|l| l.subjunctive.as_ref()) {
            return self.out(&s[i]);
        }
        self.out(&format!(
            "{}{}",
            self.stem,
            self.subjunctive_ending(person, number)
        ))
    }

    /// The bare subjunctive form (no prefix), for building the future.
    fn subjunctive_base(&self, person: Person, number: Number) -> String {
        let i = person.index(number);
        if let Some(s) = self.lex.and_then(|l| l.subjunctive.as_ref()) {
            return s[i].clone();
        }
        format!("{}{}", self.stem, self.subjunctive_ending(person, number))
    }

    fn subjunctive_ending(&self, person: Person, number: Number) -> &'static str {
        match (person, number) {
            (Person::First, Number::Singular) => self.end("وں", "ؤں"),
            (Person::First | Person::Third, Number::Plural) => self.end("یں", "ئیں"),
            (Person::Second, Number::Plural) => self.end("و", "ؤ"),
            (Person::Second | Person::Third, Number::Singular) => self.end("ے", "ئے"),
        }
    }

    /// The synthetic future (اتروں گا, اترے گی), subjunctive + the
    /// gender/number particle `گا/گے/گی` written apart.
    #[must_use]
    pub fn future(&self, person: Person, number: Number, gender: Gender) -> String {
        let base = self.subjunctive_base(person, number);
        let particle = match (gender, number) {
            (Gender::Masculine, Number::Singular) => "گا",
            (Gender::Masculine, Number::Plural) => "گے",
            (Gender::Feminine, _) => "گی",
        };
        self.out(&format!("{base} {particle}"))
    }

    /// An imperative at one politeness level, from the lexicon or rule.
    #[must_use]
    pub fn imperative(&self, politeness: Politeness) -> String {
        let i = match politeness {
            Politeness::Intimate => 0,
            Politeness::Familiar => 1,
            Politeness::Polite => 2,
        };
        if let Some(imp) = self.lex.and_then(|l| l.imperative.as_ref()) {
            return self.out(&imp[i]);
        }
        let form = match politeness {
            Politeness::Intimate => self.stem.clone(),
            Politeness::Familiar => format!("{}{}", self.stem, self.end("و", "ؤ")),
            Politeness::Polite => format!("{}{}", self.stem, "ئیے"),
        };
        self.out(&form)
    }

    /// An analytic form: an aspect stem plus the conjugated copula.
    #[must_use]
    pub fn analytic(
        &self,
        aspect: Aspect,
        mood: AuxMood,
        person: Person,
        number: Number,
        gender: Gender,
    ) -> String {
        let stem = self.aspect_stem(aspect, gender, number);
        let aux = copula(mood, person, number, gender);
        self.out(&format!("{stem} {aux}"))
    }
}

/// The conjugated copula ہونا, by auxiliary mood. The present and
/// subjunctive agree in person and number; the past and counterfactual
/// in gender and number only; the presumptive in all three (and is
/// written as two words, ہو گا).
fn copula(mood: AuxMood, person: Person, number: Number, gender: Gender) -> &'static str {
    let i = person.index(number);
    match mood {
        AuxMood::Present => ["ہوں", "ہے", "ہے", "ہیں", "ہو", "ہیں"][i],
        AuxMood::Subjunctive => ["ہوں", "ہو", "ہو", "ہوں", "ہو", "ہوں"][i],
        AuxMood::Past => match (gender, number) {
            (Gender::Masculine, Number::Singular) => "تھا",
            (Gender::Masculine, Number::Plural) => "تھے",
            (Gender::Feminine, _) => "تھی",
        },
        AuxMood::Counterfactual => match (gender, number) {
            (Gender::Masculine, Number::Singular) => "ہوتا",
            (Gender::Masculine, Number::Plural) => "ہوتے",
            (Gender::Feminine, _) => "ہوتی",
        },
        AuxMood::Presumptive => match gender {
            Gender::Masculine => ["ہوں گا", "ہو گا", "ہو گا", "ہوں گے", "ہو گے", "ہوں گے"][i],
            Gender::Feminine => ["ہوں گی", "ہو گی", "ہو گی", "ہوں گی", "ہو گی", "ہوں گی"][i],
        },
    }
}

/// The conjugation table of an Urdu verb — shared by the WebAssembly and
/// Python bindings. Person rows are [1sg, 2sg, 3sg, 1pl, 2pl, 3pl];
/// participle triples are [masc sg, masc pl, fem].
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub oblique_infinitive: String,
    /// [intimate (تو), familiar (تم), polite (آپ)].
    pub imperative: [String; 3],
    /// اتروں… — the subjunctive.
    pub subjunctive: [String; 6],
    /// اتروں گا… — the masculine synthetic future.
    pub future_masculine: [String; 6],
    /// اتروں گی… — the feminine synthetic future.
    pub future_feminine: [String; 6],
    /// [اترتا, اترتے, اترتی] — the imperfective participle.
    pub imperfective: [String; 3],
    /// [اترا, اترے, اتری] — the perfective participle.
    pub perfective: [String; 3],
}

const SLOTS: [(Person, Number); 6] = [
    (Person::First, Number::Singular),
    (Person::Second, Number::Singular),
    (Person::Third, Number::Singular),
    (Person::First, Number::Plural),
    (Person::Second, Number::Plural),
    (Person::Third, Number::Plural),
];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let ptcp = |asp: fn(&Verb, Gender, Number) -> String| {
            [
                asp(v, Gender::Masculine, Number::Singular),
                asp(v, Gender::Masculine, Number::Plural),
                asp(v, Gender::Feminine, Number::Singular),
            ]
        };
        Self {
            infinitive: v.infinitive(),
            oblique_infinitive: v.oblique_infinitive(),
            imperative: [
                v.imperative(Politeness::Intimate),
                v.imperative(Politeness::Familiar),
                v.imperative(Politeness::Polite),
            ],
            subjunctive: SLOTS.map(|(p, n)| v.subjunctive(p, n)),
            future_masculine: SLOTS.map(|(p, n)| v.future(p, n, Gender::Masculine)),
            future_feminine: SLOTS.map(|(p, n)| v.future(p, n, Gender::Feminine)),
            imperfective: ptcp(Verb::imperfective),
            perfective: ptcp(Verb::perfective),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Gender::{Feminine as F, Masculine as M};
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn regular_consonant_stem() {
        let u = v("اترنا");
        assert_eq!(u.infinitive(), "اترنا");
        assert_eq!(u.oblique_infinitive(), "اترنے");
        assert_eq!(u.subjunctive(P1, SG), "اتروں");
        assert_eq!(u.subjunctive(P2, SG), "اترے");
        assert_eq!(u.subjunctive(P2, PL), "اترو");
        assert_eq!(u.subjunctive(P1, PL), "اتریں");
        assert_eq!(u.future(P1, SG, M), "اتروں گا");
        assert_eq!(u.future(P1, SG, F), "اتروں گی");
        assert_eq!(u.future(P2, PL, M), "اترو گے");
        assert_eq!(u.imperative(Politeness::Intimate), "اتر");
        assert_eq!(u.imperative(Politeness::Familiar), "اترو");
        assert_eq!(u.imperative(Politeness::Polite), "اترئیے");
        assert_eq!(u.imperfective(M, SG), "اترتا");
        assert_eq!(u.imperfective(M, PL), "اترتے");
        assert_eq!(u.imperfective(F, SG), "اترتی");
        assert_eq!(u.perfective(M, SG), "اترا");
        assert_eq!(u.perfective(M, PL), "اترے");
        assert_eq!(u.perfective(F, SG), "اتری");
    }

    #[test]
    fn vowel_stem_glide() {
        let k = v("کھانا");
        assert_eq!(k.subjunctive(P1, SG), "کھاؤں");
        assert_eq!(k.subjunctive(P2, SG), "کھائے");
        assert_eq!(k.subjunctive(P2, PL), "کھاؤ");
        assert_eq!(k.subjunctive(P1, PL), "کھائیں");
        assert_eq!(k.imperative(Politeness::Intimate), "کھا");
        assert_eq!(k.imperative(Politeness::Familiar), "کھاؤ");
        assert_eq!(k.imperative(Politeness::Polite), "کھائیے");
        assert_eq!(k.imperfective(M, SG), "کھاتا");
        assert_eq!(k.perfective(M, SG), "کھایا");
        assert_eq!(k.perfective(M, PL), "کھائے");
        assert_eq!(k.perfective(F, SG), "کھائی");
    }

    #[test]
    fn analytic_layer() {
        let u = v("اترنا");
        assert_eq!(
            u.analytic(Aspect::Imperfective, AuxMood::Present, P1, SG, M),
            "اترتا ہوں"
        );
        assert_eq!(
            u.analytic(Aspect::Imperfective, AuxMood::Present, P3, SG, M),
            "اترتا ہے"
        );
        assert_eq!(
            u.analytic(Aspect::Imperfective, AuxMood::Past, P3, SG, F),
            "اترتی تھی"
        );
        assert_eq!(
            u.analytic(Aspect::Perfective, AuxMood::Present, P3, SG, M),
            "اترا ہے"
        );
        assert_eq!(
            u.analytic(Aspect::Progressive, AuxMood::Present, P1, SG, M),
            "اتر رہا ہوں"
        );
        assert_eq!(
            u.analytic(Aspect::Imperfective, AuxMood::Presumptive, P3, SG, M),
            "اترتا ہو گا"
        );
    }

    #[test]
    fn irregular_lexicon() {
        // کرنا: irregular perfective کیا/کیے/کی and polite imperative
        // کیجیے, regular subjunctive.
        let k = v("کرنا");
        assert_eq!(k.subjunctive(P1, SG), "کروں");
        assert_eq!(k.perfective(M, SG), "کیا");
        assert_eq!(k.perfective(F, SG), "کی");
        assert_eq!(k.imperative(Politeness::Polite), "کیجیے");
        // ہونا, جانا: suppletive perfective.
        assert_eq!(v("ہونا").perfective(M, SG), "ہوا");
        assert_eq!(v("جانا").perfective(M, SG), "گیا");
        assert_eq!(v("جانا").perfective(F, SG), "گئی");
        // دینا: contracted subjunctive and perfective.
        let d = v("دینا");
        assert_eq!(d.subjunctive(P1, SG), "دوں");
        assert_eq!(d.perfective(M, SG), "دیا");
        assert_eq!(d.imperative(Politeness::Polite), "دیجیے");
    }

    #[test]
    fn compound_lemma() {
        let a = v("حاصل کرنا");
        assert_eq!(a.infinitive(), "حاصل کرنا");
        assert_eq!(a.subjunctive(P1, SG), "حاصل کروں");
        assert_eq!(a.perfective(M, SG), "حاصل کیا");
    }

    #[test]
    fn rejects_non_verbs() {
        for input in ["", "کتاب", "run"] {
            assert_eq!(Verb::from_infinitive(input).err(), Some(Error::NotAVerb));
        }
    }
}
