//! Eastern Armenian conjugation: two productive conjugation classes,
//! four stems, and a compiled-in table of the verbs that keep an
//! archaic aorist.
//!
//! Every Armenian verb is cited by its infinitive, which ends in `-ել`
//! (գրել "write") or `-ալ` (կարդալ "read"); the class picks the
//! thematic vowel of the subjunctive and of the imperfect. Four stems
//! carry the paradigm:
//!
//! - the **present stem** (the infinitive minus its ending: գր-,
//!   կարդ-) builds the imperfective converb `գրում`, the subjunctive
//!   `գրեմ`/`կարդամ` and, with the prefix `կ-`, the conditional
//!   `կգրեմ`;
//! - the **perfect stem** builds the perfective converb `գրել`, the
//!   resultative participle `գրած` and the plural imperative `գրեք`.
//!   It equals the present stem in the `-ել` class and takes `-աց` in
//!   the `-ալ` class (կարդաց-);
//! - the **aorist stem** builds the six aorist forms — `-եց` plus
//!   `-ի/-իր/-∅/-ինք/-իք/-ին` for `-ել` verbs (գրեցի, գրեց), the same
//!   endings on the `-աց` stem for `-ալ` verbs (կարդացի);
//! - the **subject-participle stem** builds `գրող`, `կարդացող`.
//!
//! Two productive subclasses shift those stems:
//!
//! - **inchoatives** in `-անալ`/`-ենալ` (մեծանալ "grow big", վախենալ
//!   "be afraid") drop the `-ան-/-են-` in the perfect and take the
//!   vocalic aorist: մեծացա, մեծացավ, մեծացել, մեծացիր;
//! - **causatives** in `-ցնել` (ուրախացնել "make happy") replace `-ցն-`
//!   with `-ցր-` outside the present: ուրախացրի, ուրախացրած,
//!   ուրախացրու.
//!
//! The verbs with a stored aorist — the `-նել` class that drops its
//! `-ն-` (հագնել → հագա), the suppletives (գալ → եկա, ուտել → կերա,
//! անել → արեցի) and the `-առնալ` verbs (դառնալ → դարձա) — live in
//! `data/hye/verbs.tsv`.
//!
//! On top of the synthetic core sit the analytic tenses, all of them a
//! converb plus the copula: present `գրում եմ`, imperfect `գրում էի`,
//! perfect `գրել եմ`, pluperfect `գրել էի`, future `գրելու եմ`. Under
//! negation the copula moves in front of the converb and takes `չ-`:
//! `չեմ գրում`, `չէի գրել`.

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

/// Affirmative or negative: Armenian negates the synthetic forms with
/// the prefix `չ-` and the analytic ones by fronting a negated copula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Polarity {
    Positive,
    Negative,
}

/// A finite tense — the synthetic ones plus the analytic layer built
/// on the converbs and the copula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    /// `գրում եմ` — imperfective converb + copula.
    Present,
    /// `գրում էի`.
    Imperfect,
    /// `գրեցի` — the synthetic aorist.
    Aorist,
    /// `գրել եմ` — perfective converb + copula.
    Perfect,
    /// `գրել էի`.
    Pluperfect,
    /// `գրելու եմ` — future converb + copula.
    Future,
    /// `գրելու էի`.
    FutureInPast,
    /// `գրեմ` — the subjunctive future.
    SubjunctiveFuture,
    /// `գրեի` — the subjunctive past.
    SubjunctivePast,
    /// `կգրեմ` — subjunctive future under `կ-`. Negated it is analytic
    /// (`չեմ գրի`).
    Conditional,
    /// `կգրեի`; negated `չէի գրի`.
    ConditionalPast,
}

/// A non-finite form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Converb {
    /// `գրում` — the imperfective converb, base of the present.
    Imperfective,
    /// `գրել` — the perfective converb, base of the perfect.
    Perfective,
    /// `գրելու` — the future (destinative) converb.
    Future,
    /// `գրելիք` — the second future converb.
    Prospective,
    /// `գրելիս` — the simultaneous converb.
    Simultaneous,
    /// `գրի` — the connegative, base of the negative conditional.
    Connegative,
}

