//! Persian (Farsi, `pes`) conjugation: the two-stem system — a regular
//! past stem and an irregular present stem — with regular personal
//! endings and a handful of prefixes and periphrases stacked on top.
//!
//! Every Persian verb is cited by its infinitive, which ends in `ـن`
//! (رفتن "to go", کردن "to do", دیدن "to see"). Two stems carry the
//! whole paradigm:
//!
//! - the **past stem** is regular: drop the infinitive's final `ن`
//!   (رفتن → رفت, کردن → کرد, دیدن → دید). It builds the preterite, the
//!   perfect and pluperfect (through the past participle), the future
//!   and the past participle itself;
//! - the **present stem** is the locus of irregularity (رفتن → رو,
//!   کردن → کن, گفتن → گوی, دیدن → بین). It builds the aorist, the
//!   `می`-present, the `بـ`-subjunctive, the imperative and the present
//!   participle. It is *not* derivable from the infinitive for most
//!   verbs, so it is stored per lemma in `data/pes/verbs.tsv` — that
//!   file **is** the present-stem list. The one productive class is the
//!   `ـیدن` verbs (فهمیدن → فهم, رسیدن → رس), whose present stem is the
//!   infinitive minus `ـیدن`; they need no entry.
//!
//! The **personal endings** are regular across tenses. The present set
//! `[م, ی, د, یم, ید, ند]` rides the present stem (کنم … کنند); the past
//! set `[م, ی, ∅, یم, ید, ند]` rides the past stem, with a bare third
//! singular (کردم, کرد, کردند).
//!
//! Three **prefixes** and a set of **periphrases** complete the system:
//!
//! - `می` (imperfective) fronts the present indicative (می‌کنم) and the
//!   imperfect (می‌کردم);
//! - `بـ` (subjunctive/imperative) fronts the subjunctive (بکنم) and the
//!   imperative (بکن); before a vowel-initial stem it surfaces as `بیا`
//!   (آمدن → بیایم) or `بی` (افتادن → بیفتم);
//! - `نـ`/`نمی` negates (نکردم, نمی‌کنم, نکن);
//! - the **perfect** is the past participle plus the clitic copula
//!   (کرده‌ام … کرده است); the **pluperfect** adds بودن (کرده بودم); the
//!   **future** is the conjugated خواستن auxiliary before the bare past
//!   stem (خواهم کرد); the **progressives** put داشتن in front of the
//!   `می`-form (دارم می‌کنم, داشتم می‌کردم).
//!
//! Persian writes compounds as separate words — a preverb (بر گشتن) or a
//! nominal light-verb pair (بحث کردن, سفارش دادن). Only the last word
//! conjugates; the lead material is carried along unchanged and reprinted
//! in front of every form (بر می‌گردم, بحث می‌کنم). The `بـ` of the
//! subjunctive/imperative drops on the two prototypical light verbs
//! کردن/شدن (بحث کنم), and is kept elsewhere (سفارش بدهم, بر بگردم).
//!
//! All forms are stored and produced in the normalized Perso-Arabic
//! orthography of [`crate::perso_arabic`] (no ZWNJ, Arabic letters
//! folded to their Persian shapes, short vowels stripped).

use crate::perso_arabic;
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

/// Affirmative or negative. Persian negates synthetic forms with `نـ`
/// (`نمی` before the imperfective `می`), which also displaces the
/// subjunctive/imperative `بـ`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Polarity {
    Positive,
    Negative,
}

