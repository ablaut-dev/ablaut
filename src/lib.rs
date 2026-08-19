//! # ablaut
//!
//! A fast, correct German verb conjugator.
//!
//! The design follows `docs/design.md`: a small morphological core
//! generates the synthetic forms (Präsens, Präteritum, Konjunktiv I/II,
//! imperative, participles); analytic tenses are composed on top of it.
//!
//! Inflection classes: weak (the productive default), strong (ablaut),
//! mixed (changed stem + weak endings), preterite-present (modals and
//! *wissen*), and three stored suppletives (*sein*, *werden*, *tun*).
//! Everything irregular lives in `data/deu/verbs.tsv`, compiled in.

pub mod cat;
pub mod ces;
pub mod dan;
pub mod deu;
pub mod eng;
pub mod est;
pub mod fin;
pub mod fra;
pub mod gle;
#[doc(hidden)]
pub mod harness;
pub mod isl;
pub mod ita;
pub mod jpn;
pub mod por;
#[cfg(feature = "python")]
mod python;
pub mod reverse;
pub mod ron;
pub mod rus;
pub mod slv;
pub mod spa;
pub mod swe;
pub mod ukr;
#[cfg(feature = "wasm")]
mod wasm;

// Backwards-compatible root exports: the crate began as a German
// conjugator and the German API lived at the root.
pub use deu::features::{Mood, Number, Person, Tense};
pub use deu::table;
pub use deu::{AnalyticTense, Auxiliary, Error, Verb};
pub use reverse::{reverse, ReverseMatch};

/// A supported language, for the multi-language entry points in the
/// bindings. ISO 639 codes and English names are accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// German — the original engine.
    Deu,
    /// Catalan.
    Cat,
    /// Czech.
    Ces,
    /// Danish.
    Dan,
    /// English.
    Eng,
    /// Estonian.
    Est,
    /// Finnish.
    Fin,
    /// French.
    Fra,
    /// Slovenian.
    Slv,
    /// Spanish.
    Spa,
    /// Portuguese.
    Por,
    /// Irish.
    Gle,
    /// Icelandic.
    Isl,
    /// Italian.
    Ita,
    /// Romanian.
    Ron,
    /// Russian.
    Rus,
    /// Swedish.
    Swe,
    /// Ukrainian.
    Ukr,
    /// Japanese.
    Jpn,
}

impl Lang {
    /// Parse a language code ("de", "deu", "german", "fr", "fra",
    /// "french"; case-insensitive).
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_ascii_lowercase().as_str() {
            "de" | "deu" | "ger" | "german" => Some(Self::Deu),
            "ca" | "cat" | "catalan" | "català" => Some(Self::Cat),
            "cs" | "ces" | "cze" | "czech" => Some(Self::Ces),
            "da" | "dan" | "danish" => Some(Self::Dan),
            "en" | "eng" | "english" => Some(Self::Eng),
            "et" | "est" | "estonian" => Some(Self::Est),
            "fi" | "fin" | "finnish" => Some(Self::Fin),
            "fr" | "fra" | "fre" | "french" => Some(Self::Fra),
            "sl" | "slv" | "slovenian" | "slovene" => Some(Self::Slv),
            "es" | "spa" | "spanish" => Some(Self::Spa),
            "pt" | "por" | "portuguese" => Some(Self::Por),
            "ga" | "gle" | "irish" => Some(Self::Gle),
            "is" | "isl" | "ice" | "icelandic" => Some(Self::Isl),
            "it" | "ita" | "italian" => Some(Self::Ita),
            "ro" | "ron" | "rum" | "romanian" => Some(Self::Ron),
            "ru" | "rus" | "russian" => Some(Self::Rus),
            "sv" | "swe" | "swedish" => Some(Self::Swe),
            "uk" | "ukr" | "ukrainian" => Some(Self::Ukr),
            "ja" | "jpn" | "japanese" => Some(Self::Jpn),
            _ => None,
        }
    }
}

