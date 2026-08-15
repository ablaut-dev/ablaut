//! # ablaut
//!
//! A fast, correct German verb conjugator.
//!
//! The design follows `docs/ontology.md`: a small morphological core
//! generates the synthetic forms (Präsens, Präteritum, Konjunktiv I/II,
//! imperative, participles); analytic tenses are composed on top of it.
//!
//! Version 0.1 covers the weak (regular) paradigm with the full set of
//! orthographic surface rules. Strong, mixed and suppletive verbs land next,
//! as a compiled-in exception lexicon.

mod features;
mod orthography;

pub use features::{Mood, Number, Person, Tense};
use orthography::attach;

/// Six-slot ending rows (1sg, 2sg, 3sg, 1pl, 2pl, 3pl).
const PRESENT: [&str; 6] = ["e", "st", "t", "en", "t", "en"];
const PRETERITE_WEAK: [&str; 6] = ["te", "test", "te", "ten", "tet", "ten"];
const KONJUNKTIV_I: [&str; 6] = ["e", "est", "e", "en", "et", "en"];

/// A German verb, carrying the lexical facts conjugation needs (Layer A of
/// the ontology). For weak verbs everything derives from the infinitive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verb {
    infinitive: String,
    stem: String,
    /// Infinitive ends in -eln/-ern: stem ends in a schwa syllable.
    schwa_stem: bool,
    /// -ieren verbs take no ge- in the past participle.
    ieren: bool,
}

/// Errors raised when constructing a [`Verb`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// German infinitives end in -en or -n.
    InvalidInfinitive(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidInfinitive(s) => write!(f, "not a German infinitive: {s}"),
        }
    }
}

impl std::error::Error for Error {}

impl Verb {
    /// Build a weak (regular) verb from its infinitive.
    pub fn weak(infinitive: &str) -> Result<Self, Error> {
        let schwa_stem = infinitive.ends_with("eln") || infinitive.ends_with("ern");
        let stem = if schwa_stem {
            infinitive.strip_suffix('n')
        } else {
            infinitive.strip_suffix("en")
        };
        let stem = stem.ok_or_else(|| Error::InvalidInfinitive(infinitive.into()))?;
        if stem.is_empty() {
            return Err(Error::InvalidInfinitive(infinitive.into()));
        }
        Ok(Verb {
            infinitive: infinitive.to_string(),
            stem: stem.to_string(),
            schwa_stem,
            ieren: infinitive.ends_with("ieren"),
        })
    }

    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// A finite synthetic form.
    pub fn conjugate(&self, tense: Tense, mood: Mood, person: Person, number: Number) -> String {
        let i = person.index(number);
        match (tense, mood) {
            // Präsens indicative: 1pl/3pl are identical to the infinitive
            // (wir sammeln, wir kaufen).
            (Tense::Present, Mood::Indicative) if i == 3 || i == 5 => self.infinitive.clone(),
            (Tense::Present, Mood::Indicative) => attach(&self.stem, PRESENT[i], self.schwa_stem),
            // Konjunktiv I is present-stem + e-endings; weak Konjunktiv II is
            // identical to the preterite indicative. Both are tense-independent
            // in the weak paradigm, so `tense` is ignored for them.
            (_, Mood::KonjunktivI) if i == 3 || i == 5 => self.infinitive.clone(),
            (_, Mood::KonjunktivI) => attach(&self.stem, KONJUNKTIV_I[i], self.schwa_stem),
            (Tense::Preterite, Mood::Indicative) | (_, Mood::KonjunktivII) => {
                attach(&self.stem, PRETERITE_WEAK[i], self.schwa_stem)
            }
        }
    }

    /// Imperative — exists only in the 2nd person, so it takes only a number.
    pub fn imperative(&self, number: Number) -> String {
        match number {
            // Canonical 2sg with -e (kaufe!); mandatory for epenthesis and
            // schwa stems (arbeite!, sammle!).
            Number::Singular => attach(&self.stem, "e", self.schwa_stem),
            Number::Plural => self.conjugate(
                Tense::Present,
                Mood::Indicative,
                Person::Second,
                Number::Plural,
            ),
        }
    }

    /// Partizip II (gekauft, gearbeitet, studiert).
    pub fn past_participle(&self) -> String {
        let base = attach(&self.stem, "t", self.schwa_stem);
        if self.ieren {
            base
        } else {
            format!("ge{base}")
        }
    }