/// A finite tense/mood. The synthetic core plus the periphrastic layer
/// (perfect, pluperfect, future, progressive), all built off the two
/// stems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    /// `کنم` — bare present stem + present endings (the subjunctive
    /// without its prefix; Wiktionary's "aorist").
    Aorist,
    /// `می‌کنم` — the imperfective present indicative.
    Present,
    /// `بکنم` — the present subjunctive.
    Subjunctive,
    /// `کردم` — the simple past (preterite).
    Past,
    /// `می‌کردم` — the past imperfective.
    Imperfect,
    /// `کرده‌ام` — the present perfect.
    Perfect,
    /// `کرده بودم` — the pluperfect.
    Pluperfect,
    /// `خواهم کرد` — the future.
    Future,
    /// `کرده باشم` — the perfect (past) subjunctive.
    PerfectSubjunctive,
    /// `دارم می‌کنم` — the present progressive.
    PresentProgressive,
    /// `داشتم می‌کردم` — the past progressive.
    PastProgressive,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input's last word does not end in `ن`.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Persian infinitive")
    }
}

/// The present endings `[م, ی, د, یم, ید, ند]`.
const PRESENT_ENDINGS: [&str; 6] = ["م", "ی", "د", "یم", "ید", "ند"];
/// The past endings `[م, ی, ∅, یم, ید, ند]`.
const PAST_ENDINGS: [&str; 6] = ["م", "ی", "", "یم", "ید", "ند"];
/// The clitic copula of the perfect `[ام, ای, ‹ است›, ایم, اید, اند]`.
/// The third singular is a free word (کرده است), the rest are clitics.
const PERFECT_ENDINGS: [&str; 6] = ["ام", "ای", " است", "ایم", "اید", "اند"];

const MI: &str = "می";
const NA: &str = "ن";
const BE: &str = "ب";
/// The future auxiliary stem (خواستن's present stem): خواهم … خواهند.
const FUTURE_AUX: &str = "خواه";

/// Preverbs that keep the subjunctive/imperative `بـ` (بر بگردم). A
/// single leading token from this set marks a preverb compound; any
/// other leading material is a nominal light-verb compound.
const PREVERBS: [&str; 6] = ["بر", "باز", "فرو", "در", "وا", "ور"];
/// The light verbs whose `بـ` drops in a compound (بحث کنم, آب شدم).
const DROP_BE_LIGHT: [&str; 2] = ["کردن", "شدن"];

/// The compiled-in present-stem list.
static LEXICON_TSV: &str = include_str!("../data/pes/verbs.tsv");

/// A stored lexical entry: the bound present stem, and an optional
/// imperative override for the verbs whose imperative is not `بـ` + stem
/// (گفتن → بگو, آمدن → بیا).
#[derive(Debug, Clone)]
struct LexEntry {
    present_stem: String,
    imperative_sg: Option<String>,
}

