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

/// The full conjugation table of one Dutch verb. The present row is
/// [ik, jij, hij, wij, jullie, zij]; the past is [singular, plural].
#[pyclass(get_all, frozen)]
struct DutchConjugation {
    infinitive: String,
    present: Vec<String>,
    past: Vec<String>,
    imperative: String,
    present_participle: String,
    past_participle: String,
}

#[pymethods]
impl DutchConjugation {
    fn __repr__(&self) -> String {
        format!("DutchConjugation({:?})", self.infinitive)
    }
}

impl From<crate::nld::Table> for DutchConjugation {
    fn from(t: crate::nld::Table) -> Self {
        DutchConjugation {
            infinitive: t.infinitive,
            present: t.present.into(),
            past: t.past.into(),
            imperative: t.imperative,
            present_participle: t.present_participle,
            past_participle: t.past_participle,
        }
    }
}

/// The full conjugation table of one Catalan verb. Rows are
/// [jo, tu, ell/ella, nosaltres, vosaltres, ells/elles].
#[pyclass(get_all, frozen)]
struct CatalanConjugation {
    infinitive: String,
    gerund: String,
    past_participle: String,
    /// [tu, vostè, nosaltres, vosaltres, vostès]
    imperative: Vec<Option<String>>,
    present: Vec<String>,
    imperfect: Vec<String>,
    preterite: Vec<String>,
    future: Vec<String>,
    conditional: Vec<String>,
    subjunctive_present: Vec<String>,
    subjunctive_imperfect: Vec<String>,
}

#[pymethods]
impl CatalanConjugation {
    fn __repr__(&self) -> String {
        format!("CatalanConjugation({:?})", self.infinitive)
    }
}

impl From<crate::cat::Table> for CatalanConjugation {
    fn from(t: crate::cat::Table) -> Self {
        CatalanConjugation {
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
        }
    }
}

/// The conjugation of one Japanese verb: the katsuyou-kei (活用形) plus
/// the plain past た-form.
#[pyclass(get_all, frozen)]
struct JapaneseConjugation {
    /// 終止形 — dictionary form.
    terminal: String,
    /// 連用形 — continuative / masu-stem.
    continuative: String,
    /// 未然形 — irrealis.
    irrealis: String,
    /// 仮定形 — hypothetical.
    hypothetical: String,
    /// 命令形 — imperative.
    imperative: String,
    /// Plain past — た/だ-form.
    past: String,
}

#[pymethods]
impl JapaneseConjugation {
    fn __repr__(&self) -> String {
        format!("JapaneseConjugation({:?})", self.terminal)
    }
}

impl From<crate::jpn::Table> for JapaneseConjugation {
    fn from(t: crate::jpn::Table) -> Self {
        JapaneseConjugation {
            terminal: t.terminal,
            continuative: t.continuative,
            irrealis: t.irrealis,
            hypothetical: t.hypothetical,
            imperative: t.imperative,
            past: t.past,
        }
    }
}

/// The conjugation of one Eastern Armenian verb (see `ablaut::hye`).
/// Person rows are [ես, դու, նա, մենք, դուք, նրանք]; the analytic
/// tenses are spelled out (`գրում եմ`), the imperative is
/// [singular, plural].
#[pyclass(get_all, frozen)]
struct ArmenianConjugation {
    infinitive: String,
    present: [String; 6],
    imperfect: [String; 6],
    aorist: [String; 6],
    perfect: [String; 6],
    pluperfect: [String; 6],
    future: [String; 6],
    future_in_past: [String; 6],
    subjunctive_future: [String; 6],
    subjunctive_past: [String; 6],
    conditional: [String; 6],
    conditional_past: [String; 6],
    imperative: [String; 2],
    converb_imperfective: String,
    converb_perfective: String,
    converb_future: String,
    converb_simultaneous: String,
    connegative: String,
    participle_subject: String,
    participle_resultative: String,
    passive: String,
    causative: String,
}

