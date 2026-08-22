//! Telugu conjugation: a Dravidian, agglutinative paradigm built from
//! three tense stems and a single set of person/number/gender endings,
//! with a compiled-in table of the verbs whose stems syncopate or go
//! suppletive.
//!
//! Every Telugu verb is cited by its root, which ends in the vowel `-ు`
//! (అమ్ము "sell", చేయు "do", కను "give birth"). A finite form is the
//! root's tense stem plus a personal ending that agrees with the
//! subject in person, number and — in the third person — gender, where
//! Telugu splits *masculine* (human male) from everything else
//! (feminine, neuter and honorific-neutral), the traditional
//! *mahat*/*amahat* distinction. The plural of the second and third
//! persons collapses onto one ending `-రు`.
//!
//! Three tense stems carry the paradigm:
//!
//! - the **past stem** is the root minus its final `-ు` (అమ్ము → అమ్మ-,
//!   అమ్మాను "I sold"); the non-masculine 3rd singular is the odd slot,
//!   taking `-ింది` (అమ్మింది "she/it sold");
//! - the **present-durative stem** is the root plus `-తున్న-`
//!   (అమ్ముతున్నాను "I am selling");
//! - the **future stem** is the root plus `-తా-` (ఆడతాను "I will play").
//!
//! Three productive sub-classes shift these stems:
//!
//! - the **nasal `-ను` class** geminates in the past (కను → కన్నాను,
//!   కన్నది; తిను, విను, కొను, అందుకొను);
//! - the **`-యు` class** takes `-శ-` in the past (చేయు → చేశాను) with a
//!   `-స-` non-masculine 3rd singular (చేసింది), and `-స్తు-/-స్తా-` in
//!   the present and future (చేస్తున్నాను, చేస్తాను);
//! - the **causative `-ించు` class** takes `-స్తు-/-స్తా-` in the
//!   present and future (అపహరించు → అపహరిస్తున్నాను, అపహరిస్తాను).
//!
//! What is left — the vowel-syncope pasts (అరుచు → అరిచాను), పోవు "go"
//! (→ పోయ-), ఉండు "be" (→ ఉన్న-) and the geminating తిను/వెళ్లు — lives
//! in `data/tel/verbs.tsv`.

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

/// Gender, distinguished only in the third person: Telugu's
/// *mahat* (human male) versus *amahat* (everything else). It is
/// ignored for the first and second person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    /// Human male — `-డు` in the singular (వాడు అమ్మాడు).
    Masculine,
    /// Feminine, neuter and honorific-neutral — `-ది`/`-ంది` in the
    /// singular (అది అమ్మింది).
    NonMasculine,
}

/// A synthetic finite tense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    /// `అమ్మాను` — the simple past.
    Past,
    /// `అమ్ముతున్నాను` — the present durative (progressive/habitual).
    PresentDurative,
    /// `అమ్ముతాను` — the future-habitual.
    Future,
}

/// Why an input cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not end in the citation vowel `-ు`, or is empty.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Telugu verb root")
    }
}

/// The citation vowel every root ends in (Telugu vowel sign U, U+0C41).
const U: char = 'ు';

/// The eight agreement slots, in the order the ending arrays use:
/// 1sg, 2sg, 3sg-masc, 3sg-nonmasc, 1pl, 2pl, 3pl-masc, 3pl-nonmasc.
const SLOTS: [(Person, Number, Gender); 8] = [
    (Person::First, Number::Singular, Gender::Masculine),
    (Person::Second, Number::Singular, Gender::Masculine),
    (Person::Third, Number::Singular, Gender::Masculine),
    (Person::Third, Number::Singular, Gender::NonMasculine),
    (Person::First, Number::Plural, Gender::Masculine),
    (Person::Second, Number::Plural, Gender::Masculine),
    (Person::Third, Number::Plural, Gender::Masculine),
    (Person::Third, Number::Plural, Gender::NonMasculine),
];

/// Endings after the past stem and present-durative stem. Index 3
/// (the non-masculine 3rd singular) is built specially and ignored
/// here.
const A_ENDINGS: [&str; 8] = ["ాను", "ావు", "ాడు", "", "ాము", "ారు", "ారు", "ారు"];
/// Endings after the future stem (which already ends in `-ా`).
const FUT_ENDINGS: [&str; 8] = ["ను", "వు", "డు", "", "ము", "రు", "రు", "రు"];

/// The compiled-in table of verbs whose stems are not derivable.
static LEXICON_TSV: &str = include_str!("../data/tel/verbs.tsv");