/// The conjugation table of a verb in any supported language.
///
/// The per-language `Table` types are the real API surface — this
/// enum is the Rust-level counterpart of the string-dispatched
/// bindings, so `ablaut::conjugate("vorbi", Lang::Ron)` works
/// without knowing the per-language modules.
#[non_exhaustive]
pub enum Conjugation {
    Deu(Box<table::Table>),
    Cat(Box<cat::Table>),
    Ces(Box<ces::Table>),
    Dan(Box<dan::Table>),
    Eng(Box<eng::Table>),
    Est(Box<est::Table>),
    Fin(Box<fin::Table>),
    Fra(Box<fra::Table>),
    Gle(Box<gle::Table>),
    Isl(Box<isl::Table>),
    Ita(Box<ita::Table>),
    Por(Box<por::Table>),
    Ron(Box<ron::Table>),
    Rus(Box<rus::Table>),
    Slv(Box<slv::Table>),
    Spa(Box<spa::Table>),
    Swe(Box<swe::Table>),
    Ukr(Box<ukr::Table>),
    Jpn(Box<jpn::Table>),
}

/// Why `conjugate` failed: the input is not a known verb shape in
/// the requested language. Carries the language's own message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConjugateError(pub String);

impl std::fmt::Display for ConjugateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConjugateError {}

/// Conjugate an infinitive in any supported language — the single
/// Rust entry point mirroring the wasm/Python `conjugate(verb, lang)`.
pub fn conjugate(infinitive: &str, lang: Lang) -> Result<Conjugation, ConjugateError> {
    fn err(e: impl std::fmt::Display) -> ConjugateError {
        ConjugateError(e.to_string())
    }
    Ok(match lang {
        Lang::Deu => Conjugation::Deu(Box::new(table::Table::build(
            &Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Cat => Conjugation::Cat(Box::new(cat::Table::build(
            &cat::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ces => Conjugation::Ces(Box::new(ces::Table::build(
            &ces::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Dan => Conjugation::Dan(Box::new(dan::Table::build(
            &dan::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Eng => Conjugation::Eng(Box::new(eng::Table::build(
            &eng::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Est => Conjugation::Est(Box::new(est::Table::build(
            &est::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Fin => Conjugation::Fin(Box::new(fin::Table::build(
            &fin::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Fra => Conjugation::Fra(Box::new(fra::Table::build(
            &fra::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Gle => Conjugation::Gle(Box::new(gle::Table::build(
            &gle::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Isl => Conjugation::Isl(Box::new(isl::Table::build(
            &isl::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ita => Conjugation::Ita(Box::new(ita::Table::build(
            &ita::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Por => Conjugation::Por(Box::new(por::Table::build(
            &por::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ron => Conjugation::Ron(Box::new(ron::Table::build(
            &ron::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Rus => Conjugation::Rus(Box::new(rus::Table::build(
            &rus::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Slv => Conjugation::Slv(Box::new(slv::Table::build(
            &slv::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Spa => Conjugation::Spa(Box::new(spa::Table::build(
            &spa::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Swe => Conjugation::Swe(Box::new(swe::Table::build(
            &swe::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ukr => Conjugation::Ukr(Box::new(ukr::Table::build(
            &ukr::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Jpn => Conjugation::Jpn(Box::new(jpn::Table::build(
            &jpn::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
    })
}

#[cfg(test)]
mod facade_tests {
    use super::*;

    #[test]
    fn conjugate_dispatches_every_language() {
        let cases = [
            ("sprechen", Lang::Deu),
            ("mluvit", Lang::Ces),
            ("virke", Lang::Dan),
            ("run", Lang::Eng),
            ("rääkima", Lang::Est),
            ("puhua", Lang::Fin),
            ("parler", Lang::Fra),
            ("glan", Lang::Gle),
            ("kalla", Lang::Isl),
            ("parlare", Lang::Ita),
            ("falar", Lang::Por),
            ("vorbi", Lang::Ron),
            ("читать", Lang::Rus),
            ("delati", Lang::Slv),
            ("hablar", Lang::Spa),
            ("parlar", Lang::Cat),
            ("tala", Lang::Swe),
            ("читати", Lang::Ukr),
            ("食べる", Lang::Jpn),
        ];
        for (verb, lang) in cases {
            assert!(conjugate(verb, lang).is_ok(), "{verb}");
        }
        assert!(conjugate("xyz123", Lang::Fra).is_err());
    }
}
