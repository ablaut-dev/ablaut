//! Gujarati conjugation: a productive rule engine over one open verb
//! class plus a small compiled-in table of the suppletive verbs.
//!
//! Every Gujarati verb is cited by its infinitive, which ends in `-વું`
//! (કરવું "do", લખવું "write", નહાવું "bathe"). Dropping the `-વું`
//! leaves the **stem** (કર, લખ, નહા), and the whole finite system is
//! built on it:
//!
//! - the **present/subjunctive** agrees in person and number (કરું, કરે,
//!   કરીએ, કરો) — the general present, homophonous with the subjunctive;
//! - the **future** in `-શ-` agrees in person and number (કરીશ, કરશે,
//!   કરીશું, કરશો);
//! - the **imperative** by politeness — તું કર, તમે કરો, and the polite
//!   deferred forms કરજે / કરજો;
//! - the **perfective participle** (the past), agreeing in gender and
//!   number across Gujarati's three genders (કર્યો masc, કરી fem, કર્યું
//!   neut; કર્યા / કર્યાં plural);
//! - the **imperfective participle** (કરતો/કરતી/કરતું…), likewise
//!   gender/number-agreeing;
//! - two non-finite forms, the **perfective converb** કરી ("having
//!   done") and the **consecutive** કરીને, and the **verbal noun**
//!   કરવાનું.
//!
//! On top sits an **analytic layer**: the present form plus the present
//! copula છ- gives the present progressive (કરું છું), and the
//! imperfective participle plus a form of હોવું gives the past
//! progressive (કરતું હતું) and the counterfactual (કરતું હોત).
//!
//! A stem ending in a vowel takes the independent vowel signs where a
//! consonant stem takes bare matras (નહાઉં vs કરું, નહાયું vs કર્યું),
//! exactly the Indo-Aryan glide/matra split Hindi shows. Compound and
//! light-verb lemmas (`વચ્ચે આવવું`) conjugate only their last word; the
//! rest is carried along unchanged. What the rules cannot predict — the
//! suppletive past of જવું (ગયું), the `-ધ-` pasts of લેવું/દેવું/પીવું/
//! ખાવું, and the copula હોવું — lives in `data/guj/verbs.tsv`.

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

/// Grammatical gender: Gujarati distinguishes three on participles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// The politeness of the second person, which the imperative
/// distinguishes: the plain command (તું કર, તમે કરો) versus the
/// deferred/polite `-જે`/`-જો` (કરજે, કરજો).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Politeness {
    /// તું — plain intimate (bare stem).
    Intimate,
    /// તમે — plain familiar/plural (stem + ઓ/ો).
    Familiar,
    /// The deferred polite singular (કરજે).
    PoliteSingular,
    /// The deferred polite plural (કરજો).
    PolitePlural,
}

/// The aspect stem an analytic form is built on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Aspect {
    /// કરું છું — the present form plus the present copula (progressive).
    Present,
    /// કરતું હતું — the imperfective participle plus the past copula.
    Imperfective,
}

/// The copula form an analytic layer uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuxMood {
    /// છું/છે/છીએ/છો — present.
    Present,
    /// હતું — past.
    Past,
    /// હોત — counterfactual.
    Counterfactual,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input's last word does not end in `-વું`.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Gujarati infinitive")
    }
}

/// Dependent vowel signs (matras) and the independent vowels: a stem
/// ending in one of these is vowel-final and takes the independent
/// vowel endings.
const VOWEL_SIGNS: &str = "ાિીુૂૃેૈોૌઅઆઇઈઉઊએઐઓઔ";

/// The infinitive suffix, `-વું` (VA + vowel-sign U + anusvara).
const INF_SUFFIX: &str = "વું";

/// The compiled-in table of verbs whose forms are not derivable.
static LEXICON_TSV: &str = include_str!("../data/guj/verbs.tsv");

/// A stored paradigm. Every group is optional: a `-` cell falls through
/// to the productive rule.
#[derive(Debug, Clone, Default)]
struct LexEntry {
    /// Present/subjunctive [1sg, 2sg, 3, 1pl, 2pl].
    present: Option<[String; 5]>,
    /// Future [1sg, 2sg, 3, 1pl, 2pl].
    future: Option<[String; 5]>,
    /// Perfective participle [masc sg, masc pl, fem, neut sg, neut pl].
    perfective: Option<[String; 5]>,
    /// Imperative [2sg, 2pl, polite sg, polite pl].
    imperative: Option<[String; 4]>,
    /// Perfective converb (કરી).
    conjunctive: Option<String>,
}

