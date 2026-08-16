//! The full conjugation table of a verb as one plain struct — shared by the
//! WebAssembly and Python bindings.

use crate::{AnalyticTense, Auxiliary, Mood, Number, Person, Tense, Verb};

#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub zu_infinitive: String,
    pub perfect_infinitive: String,
    pub auxiliary: String,
    pub present_participle: String,
    pub past_participle: String,
    /// [2sg, 2pl]; None for verbs without an imperative (modals).
    pub imperative: [Option<String>; 2],
    /// Adhortative and polite imperatives: [wir, Sie].
    pub imperative_extended: [String; 2],
    /// Each row is [1sg, 2sg, 3sg, 1pl, 2pl, 3pl].
    pub present: [String; 6],
    pub preterite: [String; 6],
    pub konjunktiv1: [String; 6],
    pub konjunktiv2: [String; 6],
    pub perfect: [String; 6],
    pub pluperfect: [String; 6],
    pub future1: [String; 6],
    pub future2: [String; 6],
    /// würde-form (Futur I in Konjunktiv II).
    pub wuerde: [String; 6],
    pub konj1_perfect: [String; 6],
    pub konj1_future: [String; 6],
    pub konj2_pluperfect: [String; 6],
    pub konj2_future2: [String; 6],
}

const SLOTS: [(Person, Number); 6] = [
    (Person::First, Number::Singular),
    (Person::Second, Number::Singular),
    (Person::Third, Number::Singular),
    (Person::First, Number::Plural),
    (Person::Second, Number::Plural),
    (Person::Third, Number::Plural),
];

fn row(f: impl Fn(Person, Number) -> String) -> [String; 6] {
    SLOTS.map(|(p, n)| f(p, n))
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        Self {
            infinitive: v.infinitive().to_string(),
            zu_infinitive: v.zu_infinitive(),
            perfect_infinitive: v.perfect_infinitive(),
            auxiliary: match v.auxiliary() {
                Auxiliary::Haben => "haben".to_string(),
                Auxiliary::Sein => "sein".to_string(),
            },
            present_participle: v.present_participle(),
            past_participle: v.past_participle(),
            imperative: [v.imperative(Number::Singular), v.imperative(Number::Plural)],
            imperative_extended: [v.imperative_first_plural(), v.imperative_polite()],
            present: row(|p, n| v.conjugate(Tense::Present, Mood::Indicative, p, n)),
            preterite: row(|p, n| v.conjugate(Tense::Preterite, Mood::Indicative, p, n)),
            konjunktiv1: row(|p, n| v.conjugate(Tense::Present, Mood::KonjunktivI, p, n)),
            konjunktiv2: row(|p, n| v.conjugate(Tense::Present, Mood::KonjunktivII, p, n)),
            perfect: row(|p, n| v.analytic(AnalyticTense::Perfect, Mood::Indicative, p, n)),
            pluperfect: row(|p, n| v.analytic(AnalyticTense::Pluperfect, Mood::Indicative, p, n)),
            future1: row(|p, n| v.analytic(AnalyticTense::FutureI, Mood::Indicative, p, n)),
            future2: row(|p, n| v.analytic(AnalyticTense::FutureII, Mood::Indicative, p, n)),
            wuerde: row(|p, n| v.analytic(AnalyticTense::FutureI, Mood::KonjunktivII, p, n)),
            konj1_perfect: row(|p, n| v.analytic(AnalyticTense::Perfect, Mood::KonjunktivI, p, n)),
            konj1_future: row(|p, n| v.analytic(AnalyticTense::FutureI, Mood::KonjunktivI, p, n)),
            konj2_pluperfect: row(|p, n| {
                v.analytic(AnalyticTense::Pluperfect, Mood::KonjunktivII, p, n)
            }),
            konj2_future2: row(|p, n| {
                v.analytic(AnalyticTense::FutureII, Mood::KonjunktivII, p, n)
            }),
        }
    }
}