/// A participle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Participle {
    /// `գրող` — the subject (agent) participle.
    Subject,
    /// `գրած` — the resultative participle.
    Resultative,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not end in `-ել` or `-ալ`.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Armenian infinitive")
    }
}

/// The conjugation class, named for the infinitive ending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// `-ել`: գրել, խոսել.
    E,
    /// `-ալ`: կարդալ, խաղալ.
    A,
}

/// How the six aorist forms are built from the aorist stem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AoristSet {
    /// `-ի, -իր, -∅, -ինք, -իք, -ին` (գրեցի, գրեց).
    I,
    /// `-ա, -ար, -ավ, -անք, -աք, -ան` (մեծացա, մեծացավ).
    A,
    /// `-ի, -իր, -եց, -ինք, -իք, -ին` — the causative's mixed set:
    /// vocalic endings around a third singular in `-եց` (հասցրի,
    /// հասցրիր, հասցրեց).
    Mixed,
}

const AORIST_I: [&str; 6] = ["ի", "իր", "", "ինք", "իք", "ին"];
const AORIST_A: [&str; 6] = ["ա", "ար", "ավ", "անք", "աք", "ան"];
const AORIST_MIXED: [&str; 6] = ["ի", "իր", "եց", "ինք", "իք", "ին"];
const SUBJUNCTIVE_E: [&str; 6] = ["եմ", "ես", "ի", "ենք", "եք", "են"];
const SUBJUNCTIVE_A: [&str; 6] = ["ամ", "աս", "ա", "անք", "աք", "ան"];
const SUBJUNCTIVE_PAST_E: [&str; 6] = ["եի", "եիր", "եր", "եինք", "եիք", "եին"];
const SUBJUNCTIVE_PAST_A: [&str; 6] = ["այի", "այիր", "ար", "այինք", "այիք", "ային"];

/// The copula, which carries person and number in every analytic tense.
const COPULA_PRESENT: [&str; 6] = ["եմ", "ես", "է", "ենք", "եք", "են"];
const COPULA_PAST: [&str; 6] = ["էի", "էիր", "էր", "էինք", "էիք", "էին"];
const COPULA_PRESENT_NEG: [&str; 6] = ["չեմ", "չես", "չի", "չենք", "չեք", "չեն"];
const COPULA_PAST_NEG: [&str; 6] = ["չէի", "չէիր", "չէր", "չէինք", "չէիք", "չէին"];

/// The negative prefix on a synthetic form, and the negative particle
/// of the prohibitive.
const NEG: &str = "չ";
const PROHIBITIVE: &str = "մի";

/// The compiled-in table of verbs whose stems are not derivable.
static LEXICON_TSV: &str = include_str!("../data/hye/verbs.tsv");

/// A stored paradigm. Every column is optional: `-` falls through to
/// the productive rule for that stem.
#[derive(Debug, Clone, Default)]
struct LexEntry {
    imperfective: Option<String>,
    perfect: Option<String>,
    aorist: Option<String>,
    aorist_set: Option<AoristSet>,
    imperative_sg: Option<String>,
    imperative_pl: Option<String>,
    subject: Option<String>,
    passive: Option<String>,
    causative: Option<String>,
}

fn opt(s: &str) -> Option<String> {
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
                    imperfective: opt(g(1)),
                    perfect: opt(g(2)),
                    aorist: opt(g(3)),
                    aorist_set: match g(4) {
                        "1" => Some(AoristSet::I),
                        "2" => Some(AoristSet::A),
                        "3" => Some(AoristSet::Mixed),
                        _ => None,
                    },
                    imperative_sg: opt(g(5)),
                    imperative_pl: opt(g(6)),
                    subject: opt(g(7)),
                    passive: opt(g(8)),
                    causative: opt(g(9)),
                },
            );
        }
        m
    })
}