fn lexicon() -> &'static HashMap<String, LexEntry> {
    static MAP: OnceLock<HashMap<String, LexEntry>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in LEXICON_TSV.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            let imperative_sg = c
                .get(2)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty() && *s != "-")
                .map(str::to_string);
            m.insert(
                c[0].trim().to_string(),
                LexEntry {
                    present_stem: c[1].trim().to_string(),
                    imperative_sg,
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

/// A conjugatable Persian verb: its two stems and its compound frame,
/// resolved once from the infinitive and the present-stem table.
#[derive(Debug, Clone)]
pub struct Verb {
    /// The full normalized infinitive (سفارش دادن).
    infinitive: String,
    /// Invariable lead material reprinted before every form ("سفارش ",
    /// "بر "), empty for a simple verb.
    prefix: String,
    /// The conjugating word's past stem (کرد, داد).
    past_stem: String,
    /// The conjugating word's bound present stem (کن, ده, گوی).
    present_stem: String,
    /// An imperative-singular override (بگو, بیا), else `None`.
    imperative_sg: Option<String>,
    /// Whether the subjunctive/imperative `بـ` drops (کردن/شدن compound).
    drop_be: bool,
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

/// Apply the subjunctive/imperative `بـ`, with the vowel-initial
/// allomorphs `بیا` (before آ) and `بی` (before another alef).
fn be_prefix(stem: &str) -> String {
    if let Some(rest) = stem.strip_prefix('آ') {
        format!("بیا{rest}")
    } else if let Some(rest) = stem.strip_prefix('ا') {
        format!("بی{rest}")
    } else {
        format!("{BE}{stem}")
    }
}

impl Verb {
    /// Build a verb from its infinitive.
    ///
    /// ```
    /// use ablaut::pes::{Number, Person, Polarity, Tense, Verb};
    /// let v = Verb::from_infinitive("کردن").unwrap();
    /// let p = (Person::First, Number::Singular, Polarity::Positive);
    /// assert_eq!(v.form(Tense::Present, p.0, p.1, p.2), "میکنم");
    /// assert_eq!(v.form(Tense::Past, p.0, p.1, p.2), "کردم");
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::NotAVerb`] when the last word does not end in `ن`.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = perso_arabic::normalize(infinitive.trim());
        if inf.is_empty() {
            return Err(Error::NotAVerb);
        }
        // A compound conjugates only its last word; the rest leads.
        let (prefix, verb) = match inf.rfind(' ') {
            Some(i) => (inf[..=i].to_string(), inf[i + 1..].to_string()),
            None => (String::new(), inf.clone()),
        };
        // The shortest real infinitives are three letters (شدن, بردن).
        if !verb.ends_with('ن') || verb.chars().count() < 3 {
            return Err(Error::NotAVerb);
        }
        let past_stem = trim_end(&verb, 1);

        let lex = lexicon().get(&verb);
        let present_stem = if let Some(l) = lex {
            l.present_stem.clone()
        } else if verb.ends_with("یدن") {
            // The productive `ـیدن` class: present stem = infinitive − یدن.
            trim_end(&verb, 3)
        } else {
            // No stored stem and not a `ـیدن` verb: fall back to the past
            // stem. Present-based forms may then be wrong, but the verb
            // is still conjugable and past-based forms are correct.
            past_stem.clone()
        };

        let pre_tokens: Vec<&str> = prefix.split_whitespace().collect();
        let is_preverb = pre_tokens.len() == 1 && PREVERBS.contains(&pre_tokens[0]);
        let drop_be = !prefix.is_empty() && !is_preverb && DROP_BE_LIGHT.contains(&verb.as_str());

        Ok(Self {
            infinitive: inf,
            prefix,
            past_stem,
            present_stem,
            imperative_sg: lex.and_then(|l| l.imperative_sg.clone()),
            drop_be,
        })
    }

    /// The normalized infinitive.
    #[must_use]
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// Reprint the compound lead in front of a conjugated core.
    fn out(&self, core: String) -> String {
        format!("{}{core}", self.prefix)
    }

    /// The past participle (کرده), base of the perfect tenses.
    #[must_use]
    pub fn past_participle(&self) -> String {
        self.out(format!("{}ه", self.past_stem))
    }

    /// The present (agent) participle (کننده, گوینده).
    #[must_use]
    pub fn present_participle(&self) -> String {
        self.out(format!("{}نده", self.present_stem))
    }

    /// The bare past participle without the compound lead — the shared
    /// base of the perfect periphrases.
    fn bare_participle(&self) -> String {
        format!("{}ه", self.past_stem)
    }

    /// The 2nd-person imperative (بکن, بکنید); negated it is the
    /// prohibitive (نکن, نکنید).
    #[must_use]
    pub fn imperative(&self, number: Number, polarity: Polarity) -> String {
        let base = match (&self.imperative_sg, polarity) {
            // A stored imperative already includes its بـ; under negation
            // its prefix is replaced by نـ.
            (Some(imp), Polarity::Positive) => imp.clone(),
            (Some(imp), Polarity::Negative) => {
                format!("{NA}{}", imp.strip_prefix(BE).unwrap_or(imp))
            }
            (None, Polarity::Positive) if self.drop_be => self.present_stem.clone(),
            (None, Polarity::Positive) => be_prefix(&self.present_stem),
            (None, Polarity::Negative) => format!("{NA}{}", self.present_stem),
        };
        let core = match number {
            Number::Singular => base,
            Number::Plural => format!("{base}{}", PRESENT_ENDINGS[4]), // +ید
        };
        self.out(core)
    }

    /// A finite form.
    ///
    /// ```
    /// use ablaut::pes::{Number, Person, Polarity, Tense, Verb};
    /// let v = Verb::from_infinitive("دیدن").unwrap();
    /// let (p, n, pol) = (Person::Third, Number::Singular, Polarity::Positive);
    /// assert_eq!(v.form(Tense::Present, p, n, pol), "میبیند");
    /// assert_eq!(v.form(Tense::Subjunctive, p, n, pol), "ببیند");
    /// assert_eq!(v.form(Tense::Future, p, n, pol), "خواهد دید");
    /// ```
    #[must_use]
    pub fn form(&self, tense: Tense, person: Person, number: Number, polarity: Polarity) -> String {
        let i = person.index(number);
        let neg = polarity == Polarity::Negative;
        let ps = &self.present_stem;
        let past = &self.past_stem;
        let core = match tense {
            Tense::Aorist => {
                let pre = if neg { NA } else { "" };
                format!("{pre}{ps}{}", PRESENT_ENDINGS[i])
            }
            Tense::Present => {
                let pre = if neg {
                    format!("{NA}{MI}")
                } else {
                    MI.to_string()
                };
                format!("{pre}{ps}{}", PRESENT_ENDINGS[i])
            }
            Tense::Subjunctive => {
                if neg {
                    format!("{NA}{ps}{}", PRESENT_ENDINGS[i])
                } else if self.drop_be {
                    format!("{ps}{}", PRESENT_ENDINGS[i])
                } else {
                    format!("{}{}", be_prefix(ps), PRESENT_ENDINGS[i])
                }
            }
            Tense::Past => {
                let pre = if neg { NA } else { "" };
                format!("{pre}{past}{}", PAST_ENDINGS[i])
            }
            Tense::Imperfect => {
                let pre = if neg {
                    format!("{NA}{MI}")
                } else {
                    MI.to_string()
                };
                format!("{pre}{past}{}", PAST_ENDINGS[i])
            }
            Tense::Perfect => {
                let pre = if neg { NA } else { "" };
                format!("{pre}{}{}", self.bare_participle(), PERFECT_ENDINGS[i])
            }
            Tense::Pluperfect => {
                let pre = if neg { NA } else { "" };
                format!(
                    "{}{pre}بود{}",
                    pp_space(&self.bare_participle()),
                    PAST_ENDINGS[i]
                )
            }
            Tense::Future => {
                let pre = if neg { NA } else { "" };
                format!("{pre}{FUTURE_AUX}{} {past}", PRESENT_ENDINGS[i])
            }
            Tense::PerfectSubjunctive => {
                let pre = if neg { NA } else { "" };
                format!("{} {pre}باش{}", self.bare_participle(), PRESENT_ENDINGS[i])
            }
            Tense::PresentProgressive => {
                // داشتن (past-stem داشت, present-stem دار) + the می-present.
                let aux = format!("دار{}", PRESENT_ENDINGS[i]);
                let main = format!("{MI}{ps}{}", PRESENT_ENDINGS[i]);
                format!("{aux} {main}")
            }
            Tense::PastProgressive => {
                let aux = format!("داشت{}", PAST_ENDINGS[i]);
                let main = format!("{MI}{past}{}", PAST_ENDINGS[i]);
                format!("{aux} {main}")
            }
        };
        self.out(core)
    }
}

/// Glue the past participle to a following auxiliary with a space, since
/// the pluperfect's بودن is a free word (کرده بودم).
fn pp_space(participle: &str) -> String {
    format!("{participle} ")
}

/// The conjugation table of a Persian verb — shared by the WebAssembly
/// and Python bindings. Person rows are `[1sg, 2sg, 3sg, 1pl, 2pl, 3pl]`
/// and affirmative; negatives are a method away on [`Verb`].
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// `کنم` — the aorist (bare subjunctive).
    pub aorist: [String; 6],
    /// `می‌کنم` — the present indicative.
    pub present: [String; 6],
    /// `بکنم` — the present subjunctive.
    pub subjunctive: [String; 6],
    /// `کردم` — the simple past.
    pub past: [String; 6],
    /// `می‌کردم` — the past imperfective.
    pub imperfect: [String; 6],
    /// `کرده‌ام` — the present perfect.
    pub perfect: [String; 6],
    /// `کرده بودم` — the pluperfect.
    pub pluperfect: [String; 6],
    /// `خواهم کرد` — the future.
    pub future: [String; 6],
    /// `کرده باشم` — the perfect subjunctive.
    pub perfect_subjunctive: [String; 6],
    /// `دارم می‌کنم` — the present progressive.
    pub present_progressive: [String; 6],
    /// `داشتم می‌کردم` — the past progressive.
    pub past_progressive: [String; 6],
    /// `[بکن, بکنید]` — the imperative.
    pub imperative: [String; 2],
    /// `کرده` — the past participle.
    pub past_participle: String,
    /// `کننده` — the present (agent) participle.
    pub present_participle: String,
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
            infinitive: v.infinitive().to_string(),
            aorist: row(Tense::Aorist),
            present: row(Tense::Present),
            subjunctive: row(Tense::Subjunctive),
            past: row(Tense::Past),
            imperfect: row(Tense::Imperfect),
            perfect: row(Tense::Perfect),
            pluperfect: row(Tense::Pluperfect),
            future: row(Tense::Future),
            perfect_subjunctive: row(Tense::PerfectSubjunctive),
            present_progressive: row(Tense::PresentProgressive),
            past_progressive: row(Tense::PastProgressive),
            imperative: [
                v.imperative(Number::Singular, Polarity::Positive),
                v.imperative(Number::Plural, Polarity::Positive),
            ],
            past_participle: v.past_participle(),
            present_participle: v.present_participle(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Third as P3};
    use Polarity::{Negative as NEG, Positive as POS};

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn kardan_full_paradigm() {
        let k = v("کردن");
        assert_eq!(k.form(Tense::Aorist, P1, SG, POS), "کنم");
        assert_eq!(k.form(Tense::Aorist, P3, SG, POS), "کند");
        assert_eq!(k.form(Tense::Present, P1, SG, POS), "میکنم");
        assert_eq!(k.form(Tense::Present, P3, PL, POS), "میکنند");
        assert_eq!(k.form(Tense::Subjunctive, P1, SG, POS), "بکنم");
        assert_eq!(k.form(Tense::Past, P1, SG, POS), "کردم");
        assert_eq!(k.form(Tense::Past, P3, SG, POS), "کرد");
        assert_eq!(k.form(Tense::Imperfect, P1, SG, POS), "میکردم");
        assert_eq!(k.form(Tense::Perfect, P1, SG, POS), "کردهام");
        assert_eq!(k.form(Tense::Perfect, P3, SG, POS), "کرده است");
        assert_eq!(k.form(Tense::Pluperfect, P1, SG, POS), "کرده بودم");
        assert_eq!(k.form(Tense::Future, P1, SG, POS), "خواهم کرد");
        assert_eq!(k.form(Tense::Future, P3, PL, POS), "خواهند کرد");
        assert_eq!(k.form(Tense::PerfectSubjunctive, P1, SG, POS), "کرده باشم");
        assert_eq!(k.form(Tense::PresentProgressive, P1, SG, POS), "دارم میکنم");
        assert_eq!(k.form(Tense::PastProgressive, P3, SG, POS), "داشت میکرد");
        assert_eq!(k.imperative(SG, POS), "بکن");
        assert_eq!(k.imperative(PL, POS), "بکنید");
        assert_eq!(k.past_participle(), "کرده");
        assert_eq!(k.present_participle(), "کننده");
    }

    #[test]
    fn irregular_present_stems() {
        assert_eq!(v("رفتن").form(Tense::Present, P1, SG, POS), "میروم");
        assert_eq!(v("رفتن").form(Tense::Aorist, P3, SG, POS), "رود");
        assert_eq!(v("رفتن").imperative(SG, POS), "برو");
        assert_eq!(v("گفتن").form(Tense::Present, P1, SG, POS), "میگویم");
        assert_eq!(v("گفتن").form(Tense::Aorist, P3, SG, POS), "گوید");
        assert_eq!(v("گفتن").imperative(SG, POS), "بگو");
        assert_eq!(v("دیدن").form(Tense::Aorist, P1, SG, POS), "بینم");
        assert_eq!(v("دیدن").imperative(SG, POS), "ببین");
        assert_eq!(v("شدن").form(Tense::Aorist, P3, SG, POS), "شود");
        assert_eq!(v("آمدن").form(Tense::Subjunctive, P1, SG, POS), "بیایم");
        assert_eq!(v("آمدن").imperative(SG, POS), "بیا");
    }

    #[test]
    fn productive_yidan_class() {
        // Present stem = infinitive − یدن, no lexicon entry needed.
        let f = v("فهمیدن");
        assert_eq!(f.form(Tense::Present, P1, SG, POS), "میفهمم");
        assert_eq!(f.form(Tense::Past, P1, SG, POS), "فهمیدم");
        assert_eq!(f.imperative(SG, POS), "بفهم");
        assert_eq!(v("رسیدن").form(Tense::Aorist, P3, SG, POS), "رسد");
    }

    #[test]
    fn compounds() {
        // Light-verb کردن compound: بـ drops in the subjunctive.
        let b = v("بحث کردن");
        assert_eq!(b.infinitive(), "بحث کردن");
        assert_eq!(b.form(Tense::Present, P1, SG, POS), "بحث میکنم");
        assert_eq!(b.form(Tense::Subjunctive, P1, SG, POS), "بحث کنم");
        assert_eq!(b.form(Tense::Future, P1, SG, POS), "بحث خواهم کرد");
        // دادن compound keeps the بـ (and reuses دادن's stem ده).
        let s = v("سفارش دادن");
        assert_eq!(s.form(Tense::Present, P1, SG, POS), "سفارش میدهم");
        assert_eq!(s.form(Tense::Subjunctive, P1, SG, POS), "سفارش بدهم");
        // Preverb keeps the بـ, and the preverb leads می/بـ.
        let g = v("بر گشتن");
        assert_eq!(g.form(Tense::Present, P1, SG, POS), "بر میگردم");
        assert_eq!(g.form(Tense::Subjunctive, P1, SG, POS), "بر بگردم");
    }

    #[test]
    fn negation() {
        let k = v("کردن");
        assert_eq!(k.form(Tense::Present, P1, SG, NEG), "نمیکنم");
        assert_eq!(k.form(Tense::Past, P1, SG, NEG), "نکردم");
        assert_eq!(k.form(Tense::Subjunctive, P1, SG, NEG), "نکنم");
        assert_eq!(k.form(Tense::Perfect, P1, SG, NEG), "نکردهام");
        assert_eq!(k.imperative(SG, NEG), "نکن");
    }

    #[test]
    fn normalizes_input() {
        // Arabic kaf/yeh and a trailing ZWNJ are folded away.
        let k = Verb::from_infinitive("كردن").unwrap();
        assert_eq!(k.infinitive(), "کردن");
        assert_eq!(k.form(Tense::Past, P1, SG, POS), "کردم");
    }

    #[test]
    fn rejects_non_verbs() {
        for input in ["", "کتاب", "run", "کن"] {
            assert!(
                Verb::from_infinitive(input).is_err() || !input.ends_with('ن'),
                "{input}"
            );
        }
        assert_eq!(Verb::from_infinitive("").err(), Some(Error::NotAVerb));
        assert_eq!(Verb::from_infinitive("کتاب").err(), Some(Error::NotAVerb));
    }
}
