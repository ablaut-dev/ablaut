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

/// The full conjugation table of one Spanish verb. Rows are
/// [yo, tú, él/ella, nosotros, vosotros, ellos/ellas].
#[pyclass(get_all, frozen)]
struct SpanishConjugation {
    infinitive: String,
    gerund: String,
    past_participle: String,
    /// [tú, usted, nosotros, vosotros, ustedes]
    imperative: Vec<Option<String>>,
    present: Vec<String>,
    imperfect: Vec<String>,
    preterite: Vec<String>,
    future: Vec<String>,
    conditional: Vec<String>,
    subjunctive_present: Vec<String>,
    subjunctive_imperfect: Vec<String>,
    subjunctive_future: Vec<String>,
    perfecto_compuesto: Vec<String>,
    pluscuamperfecto: Vec<String>,
    preterito_anterior: Vec<String>,
    futuro_perfecto: Vec<String>,
    condicional_perfecto: Vec<String>,
    subjuntivo_perfecto: Vec<String>,
    subjuntivo_pluscuamperfecto: Vec<String>,
}

#[pymethods]
impl SpanishConjugation {
    fn __repr__(&self) -> String {
        format!("SpanishConjugation({:?})", self.infinitive)
    }
}

impl From<crate::spa::Table> for SpanishConjugation {
    fn from(t: crate::spa::Table) -> Self {
        SpanishConjugation {
            infinitive: t.infinitive,
            gerund: t.gerund,
            past_participle: t.past_participle,
            imperative: t.imperative.into(),
            present: t.present.into(),
            imperfect: t.imperfect.into(),
            preterite: t.preterite.into(),
            future: t.future.into(),
            conditional: t.conditional.into(),
            subjunctive_present: t.subjunctive_present.into(),
            subjunctive_imperfect: t.subjunctive_imperfect.into(),
            subjunctive_future: t.subjunctive_future.into(),
            perfecto_compuesto: t.perfecto_compuesto.into(),
            pluscuamperfecto: t.pluscuamperfecto.into(),
            preterito_anterior: t.preterito_anterior.into(),
            futuro_perfecto: t.futuro_perfecto.into(),
            condicional_perfecto: t.condicional_perfecto.into(),
            subjuntivo_perfecto: t.subjuntivo_perfecto.into(),
            subjuntivo_pluscuamperfecto: t.subjuntivo_pluscuamperfecto.into(),
        }
    }
}

/// The full conjugation table of one Portuguese verb. Rows are
/// [eu, tu, ele/ela, nós, vós, eles/elas].
#[pyclass(get_all, frozen)]
struct PortugueseConjugation {
    infinitive: String,
    gerund: String,
    past_participle: String,
    /// [tu, você, nós, vós, vocês]
    imperative: Vec<Option<String>>,
    personal_infinitive: Vec<String>,
    present: Vec<String>,
    imperfect: Vec<String>,
    preterite: Vec<String>,
    pluperfect: Vec<String>,
    future: Vec<String>,
    conditional: Vec<String>,
    subjunctive_present: Vec<String>,
    subjunctive_imperfect: Vec<String>,
    subjunctive_future: Vec<String>,
}

#[pymethods]
impl PortugueseConjugation {
    fn __repr__(&self) -> String {
        format!("PortugueseConjugation({:?})", self.infinitive)
    }
}

impl From<crate::por::Table> for PortugueseConjugation {
    fn from(t: crate::por::Table) -> Self {
        PortugueseConjugation {
            infinitive: t.infinitive,
            gerund: t.gerund,
            past_participle: t.past_participle,
            imperative: t.imperative.into(),
            personal_infinitive: t.personal_infinitive.into(),
            present: t.present.into(),
            imperfect: t.imperfect.into(),
            preterite: t.preterite.into(),
            pluperfect: t.pluperfect.into(),
            future: t.future.into(),
            conditional: t.conditional.into(),
            subjunctive_present: t.subjunctive_present.into(),
            subjunctive_imperfect: t.subjunctive_imperfect.into(),
            subjunctive_future: t.subjunctive_future.into(),
        }
    }
}

/// The full conjugation table of one Estonian verb. Six-slot rows
/// run [ma, sa, ta, me, te, nad].
#[pyclass(get_all, frozen)]
struct EstonianConjugation {
    infinitive: String,
    da_infinitive: String,
    present: Vec<String>,
    past: Vec<String>,
    conditional: Vec<String>,
    /// [2sg, 3sg, 1pl, 2pl]
    imperative: Vec<Option<String>>,
    present_impersonal: String,
    past_impersonal: String,
    quotative: String,
    present_participle: String,
    nud_participle: String,
    tud_participle: String,
}