fn five(s: &str) -> Option<[String; 5]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
}

fn four(s: &str) -> Option<[String; 4]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
}

fn one(s: &str) -> Option<String> {
    (s != "-" && !s.is_empty()).then(|| s.to_string())
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
                    present: five(g(1)),
                    future: five(g(2)),
                    perfective: five(g(3)),
                    imperative: four(g(4)),
                    conjunctive: one(g(5)),
                },
            );
        }
        m
    })
}

/// A conjugatable Gujarati verb.
#[derive(Debug, Clone)]
pub struct Verb {
    /// The invariable material before the verb word (`વચ્ચે ` in
    /// વચ્ચે આવવું), empty for a simple verb. Re-attached to every form.
    prefix: String,
    /// The verb word's infinitive (કરવું), without the prefix.
    infinitive: String,
    /// The stem (કર, નહા).
    stem: String,
    /// Whether the stem ends in a vowel.
    vowel: bool,
    lex: Option<&'static LexEntry>,
}

fn ends_with_vowel(s: &str) -> bool {
    s.chars().last().is_some_and(|c| VOWEL_SIGNS.contains(c))
}

/// The five present/future/imperative agreement slots, in array order:
/// 1sg, 2sg, 3 (sg = pl), 1pl, 2pl.
const SLOTS5: [(Person, Number); 5] = [
    (Person::First, Number::Singular),
    (Person::Second, Number::Singular),
    (Person::Third, Number::Singular),
    (Person::First, Number::Plural),
    (Person::Second, Number::Plural),
];

/// The five perfective/imperfective participle slots, in array order:
/// masc sg, masc pl, fem, neut sg, neut pl.
const PTCP5: [(Gender, Number); 5] = [
    (Gender::Masculine, Number::Singular),
    (Gender::Masculine, Number::Plural),
    (Gender::Feminine, Number::Singular),
    (Gender::Neuter, Number::Singular),
    (Gender::Neuter, Number::Plural),
];

