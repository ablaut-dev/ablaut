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

pub mod afr;
pub mod amh;
pub mod ara;
pub mod aze;
pub mod bel;
pub mod ben;
pub mod bul;
pub mod cat;
pub mod ces;
pub mod cym;
pub mod dan;
pub mod deu;
pub mod ell;
pub mod eng;
pub mod epo;
pub mod est;
pub mod fao;
pub mod fin;
pub mod fra;
pub mod gla;
pub mod gle;
pub mod glg;
pub mod grn;
pub mod guj;
#[doc(hidden)]
pub mod harness;
pub mod haw;
pub mod heb;
pub mod hin;
pub mod hye;
pub mod ind;
pub mod isl;
pub mod ita;
pub mod jpn;
pub mod kan;
pub mod kaz;
pub mod kor;
pub mod lat;
pub mod ltz;
pub mod mar;
pub mod mkd;
pub mod nld;
pub mod nob;
pub mod oci;
pub mod perso_arabic;
pub mod pes;
pub mod pol;
pub mod por;
#[cfg(feature = "python")]
mod python;
pub mod reverse;
pub mod ron;
pub mod rus;
pub mod slv;
pub mod spa;
pub mod sqi;
pub mod swa;
pub mod swe;
pub mod tam;
pub mod tat;
pub mod tel;
pub mod tgl;
pub mod tuk;
pub mod tur;
pub mod ukr;
pub mod urd;
pub mod uzb;
#[cfg(feature = "wasm")]
mod wasm;
pub mod ydd;
pub mod zul;

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
    /// Eastern Armenian.
    Hye,
    /// Icelandic.
    Isl,
    /// Italian.
    Ita,
    /// Korean.
    Kor,
    /// Russian.
    Rus,
    /// Dutch.
    Nld,
    /// Norwegian Bokmål.
    Nob,
    /// Romanian.
    Ron,
    /// Swedish.
    Swe,
    /// Ukrainian.
    Ukr,
    /// Japanese.
    Jpn,
    /// Turkish.
    Tur,
    /// Hindi.
    Hin,
    /// Swahili.
    Swa,
    /// Tamil.
    Tam,
    /// Telugu.
    Tel,
    /// Tagalog.
    Tgl,
    /// Persian (Farsi).
    Pes,
    /// Kannada.
    Kan,
    /// Gujarati.
    Guj,
    /// Urdu.
    Urd,
    /// Bengali.
    Ben,
    /// Marathi.
    Mar,
    /// Macedonian.
    Mkd,
    /// Afrikaans.
    Afr,
    /// Bulgarian.
    Bul,
    /// Modern Greek.
    Ell,
    /// Albanian.
    Sqi,
    /// Polish.
    Pol,
    /// Azerbaijani.
    Aze,
    /// Uzbek.
    Uzb,
    /// Turkmen.
    Tuk,
    /// Belarusian.
    Bel,
    /// Welsh.
    Cym,
    /// Faroese.
    Fao,
    /// Galician.
    Glg,
    /// Kazakh.
    Kaz,
    /// Latin.
    Lat,
    /// Luxembourgish.
    Ltz,
    /// Occitan.
    Oci,
    /// Tatar.
    Tat,
    /// Yiddish.
    Ydd,
    /// Modern Standard Arabic.
    Ara,
    /// Modern Hebrew.
    Heb,
    /// Amharic.
    Amh,
    /// Indonesian.
    Ind,
    /// Zulu.
    Zul,
    /// Esperanto.
    Epo,
    /// Scottish Gaelic.
    Gla,
    /// Paraguayan Guaraní.
    Grn,
    /// Hawaiian.
    Haw,
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
            "hy" | "hye" | "arm" | "armenian" => Some(Self::Hye),
            "is" | "isl" | "ice" | "icelandic" => Some(Self::Isl),
            "it" | "ita" | "italian" => Some(Self::Ita),
            "ko" | "kor" | "korean" => Some(Self::Kor),
            "ru" | "rus" | "russian" => Some(Self::Rus),
            "nl" | "nld" | "dutch" => Some(Self::Nld),
            "nb" | "nob" | "no" | "nor" | "norwegian" | "bokmål" | "bokmal" => Some(Self::Nob),
            "ro" | "ron" | "rum" | "romanian" => Some(Self::Ron),
            "sv" | "swe" | "swedish" => Some(Self::Swe),
            "uk" | "ukr" | "ukrainian" => Some(Self::Ukr),
            "ja" | "jpn" | "japanese" => Some(Self::Jpn),
            "tr" | "tur" | "turkish" | "türkçe" => Some(Self::Tur),
            "hi" | "hin" | "hindi" => Some(Self::Hin),
            "sw" | "swa" | "swahili" | "kiswahili" => Some(Self::Swa),
            "ta" | "tam" | "tamil" => Some(Self::Tam),
            "te" | "tel" | "telugu" => Some(Self::Tel),
            "tl" | "tgl" | "tagalog" | "filipino" => Some(Self::Tgl),
            "fa" | "pes" | "per" | "fas" | "persian" | "farsi" => Some(Self::Pes),
            "kn" | "kan" | "kannada" => Some(Self::Kan),
            "gu" | "guj" | "gujarati" | "gujrati" => Some(Self::Guj),
            "ur" | "urd" | "urdu" => Some(Self::Urd),
            "bn" | "ben" | "bengali" | "bangla" => Some(Self::Ben),
            "mr" | "mar" | "marathi" | "मराठी" => Some(Self::Mar),
            "mk" | "mkd" | "mac" | "macedonian" | "македонски" => Some(Self::Mkd),
            "af" | "afr" | "afrikaans" => Some(Self::Afr),
            "ar" | "ara" | "arabic" => Some(Self::Ara),
            "he" | "heb" | "hebrew" | "עברית" => Some(Self::Heb),
            "am" | "amh" | "amharic" | "አማርኛ" => Some(Self::Amh),
            "id" | "ind" | "indonesian" | "bahasa" => Some(Self::Ind),
            "zu" | "zul" | "zulu" | "isizulu" => Some(Self::Zul),
            "eo" | "epo" | "esperanto" => Some(Self::Epo),
            "gd" | "gla" | "gaelic" | "scottish gaelic" => Some(Self::Gla),
            "gn" | "grn" | "gug" | "guarani" | "guaraní" => Some(Self::Grn),
            "haw" | "hawaiian" | "ʻōlelo hawaiʻi" | "olelo hawaii" => Some(Self::Haw),
            "bg" | "bul" | "bulgarian" => Some(Self::Bul),
            "el" | "ell" | "gre" | "greek" => Some(Self::Ell),
            "sq" | "sqi" | "alb" | "albanian" => Some(Self::Sqi),
            "pl" | "pol" | "polish" => Some(Self::Pol),
            "az" | "aze" | "azj" | "azerbaijani" | "azeri" => Some(Self::Aze),
            "uz" | "uzb" | "uzbek" => Some(Self::Uzb),
            "tk" | "tuk" | "turkmen" => Some(Self::Tuk),
            "be" | "bel" | "belarusian" => Some(Self::Bel),
            "cy" | "cym" | "wel" | "welsh" => Some(Self::Cym),
            "fo" | "fao" | "faroese" => Some(Self::Fao),
            "gl" | "glg" | "gal" | "galician" => Some(Self::Glg),
            "kk" | "kaz" | "kazakh" => Some(Self::Kaz),
            "la" | "lat" | "latin" => Some(Self::Lat),
            "lb" | "ltz" | "luxembourgish" => Some(Self::Ltz),
            "oc" | "oci" | "occitan" => Some(Self::Oci),
            "tt" | "tat" | "tatar" => Some(Self::Tat),
            "yi" | "ydd" | "yiddish" => Some(Self::Ydd),
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
    Hye(Box<hye::Table>),
    Isl(Box<isl::Table>),
    Ita(Box<ita::Table>),
    Kor(Box<kor::Table>),
    Rus(Box<rus::Table>),
    Nld(Box<nld::Table>),
    Nob(Box<nob::Table>),
    Por(Box<por::Table>),
    Ron(Box<ron::Table>),
    Slv(Box<slv::Table>),
    Spa(Box<spa::Table>),
    Swe(Box<swe::Table>),
    Ukr(Box<ukr::Table>),
    Jpn(Box<jpn::Table>),
    Tur(Box<tur::Table>),
    Hin(Box<hin::Table>),
    Swa(Box<swa::Table>),
    Tam(Box<tam::Table>),
    Tel(Box<tel::Table>),
    Tgl(Box<tgl::Table>),
    Pes(Box<pes::Table>),
    Kan(Box<kan::Table>),
    Guj(Box<guj::Table>),
    Urd(Box<urd::Table>),
    Ben(Box<ben::Table>),
    Mar(Box<mar::Table>),
    Mkd(Box<mkd::Table>),
    Afr(Box<afr::Table>),
    Bul(Box<bul::Table>),
    Ell(Box<ell::Table>),
    Sqi(Box<sqi::Table>),
    Pol(Box<pol::Table>),
    Aze(Box<aze::Table>),
    Uzb(Box<uzb::Table>),
    Tuk(Box<tuk::Table>),
    Bel(Box<bel::Table>),
    Cym(Box<cym::Table>),
    Fao(Box<fao::Table>),
    Glg(Box<glg::Table>),
    Kaz(Box<kaz::Table>),
    Lat(Box<lat::Table>),
    Ltz(Box<ltz::Table>),
    Oci(Box<oci::Table>),
    Tat(Box<tat::Table>),
    Ydd(Box<ydd::Table>),
    /// Modern Standard Arabic.
    Ara(Box<ara::Table>),
    /// Modern Hebrew.
    Heb(Box<heb::Table>),
    /// Amharic.
    Amh(Box<amh::Table>),
    /// Indonesian.
    Ind(Box<ind::Table>),
    /// Zulu.
    Zul(Box<zul::Table>),
    /// Esperanto.
    Epo(Box<epo::Table>),
    /// Scottish Gaelic.
    Gla(Box<gla::Table>),
    /// Paraguayan Guaraní.
    Grn(Box<grn::Table>),
    /// Hawaiian.
    Haw(Box<haw::Table>),
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
        Lang::Hye => Conjugation::Hye(Box::new(hye::Table::build(
            &hye::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Isl => Conjugation::Isl(Box::new(isl::Table::build(
            &isl::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ita => Conjugation::Ita(Box::new(ita::Table::build(
            &ita::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Kor => Conjugation::Kor(Box::new(kor::Table::build(
            &kor::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Rus => Conjugation::Rus(Box::new(rus::Table::build(
            &rus::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Nld => Conjugation::Nld(Box::new(nld::Table::build(
            &nld::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Nob => Conjugation::Nob(Box::new(nob::Table::build(
            &nob::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Por => Conjugation::Por(Box::new(por::Table::build(
            &por::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ron => Conjugation::Ron(Box::new(ron::Table::build(
            &ron::Verb::from_infinitive(infinitive).map_err(err)?,
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
        Lang::Tur => Conjugation::Tur(Box::new(tur::Table::build(
            &tur::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Hin => Conjugation::Hin(Box::new(hin::Table::build(
            &hin::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Swa => Conjugation::Swa(Box::new(swa::Table::build(
            &swa::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Tam => Conjugation::Tam(Box::new(tam::Table::build(
            &tam::Verb::from_root(infinitive).map_err(err)?,
        ))),
        Lang::Tel => Conjugation::Tel(Box::new(tel::Table::build(
            &tel::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Tgl => Conjugation::Tgl(Box::new(tgl::Table::build(
            &tgl::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Pes => Conjugation::Pes(Box::new(pes::Table::build(
            &pes::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Kan => Conjugation::Kan(Box::new(kan::Table::build(
            &kan::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Guj => Conjugation::Guj(Box::new(guj::Table::build(
            &guj::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Urd => Conjugation::Urd(Box::new(urd::Table::build(
            &urd::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ben => Conjugation::Ben(Box::new(ben::Table::build(
            &ben::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Mar => Conjugation::Mar(Box::new(mar::Table::build(
            &mar::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Mkd => Conjugation::Mkd(Box::new(mkd::Table::build(
            &mkd::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Afr => Conjugation::Afr(Box::new(afr::Table::build(
            &afr::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ara => Conjugation::Ara(Box::new(ara::Table::build(
            &ara::Verb::from_lemma(infinitive).map_err(err)?,
        ))),
        Lang::Heb => Conjugation::Heb(Box::new(heb::Table::build(
            &heb::Verb::from_lemma(infinitive).map_err(err)?,
        ))),
        Lang::Amh => Conjugation::Amh(Box::new(amh::Table::build(
            &amh::Verb::from_lemma(infinitive).map_err(err)?,
        ))),
        Lang::Ind => Conjugation::Ind(Box::new(ind::Table::build(
            &ind::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Zul => Conjugation::Zul(Box::new(zul::Table::build(
            &zul::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Epo => Conjugation::Epo(Box::new(epo::Table::build(
            &epo::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Gla => Conjugation::Gla(Box::new(gla::Table::build(
            &gla::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Grn => Conjugation::Grn(Box::new(grn::Table::build(
            &grn::Verb::from_lemma(infinitive).map_err(err)?,
        ))),
        Lang::Haw => Conjugation::Haw(Box::new(haw::Table::build(
            &haw::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Bul => Conjugation::Bul(Box::new(bul::Table::build(
            &bul::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ell => Conjugation::Ell(Box::new(ell::Table::build(
            &ell::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Sqi => Conjugation::Sqi(Box::new(sqi::Table::build(
            &sqi::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Pol => Conjugation::Pol(Box::new(pol::Table::build(
            &pol::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Aze => Conjugation::Aze(Box::new(aze::Table::build(
            &aze::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Uzb => Conjugation::Uzb(Box::new(uzb::Table::build(
            &uzb::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Tuk => Conjugation::Tuk(Box::new(tuk::Table::build(
            &tuk::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Bel => Conjugation::Bel(Box::new(bel::Table::build(
            &bel::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Cym => Conjugation::Cym(Box::new(cym::Table::build(
            &cym::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Fao => Conjugation::Fao(Box::new(fao::Table::build(
            &fao::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Glg => Conjugation::Glg(Box::new(glg::Table::build(
            &glg::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Kaz => Conjugation::Kaz(Box::new(kaz::Table::build(
            &kaz::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Lat => Conjugation::Lat(Box::new(lat::Table::build(
            &lat::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ltz => Conjugation::Ltz(Box::new(ltz::Table::build(
            &ltz::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Oci => Conjugation::Oci(Box::new(oci::Table::build(
            &oci::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Tat => Conjugation::Tat(Box::new(tat::Table::build(
            &tat::Verb::from_infinitive(infinitive).map_err(err)?,
        ))),
        Lang::Ydd => Conjugation::Ydd(Box::new(ydd::Table::build(
            &ydd::Verb::from_infinitive(infinitive).map_err(err)?,
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
            ("գրել", Lang::Hye),
            ("kalla", Lang::Isl),
            ("parlare", Lang::Ita),
            ("먹다", Lang::Kor),
            ("делать", Lang::Rus),
            ("falar", Lang::Por),
            ("vorbi", Lang::Ron),
            ("delati", Lang::Slv),
            ("hablar", Lang::Spa),
            ("parlar", Lang::Cat),
            ("werken", Lang::Nld),
            ("kaste", Lang::Nob),
            ("loop", Lang::Afr),
            ("гледам", Lang::Bul),
            ("γράφω", Lang::Ell),
            ("abdikoj", Lang::Sqi),
            ("robić", Lang::Pol),
            ("sevmək", Lang::Aze),
            ("ishlamoq", Lang::Uzb),
            ("atmak", Lang::Tuk),
            ("рабіць", Lang::Bel),
            ("canu", Lang::Cym),
            ("kasta", Lang::Fao),
            ("falar", Lang::Glg),
            ("алу", Lang::Kaz),
            ("amō", Lang::Lat),
            ("schaffen", Lang::Ltz),
            ("parlar", Lang::Oci),
            ("язу", Lang::Tat),
            ("בענטשן", Lang::Ydd),
            ("جلس", Lang::Ara),
            ("שמר", Lang::Heb),
            ("ሄደ", Lang::Amh),
            ("tulis", Lang::Ind),
            ("hamba", Lang::Zul),
            ("ami", Lang::Epo),
            ("cuir", Lang::Gla),
            ("jehu", Lang::Grn),
            ("hana", Lang::Haw),
            ("tala", Lang::Swe),
            ("читати", Lang::Ukr),
            ("食べる", Lang::Jpn),
            ("gelmek", Lang::Tur),
            ("उठना", Lang::Hin),
            ("soma", Lang::Swa),
            ("అమ్ము", Lang::Tel),
            ("sulat", Lang::Tgl),
            ("کردن", Lang::Pes),
            ("ಮಾಡು", Lang::Kan),
            ("કરવું", Lang::Guj),
            ("اترنا", Lang::Urd),
            ("করা", Lang::Ben),
            ("बसणे", Lang::Mar),
            ("игра", Lang::Mkd),
        ];
        for (verb, lang) in cases {
            assert!(conjugate(verb, lang).is_ok(), "{verb}");
        }
        assert!(conjugate("xyz123", Lang::Fra).is_err());
    }

    #[test]
    fn from_code_resolves_iso1_and_iso3() {
        // Guards against alternation arms accidentally written as a single
        // string literal ("be | bel | belarusian"), which silently makes the
        // codes unreachable via wasm/python/MCP while the golden gate stays green.
        let cases = [
            ("be", "bel", Lang::Bel),
            ("cy", "cym", Lang::Cym),
            ("fo", "fao", Lang::Fao),
            ("gl", "glg", Lang::Glg),
            ("kk", "kaz", Lang::Kaz),
            ("la", "lat", Lang::Lat),
            ("lb", "ltz", Lang::Ltz),
            ("oc", "oci", Lang::Oci),
            ("tt", "tat", Lang::Tat),
            ("yi", "ydd", Lang::Ydd),
        ];
        for (iso1, iso3, lang) in cases {
            assert_eq!(Lang::from_code(iso1), Some(lang), "iso1 {iso1}");
            assert_eq!(Lang::from_code(iso3), Some(lang), "iso3 {iso3}");
        }
    }
}