/// Drop the last `n` characters of a string.
fn trim_end(s: &str, n: usize) -> String {
    let keep = s.chars().count().saturating_sub(n);
    s.chars().take(keep).collect()
}

/// A conjugatable Eastern Armenian verb: the four stems, resolved once
/// from the infinitive and the stored table.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    class: Class,
    /// The present stem (գր-, կարդ-).
    stem: String,
    /// The perfect stem (գր-, կարդաց-, մեծաց-).
    perfect: String,
    /// The aorist stem (գրեց-, կարդաց-, մեծաց-, եկ-).
    aorist: String,
    aorist_set: AoristSet,
    /// The subject-participle stem (գր-, կարդաց-).
    subject: String,
    imperative_sg: String,
    imperative_pl: String,
    imperfective: String,
    passive: String,
    causative: String,
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

impl Verb {
    /// Build a verb from its infinitive.
    ///
    /// ```
    /// use ablaut::hye::{Number, Person, Polarity, Tense, Verb};
    /// let v = Verb::from_infinitive("գրել").unwrap();
    /// assert_eq!(
    ///     v.form(Tense::Aorist, Person::First, Number::Singular, Polarity::Positive),
    ///     "գրեցի"
    /// );
    /// ```
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim().to_lowercase();
        if inf.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        let class = if inf.ends_with("ել") {
            Class::E
        } else if inf.ends_with("ալ") {
            Class::A
        } else {
            return Err(Error::NotAVerb);
        };
        let stem = trim_end(&inf, 2);
        if stem.is_empty() {
            return Err(Error::NotAVerb);
        }
        let lex = lexicon().get(&inf);

        // The productive subclasses: inchoative -անալ/-ենալ and
        // causative -ցնել shift the perfect and aorist stems.
        let inchoative = class == Class::A
            && stem.chars().count() > 3
            && (stem.ends_with("ան") || stem.ends_with("են"));
        let causative = class == Class::E && stem.chars().count() > 3 && stem.ends_with("ցն");

        let (perfect, aorist, aorist_set) = if inchoative {
            let linking = if stem.ends_with("ան") {
                "աց"
            } else {
                "եց"
            };
            let p = format!("{}{linking}", trim_end(&stem, 2));
            (p.clone(), p, AoristSet::A)
        } else if causative {
            let p = format!("{}ցր", trim_end(&stem, 2));
            (p.clone(), p, AoristSet::Mixed)
        } else if class == Class::E {
            (stem.clone(), format!("{stem}եց"), AoristSet::I)
        } else {
            let p = format!("{stem}աց");
            (p.clone(), p, AoristSet::I)
        };

        let perfect = lex.and_then(|l| l.perfect.clone()).unwrap_or(perfect);
        let aorist = lex.and_then(|l| l.aorist.clone()).unwrap_or(aorist);
        let aorist_set = lex.and_then(|l| l.aorist_set).unwrap_or(aorist_set);

        // The subject participle is built on the present stem in the
        // `-ել` class (գրող, հագնող, ուրախացնող) and on the perfect
        // stem in the `-ալ` class (կարդացող, մեծացող).
        let subject = match class {
            Class::E => stem.clone(),
            Class::A => perfect.clone(),
        };
        let subject = lex.and_then(|l| l.subject.clone()).unwrap_or(subject);

        let imperative_sg = if inchoative || causative {
            let suffix = if causative { "ու" } else { "իր" };
            format!("{perfect}{suffix}")
        } else {
            match class {
                Class::E => format!("{stem}իր"),
                Class::A => format!("{stem}ա"),
            }
        };
        let imperative_sg = lex
            .and_then(|l| l.imperative_sg.clone())
            .unwrap_or(imperative_sg);
        let imperative_pl = lex
            .and_then(|l| l.imperative_pl.clone())
            .unwrap_or_else(|| format!("{perfect}եք"));