impl Verb {
    /// Build a verb from its infinitive.
    ///
    /// ```
    /// use ablaut::guj::{Gender, Number, Person, Verb};
    /// let v = Verb::from_infinitive("કરવું").unwrap();
    /// assert_eq!(v.present(Person::First, Number::Singular), "કરું");
    /// assert_eq!(
    ///     v.perfective(Gender::Neuter, Number::Singular),
    ///     "કર્યું"
    /// );
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::NotAVerb`] when the last word does not end in
    /// `-વું`.
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

    fn slot5(person: Person, number: Number) -> usize {
        match (person, number) {
            (Person::First, Number::Singular) => 0,
            (Person::Second, Number::Singular) => 1,
            (Person::Third, _) => 2,
            (Person::First, Number::Plural) => 3,
            (Person::Second, Number::Plural) => 4,
        }
    }

    /// The verbal noun / gerund (કરવાનું).
    #[must_use]
    pub fn verbal_noun(&self) -> String {
        self.out(&format!("{}વાનું", self.stem))
    }

    /// The perfective converb (કરી), "having done".
    #[must_use]
    pub fn conjunctive(&self) -> String {
        if let Some(c) = self.lex.and_then(|l| l.conjunctive.as_ref()) {
            return self.out(c);
        }
        self.out(&format!("{}{}", self.stem, self.end("ી", "ઈ")))
    }

    /// The consecutive converb (કરીને).
    #[must_use]
    pub fn consecutive(&self) -> String {
        format!("{}ને", self.conjunctive())
    }

    fn present_ending(&self, person: Person, number: Number) -> &'static str {
        match Self::slot5(person, number) {
            0 => self.end("ું", "ઉં"),
            1 | 2 => self.end("ે", "ય"),
            3 => self.end("ીએ", "ઈએ"),
            _ => self.end("ો", "ઓ"),
        }
    }

    /// The present/subjunctive (કરું, કરે, કરીએ, કરો), from the lexicon
    /// or the rule. The third person does not distinguish singular from
    /// plural.
    #[must_use]
    pub fn present(&self, person: Person, number: Number) -> String {
        let i = Self::slot5(person, number);
        if let Some(p) = self.lex.and_then(|l| l.present.as_ref()) {
            return self.out(&p[i]);
        }
        self.out(&format!(
            "{}{}",
            self.stem,
            self.present_ending(person, number)
        ))
    }

    /// The future in `-શ-` (કરીશ, કરશે), from the lexicon or the rule.
    #[must_use]
    pub fn future(&self, person: Person, number: Number) -> String {
        let i = Self::slot5(person, number);
        if let Some(f) = self.lex.and_then(|l| l.future.as_ref()) {
            return self.out(&f[i]);
        }
        let ending = match i {
            0 | 1 => self.end("ીશ", "ઈશ"),
            2 => "શે",
            3 => self.end("ીશું", "ઈશું"),
            _ => "શો",
        };
        self.out(&format!("{}{ending}", self.stem))
    }

    /// The perfective participle (the past): કર્યો/કરી/કર્યું, agreeing
    /// in gender and number. From the lexicon or the rule.
    #[must_use]
    pub fn perfective(&self, gender: Gender, number: Number) -> String {
        let i = match (gender, number) {
            (Gender::Masculine, Number::Singular) => 0,
            (Gender::Masculine, Number::Plural) => 1,
            (Gender::Feminine, _) => 2,
            (Gender::Neuter, Number::Singular) => 3,
            (Gender::Neuter, Number::Plural) => 4,
        };
        if let Some(p) = self.lex.and_then(|l| l.perfective.as_ref()) {
            return self.out(&p[i]);
        }
        // Feminine is the bare -ી/-ઈ form; the others build on the
        // -ય- glide base.
        if gender == Gender::Feminine {
            return self.out(&format!("{}{}", self.stem, self.end("ી", "ઈ")));
        }
        let ybase = format!("{}{}", self.stem, self.end("્ય", "ય"));
        let ending = match i {
            0 => "ો",
            1 => "ા",
            3 => "ું",
            _ => "ાં",
        };
        self.out(&format!("{ybase}{ending}"))
    }

    /// The imperfective participle (કરતો/કરતી/કરતું…), agreeing in
    /// gender and number.
    #[must_use]
    pub fn imperfective(&self, gender: Gender, number: Number) -> String {
        let base = format!("{}ત", self.stem);
        let ending = match (gender, number) {
            (Gender::Masculine, Number::Singular) => "ો",
            (Gender::Masculine, Number::Plural) => "ા",
            (Gender::Feminine, _) => "ી",
            (Gender::Neuter, Number::Singular) => "ું",
            (Gender::Neuter, Number::Plural) => "ાં",
        };
        self.out(&format!("{base}{ending}"))
    }

    /// The bare conditional/counterfactual stem `stem + ત` (કરત), the
    /// invariant form UniMorph labels LGSPEC3.
    #[must_use]
    pub fn conditional(&self) -> String {
        self.out(&format!("{}ત", self.stem))
    }

    /// An imperative at one politeness level, from the lexicon or rule.
    #[must_use]
    pub fn imperative(&self, politeness: Politeness) -> String {
        let i = match politeness {
            Politeness::Intimate => 0,
            Politeness::Familiar => 1,
            Politeness::PoliteSingular => 2,
            Politeness::PolitePlural => 3,
        };
        if let Some(imp) = self.lex.and_then(|l| l.imperative.as_ref()) {
            return self.out(&imp[i]);
        }
        let form = match politeness {
            Politeness::Intimate => self.stem.clone(),
            Politeness::Familiar => format!("{}{}", self.stem, self.end("ો", "ઓ")),
            Politeness::PoliteSingular => format!("{}જે", self.stem),
            Politeness::PolitePlural => format!("{}જો", self.stem),
        };
        self.out(&form)
    }

    /// The present copula છ-, agreeing in person and number.
    fn present_copula(person: Person, number: Number) -> &'static str {
        match Self::slot5(person, number) {
            0 => "છું",
            1 | 2 => "છે",
            3 => "છીએ",
            _ => "છો",
        }
    }

    /// An analytic form: an aspect stem plus a copula.
    ///
    /// - [`Aspect::Present`] + [`AuxMood::Present`] → the present
    ///   progressive (કરું છું), agreeing in person and number;
    /// - [`Aspect::Imperfective`] + [`AuxMood::Past`] → the past
    ///   progressive (કરતું હતું), the copula agreeing with the subject
    ///   in gender and number;
    /// - [`Aspect::Imperfective`] + [`AuxMood::Counterfactual`] → કરતું
    ///   હોત.
    #[must_use]
    pub fn analytic(
        &self,
        aspect: Aspect,
        mood: AuxMood,
        person: Person,
        number: Number,
        gender: Gender,
    ) -> String {
        match aspect {
            Aspect::Present => {
                let base = self.present(person, number);
                let aux = Self::present_copula(person, number);
                format!("{base} {aux}")
            }
            Aspect::Imperfective => {
                let base = self.imperfective(gender, number);
                let aux = match (mood, gender, number) {
                    (AuxMood::Counterfactual, ..) => "હોત",
                    (_, Gender::Masculine, Number::Singular) => "હતો",
                    (_, Gender::Masculine, Number::Plural) => "હતા",
                    (_, Gender::Feminine, _) => "હતી",
                    (_, Gender::Neuter, Number::Singular) => "હતું",
                    (_, Gender::Neuter, Number::Plural) => "હતાં",
                };
                format!("{base} {aux}")
            }
        }
    }
}

/// The conjugation table of a Gujarati verb — shared by the WebAssembly
/// and Python bindings. Person rows are [1sg, 2sg, 3, 1pl, 2pl] (the
/// third person does not split singular from plural); participle rows
/// are [masc sg, masc pl, fem, neut sg, neut pl].
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub verbal_noun: String,
    /// કરી — the perfective converb.
    pub conjunctive: String,
    /// કરીને — the consecutive converb.
    pub consecutive: String,
    /// [કરું, કરે, કરે, કરીએ, કરો] — the present/subjunctive.
    pub present: [String; 5],
    /// [કરીશ, કરીશ, કરશે, કરીશું, કરશો] — the future.
    pub future: [String; 5],
    /// [કર, કરો, કરજે, કરજો] — imperative [2sg, 2pl, polite sg,
    /// polite pl].
    pub imperative: [String; 4],
    /// [કર્યો, કર્યા, કરી, કર્યું, કર્યાં] — the perfective participle.
    pub perfective: [String; 5],
    /// [કરતો, કરતા, કરતી, કરતું, કરતાં] — the imperfective participle.
    pub imperfective: [String; 5],
    /// [કરું છું, …] — the present progressive.
    pub present_progressive: [String; 5],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        Self {
            infinitive: v.infinitive(),
            verbal_noun: v.verbal_noun(),
            conjunctive: v.conjunctive(),
            consecutive: v.consecutive(),
            present: SLOTS5.map(|(p, n)| v.present(p, n)),
            future: SLOTS5.map(|(p, n)| v.future(p, n)),
            imperative: [
                v.imperative(Politeness::Intimate),
                v.imperative(Politeness::Familiar),
                v.imperative(Politeness::PoliteSingular),
                v.imperative(Politeness::PolitePlural),
            ],
            perfective: PTCP5.map(|(g, n)| v.perfective(g, n)),
            imperfective: PTCP5.map(|(g, n)| v.imperfective(g, n)),
            present_progressive: SLOTS5
                .map(|(p, n)| v.analytic(Aspect::Present, AuxMood::Present, p, n, Gender::Neuter)),
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
        let k = v("કરવું");
        assert_eq!(k.infinitive(), "કરવું");
        assert_eq!(k.present(P1, SG), "કરું");
        assert_eq!(k.present(P2, SG), "કરે");
        assert_eq!(k.present(P3, SG), "કરે");
        assert_eq!(k.present(P1, PL), "કરીએ");
        assert_eq!(k.present(P2, PL), "કરો");
        assert_eq!(k.future(P1, SG), "કરીશ");
        assert_eq!(k.future(P2, SG), "કરીશ");
        assert_eq!(k.future(P3, SG), "કરશે");
        assert_eq!(k.future(P1, PL), "કરીશું");
        assert_eq!(k.future(P2, PL), "કરશો");
        assert_eq!(k.perfective(M, SG), "કર્યો");
        assert_eq!(k.perfective(M, PL), "કર્યા");
        assert_eq!(k.perfective(F, SG), "કરી");
        assert_eq!(k.perfective(N, SG), "કર્યું");
        assert_eq!(k.perfective(N, PL), "કર્યાં");
        assert_eq!(k.imperfective(N, SG), "કરતું");
        assert_eq!(k.imperfective(M, SG), "કરતો");
        assert_eq!(k.imperative(Politeness::Intimate), "કર");
        assert_eq!(k.imperative(Politeness::Familiar), "કરો");
        assert_eq!(k.imperative(Politeness::PoliteSingular), "કરજે");
        assert_eq!(k.imperative(Politeness::PolitePlural), "કરજો");
        assert_eq!(k.conjunctive(), "કરી");
        assert_eq!(k.consecutive(), "કરીને");
        assert_eq!(k.verbal_noun(), "કરવાનું");
        assert_eq!(k.conditional(), "કરત");
    }

    #[test]
    fn vowel_stem() {
        let n = v("નહાવું");
        assert_eq!(n.present(P1, SG), "નહાઉં");
        assert_eq!(n.present(P2, SG), "નહાય");
        assert_eq!(n.present(P1, PL), "નહાઈએ");
        assert_eq!(n.present(P2, PL), "નહાઓ");
        assert_eq!(n.future(P1, SG), "નહાઈશ");
        assert_eq!(n.future(P3, SG), "નહાશે");
        assert_eq!(n.future(P2, PL), "નહાશો");
        assert_eq!(n.perfective(N, SG), "નહાયું");
        assert_eq!(n.perfective(M, SG), "નહાયો");
        assert_eq!(n.perfective(F, SG), "નહાઈ");
        assert_eq!(n.conjunctive(), "નહાઈ");
        assert_eq!(n.imperative(Politeness::Intimate), "નહા");
        assert_eq!(n.verbal_noun(), "નહાવાનું");
    }

    #[test]
    fn analytic_layer() {
        let k = v("કરવું");
        assert_eq!(
            k.analytic(Aspect::Present, AuxMood::Present, P1, SG, N),
            "કરું છું"
        );
        assert_eq!(
            k.analytic(Aspect::Present, AuxMood::Present, P2, SG, N),
            "કરે છે"
        );
        assert_eq!(
            k.analytic(Aspect::Present, AuxMood::Present, P1, PL, N),
            "કરીએ છીએ"
        );
        assert_eq!(
            k.analytic(Aspect::Present, AuxMood::Present, P2, PL, N),
            "કરો છો"
        );
        assert_eq!(
            k.analytic(Aspect::Imperfective, AuxMood::Past, P3, SG, N),
            "કરતું હતું"
        );
        assert_eq!(
            k.analytic(Aspect::Imperfective, AuxMood::Past, P3, SG, M),
            "કરતો હતો"
        );
        assert_eq!(
            k.analytic(Aspect::Imperfective, AuxMood::Counterfactual, P3, SG, N),
            "કરતું હોત"
        );
    }

    #[test]
    fn suppletive_lexicon() {
        // જવું "go": suppletive past ગયું, present stem જા-.
        let j = v("જવું");
        assert_eq!(j.present(P1, SG), "જાઉં");
        assert_eq!(j.present(P3, SG), "જાય");
        assert_eq!(j.perfective(N, SG), "ગયું");
        assert_eq!(j.perfective(M, SG), "ગયો");
        assert_eq!(j.perfective(F, SG), "ગઈ");
        assert_eq!(j.future(P1, SG), "જઈશ");
        assert_eq!(j.conjunctive(), "જઈ");
        assert_eq!(j.imperative(Politeness::Intimate), "જા");
        // The imperfective is still regular (જતું).
        assert_eq!(j.imperfective(N, SG), "જતું");
        // ખાવું "eat": irregular past ખાધું, regular present.
        let kh = v("ખાવું");
        assert_eq!(kh.present(P1, SG), "ખાઉં");
        assert_eq!(kh.perfective(N, SG), "ખાધું");
        assert_eq!(kh.perfective(F, SG), "ખાધી");
        // લેવું "take": past લીધું, present stem લે.
        let l = v("લેવું");
        assert_eq!(l.present(P2, SG), "લે");
        assert_eq!(l.perfective(N, SG), "લીધું");
        assert_eq!(l.conjunctive(), "લઈ");
    }

    #[test]
    fn compound_lemma() {
        let a = v("વચ્ચે આવવું");
        assert_eq!(a.infinitive(), "વચ્ચે આવવું");
        assert_eq!(a.present(P1, SG), "વચ્ચે આવું");
        assert_eq!(a.perfective(N, SG), "વચ્ચે આવ્યું");
    }

    #[test]
    fn rejects_non_verbs() {
        for input in ["", "ઘર", "run", "કર"] {
            assert_eq!(Verb::from_infinitive(input).err(), Some(Error::NotAVerb));
        }
    }
}