    /// Partizip I (kaufend).
    pub fn present_participle(&self) -> String {
        format!("{}d", self.infinitive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Mood::*;
    use Number::*;
    use Person::*;
    use Tense::*;

    fn row(v: &Verb, tense: Tense, mood: Mood) -> [String; 6] {
        [
            v.conjugate(tense, mood, First, Singular),
            v.conjugate(tense, mood, Second, Singular),
            v.conjugate(tense, mood, Third, Singular),
            v.conjugate(tense, mood, First, Plural),
            v.conjugate(tense, mood, Second, Plural),
            v.conjugate(tense, mood, Third, Plural),
        ]
    }

    #[test]
    fn kaufen_plain_weak() {
        let v = Verb::weak("kaufen").unwrap();
        assert_eq!(
            row(&v, Present, Indicative),
            ["kaufe", "kaufst", "kauft", "kaufen", "kauft", "kaufen"]
        );
        assert_eq!(
            row(&v, Preterite, Indicative),
            ["kaufte", "kauftest", "kaufte", "kauften", "kauftet", "kauften"]
        );
        assert_eq!(v.past_participle(), "gekauft");
        assert_eq!(v.present_participle(), "kaufend");
    }

    #[test]
    fn arbeiten_epenthesis() {
        let v = Verb::weak("arbeiten").unwrap();
        assert_eq!(
            row(&v, Present, Indicative),
            ["arbeite", "arbeitest", "arbeitet", "arbeiten", "arbeitet", "arbeiten"]
        );
        assert_eq!(
            row(&v, Preterite, Indicative),
            ["arbeitete", "arbeitetest", "arbeitete", "arbeiteten", "arbeitetet", "arbeiteten"]
        );
        assert_eq!(v.past_participle(), "gearbeitet");
        assert_eq!(v.imperative(Singular), "arbeite");
    }

    #[test]
    fn atmen_rechnen_epenthesis_after_obstruent() {
        let atmen = Verb::weak("atmen").unwrap();
        assert_eq!(atmen.conjugate(Present, Indicative, Second, Singular), "atmest");
        assert_eq!(atmen.conjugate(Present, Indicative, Third, Singular), "atmet");
        let rechnen = Verb::weak("rechnen").unwrap();
        assert_eq!(rechnen.conjugate(Present, Indicative, Second, Singular), "rechnest");
        assert_eq!(rechnen.past_participle(), "gerechnet");
    }

    #[test]
    fn lernen_wohnen_no_epenthesis() {
        let lernen = Verb::weak("lernen").unwrap();
        assert_eq!(lernen.conjugate(Present, Indicative, Second, Singular), "lernst");
        let wohnen = Verb::weak("wohnen").unwrap();
        assert_eq!(wohnen.conjugate(Present, Indicative, Second, Singular), "wohnst");
        assert_eq!(wohnen.past_participle(), "gewohnt");
    }

    #[test]
    fn tanzen_s_coalescence() {
        let v = Verb::weak("tanzen").unwrap();
        assert_eq!(v.conjugate(Present, Indicative, Second, Singular), "tanzt");
        assert_eq!(v.conjugate(Preterite, Indicative, Second, Singular), "tanztest");
    }

    #[test]
    fn sammeln_schwa_elision() {
        let v = Verb::weak("sammeln").unwrap();
        assert_eq!(
            row(&v, Present, Indicative),
            ["sammle", "sammelst", "sammelt", "sammeln", "sammelt", "sammeln"]
        );
        assert_eq!(v.past_participle(), "gesammelt");
        assert_eq!(v.imperative(Singular), "sammle");
        assert_eq!(v.present_participle(), "sammelnd");
    }

    #[test]
    fn wandern_schwa_stem() {
        let v = Verb::weak("wandern").unwrap();
        assert_eq!(
            row(&v, Present, Indicative),
            ["wandere", "wanderst", "wandert", "wandern", "wandert", "wandern"]
        );
        assert_eq!(v.past_participle(), "gewandert");
        assert_eq!(
            row(&v, Preterite, Indicative),
            ["wanderte", "wandertest", "wanderte", "wanderten", "wandertet", "wanderten"]
        );
    }

    #[test]
    fn spielen_is_not_a_schwa_stem() {
        // stem ends in "el" on the surface but the infinitive is -en:
        // no elision may fire.
        let v = Verb::weak("spielen").unwrap();
        assert_eq!(v.conjugate(Present, Indicative, First, Singular), "spiele");
    }

    #[test]
    fn studieren_no_ge_participle() {
        let v = Verb::weak("studieren").unwrap();
        assert_eq!(v.past_participle(), "studiert");
        assert_eq!(v.conjugate(Preterite, Indicative, Third, Singular), "studierte");
    }

    #[test]
    fn konjunktiv_weak() {
        let v = Verb::weak("kaufen").unwrap();
        assert_eq!(
            row(&v, Present, KonjunktivI),
            ["kaufe", "kaufest", "kaufe", "kaufen", "kaufet", "kaufen"]
        );
        // Weak Konjunktiv II is identical to the preterite indicative.
        assert_eq!(row(&v, Present, KonjunktivII), row(&v, Preterite, Indicative));
        let s = Verb::weak("sammeln").unwrap();
        assert_eq!(s.conjugate(Present, KonjunktivI, Second, Singular), "sammelst");
    }

    #[test]
    fn invalid_infinitive() {
        assert!(Verb::weak("kauf").is_err());
        assert!(Verb::weak("n").is_err());
    }
}