        let imperfective = lex
            .and_then(|l| l.imperfective.clone())
            .unwrap_or_else(|| format!("{stem}ում"));

        // The passive is built on the present stem in the `-ել` class
        // and on the perfect stem in the `-ալ` class; the causative
        // takes `-ցնել` after the inchoative's linking vowel.
        // The passive drops the causative's `-ն-` (հասցնել →
        // հասցվել); elsewhere it is built on the present stem in the
        // `-ել` class and on the perfect stem in the `-ալ` class.
        let passive = lex.and_then(|l| l.passive.clone()).unwrap_or_else(|| {
            if causative {
                format!("{}վել", trim_end(&stem, 1))
            } else {
                let base = if class == Class::E { &stem } else { &perfect };
                format!("{base}վել")
            }
        });
        let causative_inf = lex.and_then(|l| l.causative.clone()).unwrap_or_else(|| {
            if inchoative {
                format!("{perfect}նել")
            } else if class == Class::E {
                format!("{stem}եցնել")
            } else {
                format!("{stem}ացնել")
            }
        });

        Ok(Self {
            infinitive: inf,
            class,
            stem,
            perfect,
            aorist,
            aorist_set,
            subject,
            imperative_sg,
            imperative_pl,
            imperfective,
            passive,
            causative: causative_inf,
        })
    }

    /// The normalized infinitive (`գրել`), or its negation (`չգրել`).
    #[must_use]
    pub fn infinitive(&self, polarity: Polarity) -> String {
        self.negate(&self.infinitive, polarity)
    }

    /// The conjugation class.
    #[must_use]
    pub const fn class(&self) -> Class {
        self.class
    }

    fn negate(&self, form: &str, polarity: Polarity) -> String {
        match polarity {
            Polarity::Positive => form.to_string(),
            Polarity::Negative => format!("{NEG}{form}"),
        }
    }

    /// A converb (the non-finite base of the analytic tenses).
    #[must_use]
    pub fn converb(&self, converb: Converb, polarity: Polarity) -> String {
        let form = match converb {
            Converb::Imperfective => self.imperfective.clone(),
            Converb::Perfective => format!("{}ել", self.perfect),
            Converb::Future => format!("{}ու", self.infinitive),
            Converb::Prospective => format!("{}իք", self.infinitive),
            Converb::Simultaneous => format!("{}իս", self.infinitive),
            Converb::Connegative => match self.class {
                Class::E => format!("{}ի", self.stem),
                Class::A => format!("{}ա", self.stem),
            },
        };
        self.negate(&form, polarity)
    }

    /// A participle.
    #[must_use]
    pub fn participle(&self, participle: Participle, polarity: Polarity) -> String {
        let form = match participle {
            Participle::Subject => format!("{}ող", self.subject),
            Participle::Resultative => format!("{}ած", self.perfect),
        };
        self.negate(&form, polarity)
    }

    /// The second-person imperative (`գրիր`, `գրեք`); negated it is the
    /// prohibitive `մի գրիր`.
    #[must_use]
    pub fn imperative(&self, number: Number, polarity: Polarity) -> String {
        let form = match number {
            Number::Singular => &self.imperative_sg,
            Number::Plural => &self.imperative_pl,
        };
        match polarity {
            Polarity::Positive => form.clone(),
            Polarity::Negative => format!("{PROHIBITIVE} {form}"),
        }
    }

    /// The passive infinitive (`գրվել`).
    #[must_use]
    pub fn passive(&self) -> &str {
        &self.passive
    }

    /// The causative infinitive (`գրեցնել`).
    #[must_use]
    pub fn causative(&self) -> &str {
        &self.causative
    }

    fn aorist_form(&self, i: usize) -> String {
        let endings = match self.aorist_set {
            AoristSet::I => AORIST_I,
            AoristSet::A => AORIST_A,
            AoristSet::Mixed => AORIST_MIXED,
        };
        format!("{}{}", self.aorist, endings[i])
    }

    fn subjunctive_form(&self, i: usize, past: bool) -> String {
        let endings = match (self.class, past) {
            (Class::E, false) => SUBJUNCTIVE_E,
            (Class::A, false) => SUBJUNCTIVE_A,
            (Class::E, true) => SUBJUNCTIVE_PAST_E,
            (Class::A, true) => SUBJUNCTIVE_PAST_A,
        };
        format!("{}{}", self.stem, endings[i])
    }

    /// An analytic tense: the converb with the copula behind it, or —
    /// under negation — the negated copula in front.
    fn analytic(&self, converb: &str, i: usize, past: bool, polarity: Polarity) -> String {
        match polarity {
            Polarity::Positive => {
                let copula = if past { COPULA_PAST } else { COPULA_PRESENT };
                format!("{converb} {}", copula[i])
            }
            Polarity::Negative => {
                let copula = if past {
                    COPULA_PAST_NEG
                } else {
                    COPULA_PRESENT_NEG
                };
                format!("{} {converb}", copula[i])
            }
        }
    }

    /// A finite form.
    ///
    /// ```
    /// use ablaut::hye::{Number, Person, Polarity, Tense, Verb};
    /// let v = Verb::from_infinitive("կարդալ").unwrap();
    /// let p = (Person::Third, Number::Singular);
    /// assert_eq!(v.form(Tense::Present, p.0, p.1, Polarity::Positive), "կարդում է");
    /// assert_eq!(v.form(Tense::Present, p.0, p.1, Polarity::Negative), "չի կարդում");
    /// assert_eq!(v.form(Tense::Aorist, p.0, p.1, Polarity::Positive), "կարդաց");
    /// ```
    #[must_use]
    pub fn form(&self, tense: Tense, person: Person, number: Number, polarity: Polarity) -> String {
        let i = person.index(number);
        match tense {
            Tense::Aorist => self.negate(&self.aorist_form(i), polarity),
            Tense::SubjunctiveFuture => self.negate(&self.subjunctive_form(i, false), polarity),
            Tense::SubjunctivePast => self.negate(&self.subjunctive_form(i, true), polarity),
            // The conditional prefixes `կ-` to the subjunctive; its
            // negation is analytic, on the connegative.
            Tense::Conditional | Tense::ConditionalPast => {
                let past = tense == Tense::ConditionalPast;
                match polarity {
                    Polarity::Positive => format!("կ{}", self.subjunctive_form(i, past)),
                    Polarity::Negative => {
                        let copula = if past {
                            COPULA_PAST_NEG
                        } else {
                            COPULA_PRESENT_NEG
                        };
                        format!(
                            "{} {}",
                            copula[i],
                            self.converb(Converb::Connegative, Polarity::Positive)
                        )
                    }
                }
            }
            Tense::Present => self.analytic(&self.imperfective, i, false, polarity),
            Tense::Imperfect => self.analytic(&self.imperfective, i, true, polarity),
            Tense::Perfect => self.analytic(
                &self.converb(Converb::Perfective, Polarity::Positive),
                i,
                false,
                polarity,
            ),
            Tense::Pluperfect => self.analytic(
                &self.converb(Converb::Perfective, Polarity::Positive),
                i,
                true,
                polarity,
            ),
            Tense::Future => self.analytic(
                &self.converb(Converb::Future, Polarity::Positive),
                i,
                false,
                polarity,
            ),
            Tense::FutureInPast => self.analytic(
                &self.converb(Converb::Future, Polarity::Positive),
                i,
                true,
                polarity,
            ),
        }
    }
}