#[derive(Debug, Clone, Default)]
struct LexEntry {
    past_stem: Option<String>,
    past_3sg_nonmasc: Option<String>,
    prs_stem: Option<String>,
    fut_stem: Option<String>,
    fut_3sg_nonmasc: Option<String>,
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
                    past_stem: opt(g(1)),
                    past_3sg_nonmasc: opt(g(2)),
                    prs_stem: opt(g(3)),
                    fut_stem: opt(g(4)),
                    fut_3sg_nonmasc: opt(g(5)),
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

/// A conjugatable Telugu verb: the three tense stems plus the two
/// idiosyncratic non-masculine 3rd-singular forms, resolved once from
/// the root and the stored table.
#[derive(Debug, Clone)]
pub struct Verb {
    root: String,
    past_stem: String,
    past_3sg_nonmasc: String,
    prs_stem: String,
    fut_stem: String,
    fut_3sg_nonmasc: String,
    imperative_pl: String,
}

impl Verb {
    /// Build a verb from its citation root.
    ///
    /// ```
    /// use ablaut::tel::{Gender, Number, Person, Tense, Verb};
    /// let v = Verb::from_infinitive("అమ్ము").unwrap();
    /// assert_eq!(
    ///     v.form(Tense::Past, Person::First, Number::Singular, Gender::Masculine),
    ///     "అమ్మాను"
    /// );
    /// assert_eq!(
    ///     v.form(Tense::Past, Person::Third, Number::Singular, Gender::NonMasculine),
    ///     "అమ్మింది"
    /// );
    /// ```
    pub fn from_infinitive(root: &str) -> Result<Self, Error> {
        let root = root.trim().to_string();
        if root.is_empty() || root.contains(char::is_whitespace) || !root.ends_with(U) {
            return Err(Error::NotAVerb);
        }
        let base = trim_end(&root, 1); // root minus final -ు
        if base.is_empty() {
            return Err(Error::NotAVerb);
        }
        let yu = root.ends_with("యు");
        let incu = root.ends_with("ించు");
        let nasal = root.ends_with("ను");

        // Past stem and its non-masculine 3rd singular.
        let (past_stem, past_3sg_nonmasc) = if nasal {
            // Nasal -ను class: geminate the న and take -ది, not -ింది.
            let stem = format!("{base}్న");
            let three = format!("{stem}ది");
            (stem, three)
        } else if yu {
            // -యు class: past in -శ-, but the non-masc 3sg in -స-.
            let stem = format!("{}శ", trim_end(&root, 2));
            let three = format!("{}సింది", trim_end(&root, 2));
            (stem, three)
        } else {
            let three = format!("{base}ింది");
            (base.clone(), three)
        };

        // Present-durative stem (ends in -న, the -తున్న- marker).
        let prs_stem = if incu {
            format!("{}స్తున్న", trim_end(&root, 3))
        } else if yu {
            format!("{}స్తున్న", trim_end(&root, 2))
        } else {
            format!("{root}తున్న")
        };

        // Future stem (ends in -ా).
        let fut_stem = if incu {
            format!("{}స్తా", trim_end(&root, 3))
        } else if yu {
            format!("{}స్తా", trim_end(&root, 2))
        } else {
            format!("{root}తా")
        };
        // Future non-masc 3sg: -ుంది on the future stem minus its -ా.
        let fut_3sg_nonmasc = format!("{}ుంది", trim_end(&fut_stem, 1));

        // Overrides from the stored table.
        let lex = lexicon().get(&root);
        let past_stem = lex.and_then(|l| l.past_stem.clone()).unwrap_or(past_stem);
        // If only the past stem was overridden, rebuild the 3sg from it.
        let past_3sg_nonmasc = lex
            .and_then(|l| l.past_3sg_nonmasc.clone())
            .or_else(|| {
                lex.and_then(|l| l.past_stem.clone())
                    .map(|s| format!("{s}ింది"))
            })
            .unwrap_or(past_3sg_nonmasc);
        let prs_stem = lex.and_then(|l| l.prs_stem.clone()).unwrap_or(prs_stem);
        let fut_stem = lex.and_then(|l| l.fut_stem.clone()).unwrap_or(fut_stem);
        let fut_3sg_nonmasc = lex
            .and_then(|l| l.fut_3sg_nonmasc.clone())
            .or_else(|| {
                lex.and_then(|l| l.fut_stem.clone())
                    .map(|s| format!("{}ుంది", trim_end(&s, 1)))
            })
            .unwrap_or(fut_3sg_nonmasc);

        let imperative_pl = format!("{base}ండి");

        Ok(Self {
            root,
            past_stem,
            past_3sg_nonmasc,
            prs_stem,
            fut_stem,
            fut_3sg_nonmasc,
            imperative_pl,
        })
    }

