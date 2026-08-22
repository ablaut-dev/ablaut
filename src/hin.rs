//! Hindi conjugation: a productive rule engine over one open verb class
//! plus a small compiled-in table of the verbs with an unpredictable
//! perfective or subjunctive stem.
//!
//! Every Hindi verb is cited by its infinitive, which ends in `-ना`
//! (उठना "rise", खाना "eat"). Dropping the `-ना` leaves the **stem**
//! (उठ, खा), and the whole finite system is built on it:
//!
//! - the **subjunctive** agrees in person and number (उठूं, उठे, उठें,
//!   उठो) — the one synthetic non-participle finite form;
//! - the **synthetic future** in `-गा/-गे/-गी` is the subjunctive plus a
//!   gender/number-agreeing suffix (उठूंगा, उठेगी, उठेंगे);
//! - three **imperatives** by politeness — तू उठ, तुम उठो, आप उठिए;
//! - the **imperfective** and **perfective participles**, each agreeing
//!   in gender and number (उठता/उठते/उठती, उठा/उठे/उठी).
//!
//! On top of that sits a large but regular **analytic layer**: an aspect
//! stem (imperfective participle, perfective participle, or `stem रहा`
//! for the progressive) followed by the conjugated copula होना. Three
//! aspects (habitual, perfect, progressive) each cross five auxiliary
//! tense-moods (present, past, subjunctive, presumptive, counterfactual)
//! and agree in gender and number.
//!
//! A stem ending in a vowel takes the euphonic य-glide and independent
//! vowel signs where a consonant stem takes bare matras (खाया vs उठा,
//! खाए vs उठे). Compound and light-verb lemmas (`अध्ययन करना`,
//! `हो जाना`) conjugate only their last word; the rest is carried along
//! unchanged. What the rules cannot predict — the suppletive and
//! contracted stems of होना, जाना, करना, देना, लेना, पीना and their kin
//! — lives in `data/hin/verbs.tsv`.

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
/// distinguishes: तू (intimate), तुम (familiar), आप (polite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Politeness {
    Intimate,
    Familiar,
    Polite,
}

/// A participle, agreeing in gender and number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Aspect {
    /// उठता — the imperfective (habitual) participle.
    Imperfective,
    /// उठा — the perfective participle.
    Perfective,
    /// उठ रहा — the progressive (stem + रहा).
    Progressive,
}

/// The tense-mood of the copula होना in an analytic form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuxMood {
    /// है — present (उठता है).
    Present,
    /// था — past (उठता था).
    Past,
    /// हो — subjunctive (उठता हो).
    Subjunctive,
    /// होगा — presumptive / future (उठता होगा).
    Presumptive,
    /// होता — counterfactual (उठता होता).
    Counterfactual,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input's last word does not end in `-ना`.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Hindi infinitive")
    }
}

/// Dependent vowel signs (matras) and the independent vowels: a stem
/// ending in one of these is vowel-final and takes the य-glide.
const VOWEL_SIGNS: &str = "ािीुूृेैोौअआइईउऊएऐओऔ";

/// The compiled-in table of verbs whose stems are not derivable.
static LEXICON_TSV: &str = include_str!("../data/hin/verbs.tsv");

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
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
}

fn six(s: &str) -> Option<[String; 6]> {
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
                    perfective: triple(g(1)),
                    subjunctive: six(g(2)),
                    imperative: triple(g(3)),
                },
            );
        }
        m
    })
}

/// A conjugatable Hindi verb.
#[derive(Debug, Clone)]
pub struct Verb {
    /// The invariable material before the verb word (`अध्ययन ` in
    /// अध्ययन करना), empty for a simple verb. Re-attached to every form.
    prefix: String,
    /// The verb word's infinitive (करना), without the prefix.
    infinitive: String,
    /// The stem (उठ, खा, कर).
    stem: String,
    /// Whether the stem ends in a vowel.
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
    s.chars().last().is_some_and(|c| VOWEL_SIGNS.contains(c))
}