#[pymethods]
impl ArmenianConjugation {
    fn __repr__(&self) -> String {
        format!("ArmenianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::hye::Table> for ArmenianConjugation {
    fn from(t: crate::hye::Table) -> Self {
        ArmenianConjugation {
            infinitive: t.infinitive,
            present: t.present,
            imperfect: t.imperfect,
            aorist: t.aorist,
            perfect: t.perfect,
            pluperfect: t.pluperfect,
            future: t.future,
            future_in_past: t.future_in_past,
            subjunctive_future: t.subjunctive_future,
            subjunctive_past: t.subjunctive_past,
            conditional: t.conditional,
            conditional_past: t.conditional_past,
            imperative: t.imperative,
            converb_imperfective: t.converb_imperfective,
            converb_perfective: t.converb_perfective,
            converb_future: t.converb_future,
            converb_simultaneous: t.converb_simultaneous,
            connegative: t.connegative,
            participle_subject: t.participle_subject,
            participle_resultative: t.participle_resultative,
            passive: t.passive,
            causative: t.causative,
        }
    }
}

/// The conjugation of one Turkish verb (see `ablaut::tur`). Person rows
/// are [ben, sen, o, biz, siz, onlar]; the imperative is
/// [2sg, 2pl]. Rows are affirmative.
#[pyclass(get_all, frozen)]
struct TurkishConjugation {
    infinitive: String,
    aorist: [String; 6],
    progressive: [String; 6],
    future: [String; 6],
    past: [String; 6],
    evidential: [String; 6],
    necessitative: [String; 6],
    imperative: [String; 2],
}

#[pymethods]
impl TurkishConjugation {
    fn __repr__(&self) -> String {
        format!("TurkishConjugation({:?})", self.infinitive)
    }
}

impl From<crate::tur::Table> for TurkishConjugation {
    fn from(t: crate::tur::Table) -> Self {
        TurkishConjugation {
            infinitive: t.infinitive,
            aorist: t.aorist,
            progressive: t.progressive,
            future: t.future,
            past: t.past,
            evidential: t.evidential,
            necessitative: t.necessitative,
            imperative: t.imperative,
        }
    }
}

/// The conjugation of one Tamil verb (see `ablaut::tam`). Finite rows
/// are [1sg, 1pl, 2sg, 2pl, 3sg m, 3sg f, 3sg hon, 3sg neut, 3pl epicene,
/// 3pl neut]; the imperative is [singular, plural].
#[pyclass(get_all, frozen)]
struct TamilConjugation {
    root: String,
    present: [String; 10],
    past: [String; 10],
    future: [String; 10],
    infinitive: String,
    adverbial: String,
    relative_past: String,
    relative_present: String,
    relative_future: String,
    conditional: String,
    imperative: [String; 2],
}

#[pymethods]
impl TamilConjugation {
    fn __repr__(&self) -> String {
        format!("TamilConjugation({:?})", self.root)
    }
}

impl From<crate::tam::Table> for TamilConjugation {
    fn from(t: crate::tam::Table) -> Self {
        TamilConjugation {
            root: t.root,
            present: t.present,
            past: t.past,
            future: t.future,
            infinitive: t.infinitive,
            adverbial: t.adverbial,
            relative_past: t.relative_past,
            relative_present: t.relative_present,
            relative_future: t.relative_future,
            conditional: t.conditional,
            imperative: t.imperative,
        }
    }
}

/// The conjugation of one Hindi verb (see `ablaut::hin`). Person rows
/// are [1sg, 2sg, 3sg, 1pl, 2pl, 3pl]; participle triples are
/// [masc sg, masc pl, fem]; the imperative is
/// [intimate (तू), familiar (तुम), polite (आप)].
#[pyclass(get_all, frozen)]
struct HindiConjugation {
    infinitive: String,
    oblique_infinitive: String,
    imperative: [String; 3],
    subjunctive: [String; 6],
    future_masculine: [String; 6],
    future_feminine: [String; 6],
    imperfective: [String; 3],
    perfective: [String; 3],
}

#[pymethods]
impl HindiConjugation {
    fn __repr__(&self) -> String {
        format!("HindiConjugation({:?})", self.infinitive)
    }
}

impl From<crate::hin::Table> for HindiConjugation {
    fn from(t: crate::hin::Table) -> Self {
        HindiConjugation {
            infinitive: t.infinitive,
            oblique_infinitive: t.oblique_infinitive,
            imperative: t.imperative,
            subjunctive: t.subjunctive,
            future_masculine: t.future_masculine,
            future_feminine: t.future_feminine,
            imperfective: t.imperfective,
            perfective: t.perfective,
        }
    }
}

/// The conjugation of one Urdu verb (see `ablaut::urd`). Urdu is
/// Hindustani in the Perso-Arabic script, so the layout matches
/// [`HindiConjugation`]: person rows are [1sg, 2sg, 3sg, 1pl, 2pl, 3pl];
/// participle triples are [masc sg, masc pl, fem]; the imperative is
/// [intimate (تو), familiar (تم), polite (آپ)]; the synthetic future is
/// written apart (اتروں گا).
#[pyclass(get_all, frozen)]
struct UrduConjugation {
    infinitive: String,
    oblique_infinitive: String,
    imperative: [String; 3],
    subjunctive: [String; 6],
    future_masculine: [String; 6],
    future_feminine: [String; 6],
    imperfective: [String; 3],
    perfective: [String; 3],
}

#[pymethods]
impl UrduConjugation {
    fn __repr__(&self) -> String {
        format!("UrduConjugation({:?})", self.infinitive)
    }
}

impl From<crate::urd::Table> for UrduConjugation {
    fn from(t: crate::urd::Table) -> Self {
        UrduConjugation {
            infinitive: t.infinitive,
            oblique_infinitive: t.oblique_infinitive,
            imperative: t.imperative,
            subjunctive: t.subjunctive,
            future_masculine: t.future_masculine,
            future_feminine: t.future_feminine,
            imperfective: t.imperfective,
            perfective: t.perfective,
        }
    }
}

/// The conjugation of one Tagalog verb (see `ablaut::tgl`). Each voice
/// row is [perfective, imperfective, contemplated]; `patient` holds
/// empty strings when the root takes no patient voice.
#[pyclass(get_all, frozen)]
struct TagalogConjugation {
    infinitive: String,
    actor: [String; 3],
    patient: [String; 3],
}

#[pymethods]
impl TagalogConjugation {
    fn __repr__(&self) -> String {
        format!("TagalogConjugation({:?})", self.infinitive)
    }
}

impl From<crate::tgl::Table> for TagalogConjugation {
    fn from(t: crate::tgl::Table) -> Self {
        TagalogConjugation {
            infinitive: t.infinitive,
            actor: t.actor,
            patient: t.patient,
        }
    }
}

/// The conjugation of one Gujarati verb (see `ablaut::guj`). Person
/// rows are [1sg, 2sg, 3, 1pl, 2pl] (the third person does not split
/// singular from plural); participle rows are [masc sg, masc pl, fem,
/// neut sg, neut pl]; the imperative is [2sg, 2pl, polite sg, polite pl].
#[pyclass(get_all, frozen)]
struct GujaratiConjugation {
    infinitive: String,
    verbal_noun: String,
    conjunctive: String,
    consecutive: String,
    present: [String; 5],
    future: [String; 5],
    imperative: [String; 4],
    perfective: [String; 5],
    imperfective: [String; 5],
    present_progressive: [String; 5],
}

#[pymethods]
impl GujaratiConjugation {
    fn __repr__(&self) -> String {
        format!("GujaratiConjugation({:?})", self.infinitive)
    }
}

impl From<crate::guj::Table> for GujaratiConjugation {
    fn from(t: crate::guj::Table) -> Self {
        GujaratiConjugation {
            infinitive: t.infinitive,
            verbal_noun: t.verbal_noun,
            conjunctive: t.conjunctive,
            consecutive: t.consecutive,
            present: t.present,
            future: t.future,
            imperative: t.imperative,
            perfective: t.perfective,
            imperfective: t.imperfective,
            present_progressive: t.present_progressive,
        }
    }
}

/// The conjugation of one Bengali verb (see `ablaut::ben`). Every
/// person row is [আমি, তুই, তুমি, সে, আপনি] — first, second intimate,
/// second familiar, third ordinary, honorific.
#[pyclass(get_all, frozen)]
struct BengaliConjugation {
    infinitive: String,
    verbal_infinitive: String,
    perfective: String,
    habitual_participle: String,
    progressive_participle: String,
    conditional: String,
    present: [String; 5],
    past: [String; 5],
    future: [String; 5],
    habitual: [String; 5],
    present_progressive: [String; 5],
    past_progressive: [String; 5],
    present_perfect: [String; 5],
    past_perfect: [String; 5],
}

#[pymethods]
impl BengaliConjugation {
    fn __repr__(&self) -> String {
        format!("BengaliConjugation({:?})", self.infinitive)
    }
}

impl From<crate::ben::Table> for BengaliConjugation {
    fn from(t: crate::ben::Table) -> Self {
        BengaliConjugation {
            infinitive: t.infinitive,
            verbal_infinitive: t.verbal_infinitive,
            perfective: t.perfective,
            habitual_participle: t.habitual_participle,
            progressive_participle: t.progressive_participle,
            conditional: t.conditional,
            present: t.present,
            past: t.past,
            future: t.future,
            habitual: t.habitual,
            present_progressive: t.present_progressive,
            past_progressive: t.past_progressive,
            present_perfect: t.present_perfect,
            past_perfect: t.past_perfect,
        }
    }
}

/// The conjugation of one Marathi verb (see `ablaut::mar`). Person rows
/// are [1sg, 2sg, 3sg, 1pl, 2pl, 3pl]; the subjunctive row is [masc sg,
/// fem sg, neut sg, masc pl, fem pl, neut pl]; the imperative is [2sg,
/// 2pl, 1, 3sg, 3pl].
#[pyclass(get_all, frozen)]
struct MarathiConjugation {
    infinitive: String,
    completive: String,
    purposive: String,
    prospective: String,
    present_masculine: [String; 6],
    present_feminine: [String; 6],
    perfective_masculine: [String; 6],
    perfective_feminine: [String; 6],
    subjunctive: [String; 6],
    future: [String; 6],
    imperative: [String; 5],
}

#[pymethods]
impl MarathiConjugation {
    fn __repr__(&self) -> String {
        format!("MarathiConjugation({:?})", self.infinitive)
    }
}

impl From<crate::mar::Table> for MarathiConjugation {
    fn from(t: crate::mar::Table) -> Self {
        MarathiConjugation {
            infinitive: t.infinitive,
            completive: t.completive,
            purposive: t.purposive,
            prospective: t.prospective,
            present_masculine: t.present_masculine,
            present_feminine: t.present_feminine,
            perfective_masculine: t.perfective_masculine,
            perfective_feminine: t.perfective_feminine,
            subjunctive: t.subjunctive,
            future: t.future,
            imperative: t.imperative,
        }
    }
}

/// The full conjugation table of one Macedonian verb. No infinitive
/// (Macedonian has none); the lemma is the 3sg present. Present and
/// imperfect rows are [1sg…3pl]; the imperfect l-participle is
/// [masc sg, fem sg, neut sg, pl].
#[pyclass(get_all, frozen)]
struct MacedonianConjugation {
    lemma: String,
    present: Vec<String>,
    imperfect: Vec<String>,
    imperfect_participle: Vec<String>,
    /// [2sg, 2pl]
    imperative: Vec<Option<String>>,
    passive_participle: Option<String>,
    converb: Option<String>,
    verbal_noun: Option<String>,
}

#[pymethods]
impl MacedonianConjugation {
    fn __repr__(&self) -> String {
        format!("MacedonianConjugation({:?})", self.lemma)
    }
}

impl From<crate::mkd::Table> for MacedonianConjugation {
    fn from(t: crate::mkd::Table) -> Self {
        MacedonianConjugation {
            lemma: t.lemma,
            present: t.present.into(),
            imperfect: t.imperfect.into(),
            imperfect_participle: t.imperfect_participle.into(),
            imperative: t.imperative.into(),
            passive_participle: t.passive_participle,
            converb: t.converb,
            verbal_noun: t.verbal_noun,
        }
    }
}

/// The four whole-word surface forms of one Korean verb (see
/// `ablaut::kor`).
#[pyclass(get_all, frozen)]
struct KoreanConjugation {
    infinitive: String,
    intimate: String,
    connective_ni: String,
    adnominal_present: String,
    formal_present: String,
}

#[pymethods]
impl KoreanConjugation {
    fn __repr__(&self) -> String {
        format!("KoreanConjugation({:?})", self.infinitive)
    }
}

impl From<crate::kor::Table> for KoreanConjugation {
    fn from(t: crate::kor::Table) -> Self {
        KoreanConjugation {
            infinitive: t.infinitive,
            intimate: t.intimate,
            connective_ni: t.connective_ni,
            adnominal_present: t.adnominal_present,
            formal_present: t.formal_present,
        }
    }
}

/// The finite conjugation of one Telugu verb (see `ablaut::tel`).
/// Every row is [1sg, 2sg, 3sg-masc, 3sg-nonmasc, 1pl, 2pl, 3pl-masc,
/// 3pl-nonmasc].
#[pyclass(get_all, frozen)]
struct TeluguConjugation {
    infinitive: String,
    past: Vec<String>,
    present_durative: Vec<String>,
    future: Vec<String>,
    /// [2sg, 2pl].
    imperative: Vec<String>,
}

#[pymethods]
impl TeluguConjugation {
    fn __repr__(&self) -> String {
        format!("TeluguConjugation({:?})", self.infinitive)
    }
}

impl From<crate::tel::Table> for TeluguConjugation {
    fn from(t: crate::tel::Table) -> Self {
        TeluguConjugation {
            infinitive: t.infinitive,
            past: t.past.to_vec(),
            present_durative: t.present_durative.to_vec(),
            future: t.future.to_vec(),
            imperative: t.imperative.to_vec(),
        }
    }
}

/// The finite conjugation of one Kannada verb (see `ablaut::kan`).
/// Every row is [1sg, 2sg, 3sg-masc, 3sg-fem, 3sg-neut, 1pl, 2pl,
/// 3pl-masc, 3pl-fem, 3pl-neut].
#[pyclass(get_all, frozen)]
struct KannadaConjugation {
    infinitive: String,
    past: Vec<String>,
    present: Vec<String>,
    future: Vec<String>,
    /// [2sg, 2pl].
    imperative: Vec<String>,
}

#[pymethods]
impl KannadaConjugation {
    fn __repr__(&self) -> String {
        format!("KannadaConjugation({:?})", self.infinitive)
    }
}

impl From<crate::kan::Table> for KannadaConjugation {
    fn from(t: crate::kan::Table) -> Self {
        KannadaConjugation {
            infinitive: t.infinitive,
            past: t.past.to_vec(),
            present: t.present.to_vec(),
            future: t.future.to_vec(),
            imperative: t.imperative.to_vec(),
        }
    }
}

/// The finite conjugation of one Russian verb. Person rows are
/// [я, ты, он/она/оно, мы, вы, они].
#[pyclass(get_all, frozen)]
struct RussianConjugation {
    infinitive: String,
    /// Present (imperfective) or simple future (perfective).
    non_past: Vec<String>,
    /// [masc, fem, neut] singular past.
    past_singular: Vec<String>,
    past_plural: String,
    /// [2sg, 2pl].
    imperative: Vec<String>,
}

#[pymethods]
impl RussianConjugation {
    fn __repr__(&self) -> String {
        format!("RussianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::rus::Table> for RussianConjugation {
    fn from(t: crate::rus::Table) -> Self {
        RussianConjugation {
            infinitive: t.infinitive,
            non_past: t.non_past.into(),
            past_singular: t.past_singular.into(),
            past_plural: t.past_plural,
            imperative: t.imperative.into(),
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
    perfeito_composto: Vec<String>,
    mais_que_perfeito_composto: Vec<String>,
    futuro_composto: Vec<String>,
    condicional_composto: Vec<String>,
    conjuntivo_perfeito: Vec<String>,
    conjuntivo_mais_que_perfeito: Vec<String>,
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
            perfeito_composto: t.perfeito_composto.into(),
            mais_que_perfeito_composto: t.mais_que_perfeito_composto.into(),
            futuro_composto: t.futuro_composto.into(),
            condicional_composto: t.condicional_composto.into(),
            conjuntivo_perfeito: t.conjuntivo_perfeito.into(),
            conjuntivo_mais_que_perfeito: t.conjuntivo_mais_que_perfeito.into(),
        }
    }
}

/// The full conjugation table of one Irish verb. Rows run [base,
/// 1sg, 2sg, 1pl, 2pl, 3pl, autonomous]; None marks analytic-only
/// slots (filled with pronouns in running text).
#[pyclass(get_all, frozen)]
struct IrishConjugation {
    lemma: String,
    verbal_noun: String,
    verbal_adjective: String,
    present: Vec<Option<String>>,
    past: Vec<Option<String>>,
    past_habitual: Vec<Option<String>>,
    future: Vec<Option<String>>,
    conditional: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    subjunctive: Vec<Option<String>>,
}

#[pymethods]
impl IrishConjugation {
    fn __repr__(&self) -> String {
        format!("IrishConjugation({:?})", self.lemma)
    }
}

impl From<crate::gle::Table> for IrishConjugation {
    fn from(t: crate::gle::Table) -> Self {
        IrishConjugation {
            lemma: t.lemma,
            verbal_noun: t.verbal_noun,
            verbal_adjective: t.verbal_adjective,
            present: t.present.into(),
            past: t.past.into(),
            past_habitual: t.past_habitual.into(),
            future: t.future.into(),
            conditional: t.conditional.into(),
            imperative: t.imperative.into(),
            subjunctive: t.subjunctive.into(),
        }
    }
}

/// The full conjugation table of one Finnish verb. Six-slot rows run
/// [minä, sinä, hän, me, te, he].
#[pyclass(get_all, frozen)]
struct FinnishConjugation {
    infinitive: String,
    present: Vec<String>,
    past: Vec<String>,
    conditional: Vec<String>,
    potential: Vec<String>,
    /// [2sg, 3sg, 1pl, 2pl, 3pl]
    imperative: Vec<Option<String>>,
    present_passive: String,
    past_passive: String,
    conditional_passive: String,
    potential_passive: String,
    imperative_passive: String,
    present_participle: String,
    past_participle: String,
    present_passive_participle: String,
    past_passive_participle: String,
    perfekti: Vec<String>,
    pluskvamperfekti: Vec<String>,
}

#[pymethods]
impl FinnishConjugation {
    fn __repr__(&self) -> String {
        format!("FinnishConjugation({:?})", self.infinitive)
    }
}

impl From<crate::fin::Table> for FinnishConjugation {
    fn from(t: crate::fin::Table) -> Self {
        FinnishConjugation {
            infinitive: t.infinitive,
            present: t.present.into(),
            past: t.past.into(),
            conditional: t.conditional.into(),
            potential: t.potential.into(),
            imperative: t.imperative.into(),
            present_passive: t.present_passive,
            past_passive: t.past_passive,
            conditional_passive: t.conditional_passive,
            potential_passive: t.potential_passive,
            imperative_passive: t.imperative_passive,
            present_participle: t.present_participle,
            past_participle: t.past_participle,
            present_passive_participle: t.present_passive_participle,
            past_passive_participle: t.past_passive_participle,
            perfekti: t.perfekti.into(),
            pluskvamperfekti: t.pluskvamperfekti.into(),
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
    perfect: Vec<String>,
    pluperfect: Vec<String>,
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
            perfect: t.perfect.into(),
            pluperfect: t.pluperfect.into(),
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
    preteklik: Vec<String>,
    prihodnjik: Vec<String>,
    pogojnik: Vec<String>,
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
            preteklik: t.preteklik.into(),
            prihodnjik: t.prihodnjik.into(),
            pogojnik: t.pogojnik.into(),
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
    minuly_cas: Vec<String>,
    kondicional: Vec<String>,
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
            minuly_cas: t.minuly_cas.into(),
            kondicional: t.kondicional.into(),
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

/// The full conjugation table of one Norwegian Bokmål verb (single
/// forms — no person/number agreement).
#[pyclass(get_all, frozen)]
struct NorwegianConjugation {
    infinitive: String,
    present: Option<String>,
    past: Option<String>,
    past_participle: Option<String>,
    imperative: Option<String>,
    present_participle: Option<String>,
    present_passive: Option<String>,
}

#[pymethods]
impl NorwegianConjugation {
    fn __repr__(&self) -> String {
        format!("NorwegianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::nob::Table> for NorwegianConjugation {
    fn from(t: crate::nob::Table) -> Self {
        NorwegianConjugation {
            infinitive: t.infinitive,
            present: t.present,
            past: t.past,
            past_participle: t.past_participle,
            imperative: t.imperative,
            present_participle: t.present_participle,
            present_passive: t.present_passive,
        }
    }
}

/// person/number agreement; the past is periphrastic bar a closed set).
#[pyclass(get_all, frozen)]
struct AfrikaansConjugation {
    infinitive: String,
    present: Option<String>,
    past: Option<String>,
    perfect: Option<String>,
    past_participle: Option<String>,
    present_participle: Option<String>,
    imperative: Option<String>,
}

#[pymethods]
impl AfrikaansConjugation {
    fn __repr__(&self) -> String {
        format!("AfrikaansConjugation({:?})", self.infinitive)
    }
}

impl From<crate::afr::Table> for AfrikaansConjugation {
    fn from(t: crate::afr::Table) -> Self {
        AfrikaansConjugation {
            infinitive: t.infinitive,
            present: t.present,
            past: t.past,
            perfect: t.perfect,
            past_participle: t.past_participle,
            present_participle: t.present_participle,
            imperative: t.imperative,
        }
    }
}

/// 2sg/2pl (imperative) and masc/fem/neut/pl indefinite (participles).
#[pyclass(get_all, frozen)]
struct BulgarianConjugation {
    present: String,
    present_all: Vec<Option<String>>,
    aorist: Vec<Option<String>>,
    imperfect: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    past_participle: Vec<Option<String>>,
    present_participle: Vec<Option<String>>,
    passive_participle: Vec<Option<String>>,
    verbal_adverb: Option<String>,
    verbal_noun: Option<String>,
}

#[pymethods]
impl BulgarianConjugation {
    fn __repr__(&self) -> String {
        format!("BulgarianConjugation({:?})", self.present)
    }
}

impl From<crate::bul::Table> for BulgarianConjugation {
    fn from(t: crate::bul::Table) -> Self {
        BulgarianConjugation {
            present: t.present,
            present_all: t.present_all,
            aorist: t.aorist,
            imperfect: t.imperfect,
            imperative: t.imperative,
            past_participle: t.past_participle,
            present_participle: t.present_participle,
            passive_participle: t.passive_participle,
            verbal_adverb: t.verbal_adverb,
            verbal_noun: t.verbal_noun,
        }
    }
}

/// (present, imperfect, aorist) and 2sg/2pl (imperative).
#[pyclass(get_all, frozen)]
struct GreekConjugation {
    present: String,
    present_all: Vec<Option<String>>,
    imperfect: Vec<Option<String>>,
    aorist: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    gerund: Option<String>,
}

#[pymethods]
impl GreekConjugation {
    fn __repr__(&self) -> String {
        format!("GreekConjugation({:?})", self.present)
    }
}

impl From<crate::ell::Table> for GreekConjugation {
    fn from(t: crate::ell::Table) -> Self {
        GreekConjugation {
            present: t.present,
            present_all: t.present_all,
            imperfect: t.imperfect,
            aorist: t.aorist,
            imperative: t.imperative,
            gerund: t.gerund,
        }
    }
}

/// (present, imperfect, aorist, admirative) and 2sg/2pl (imperative).
#[pyclass(get_all, frozen)]
struct AlbanianConjugation {
    present: String,
    present_all: Vec<Option<String>>,
    imperfect: Vec<Option<String>>,
    aorist: Vec<Option<String>>,
    admirative: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    participle: Option<String>,
}

#[pymethods]
impl AlbanianConjugation {
    fn __repr__(&self) -> String {
        format!("AlbanianConjugation({:?})", self.present)
    }
}

impl From<crate::sqi::Table> for AlbanianConjugation {
    fn from(t: crate::sqi::Table) -> Self {
        AlbanianConjugation {
            present: t.present,
            present_all: t.present_all,
            imperfect: t.imperfect,
            aorist: t.aorist,
            admirative: t.admirative,
            imperative: t.imperative,
            participle: t.participle,
        }
    }
}

/// masc/fem/neut (3sg) then virile/non-virile (3pl); imperative 2sg/1pl/2pl.
#[pyclass(get_all, frozen)]
struct PolishConjugation {
    infinitive: String,
    present: Vec<Option<String>>,
    future: Vec<Option<String>>,
    past: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
}

#[pymethods]
impl PolishConjugation {
    fn __repr__(&self) -> String {
        format!("PolishConjugation({:?})", self.infinitive)
    }
}

impl From<crate::pol::Table> for PolishConjugation {
    fn from(t: crate::pol::Table) -> Self {
        PolishConjugation {
            infinitive: t.infinitive,
            present: t.present,
            future: t.future,
            past: t.past,
            imperative: t.imperative,
        }
    }
}

/// future, aorist) and 2sg/2pl (imperative).
#[pyclass(get_all, frozen)]
struct AzerbaijaniConjugation {
    infinitive: String,
    present: Vec<Option<String>>,
    past: Vec<Option<String>>,
    future: Vec<Option<String>>,
    aorist: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
}

#[pymethods]
impl AzerbaijaniConjugation {
    fn __repr__(&self) -> String {
        format!("AzerbaijaniConjugation({:?})", self.infinitive)
    }
}

impl From<crate::aze::Table> for AzerbaijaniConjugation {
    fn from(t: crate::aze::Table) -> Self {
        AzerbaijaniConjugation {
            infinitive: t.infinitive,
            present: t.present,
            past: t.past,
            future: t.future,
            aorist: t.aorist,
            imperative: t.imperative,
        }
    }
}

/// infinitive. Each vector field runs 3sg / 3pl.
#[pyclass(get_all, frozen)]
struct UzbekConjugation {
    infinitive: String,
    present: Vec<Option<String>>,
    past: Vec<Option<String>>,
    future: Vec<Option<String>>,
    aorist: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
}

#[pymethods]
impl UzbekConjugation {
    fn __repr__(&self) -> String {
        format!("UzbekConjugation({:?})", self.infinitive)
    }
}

impl From<crate::uzb::Table> for UzbekConjugation {
    fn from(t: crate::uzb::Table) -> Self {
        UzbekConjugation {
            infinitive: t.infinitive,
            present: t.present,
            past: t.past,
            future: t.future,
            aorist: t.aorist,
            imperative: t.imperative,
        }
    }
}

/// aorist) and 2sg/2pl (imperative).
#[pyclass(get_all, frozen)]
struct TurkmenConjugation {
    infinitive: String,
    present: Vec<Option<String>>,
    past: Vec<Option<String>>,
    aorist: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
}

#[pymethods]
impl TurkmenConjugation {
    fn __repr__(&self) -> String {
        format!("TurkmenConjugation({:?})", self.infinitive)
    }
}

impl From<crate::tuk::Table> for TurkmenConjugation {
    fn from(t: crate::tuk::Table) -> Self {
        TurkmenConjugation {
            infinitive: t.infinitive,
            present: t.present,
            past: t.past,
            aorist: t.aorist,
            imperative: t.imperative,
        }
    }
}

/// The conjugation table of one Amharic verb.
#[pyclass(get_all, frozen)]
struct AmharicConjugation {
    lemma: String,
    perfective: Vec<Option<String>>,
    perfect: Vec<Option<String>>,
    imperfective: Vec<Option<String>>,
    imperfective_nfin: Vec<Option<String>>,
    jussive: Vec<Option<String>>,
}

#[pymethods]
impl AmharicConjugation {
    fn __repr__(&self) -> String {
        format!("AmharicConjugation({:?})", self.lemma)
    }
}

impl From<crate::amh::Table> for AmharicConjugation {
    fn from(t: crate::amh::Table) -> Self {
        AmharicConjugation {
            lemma: t.lemma,
            perfective: t.perfective,
            perfect: t.perfect,
            imperfective: t.imperfective,
            imperfective_nfin: t.imperfective_nfin,
            jussive: t.jussive,
        }
    }
}

/// The conjugation table of one Modern Hebrew verb.
#[pyclass(get_all, frozen)]
struct HebrewConjugation {
    lemma: String,
    past: Vec<Option<String>>,
    present: Vec<Option<String>>,
    future: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    infinitive: Option<String>,
}

#[pymethods]
impl HebrewConjugation {
    fn __repr__(&self) -> String {
        format!("HebrewConjugation({:?})", self.lemma)
    }
}

impl From<crate::heb::Table> for HebrewConjugation {
    fn from(t: crate::heb::Table) -> Self {
        HebrewConjugation {
            lemma: t.lemma,
            past: t.past,
            present: t.present,
            future: t.future,
            imperative: t.imperative,
            infinitive: t.infinitive,
        }
    }
}

/// The conjugation table of one Modern Standard Arabic verb.
#[pyclass(get_all, frozen)]
struct ArabicConjugation {
    lemma: String,
    perfect: Vec<Option<String>>,
    imperfect: Vec<Option<String>>,
    subjunctive: Vec<Option<String>>,
    jussive: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    perfect_passive: Vec<Option<String>>,
    imperfect_passive: Vec<Option<String>>,
    subjunctive_passive: Vec<Option<String>>,
    jussive_passive: Vec<Option<String>>,
}

#[pymethods]
impl ArabicConjugation {
    fn __repr__(&self) -> String {
        format!("ArabicConjugation({:?})", self.lemma)
    }
}

impl From<crate::ara::Table> for ArabicConjugation {
    fn from(t: crate::ara::Table) -> Self {
        ArabicConjugation {
            lemma: t.lemma,
            perfect: t.perfect,
            imperfect: t.imperfect,
            subjunctive: t.subjunctive,
            jussive: t.jussive,
            imperative: t.imperative,
            perfect_passive: t.perfect_passive,
            imperfect_passive: t.imperfect_passive,
            subjunctive_passive: t.subjunctive_passive,
            jussive_passive: t.jussive_passive,
        }
    }
}

/// The conjugation table of one Belarusian verb.
#[pyclass(get_all, frozen)]
struct BelarusianConjugation {
    infinitive: String,
    present: Vec<Option<String>>,
    future: Vec<Option<String>>,
    past: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
}

#[pymethods]
impl BelarusianConjugation {
    fn __repr__(&self) -> String {
        format!("BelarusianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::bel::Table> for BelarusianConjugation {
    fn from(t: crate::bel::Table) -> Self {
        BelarusianConjugation {
            infinitive: t.infinitive,
            present: t.present,
            future: t.future,
            past: t.past,
            imperative: t.imperative,
        }
    }
}

/// The conjugation table of one Welsh verb.
#[pyclass(get_all, frozen)]
struct WelshConjugation {
    verbal_noun: String,
    present: Vec<Option<String>>,
    imperfect: Vec<Option<String>>,
    preterite: Vec<Option<String>>,
    pluperfect: Vec<Option<String>>,
    subjunctive: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    participle: Option<String>,
}

#[pymethods]
impl WelshConjugation {
    fn __repr__(&self) -> String {
        format!("WelshConjugation({:?})", self.verbal_noun)
    }
}

impl From<crate::cym::Table> for WelshConjugation {
    fn from(t: crate::cym::Table) -> Self {
        WelshConjugation {
            verbal_noun: t.verbal_noun,
            present: t.present,
            imperfect: t.imperfect,
            preterite: t.preterite,
            pluperfect: t.pluperfect,
            subjunctive: t.subjunctive,
            imperative: t.imperative,
            participle: t.participle,
        }
    }
}

/// The conjugation table of one Faroese verb.
#[pyclass(get_all, frozen)]
struct FaroeseConjugation {
    infinitive: String,
    present: Vec<Option<String>>,
    past: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    supine: Option<String>,
    present_participle: Option<String>,
    past_participle: Option<String>,
}

#[pymethods]
impl FaroeseConjugation {
    fn __repr__(&self) -> String {
        format!("FaroeseConjugation({:?})", self.infinitive)
    }
}

impl From<crate::fao::Table> for FaroeseConjugation {
    fn from(t: crate::fao::Table) -> Self {
        FaroeseConjugation {
            infinitive: t.infinitive,
            present: t.present,
            past: t.past,
            imperative: t.imperative,
            supine: t.supine,
            present_participle: t.present_participle,
            past_participle: t.past_participle,
        }
    }
}

/// The conjugation table of one Galician verb.
#[pyclass(get_all, frozen)]
struct GalicianConjugation {
    infinitive: String,
    gerund: Option<String>,
    participle: Vec<Option<String>>,
    present: Vec<Option<String>>,
    imperfect: Vec<Option<String>>,
    preterite: Vec<Option<String>>,
    pluperfect: Vec<Option<String>>,
    future: Vec<Option<String>>,
    conditional: Vec<Option<String>>,
    present_subjunctive: Vec<Option<String>>,
    imperfect_subjunctive: Vec<Option<String>>,
    future_subjunctive: Vec<Option<String>>,
    personal_infinitive: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
}

#[pymethods]
impl GalicianConjugation {
    fn __repr__(&self) -> String {
        format!("GalicianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::glg::Table> for GalicianConjugation {
    fn from(t: crate::glg::Table) -> Self {
        GalicianConjugation {
            infinitive: t.infinitive,
            gerund: t.gerund,
            participle: t.participle,
            present: t.present,
            imperfect: t.imperfect,
            preterite: t.preterite,
            pluperfect: t.pluperfect,
            future: t.future,
            conditional: t.conditional,
            present_subjunctive: t.present_subjunctive,
            imperfect_subjunctive: t.imperfect_subjunctive,
            future_subjunctive: t.future_subjunctive,
            personal_infinitive: t.personal_infinitive,
            imperative: t.imperative,
        }
    }
}

/// The conjugation table of one Kazakh verb.
#[pyclass(get_all, frozen)]
struct KazakhConjugation {
    infinitive: String,
    aorist: Vec<Option<String>>,
    past: Vec<Option<String>>,
    future: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
}

#[pymethods]
impl KazakhConjugation {
    fn __repr__(&self) -> String {
        format!("KazakhConjugation({:?})", self.infinitive)
    }
}

impl From<crate::kaz::Table> for KazakhConjugation {
    fn from(t: crate::kaz::Table) -> Self {
        KazakhConjugation {
            infinitive: t.infinitive,
            aorist: t.aorist,
            past: t.past,
            future: t.future,
            imperative: t.imperative,
        }
    }
}

/// The conjugation table of one Latin verb.
#[pyclass(get_all, frozen)]
struct LatinConjugation {
    citation: String,
    infinitive: Option<String>,
    present: Vec<Option<String>>,
    imperfect: Vec<Option<String>>,
    future: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
}

#[pymethods]
impl LatinConjugation {
    fn __repr__(&self) -> String {
        format!("LatinConjugation({:?})", self.citation)
    }
}

impl From<crate::lat::Table> for LatinConjugation {
    fn from(t: crate::lat::Table) -> Self {
        LatinConjugation {
            citation: t.citation,
            infinitive: t.infinitive,
            present: t.present,
            imperfect: t.imperfect,
            future: t.future,
            imperative: t.imperative,
        }
    }
}

/// The conjugation table of one Luxembourgish verb.
#[pyclass(get_all, frozen)]
struct LuxembourgishConjugation {
    infinitive: String,
    present: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    past_participle: Option<String>,
}

#[pymethods]
impl LuxembourgishConjugation {
    fn __repr__(&self) -> String {
        format!("LuxembourgishConjugation({:?})", self.infinitive)
    }
}

impl From<crate::ltz::Table> for LuxembourgishConjugation {
    fn from(t: crate::ltz::Table) -> Self {
        LuxembourgishConjugation {
            infinitive: t.infinitive,
            present: t.present,
            imperative: t.imperative,
            past_participle: t.past_participle,
        }
    }
}

/// The conjugation table of one Occitan verb.
#[pyclass(get_all, frozen)]
struct OccitanConjugation {
    infinitive: String,
    present: Vec<Option<String>>,
    imperfect: Vec<Option<String>>,
    preterite: Vec<Option<String>>,
    future: Vec<Option<String>>,
    conditional: Vec<Option<String>>,
    present_subjunctive: Vec<Option<String>>,
    imperfect_subjunctive: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    past_participle: Option<String>,
    gerund: Option<String>,
}

#[pymethods]
impl OccitanConjugation {
    fn __repr__(&self) -> String {
        format!("OccitanConjugation({:?})", self.infinitive)
    }
}

impl From<crate::oci::Table> for OccitanConjugation {
    fn from(t: crate::oci::Table) -> Self {
        OccitanConjugation {
            infinitive: t.infinitive,
            present: t.present,
            imperfect: t.imperfect,
            preterite: t.preterite,
            future: t.future,
            conditional: t.conditional,
            present_subjunctive: t.present_subjunctive,
            imperfect_subjunctive: t.imperfect_subjunctive,
            imperative: t.imperative,
            past_participle: t.past_participle,
            gerund: t.gerund,
        }
    }
}

/// The conjugation table of one Tatar verb.
#[pyclass(get_all, frozen)]
struct TatarConjugation {
    infinitive: String,
    present: Vec<Option<String>>,
    past: Vec<Option<String>>,
    future: Vec<Option<String>>,
    conditional: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
}

#[pymethods]
impl TatarConjugation {
    fn __repr__(&self) -> String {
        format!("TatarConjugation({:?})", self.infinitive)
    }
}

impl From<crate::tat::Table> for TatarConjugation {
    fn from(t: crate::tat::Table) -> Self {
        TatarConjugation {
            infinitive: t.infinitive,
            present: t.present,
            past: t.past,
            future: t.future,
            conditional: t.conditional,
            imperative: t.imperative,
        }
    }
}

/// The conjugation table of one Yiddish verb.
#[pyclass(get_all, frozen)]
struct YiddishConjugation {
    infinitive: String,
    present: Vec<Option<String>>,
    imperative: Vec<Option<String>>,
    present_participle: Option<String>,
    past_participle: Option<String>,
}

#[pymethods]
impl YiddishConjugation {
    fn __repr__(&self) -> String {
        format!("YiddishConjugation({:?})", self.infinitive)
    }
}

impl From<crate::ydd::Table> for YiddishConjugation {
    fn from(t: crate::ydd::Table) -> Self {
        YiddishConjugation {
            infinitive: t.infinitive,
            present: t.present,
            imperative: t.imperative,
            present_participle: t.present_participle,
            past_participle: t.past_participle,
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
    present_row: Vec<String>,
    past_row: Vec<String>,
    present_perfect_row: Vec<String>,
    past_perfect_row: Vec<String>,
    future_row: Vec<String>,
    future_perfect_row: Vec<String>,
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
            present_row: t.present_row.into(),
            past_row: t.past_row.into(),
            present_perfect_row: t.present_perfect_row.into(),
            past_perfect_row: t.past_perfect_row.into(),
            future_row: t.future_row.into(),
            future_perfect_row: t.future_perfect_row.into(),
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
    perfect_compus: Vec<String>,
    viitor: Vec<String>,
    conditional_prezent: Vec<String>,
    subjunctive_perfect: Vec<String>,
    conditional_perfect: Vec<String>,
    viitor_anterior: Vec<String>,
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
            perfect_compus: t.perfect_compus.into(),
            viitor: t.viitor.into(),
            conditional_prezent: t.conditional_prezent.into(),
            subjunctive_perfect: t.subjunctive_perfect.into(),
            conditional_perfect: t.conditional_perfect.into(),
            viitor_anterior: t.viitor_anterior.into(),
        }
    }
}

/// The full conjugation table of one Italian verb. Rows are
/// [io, tu, lui/lei, noi, voi, loro].
#[pyclass(get_all, frozen)]
struct ItalianConjugation {
    infinitive: String,
    auxiliary: String,
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
    passato_prossimo: Vec<String>,
    trapassato_prossimo: Vec<String>,
    futuro_anteriore: Vec<String>,
    condizionale_passato: Vec<String>,
    congiuntivo_passato: Vec<String>,
    congiuntivo_trapassato: Vec<String>,
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
            auxiliary: t.auxiliary,
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
            passato_prossimo: t.passato_prossimo.into(),
            trapassato_prossimo: t.trapassato_prossimo.into(),
            futuro_anteriore: t.futuro_anteriore.into(),
            condizionale_passato: t.condizionale_passato.into(),
            congiuntivo_passato: t.congiuntivo_passato.into(),
            congiuntivo_trapassato: t.congiuntivo_trapassato.into(),
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
    presens_row: Option<Vec<String>>,
    preteritum_row: Option<Vec<String>>,
    perfekt_row: Option<Vec<String>>,
    pluskvamperfekt_row: Option<Vec<String>>,
    futurum_row: Vec<String>,
    konditionalis_row: Vec<String>,
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
            presens_row: t.presens_row.map(Into::into),
            preteritum_row: t.preteritum_row.map(Into::into),
            perfekt_row: t.perfekt_row.map(Into::into),
            pluskvamperfekt_row: t.pluskvamperfekt_row.map(Into::into),
            futurum_row: t.futurum_row.into(),
            konditionalis_row: t.konditionalis_row.into(),
        }
    }
}

/// The conjugation table of one Ukrainian verb. The present row is
/// [1sg, 2sg, 3sg, 1pl, 2pl, 3pl]; the l-past is [masc sg, fem sg,
/// neut sg, plural].
#[pyclass(get_all, frozen)]
struct UkrainianConjugation {
    infinitive: String,
    present: Vec<String>,
    /// [2sg, 1pl, 2pl]
    imperative: Vec<Option<String>>,
    past: Vec<String>,
}

#[pymethods]
impl UkrainianConjugation {
    fn __repr__(&self) -> String {
        format!("UkrainianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::ukr::Table> for UkrainianConjugation {
    fn from(t: crate::ukr::Table) -> Self {
        UkrainianConjugation {
            infinitive: t.infinitive,
            present: t.present.into(),
            imperative: t.imperative.into(),
            past: t.past.into(),
        }
    }
}

/// The synthetic active paradigm of one Icelandic verb. Person rows are
/// [ég, þú, hann/hún, við, þið, þeir].
#[pyclass(get_all, frozen)]
struct IcelandicConjugation {
    infinitive: String,
    supine: String,
    present_participle: String,
    /// [þú, þið]
    imperative: Vec<Option<String>>,
    present: Vec<String>,
    past: Vec<String>,
    subjunctive_present: Vec<String>,
    subjunctive_past: Vec<String>,
}

#[pymethods]
impl IcelandicConjugation {
    fn __repr__(&self) -> String {
        format!("IcelandicConjugation({:?})", self.infinitive)
    }
}

impl From<crate::isl::Table> for IcelandicConjugation {
    fn from(t: crate::isl::Table) -> Self {
        IcelandicConjugation {
            infinitive: t.infinitive,
            supine: t.supine,
            present_participle: t.present_participle,
            imperative: t.imperative.into(),
            present: t.present.into(),
            past: t.past.into(),
            subjunctive_present: t.subjunctive_present.into(),
            subjunctive_past: t.subjunctive_past.into(),
        }
    }
}

/// The person-core conjugation of one Swahili verb. Six-slot rows run
/// [1sg, 2sg, 3sg (class 1), 1pl, 2pl, 3pl (class 2)]; the full
/// noun-class matrix is reached through the Rust `swa::Verb::form` API.
#[pyclass(get_all, frozen)]
struct SwahiliConjugation {
    infinitive: String,
    infinitive_negative: String,
    /// [singular, plural]
    imperative: Vec<String>,
    habitual: String,
    present: Vec<String>,
    present_negative: Vec<String>,
    past: Vec<String>,
    future: Vec<String>,
    perfect: Vec<String>,
    subjunctive: Vec<String>,
    gnomic: Vec<String>,
}

#[pymethods]
impl SwahiliConjugation {
    fn __repr__(&self) -> String {
        format!("SwahiliConjugation({:?})", self.infinitive)
    }
}

impl From<crate::swa::Table> for SwahiliConjugation {
    fn from(t: crate::swa::Table) -> Self {
        SwahiliConjugation {
            infinitive: t.infinitive,
            infinitive_negative: t.infinitive_negative,
            imperative: t.imperative.into(),
            habitual: t.habitual,
            present: t.present.into(),
            present_negative: t.present_negative.into(),
            past: t.past.into(),
            future: t.future.into(),
            perfect: t.perfect.into(),
            subjunctive: t.subjunctive.into(),
            gnomic: t.gnomic.into(),
        }
    }
}

/// The conjugation of one Persian verb (see `ablaut::pes`). Person rows
/// are [1sg, 2sg, 3sg, 1pl, 2pl, 3pl] and affirmative; the imperative is
/// [2sg, 2pl]. Forms are in normalized Perso-Arabic orthography.
#[pyclass(get_all, frozen)]
struct PersianConjugation {
    infinitive: String,
    aorist: [String; 6],
    present: [String; 6],
    subjunctive: [String; 6],
    past: [String; 6],
    imperfect: [String; 6],
    perfect: [String; 6],
    pluperfect: [String; 6],
    future: [String; 6],
    perfect_subjunctive: [String; 6],
    present_progressive: [String; 6],
    past_progressive: [String; 6],
    imperative: [String; 2],
    past_participle: String,
    present_participle: String,
}

#[pymethods]
impl PersianConjugation {
    fn __repr__(&self) -> String {
        format!("PersianConjugation({:?})", self.infinitive)
    }
}

impl From<crate::pes::Table> for PersianConjugation {
    fn from(t: crate::pes::Table) -> Self {
        PersianConjugation {
            infinitive: t.infinitive,
            aorist: t.aorist,
            present: t.present,
            subjunctive: t.subjunctive,
            past: t.past,
            imperfect: t.imperfect,
            perfect: t.perfect,
            pluperfect: t.pluperfect,
            future: t.future,
            perfect_subjunctive: t.perfect_subjunctive,
            present_progressive: t.present_progressive,
            past_progressive: t.past_progressive,
            imperative: t.imperative,
            past_participle: t.past_participle,
            present_participle: t.present_participle,
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
        Some(crate::Lang::Gle) => {
            let v = crate::gle::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(IrishConjugation::from(crate::gle::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Isl) => {
            let v = crate::isl::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(IcelandicConjugation::from(crate::isl::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Fin) => {
            let v = crate::fin::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(FinnishConjugation::from(crate::fin::Table::build(&v))
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
        Some(crate::Lang::Nob) => {
            let v = crate::nob::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(NorwegianConjugation::from(crate::nob::Table::build(&v))
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
        Some(crate::Lang::Cat) => {
            let v = crate::cat::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(CatalanConjugation::from(crate::cat::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Rus) => {
            let v = crate::rus::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(RussianConjugation::from(crate::rus::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Nld) => {
            let v = crate::nld::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(DutchConjugation::from(crate::nld::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Ukr) => {
            let v = crate::ukr::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(UkrainianConjugation::from(crate::ukr::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Jpn) => {
            let v = crate::jpn::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(JapaneseConjugation::from(crate::jpn::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Hye) => {
            let v = crate::hye::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(ArmenianConjugation::from(crate::hye::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Kor) => {
            let v = crate::kor::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(KoreanConjugation::from(crate::kor::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Tur) => {
            let v = crate::tur::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(TurkishConjugation::from(crate::tur::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Hin) => {
            let v = crate::hin::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(HindiConjugation::from(crate::hin::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Swa) => {
            let v = crate::swa::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(SwahiliConjugation::from(crate::swa::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Tam) => {
            let v = crate::tam::Verb::from_root(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(TamilConjugation::from(crate::tam::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Tel) => {
            let v = crate::tel::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(TeluguConjugation::from(crate::tel::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Tgl) => {
            let v = crate::tgl::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(TagalogConjugation::from(crate::tgl::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Pes) => {
            let v = crate::pes::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(PersianConjugation::from(crate::pes::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Kan) => {
            let v = crate::kan::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(KannadaConjugation::from(crate::kan::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Guj) => {
            let v = crate::guj::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(GujaratiConjugation::from(crate::guj::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Urd) => {
            let v = crate::urd::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(UrduConjugation::from(crate::urd::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Ben) => {
            let v = crate::ben::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(BengaliConjugation::from(crate::ben::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Mar) => {
            let v = crate::mar::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(MarathiConjugation::from(crate::mar::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Mkd) => {
            let v = crate::mkd::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(MacedonianConjugation::from(crate::mkd::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Afr) => {
            let v = crate::afr::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(AfrikaansConjugation::from(crate::afr::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Bul) => {
            let v = crate::bul::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(BulgarianConjugation::from(crate::bul::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Ell) => {
            let v = crate::ell::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(GreekConjugation::from(crate::ell::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Sqi) => {
            let v = crate::sqi::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(AlbanianConjugation::from(crate::sqi::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Pol) => {
            let v = crate::pol::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(PolishConjugation::from(crate::pol::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Aze) => {
            let v = crate::aze::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(AzerbaijaniConjugation::from(crate::aze::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Uzb) => {
            let v = crate::uzb::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(UzbekConjugation::from(crate::uzb::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Tuk) => {
            let v = crate::tuk::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(TurkmenConjugation::from(crate::tuk::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Ara) => {
            let v = crate::ara::Verb::from_lemma(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(ArabicConjugation::from(crate::ara::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Heb) => {
            let v = crate::heb::Verb::from_lemma(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(HebrewConjugation::from(crate::heb::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Amh) => {
            let v = crate::amh::Verb::from_lemma(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(AmharicConjugation::from(crate::amh::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Bel) => {
            let v = crate::bel::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(BelarusianConjugation::from(crate::bel::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Cym) => {
            let v = crate::cym::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(WelshConjugation::from(crate::cym::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Fao) => {
            let v = crate::fao::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(FaroeseConjugation::from(crate::fao::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Glg) => {
            let v = crate::glg::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(GalicianConjugation::from(crate::glg::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Kaz) => {
            let v = crate::kaz::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(KazakhConjugation::from(crate::kaz::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Lat) => {
            let v = crate::lat::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(LatinConjugation::from(crate::lat::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Ltz) => {
            let v = crate::ltz::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(LuxembourgishConjugation::from(crate::ltz::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Oci) => {
            let v = crate::oci::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(OccitanConjugation::from(crate::oci::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Tat) => {
            let v = crate::tat::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(TatarConjugation::from(crate::tat::Table::build(&v))
                .into_pyobject(py)?
                .into())
        }
        Some(crate::Lang::Ydd) => {
            let v = crate::ydd::Verb::from_infinitive(infinitive)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(YiddishConjugation::from(crate::ydd::Table::build(&v))
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
    m.add_class::<CatalanConjugation>()?;
    m.add_class::<RussianConjugation>()?;
    m.add_class::<DutchConjugation>()?;
    m.add_class::<ArmenianConjugation>()?;
    m.add_class::<HindiConjugation>()?;
    m.add_class::<MarathiConjugation>()?;
    m.add_class::<MacedonianConjugation>()?;
    m.add_class::<AfrikaansConjugation>()?;
    m.add_class::<BulgarianConjugation>()?;
    m.add_class::<GreekConjugation>()?;
    m.add_class::<AlbanianConjugation>()?;
    m.add_class::<PolishConjugation>()?;
    m.add_class::<AzerbaijaniConjugation>()?;
    m.add_class::<UzbekConjugation>()?;
    m.add_class::<TurkmenConjugation>()?;
    m.add_class::<BelarusianConjugation>()?;
    m.add_class::<WelshConjugation>()?;
    m.add_class::<FaroeseConjugation>()?;
    m.add_class::<GalicianConjugation>()?;
    m.add_class::<KazakhConjugation>()?;
    m.add_class::<LatinConjugation>()?;
    m.add_class::<LuxembourgishConjugation>()?;
    m.add_class::<OccitanConjugation>()?;
    m.add_class::<TatarConjugation>()?;
    m.add_class::<YiddishConjugation>()?;
    m.add_class::<ArabicConjugation>()?;
    m.add_class::<HebrewConjugation>()?;
    m.add_class::<AmharicConjugation>()?;
    m.add_class::<KoreanConjugation>()?;
    m.add_class::<TurkishConjugation>()?;
    m.add_class::<PortugueseConjugation>()?;
    m.add_class::<RomanianConjugation>()?;
    m.add_class::<EnglishConjugation>()?;
    m.add_class::<DanishConjugation>()?;
    m.add_class::<NorwegianConjugation>()?;
    m.add_class::<CzechConjugation>()?;
    m.add_class::<SlovenianConjugation>()?;
    m.add_class::<EstonianConjugation>()?;
    m.add_class::<FinnishConjugation>()?;
    m.add_class::<IrishConjugation>()?;
    m.add_class::<ItalianConjugation>()?;
    m.add_class::<SwedishConjugation>()?;
    m.add_class::<UkrainianConjugation>()?;
    m.add_class::<IcelandicConjugation>()?;
    m.add_class::<JapaneseConjugation>()?;
    m.add_class::<SwahiliConjugation>()?;
    m.add_class::<TamilConjugation>()?;
    m.add_class::<TeluguConjugation>()?;
    m.add_class::<TagalogConjugation>()?;
    m.add_class::<PersianConjugation>()?;
    m.add_class::<KannadaConjugation>()?;
    m.add_class::<GujaratiConjugation>()?;
    m.add_class::<UrduConjugation>()?;
    m.add_class::<BengaliConjugation>()?;
    m.add_function(wrap_pyfunction!(conjugate, m)?)?;
    Ok(())
}