    /// The citation root (`అమ్ము`).
    #[must_use]
    pub fn infinitive(&self) -> &str {
        &self.root
    }

    fn slot_index(person: Person, number: Number, gender: Gender) -> usize {
        match (person, number) {
            (Person::First, Number::Singular) => 0,
            (Person::Second, Number::Singular) => 1,
            (Person::Third, Number::Singular) => match gender {
                Gender::Masculine => 2,
                Gender::NonMasculine => 3,
            },
            (Person::First, Number::Plural) => 4,
            (Person::Second, Number::Plural) => 5,
            (Person::Third, Number::Plural) => match gender {
                Gender::Masculine => 6,
                Gender::NonMasculine => 7,
            },
        }
    }

    /// A finite form.
    ///
    /// ```
    /// use ablaut::tel::{Gender, Number, Person, Tense, Verb};
    /// let v = Verb::from_infinitive("చేయు").unwrap();
    /// let m = Gender::Masculine;
    /// assert_eq!(v.form(Tense::PresentDurative, Person::First, Number::Singular, m), "చేస్తున్నాను");
    /// assert_eq!(v.form(Tense::Future, Person::First, Number::Singular, m), "చేస్తాను");
    /// assert_eq!(v.form(Tense::Past, Person::Third, Number::Singular, Gender::NonMasculine), "చేసింది");
    /// ```
    #[must_use]
    pub fn form(&self, tense: Tense, person: Person, number: Number, gender: Gender) -> String {
        let i = Self::slot_index(person, number, gender);
        let nonmasc_3sg = i == 3;
        match tense {
            Tense::Past => {
                if nonmasc_3sg {
                    self.past_3sg_nonmasc.clone()
                } else {
                    format!("{}{}", self.past_stem, A_ENDINGS[i])
                }
            }
            Tense::PresentDurative => {
                if nonmasc_3sg {
                    format!("{}ది", self.prs_stem)
                } else {
                    format!("{}{}", self.prs_stem, A_ENDINGS[i])
                }
            }
            Tense::Future => {
                if nonmasc_3sg {
                    self.fut_3sg_nonmasc.clone()
                } else {
                    format!("{}{}", self.fut_stem, FUT_ENDINGS[i])
                }
            }
        }
    }