#[pymethods]
impl EstonianConjugation {
    fn __repr__(&self) -> String {
        format!("EstonianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::est::Table> for EstonianConjugation {
    fn from(t: crate::est::Table) -> Self {
        EstonianConjugation {
            infinitive: t.infinitive,
            da_infinitive: t.da_infinitive,
            present: t.present.into(),
            past: t.past.into(),
            conditional: t.conditional.into(),
            imperative: t.imperative.into(),
            present_impersonal: t.present_impersonal,
            past_impersonal: t.past_impersonal,
            quotative: t.quotative,
            present_participle: t.present_participle,
            nud_participle: t.nud_participle,
            tud_participle: t.tud_participle,
        }
    }
}

/// The full conjugation table of one Slovenian verb. Nine-slot rows
/// run [sg1, sg2, sg3, du1, du2, du3, pl1, pl2, pl3]; participles
/// run [m, f, n] × [sg, du, pl].
#[pyclass(get_all, frozen)]
struct SlovenianConjugation {
    infinitive: String,
    supine: String,
    present: Vec<String>,
    /// [2sg, 1du, 2du, 1pl, 2pl]
    imperative: Vec<Option<String>>,
    participle: Vec<String>,
}

#[pymethods]
impl SlovenianConjugation {
    fn __repr__(&self) -> String {
        format!("SlovenianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::slv::Table> for SlovenianConjugation {
    fn from(t: crate::slv::Table) -> Self {
        SlovenianConjugation {
            infinitive: t.infinitive,
            supine: t.supine,
            present: t.present.into(),
            imperative: t.imperative.into(),
            participle: t.participle.into(),
        }
    }
}

/// The full conjugation table of one Czech verb. Person rows are
/// [já, ty, on/ona, my, vy, oni]; participle rows are
/// [masc sg, fem sg, neut sg, masc-anim pl, fem pl, neut pl].
#[pyclass(get_all, frozen)]
struct CzechConjugation {
    infinitive: String,
    present: Vec<String>,
    /// [2sg, 1pl, 2pl]
    imperative: Vec<Option<String>>,
    past_participle: Vec<String>,
    passive_participle: Option<Vec<String>>,
    /// [masc, fem/neut, plural]
    transgressive: Vec<Option<String>>,
}

#[pymethods]
impl CzechConjugation {
    fn __repr__(&self) -> String {
        format!("CzechConjugation({:?})", self.infinitive)
    }
}

impl From<crate::ces::Table> for CzechConjugation {
    fn from(t: crate::ces::Table) -> Self {
        CzechConjugation {
            infinitive: t.infinitive,
            present: t.present.into(),
            imperative: t.imperative.into(),
            past_participle: t.past_participle.into(),
            passive_participle: t.passive_participle.map(Into::into),
            transgressive: t.transgressive.into(),
        }
    }
}

/// The full conjugation table of one Danish verb (single forms — no
/// person/number agreement).
#[pyclass(get_all, frozen)]
struct DanishConjugation {
    infinitive: String,
    present: Option<String>,
    past: Option<String>,
    past_participle: Option<String>,
    imperative: Option<String>,
    present_participle: Option<String>,
    infinitive_passive: Option<String>,
    present_passive: Option<String>,
    past_passive: Option<String>,
}

#[pymethods]
impl DanishConjugation {
    fn __repr__(&self) -> String {
        format!("DanishConjugation({:?})", self.infinitive)
    }
}

impl From<crate::dan::Table> for DanishConjugation {
    fn from(t: crate::dan::Table) -> Self {
        DanishConjugation {
            infinitive: t.infinitive,
            present: t.present,
            past: t.past,
            past_participle: t.past_participle,
            imperative: t.imperative,
            present_participle: t.present_participle,
            infinitive_passive: t.infinitive_passive,
            present_passive: t.present_passive,
            past_passive: t.past_passive,
        }
    }
}

/// The full conjugation table of one English verb.
#[pyclass(get_all, frozen)]
struct EnglishConjugation {
    infinitive: String,
    past: String,
    past_participle: String,
    present_participle: String,
    third_singular: String,
}

#[pymethods]
impl EnglishConjugation {
    fn __repr__(&self) -> String {
        format!("EnglishConjugation({:?})", self.infinitive)
    }
}

impl From<crate::eng::Table> for EnglishConjugation {
    fn from(t: crate::eng::Table) -> Self {
        EnglishConjugation {
            infinitive: t.infinitive,
            past: t.past,
            past_participle: t.past_participle,
            present_participle: t.present_participle,
            third_singular: t.third_singular,
        }
    }
}

/// The full conjugation table of one Romanian verb. Rows are
/// [eu, tu, el/ea, noi, voi, ei/ele].
#[pyclass(get_all, frozen)]
struct RomanianConjugation {
    infinitive: String,
    gerund: String,
    participle: String,
    /// [tu, voi]
    imperative: Vec<String>,
    present: Vec<String>,
    imperfect: Vec<String>,
    simple_perfect: Vec<String>,
    pluperfect: Vec<String>,
    subjunctive: Vec<String>,
}

#[pymethods]
impl RomanianConjugation {
    fn __repr__(&self) -> String {
        format!("RomanianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::ron::Table> for RomanianConjugation {
    fn from(t: crate::ron::Table) -> Self {
        RomanianConjugation {
            infinitive: t.infinitive,
            gerund: t.gerund,
            participle: t.participle,
            imperative: t.imperative.into(),
            present: t.present.into(),
            imperfect: t.imperfect.into(),
            simple_perfect: t.simple_perfect.into(),
            pluperfect: t.pluperfect.into(),
            subjunctive: t.subjunctive.into(),
        }
    }
}

/// The full conjugation table of one Italian verb. Rows are
/// [io, tu, lui/lei, noi, voi, loro].
#[pyclass(get_all, frozen)]
struct ItalianConjugation {
    infinitive: String,
    gerund: String,
    present_participle: String,
    past_participle: String,
    /// [tu, Lei, noi, voi, Loro]
    imperative: Vec<Option<String>>,
    present: Vec<String>,
    imperfect: Vec<String>,
    past_historic: Vec<String>,
    future: Vec<String>,
    conditional: Vec<String>,
    subjunctive_present: Vec<String>,
    subjunctive_imperfect: Vec<String>,
}

#[pymethods]
impl ItalianConjugation {
    fn __repr__(&self) -> String {
        format!("ItalianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::ita::Table> for ItalianConjugation {
    fn from(t: crate::ita::Table) -> Self {
        ItalianConjugation {
            infinitive: t.infinitive,
            gerund: t.gerund,
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
        }
    }
}

/// The principal parts of one Swedish verb.
#[pyclass(get_all, frozen)]
struct SwedishConjugation {
    infinitive: String,
    present: Option<String>,
    past: Option<String>,
    supine: Option<String>,
    imperative: Option<String>,
    infinitive_passive: Option<String>,
    present_passive: Option<String>,
    past_passive: Option<String>,
    supine_passive: Option<String>,
    subjunctive_past: Option<String>,
}

#[pymethods]
impl SwedishConjugation {
    fn __repr__(&self) -> String {
        format!("SwedishConjugation({:?})", self.infinitive)
    }
}

impl From<crate::swe::Table> for SwedishConjugation {
    fn from(t: crate::swe::Table) -> Self {
        SwedishConjugation {
            infinitive: t.infinitive,
            present: t.present,
            past: t.past,
            supine: t.supine,
            imperative: t.imperative,
            infinitive_passive: t.infinitive_passive,
            present_passive: t.present_passive,
            past_passive: t.past_passive,
            supine_passive: t.supine_passive,
            subjunctive_past: t.subjunctive_past,
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
        Some(crate::Lang::Swe) => {
            let v = crate::swe::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(SwedishConjugation::from(crate::swe::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Ita) => {
            let v = crate::ita::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(ItalianConjugation::from(crate::ita::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Est) => {
            let v = crate::est::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(EstonianConjugation::from(crate::est::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Slv) => {
            let v = crate::slv::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(SlovenianConjugation::from(crate::slv::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Ces) => {
            let v = crate::ces::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(CzechConjugation::from(crate::ces::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Dan) => {
            let v = crate::dan::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(DanishConjugation::from(crate::dan::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Eng) => {
            let v = crate::eng::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(EnglishConjugation::from(crate::eng::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Ron) => {
            let v = crate::ron::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(RomanianConjugation::from(crate::ron::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Por) => {
            let v = crate::por::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(PortugueseConjugation::from(crate::por::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Spa) => {
            let v = crate::spa::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(SpanishConjugation::from(crate::spa::Table::build(&v))
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
    m.add_class::<SpanishConjugation>()?;
    m.add_class::<PortugueseConjugation>()?;
    m.add_class::<RomanianConjugation>()?;
    m.add_class::<EnglishConjugation>()?;
    m.add_class::<DanishConjugation>()?;
    m.add_class::<CzechConjugation>()?;
    m.add_class::<SlovenianConjugation>()?;
    m.add_class::<EstonianConjugation>()?;
    m.add_class::<ItalianConjugation>()?;
    m.add_class::<SwedishConjugation>()?;
    m.add_function(wrap_pyfunction!(conjugate, m)?)?;
    Ok(())
}