/// The full conjugation table of an Armenian verb — shared by the
/// WebAssembly and Python bindings. Every six-slot row is
/// `[1sg, 2sg, 3sg, 1pl, 2pl, 3pl]`; the rows are affirmative, and the
/// negatives are a method away on [`Verb`].
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// `գրում եմ`.
    pub present: [String; 6],
    /// `գրում էի`.
    pub imperfect: [String; 6],
    /// `գրեցի` — the synthetic aorist.
    pub aorist: [String; 6],
    /// `գրել եմ`.
    pub perfect: [String; 6],
    /// `գրել էի`.
    pub pluperfect: [String; 6],
    /// `գրելու եմ`.
    pub future: [String; 6],
    /// `գրելու էի`.
    pub future_in_past: [String; 6],
    /// `գրեմ`.
    pub subjunctive_future: [String; 6],
    /// `գրեի`.
    pub subjunctive_past: [String; 6],
    /// `կգրեմ`.
    pub conditional: [String; 6],
    /// `կգրեի`.
    pub conditional_past: [String; 6],
    /// `[գրիր, գրեք]`.
    pub imperative: [String; 2],
    /// `գրում`.
    pub converb_imperfective: String,
    /// `գրել`.
    pub converb_perfective: String,
    /// `գրելու`.
    pub converb_future: String,
    /// `գրելիս`.
    pub converb_simultaneous: String,
    /// `գրի` — the connegative.
    pub connegative: String,
    /// `գրող`.
    pub participle_subject: String,
    /// `գրած`.
    pub participle_resultative: String,
    /// `գրվել`.
    pub passive: String,
    /// `գրեցնել`.
    pub causative: String,
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
        let row = |t: Tense| SLOTS.map(|(p, n)| v.form(t, p, n, Polarity::Positive));
        Self {
            infinitive: v.infinitive(Polarity::Positive),
            present: row(Tense::Present),
            imperfect: row(Tense::Imperfect),
            aorist: row(Tense::Aorist),
            perfect: row(Tense::Perfect),
            pluperfect: row(Tense::Pluperfect),
            future: row(Tense::Future),
            future_in_past: row(Tense::FutureInPast),
            subjunctive_future: row(Tense::SubjunctiveFuture),
            subjunctive_past: row(Tense::SubjunctivePast),
            conditional: row(Tense::Conditional),
            conditional_past: row(Tense::ConditionalPast),
            imperative: [
                v.imperative(Number::Singular, Polarity::Positive),
                v.imperative(Number::Plural, Polarity::Positive),
            ],
            converb_imperfective: v.converb(Converb::Imperfective, Polarity::Positive),
            converb_perfective: v.converb(Converb::Perfective, Polarity::Positive),
            converb_future: v.converb(Converb::Future, Polarity::Positive),
            converb_simultaneous: v.converb(Converb::Simultaneous, Polarity::Positive),
            connegative: v.converb(Converb::Connegative, Polarity::Positive),
            participle_subject: v.participle(Participle::Subject, Polarity::Positive),
            participle_resultative: v.participle(Participle::Resultative, Polarity::Positive),
            passive: v.passive().to_string(),
            causative: v.causative().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};
    use Polarity::{Negative as NEGP, Positive as POS};

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn e_class_regular() {
        let g = v("գրել");
        assert_eq!(g.class(), Class::E);
        assert_eq!(g.form(Tense::Aorist, P1, SG, POS), "գրեցի");
        assert_eq!(g.form(Tense::Aorist, P3, SG, POS), "գրեց");
        assert_eq!(g.form(Tense::Aorist, P3, PL, NEGP), "չգրեցին");
        assert_eq!(g.form(Tense::SubjunctiveFuture, P1, SG, POS), "գրեմ");
        assert_eq!(g.form(Tense::SubjunctiveFuture, P3, SG, POS), "գրի");
        assert_eq!(g.form(Tense::SubjunctivePast, P3, SG, POS), "գրեր");
        assert_eq!(g.form(Tense::Conditional, P1, SG, POS), "կգրեմ");
        assert_eq!(g.form(Tense::ConditionalPast, P1, SG, POS), "կգրեի");
        assert_eq!(g.imperative(SG, POS), "գրիր");
        assert_eq!(g.imperative(PL, POS), "գրեք");
        assert_eq!(g.imperative(SG, NEGP), "մի գրիր");
        assert_eq!(g.converb(Converb::Imperfective, POS), "գրում");
        assert_eq!(g.converb(Converb::Perfective, POS), "գրել");
        assert_eq!(g.converb(Converb::Future, POS), "գրելու");
        assert_eq!(g.converb(Converb::Simultaneous, POS), "գրելիս");
        assert_eq!(g.converb(Converb::Connegative, POS), "գրի");
        assert_eq!(g.participle(Participle::Subject, POS), "գրող");
        assert_eq!(g.participle(Participle::Resultative, NEGP), "չգրած");
        assert_eq!(g.passive(), "գրվել");
        assert_eq!(g.causative(), "գրեցնել");
    }

    #[test]
    fn a_class_regular() {
        let k = v("կարդալ");
        assert_eq!(k.class(), Class::A);
        assert_eq!(k.form(Tense::Aorist, P1, SG, POS), "կարդացի");
        assert_eq!(k.form(Tense::Aorist, P3, SG, POS), "կարդաց");
        assert_eq!(k.form(Tense::SubjunctiveFuture, P1, SG, POS), "կարդամ");
        assert_eq!(k.form(Tense::SubjunctivePast, P1, SG, POS), "կարդայի");
        assert_eq!(k.form(Tense::SubjunctivePast, P3, SG, POS), "կարդար");
        assert_eq!(k.converb(Converb::Imperfective, POS), "կարդում");
        assert_eq!(k.converb(Converb::Perfective, POS), "կարդացել");
        assert_eq!(k.participle(Participle::Subject, POS), "կարդացող");
        assert_eq!(k.participle(Participle::Resultative, POS), "կարդացած");
        assert_eq!(k.imperative(SG, POS), "կարդա");
        assert_eq!(k.imperative(PL, POS), "կարդացեք");
        assert_eq!(k.causative(), "կարդացնել");
    }

    #[test]
    fn analytic_tenses() {
        let g = v("գրել");
        assert_eq!(g.form(Tense::Present, P1, SG, POS), "գրում եմ");
        assert_eq!(g.form(Tense::Present, P3, SG, NEGP), "չի գրում");
        assert_eq!(g.form(Tense::Imperfect, P1, SG, POS), "գրում էի");
        assert_eq!(g.form(Tense::Imperfect, P3, PL, NEGP), "չէին գրում");
        assert_eq!(g.form(Tense::Perfect, P2, SG, POS), "գրել ես");
        assert_eq!(g.form(Tense::Pluperfect, P1, PL, POS), "գրել էինք");
        assert_eq!(g.form(Tense::Future, P1, SG, POS), "գրելու եմ");
        assert_eq!(g.form(Tense::FutureInPast, P3, SG, POS), "գրելու էր");
        // The negative conditional is analytic, on the connegative.
        assert_eq!(g.form(Tense::Conditional, P1, SG, NEGP), "չեմ գրի");
        assert_eq!(g.form(Tense::ConditionalPast, P3, SG, NEGP), "չէր գրի");
    }

    #[test]
    fn inchoative_class() {
        let m = v("մեծանալ");
        assert_eq!(m.form(Tense::Aorist, P1, SG, POS), "մեծացա");
        assert_eq!(m.form(Tense::Aorist, P3, SG, POS), "մեծացավ");
        assert_eq!(m.form(Tense::SubjunctiveFuture, P1, SG, POS), "մեծանամ");
        assert_eq!(m.form(Tense::Present, P1, SG, POS), "մեծանում եմ");
        assert_eq!(m.converb(Converb::Perfective, POS), "մեծացել");
        assert_eq!(m.participle(Participle::Resultative, POS), "մեծացած");
        assert_eq!(m.imperative(SG, POS), "մեծացիր");
        assert_eq!(m.imperative(PL, POS), "մեծացեք");
        assert_eq!(m.causative(), "մեծացնել");
        // The -ենալ members take the same shape with -եց-.
        let u = v("վախենալ");
        assert_eq!(u.form(Tense::Aorist, P1, SG, POS), "վախեցա");
        assert_eq!(u.converb(Converb::Perfective, POS), "վախեցել");
        assert_eq!(u.causative(), "վախեցնել");
    }

    #[test]
    fn causative_class() {
        let c = v("ուրախացնել");
        assert_eq!(c.form(Tense::Aorist, P1, SG, POS), "ուրախացրի");
        assert_eq!(c.form(Tense::Aorist, P3, SG, POS), "ուրախացրեց");
        assert_eq!(c.passive(), "ուրախացվել");
        assert_eq!(c.form(Tense::SubjunctiveFuture, P1, SG, POS), "ուրախացնեմ");
        assert_eq!(c.converb(Converb::Perfective, POS), "ուրախացրել");
        assert_eq!(c.participle(Participle::Resultative, POS), "ուրախացրած");
        // The subject participle stays on the present stem.
        assert_eq!(c.participle(Participle::Subject, POS), "ուրախացնող");
        assert_eq!(c.imperative(SG, POS), "ուրախացրու");
        assert_eq!(c.imperative(PL, POS), "ուրախացրեք");
    }

    #[test]
    fn stored_irregulars() {
        // The -նել class drops its -ն- outside the present.
        let h = v("հագնել");
        assert_eq!(h.form(Tense::Aorist, P1, SG, POS), "հագա");
        assert_eq!(h.form(Tense::Aorist, P3, SG, POS), "հագավ");
        assert_eq!(h.converb(Converb::Perfective, POS), "հագել");
        assert_eq!(h.converb(Converb::Imperfective, POS), "հագնում");
        assert_eq!(h.participle(Participle::Subject, POS), "հագնող");
        assert_eq!(h.imperative(SG, POS), "հագիր");
        // Suppletive stems.
        let g = v("գալ");
        assert_eq!(g.form(Tense::Aorist, P1, SG, POS), "եկա");
        assert_eq!(g.form(Tense::Aorist, P3, SG, POS), "եկավ");
        assert_eq!(g.converb(Converb::Imperfective, POS), "գալիս");
        assert_eq!(g.converb(Converb::Perfective, POS), "եկել");
        assert_eq!(g.form(Tense::SubjunctiveFuture, P1, SG, POS), "գամ");
        assert_eq!(g.imperative(SG, POS), "արի");
        let t = v("տալ");
        assert_eq!(t.form(Tense::Aorist, P1, SG, POS), "տվեցի");
        assert_eq!(t.converb(Converb::Perfective, POS), "տվել");
        assert_eq!(t.imperative(SG, POS), "տուր");
        let u = v("ուտել");
        assert_eq!(u.form(Tense::Aorist, P1, SG, POS), "կերա");
        assert_eq!(u.converb(Converb::Perfective, POS), "կերել");
        assert_eq!(u.imperative(SG, POS), "կեր");
        let l = v("լինել");
        assert_eq!(l.form(Tense::Aorist, P3, SG, POS), "եղավ");
        assert_eq!(l.form(Tense::Present, P1, SG, POS), "լինում եմ");
        let a = v("անել");
        assert_eq!(a.form(Tense::Aorist, P1, SG, POS), "արեցի");
        assert_eq!(a.imperative(SG, POS), "արա");
    }

    #[test]
    fn rejects_non_verbs() {
        for input in ["տուն", "ել", "write", "գրել գիրք", ""] {
            assert_eq!(Verb::from_infinitive(input).err(), Some(Error::NotAVerb));
        }
    }

    #[test]
    fn table_rows_are_filled() {
        let t = Table::build(&v("խոսել"));
        assert_eq!(t.present[0], "խոսում եմ");
        assert_eq!(t.aorist[0], "խոսեցի");
        assert_eq!(t.conditional[2], "կխոսի");
        assert_eq!(t.imperative, ["խոսիր".to_string(), "խոսեք".to_string()]);
        assert_eq!(t.participle_subject, "խոսող");
    }
}
