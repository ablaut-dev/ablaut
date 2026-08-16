//! Python bindings (feature `python`, built with maturin):
//! `pip install ablaut` → `ablaut.conjugate("aufstehen").present`.

use crate::table::Table;
use crate::Verb;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// The full conjugation table of one verb. Rows are
/// [ich, du, er/sie/es, wir, ihr, sie]; imperative entries are None for
/// verbs without one (the modals).
#[pyclass(get_all, frozen)]
struct Conjugation {
    infinitive: String,
    zu_infinitive: String,
    perfect_infinitive: String,
    auxiliary: String,
    present_participle: String,
    past_participle: String,
    /// [2sg, 2pl]
    imperative: Vec<Option<String>>,
    /// [wir, Sie]
    imperative_extended: Vec<String>,
    present: Vec<String>,
    preterite: Vec<String>,
    konjunktiv1: Vec<String>,
    konjunktiv2: Vec<String>,
    perfect: Vec<String>,
    pluperfect: Vec<String>,
    future1: Vec<String>,
    future2: Vec<String>,
    wuerde: Vec<String>,
    konj1_perfect: Vec<String>,
    konj1_future: Vec<String>,
    konj2_pluperfect: Vec<String>,
    konj2_future2: Vec<String>,
}

#[pymethods]
impl Conjugation {
    fn __repr__(&self) -> String {
        format!(
            "Conjugation({:?}, auxiliary={:?})",
            self.infinitive, self.auxiliary
        )
    }
}

impl From<Table> for Conjugation {
    fn from(t: Table) -> Self {
        Conjugation {
            infinitive: t.infinitive,
            zu_infinitive: t.zu_infinitive,
            perfect_infinitive: t.perfect_infinitive,
            auxiliary: t.auxiliary,
            present_participle: t.present_participle,
            past_participle: t.past_participle,
            imperative: t.imperative.into(),
            imperative_extended: t.imperative_extended.into(),
            present: t.present.into(),
            preterite: t.preterite.into(),
            konjunktiv1: t.konjunktiv1.into(),
            konjunktiv2: t.konjunktiv2.into(),
            perfect: t.perfect.into(),
            pluperfect: t.pluperfect.into(),
            future1: t.future1.into(),
            future2: t.future2.into(),
            wuerde: t.wuerde.into(),
            konj1_perfect: t.konj1_perfect.into(),
            konj1_future: t.konj1_future.into(),
            konj2_pluperfect: t.konj2_pluperfect.into(),
            konj2_future2: t.konj2_future2.into(),
        }
    }
}

/// The full conjugation table of one French verb. Rows are
/// [je, tu, il/elle, nous, vous, ils/elles].
#[pyclass(get_all, frozen)]
struct FrenchConjugation {
    infinitive: String,
    auxiliary: String,
    present_participle: String,
    past_participle: String,
    /// [tu, nous, vous]
    imperative: Vec<Option<String>>,
    present: Vec<String>,
    imperfect: Vec<String>,
    past_historic: Vec<String>,
    future: Vec<String>,
    conditional: Vec<String>,
    subjunctive_present: Vec<String>,
    subjunctive_imperfect: Vec<String>,
    passe_compose: Vec<String>,
    plus_que_parfait: Vec<String>,
    passe_anterieur: Vec<String>,
    futur_anterieur: Vec<String>,
    conditionnel_passe: Vec<String>,
    subjonctif_passe: Vec<String>,
    subjonctif_plus_que_parfait: Vec<String>,
}

#[pymethods]
impl FrenchConjugation {
    fn __repr__(&self) -> String {
        format!("FrenchConjugation({:?})", self.infinitive)
    }
}

impl From<crate::fra::Table> for FrenchConjugation {
    fn from(t: crate::fra::Table) -> Self {
        FrenchConjugation {
            infinitive: t.infinitive,
            auxiliary: t.auxiliary,
            present_participle: t.present_participle,
            past_participle: t.past_participle,
            imperative: t.imperative.into(),
            present: t.present.into(),
            imperfect: t.imperfect.into(),
            past_historic: t.past_historic.into(),
            future: t.future.into(),
            conditional: t.conditional.into(),
            subjunctive_present: t.subjunctive_present.into(),
            subjunctive_imperfect: t.subjunctive_imperfect.into(),
            passe_compose: t.passe_compose.into(),
            plus_que_parfait: t.plus_que_parfait.into(),
            passe_anterieur: t.passe_anterieur.into(),
            futur_anterieur: t.futur_anterieur.into(),
            conditionnel_passe: t.conditionnel_passe.into(),
            subjonctif_passe: t.subjonctif_passe.into(),
            subjonctif_plus_que_parfait: t.subjonctif_plus_que_parfait.into(),
        }
    }
}

/// Conjugate an infinitive: `ablaut.conjugate("aufstehen").present[0]`
/// → "stehe auf", `ablaut.conjugate("parler", lang="fra").present[0]`
/// → "parle". Raises ValueError for unknown languages and for strings
/// that are not infinitives of the requested language.
#[pyfunction]
#[pyo3(signature = (infinitive, lang = "deu"))]
fn conjugate(py: Python<'_>, infinitive: &str, lang: &str) -> PyResult<PyObject> {
    match crate::Lang::from_code(lang) {
        Some(crate::Lang::Deu) => {
            let v = Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(Conjugation::from(Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Fra) => {
            let v = crate::fra::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(FrenchConjugation::from(crate::fra::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        None => Err(PyValueError::new_err(format!("unknown language: {lang}"))),
    }
}

#[pymodule]
fn ablaut(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Conjugation>()?;
    m.add_class::<FrenchConjugation>()?;
    m.add_function(wrap_pyfunction!(conjugate, m)?)?;
    Ok(())
}