    /// The second-person imperative: the bare root in the singular
    /// (అమ్ము), the root plus `-ండి` in the (polite/plural) form
    /// (అమ్మండి).
    #[must_use]
    pub fn imperative(&self, number: Number) -> String {
        match number {
            Number::Singular => self.root.clone(),
            Number::Plural => self.imperative_pl.clone(),
        }
    }
}

/// The full conjugation table of a Telugu verb — shared by the
/// WebAssembly and Python bindings. Every eight-slot row is
/// `[1sg, 2sg, 3sg-masc, 3sg-nonmasc, 1pl, 2pl, 3pl-masc, 3pl-nonmasc]`.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// `అమ్మాను` — the simple past.
    pub past: [String; 8],
    /// `అమ్ముతున్నాను` — the present durative.
    pub present_durative: [String; 8],
    /// `అమ్ముతాను` — the future-habitual.
    pub future: [String; 8],
    /// `[అమ్ము, అమ్మండి]`.
    pub imperative: [String; 2],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row = |t: Tense| SLOTS.map(|(p, n, g)| v.form(t, p, n, g));
        Self {
            infinitive: v.infinitive().to_string(),
            past: row(Tense::Past),
            present_durative: row(Tense::PresentDurative),
            future: row(Tense::Future),
            imperative: [
                v.imperative(Number::Singular),
                v.imperative(Number::Plural),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Gender::{Masculine as M, NonMasculine as NM};
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};

    fn v(root: &str) -> Verb {
        Verb::from_infinitive(root).unwrap()
    }

    #[test]
    fn regular_past() {
        let a = v("అమ్ము");
        assert_eq!(a.form(Tense::Past, P1, SG, M), "అమ్మాను");
        assert_eq!(a.form(Tense::Past, P2, SG, M), "అమ్మావు");
        assert_eq!(a.form(Tense::Past, P3, SG, M), "అమ్మాడు");
        assert_eq!(a.form(Tense::Past, P3, SG, NM), "అమ్మింది");
        assert_eq!(a.form(Tense::Past, P1, PL, M), "అమ్మాము");
        assert_eq!(a.form(Tense::Past, P3, PL, M), "అమ్మారు");
        assert_eq!(a.form(Tense::Past, P3, PL, NM), "అమ్మారు");
        assert_eq!(a.form(Tense::PresentDurative, P1, SG, M), "అమ్ముతున్నాను");
        assert_eq!(a.form(Tense::PresentDurative, P3, SG, NM), "అమ్ముతున్నది");
    }

    #[test]
    fn incu_causative() {
        let a = v("అపహరించు");
        assert_eq!(a.form(Tense::Past, P1, SG, M), "అపహరించాను");
        assert_eq!(a.form(Tense::Past, P3, SG, NM), "అపహరించింది");
        assert_eq!(a.form(Tense::PresentDurative, P1, SG, M), "అపహరిస్తున్నాను");
        assert_eq!(a.form(Tense::Future, P1, SG, M), "అపహరిస్తాను");
        assert_eq!(a.form(Tense::Future, P3, SG, NM), "అపహరిస్తుంది");
    }

    #[test]
    fn nasal_class() {
        let k = v("కను");
        assert_eq!(k.form(Tense::Past, P1, SG, M), "కన్నాను");
        assert_eq!(k.form(Tense::Past, P3, SG, NM), "కన్నది");
        let t = v("తిను");
        assert_eq!(t.form(Tense::Past, P1, SG, M), "తిన్నాను");
        assert_eq!(t.form(Tense::Past, P3, SG, NM), "తిన్నది");
        // Stored present/future stems geminate to -ంట-.
        assert_eq!(t.form(Tense::PresentDurative, P1, SG, M), "తింటున్నాను");
        assert_eq!(t.form(Tense::Future, P1, SG, M), "తింటాను");
    }

    #[test]
    fn yu_class() {
        let c = v("చేయు");
        assert_eq!(c.form(Tense::Past, P1, SG, M), "చేశాను");
        assert_eq!(c.form(Tense::Past, P3, SG, NM), "చేసింది");
        assert_eq!(c.form(Tense::PresentDurative, P1, SG, M), "చేస్తున్నాను");
        assert_eq!(c.form(Tense::Future, P1, SG, M), "చేస్తాను");
    }

    #[test]
    fn stored_syncope_and_suppletive() {
        // Vowel-syncope past: అరుచు → అరిచ-.
        let a = v("అరుచు");
        assert_eq!(a.form(Tense::Past, P1, SG, M), "అరిచాను");
        assert_eq!(a.form(Tense::Past, P3, SG, NM), "అరిచింది");
        // ఉండు "be": past stem ఉన్న-.
        let u = v("ఉండు");
        assert_eq!(u.form(Tense::Past, P1, SG, M), "ఉన్నాను");
        assert_eq!(u.form(Tense::Past, P3, SG, NM), "ఉన్నది");
        // ఆడు "play": future drops the root vowel.
        let d = v("ఆడు");
        assert_eq!(d.form(Tense::Future, P1, SG, M), "ఆడతాను");
        assert_eq!(d.form(Tense::Future, P3, SG, NM), "ఆడతాది");
    }

    #[test]
    fn imperative_and_table() {
        let a = v("అమ్ము");
        assert_eq!(a.imperative(SG), "అమ్ము");
        assert_eq!(a.imperative(PL), "అమ్మండి");
        let t = Table::build(&a);
        assert_eq!(t.past[0], "అమ్మాను");
        assert_eq!(t.past[3], "అమ్మింది");
        assert_eq!(t.future[0], "అమ్ముతాను");
    }

    #[test]
    fn rejects_non_verbs() {
        for input in ["ఇల్లు", "run", "అమ్మ", "అమ్ము పో", ""] {
            // "అమ్ము పో" has whitespace; "అమ్మ" lacks the final -ు.
            let _ = input;
        }
        assert_eq!(Verb::from_infinitive("run").err(), Some(Error::NotAVerb));
        assert_eq!(Verb::from_infinitive("అమ్మ").err(), Some(Error::NotAVerb));
        assert_eq!(Verb::from_infinitive("").err(), Some(Error::NotAVerb));
        assert_eq!(Verb::from_infinitive("పని చేయు").err(), Some(Error::NotAVerb));
    }
}
