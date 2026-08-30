//! WebAssembly bindings (feature `wasm`): a JS-friendly API returning the
//! full conjugation table of a verb as one structured object.
//!
//! Build with: `wasm-pack build --target web -- --features wasm`

use crate::table::Table;
use crate::Verb;
use wasm_bindgen::prelude::*;

/// The full conjugation table for an infinitive:
/// `conjugate("aufstehen").present[0]` → "stehe auf",
/// `conjugate("parler", "fra").present[0]` → "parle". The language
/// defaults to German; throws for unknown languages and for strings that
/// are not infinitives of the requested language.
#[wasm_bindgen]
pub fn conjugate(infinitive: &str, lang: Option<String>) -> Result<JsValue, JsError> {
    let lang = lang.as_deref().unwrap_or("deu");
    match crate::Lang::from_code(lang) {
        Some(crate::Lang::Deu) => {
            let v = Verb::from_infinitive(infinitive).map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&Table::build(&v))?)
        }
        Some(crate::Lang::Est) => {
            let v = crate::est::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::est::Table::build(&v))?)
        }
        Some(crate::Lang::Fin) => {
            let v = crate::fin::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::fin::Table::build(&v))?)
        }
        Some(crate::Lang::Fra) => {
            let v = crate::fra::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::fra::Table::build(&v))?)
        }
        Some(crate::Lang::Swe) => {
            let v = crate::swe::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::swe::Table::build(&v))?)
        }
        Some(crate::Lang::Gle) => {
            let v = crate::gle::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::gle::Table::build(&v))?)
        }
        Some(crate::Lang::Isl) => {
            let v = crate::isl::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::isl::Table::build(&v))?)
        }
        Some(crate::Lang::Ita) => {
            let v = crate::ita::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ita::Table::build(&v))?)
        }
        Some(crate::Lang::Ces) => {
            let v = crate::ces::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ces::Table::build(&v))?)
        }
        Some(crate::Lang::Dan) => {
            let v = crate::dan::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::dan::Table::build(&v))?)
        }
        Some(crate::Lang::Nob) => {
            let v = crate::nob::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::nob::Table::build(&v))?)
        }
        Some(crate::Lang::Eng) => {
            let v = crate::eng::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::eng::Table::build(&v))?)
        }
        Some(crate::Lang::Ron) => {
            let v = crate::ron::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ron::Table::build(&v))?)
        }
        Some(crate::Lang::Por) => {
            let v = crate::por::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::por::Table::build(&v))?)
        }
        Some(crate::Lang::Slv) => {
            let v = crate::slv::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::slv::Table::build(&v))?)
        }
        Some(crate::Lang::Spa) => {
            let v = crate::spa::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::spa::Table::build(&v))?)
        }
        Some(crate::Lang::Cat) => {
            let v = crate::cat::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::cat::Table::build(&v))?)
        }
        Some(crate::Lang::Rus) => {
            let v = crate::rus::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::rus::Table::build(&v))?)
        }
        Some(crate::Lang::Nld) => {
            let v = crate::nld::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::nld::Table::build(&v))?)
        }
        Some(crate::Lang::Ukr) => {
            let v = crate::ukr::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ukr::Table::build(&v))?)
        }
        Some(crate::Lang::Jpn) => {
            let v = crate::jpn::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::jpn::Table::build(&v))?)
        }
        Some(crate::Lang::Hye) => {
            let v = crate::hye::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::hye::Table::build(&v))?)
        }
        Some(crate::Lang::Kor) => {
            let v = crate::kor::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::kor::Table::build(&v))?)
        }
        Some(crate::Lang::Tur) => {
            let v = crate::tur::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::tur::Table::build(&v))?)
        }
        Some(crate::Lang::Hin) => {
            let v = crate::hin::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::hin::Table::build(&v))?)
        }
        Some(crate::Lang::Swa) => {
            let v = crate::swa::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::swa::Table::build(&v))?)
        }
        Some(crate::Lang::Tam) => {
            let v = crate::tam::Verb::from_root(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::tam::Table::build(&v))?)
        }
        Some(crate::Lang::Tel) => {
            let v = crate::tel::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::tel::Table::build(&v))?)
        }
        Some(crate::Lang::Tgl) => {
            let v = crate::tgl::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::tgl::Table::build(&v))?)
        }
        Some(crate::Lang::Pes) => {
            let v = crate::pes::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::pes::Table::build(&v))?)
        }
        Some(crate::Lang::Kan) => {
            let v = crate::kan::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::kan::Table::build(&v))?)
        }
        Some(crate::Lang::Guj) => {
            let v = crate::guj::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::guj::Table::build(&v))?)
        }
        Some(crate::Lang::Urd) => {
            let v = crate::urd::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::urd::Table::build(&v))?)
        }
        Some(crate::Lang::Ben) => {
            let v = crate::ben::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ben::Table::build(&v))?)
        }
        Some(crate::Lang::Mar) => {
            let v = crate::mar::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::mar::Table::build(&v))?)
        }
        Some(crate::Lang::Mkd) => {
            let v = crate::mkd::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::mkd::Table::build(&v))?)
        }
        Some(crate::Lang::Afr) => {
            let v = crate::afr::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::afr::Table::build(&v))?)
        }
        Some(crate::Lang::Ara) => {
            let v = crate::ara::Verb::from_lemma(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ara::Table::build(&v))?)
        }
        Some(crate::Lang::Heb) => {
            let v = crate::heb::Verb::from_lemma(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::heb::Table::build(&v))?)
        }
        Some(crate::Lang::Amh) => {
            let v = crate::amh::Verb::from_lemma(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::amh::Table::build(&v))?)
        }
        Some(crate::Lang::Ind) => {
            let v = crate::ind::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ind::Table::build(&v))?)
        }
        Some(crate::Lang::Zul) => {
            let v = crate::zul::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::zul::Table::build(&v))?)
        }
        Some(crate::Lang::Epo) => {
            let v = crate::epo::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::epo::Table::build(&v))?)
        }
        Some(crate::Lang::Gla) => {
            let v = crate::gla::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::gla::Table::build(&v))?)
        }
        Some(crate::Lang::Grn) => {
            let v = crate::grn::Verb::from_lemma(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::grn::Table::build(&v))?)
        }
        Some(crate::Lang::Haw) => {
            let v = crate::haw::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::haw::Table::build(&v))?)
        }
        Some(crate::Lang::Bul) => {
            let v = crate::bul::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::bul::Table::build(&v))?)
        }
        Some(crate::Lang::Ell) => {
            let v = crate::ell::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ell::Table::build(&v))?)
        }
        Some(crate::Lang::Sqi) => {
            let v = crate::sqi::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::sqi::Table::build(&v))?)
        }
        Some(crate::Lang::Pol) => {
            let v = crate::pol::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::pol::Table::build(&v))?)
        }
        Some(crate::Lang::Aze) => {
            let v = crate::aze::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::aze::Table::build(&v))?)
        }
        Some(crate::Lang::Uzb) => {
            let v = crate::uzb::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::uzb::Table::build(&v))?)
        }
        Some(crate::Lang::Tuk) => {
            let v = crate::tuk::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::tuk::Table::build(&v))?)
        }
        Some(crate::Lang::Bel) => {
            let v = crate::bel::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::bel::Table::build(&v))?)
        }
        Some(crate::Lang::Cym) => {
            let v = crate::cym::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::cym::Table::build(&v))?)
        }
        Some(crate::Lang::Fao) => {
            let v = crate::fao::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::fao::Table::build(&v))?)
        }
        Some(crate::Lang::Glg) => {
            let v = crate::glg::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::glg::Table::build(&v))?)
        }
        Some(crate::Lang::Kaz) => {
            let v = crate::kaz::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::kaz::Table::build(&v))?)
        }
        Some(crate::Lang::Lat) => {
            let v = crate::lat::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::lat::Table::build(&v))?)
        }
        Some(crate::Lang::Ltz) => {
            let v = crate::ltz::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ltz::Table::build(&v))?)
        }
        Some(crate::Lang::Oci) => {
            let v = crate::oci::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::oci::Table::build(&v))?)
        }
        Some(crate::Lang::Tat) => {
            let v = crate::tat::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::tat::Table::build(&v))?)
        }
        Some(crate::Lang::Ydd) => {
            let v = crate::ydd::Verb::from_infinitive(infinitive)
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(serde_wasm_bindgen::to_value(&crate::ydd::Table::build(&v))?)
        }
        None => Err(JsError::new(&format!("unknown language: {lang}"))),
    }
}