impl Verb {
    /// Build a verb from its infinitive.
    ///
    /// ```
    /// use ablaut::hin::{Gender, Number, Person, Verb};
    /// let v = Verb::from_infinitive("उठना").unwrap();
    /// assert_eq!(v.subjunctive(Person::First, Number::Singular), "उठूं");
    /// assert_eq!(
    ///     v.future(Person::Third, Number::Singular, Gender::Masculine),
    ///     "उठेगा"
    /// );
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::NotAVerb`] when the last word does not end in
    /// `-ना`.
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
        if !word.ends_with("ना") {
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
        format!("{}{}", self.prefix, self.infinitive)
    }

    /// The oblique infinitive (उठने).
    #[must_use]
    pub fn oblique_infinitive(&self) -> String {
        self.out(&format!("{}ने", self.stem))
    }

    /// Re-attach the invariable prefix.
    fn out(&self, form: &str) -> String {
        format!("{}{}", self.prefix, form)
    }

    /// One of a pair of endings, choosing the vowel-stem variant.
    fn end(&self, consonant: &'static str, vowel: &'static str) -> &'static str {
        if self.vowel {
            vowel
        } else {
            consonant
        }
    }

    /// The imperfective participle stem+ता/ते/ती.
    #[must_use]
    pub fn imperfective(&self, gender: Gender, number: Number) -> String {
        let end = match (gender, number) {
            (Gender::Masculine, Number::Singular) => "ता",
            (Gender::Masculine, Number::Plural) => "ते",
            (Gender::Feminine, _) => "ती",
        };
        self.out(&format!("{}{end}", self.stem))
    }

    /// The perfective participle (उठा/उठे/उठी), from the lexicon or the
    /// rule.
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
            0 => self.end("ा", "या"),
            1 => self.end("े", "ए"),
            _ => self.end("ी", "ई"),
        };
        self.out(&format!("{}{end}", self.stem))
    }

    /// The `stem रहा` progressive base, agreeing in gender and number.
    fn progressive(&self, gender: Gender, number: Number) -> String {
        let raha = match (gender, number) {
            (Gender::Masculine, Number::Singular) => "रहा",
            (Gender::Masculine, Number::Plural) => "रहे",
            (Gender::Feminine, _) => "रही",
        };
        format!("{} {raha}", self.stem)
    }

    /// The aspect stem an analytic form is built on (without the prefix).
    fn aspect_stem(&self, aspect: Aspect, gender: Gender, number: Number) -> String {
        match aspect {
            Aspect::Imperfective => {
                let end = match (gender, number) {
                    (Gender::Masculine, Number::Singular) => "ता",
                    (Gender::Masculine, Number::Plural) => "ते",
                    (Gender::Feminine, _) => "ती",
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

    /// The subjunctive (उठूं, उठे…), from the lexicon or the rule.
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
            (Person::First, Number::Singular) => self.end("ूं", "ऊं"),
            (Person::First | Person::Third, Number::Plural) => self.end("ें", "एं"),
            (Person::Second, Number::Plural) => self.end("ो", "ओ"),
            (Person::Second | Person::Third, Number::Singular) => self.end("े", "ए"),
        }
    }

    /// The synthetic future (उठूंगा, उठेगी), subjunctive + `-गा/-गे/-गी`.
    #[must_use]
    pub fn future(&self, person: Person, number: Number, gender: Gender) -> String {
        let base = self.subjunctive_base(person, number);
        let suffix = match (gender, number) {
            (Gender::Masculine, Number::Singular) => "गा",
            (Gender::Masculine, Number::Plural) => "गे",
            (Gender::Feminine, _) => "गी",
        };
        self.out(&format!("{base}{suffix}"))
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
            Politeness::Familiar => format!("{}{}", self.stem, self.end("ो", "ओ")),
            Politeness::Polite => format!("{}{}", self.stem, self.end("िए", "इए")),
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

/// The conjugated copula होना, by auxiliary mood. The present and
/// subjunctive agree in person and number; the past and counterfactual
/// in gender and number only; the presumptive in all three.
fn copula(mood: AuxMood, person: Person, number: Number, gender: Gender) -> &'static str {
    let i = person.index(number);
    match mood {
        AuxMood::Present => ["हूं", "है", "है", "हैं", "हो", "हैं"][i],
        AuxMood::Subjunctive => ["हूं", "हो", "हो", "हों", "हो", "हों"][i],
        AuxMood::Past => match (gender, number) {
            (Gender::Masculine, Number::Singular) => "था",
            (Gender::Masculine, Number::Plural) => "थे",
            (Gender::Feminine, _) => "थी",
        },
        AuxMood::Counterfactual => match (gender, number) {
            (Gender::Masculine, Number::Singular) => "होता",
            (Gender::Masculine, Number::Plural) => "होते",
            (Gender::Feminine, _) => "होती",
        },
        AuxMood::Presumptive => match gender {
            Gender::Masculine => ["हूंगा", "होगा", "होगा", "होंगे", "होगे", "होंगे"][i],
            Gender::Feminine => ["हूंगी", "होगी", "होगी", "होंगी", "होगी", "होंगी"][i],
        },
    }
}

/// The conjugation table of a Hindi verb — shared by the WebAssembly and
/// Python bindings. Person rows are [1sg, 2sg, 3sg, 1pl, 2pl, 3pl];
/// participle triples are [masc sg, masc pl, fem].
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub oblique_infinitive: String,
    /// [intimate (तू), familiar (तुम), polite (आप)].
    pub imperative: [String; 3],
    /// उठूं… — the subjunctive.
    pub subjunctive: [String; 6],
    /// उठूंगा… — the masculine synthetic future.
    pub future_masculine: [String; 6],
    /// उठूंगी… — the feminine synthetic future.
    pub future_feminine: [String; 6],
    /// [उठता, उठते, उठती] — the imperfective participle.
    pub imperfective: [String; 3],
    /// [उठा, उठे, उठी] — the perfective participle.
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
        let u = v("उठना");
        assert_eq!(u.infinitive(), "उठना");
        assert_eq!(u.oblique_infinitive(), "उठने");
        assert_eq!(u.subjunctive(P1, SG), "उठूं");
        assert_eq!(u.subjunctive(P2, SG), "उठे");
        assert_eq!(u.subjunctive(P2, PL), "उठो");
        assert_eq!(u.subjunctive(P1, PL), "उठें");
        assert_eq!(u.future(P1, SG, M), "उठूंगा");
        assert_eq!(u.future(P1, SG, F), "उठूंगी");
        assert_eq!(u.future(P2, PL, M), "उठोगे");
        assert_eq!(u.future(P1, PL, M), "उठेंगे");
        assert_eq!(u.imperative(Politeness::Intimate), "उठ");
        assert_eq!(u.imperative(Politeness::Familiar), "उठो");
        assert_eq!(u.imperative(Politeness::Polite), "उठिए");
        assert_eq!(u.imperfective(M, SG), "उठता");
        assert_eq!(u.imperfective(M, PL), "उठते");
        assert_eq!(u.imperfective(F, SG), "उठती");
        assert_eq!(u.perfective(M, SG), "उठा");
        assert_eq!(u.perfective(M, PL), "उठे");
        assert_eq!(u.perfective(F, SG), "उठी");
    }

    #[test]
    fn vowel_stem_glide() {
        let k = v("खाना");
        assert_eq!(k.subjunctive(P1, SG), "खाऊं");
        assert_eq!(k.subjunctive(P2, SG), "खाए");
        assert_eq!(k.subjunctive(P2, PL), "खाओ");
        assert_eq!(k.future(P2, SG, M), "खाएगा");
        assert_eq!(k.imperative(Politeness::Intimate), "खा");
        assert_eq!(k.imperative(Politeness::Familiar), "खाओ");
        assert_eq!(k.imperative(Politeness::Polite), "खाइए");
        assert_eq!(k.imperfective(M, SG), "खाता");
        assert_eq!(k.perfective(M, SG), "खाया");
        assert_eq!(k.perfective(M, PL), "खाए");
        assert_eq!(k.perfective(F, SG), "खाई");
    }

    #[test]
    fn analytic_layer() {
        let u = v("उठना");
        assert_eq!(
            u.analytic(Aspect::Imperfective, AuxMood::Present, P1, SG, M),
            "उठता हूं"
        );
        assert_eq!(
            u.analytic(Aspect::Imperfective, AuxMood::Present, P3, SG, M),
            "उठता है"
        );
        assert_eq!(
            u.analytic(Aspect::Imperfective, AuxMood::Present, P1, PL, M),
            "उठते हैं"
        );
        assert_eq!(
            u.analytic(Aspect::Imperfective, AuxMood::Past, P3, SG, F),
            "उठती थी"
        );
        assert_eq!(
            u.analytic(Aspect::Perfective, AuxMood::Present, P3, SG, M),
            "उठा है"
        );
        assert_eq!(
            u.analytic(Aspect::Progressive, AuxMood::Present, P1, SG, M),
            "उठ रहा हूं"
        );
        assert_eq!(
            u.analytic(Aspect::Progressive, AuxMood::Past, P3, SG, F),
            "उठ रही थी"
        );
        assert_eq!(
            u.analytic(Aspect::Imperfective, AuxMood::Presumptive, P3, SG, M),
            "उठता होगा"
        );
        assert_eq!(
            u.analytic(Aspect::Perfective, AuxMood::Counterfactual, P1, PL, M),
            "उठे होते"
        );
    }

    #[test]
    fn irregular_perfective() {
        // करना: irregular perfective किया/किए/की, regular elsewhere.
        let k = v("करना");
        assert_eq!(k.subjunctive(P1, SG), "करूं");
        assert_eq!(k.perfective(M, SG), "किया");
        assert_eq!(k.perfective(F, SG), "की");
        assert_eq!(
            k.analytic(Aspect::Perfective, AuxMood::Present, P3, SG, M),
            "किया है"
        );
        // होना, जाना: suppletive perfective.
        assert_eq!(v("होना").perfective(M, SG), "हुआ");
        assert_eq!(v("जाना").perfective(M, SG), "गया");
        assert_eq!(v("जाना").perfective(F, SG), "गई");
        // देना: contracted subjunctive and perfective.
        let d = v("देना");
        assert_eq!(d.subjunctive(P1, SG), "दूं");
        assert_eq!(d.future(P1, SG, M), "दूंगा");
        assert_eq!(d.perfective(M, SG), "दिया");
        assert_eq!(d.imperative(Politeness::Polite), "दीजिए");
    }

    #[test]
    fn compound_lemma() {
        let a = v("अध्ययन करना");
        assert_eq!(a.infinitive(), "अध्ययन करना");
        assert_eq!(a.subjunctive(P1, SG), "अध्ययन करूं");
        assert_eq!(a.perfective(M, SG), "अध्ययन किया");
        assert_eq!(
            a.analytic(Aspect::Imperfective, AuxMood::Present, P3, SG, M),
            "अध्ययन करता है"
        );
    }

    #[test]
    fn rejects_non_verbs() {
        for input in ["", "किताब", "run"] {
            assert_eq!(Verb::from_infinitive(input).err(), Some(Error::NotAVerb));
        }
    }
}
