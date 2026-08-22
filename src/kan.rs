//! Kannada conjugation: a Dravidian, agglutinative paradigm built from
//! three tense stems and one set of person/number/gender endings, with
//! a compiled-in table of the verbs whose past stem is not derivable
//! from the root.
//!
//! Every Kannada verb is cited by its root. Most roots end in the vowel
//! `-ు` (ಮಾಡು "do", ಓಡು "run", ಬಯಸು "wish"); a minority end in another
//! vowel (ನಡೆ "walk", ಕುಡಿ "drink", ಮೀ "bathe"). A finite form is the
//! root's tense stem plus a personal ending that agrees with the
//! subject in person, number and — in the third person — a three-way
//! gender: masculine (human male), feminine (human female) and neuter
//! (everything else). The second- and third-person plural human
//! genders share one ending, so `3;PL;MASC` and `3;PL;FEM` coincide.
//!
//! Three tense stems carry the paradigm. For a `-ు` root, with `base`
//! the root minus its final `-ు`:
//!
//! - the **past stem** is `base + -ಇದ-` (ಮಾಡು → ಮಾಡಿದ-, ಮಾಡಿದೆನು "I
//!   did"); its neuter 3rd singular is the odd slot, `base + -ಇತು`
//!   (ಮಾಡಿತು "it did");
//! - the **present stem** is `base + -ುತ್ತ-` (ಮಾಡುತ್ತ-, ಮಾಡುತ್ತೇನೆ "I
//!   do");
//! - the **future stem** is `base + -ುವ-` (ಮಾಡುವ-, ಮಾಡುವೆನು "I will
//!   do").
//!
//! A root ending in another vowel takes a glide `-ಯ-` before the
//! present and future markers (ನಡೆ → ನಡೆಯುತ್ತ-, ನಡೆಯುವ-) and forms its
//! past on `root + -ದ-` (ನಡೆದ-), with the neuter 3rd singular
//! `root + -ಯಿತು` (ನಡೆಯಿತು).
//!
//! What is left — the verbs whose past stem is suppletive or contracted
//! (ಆಗು → ಆದ-, ಹೋಗು → ಹೋದ-, ಕೊಡು → ಕೊಟ್ಟ-, ಕೊಲ್ಲು → ಕೊಂದ-, ಗೆಲ್ಲು →
//! ಗೆದ್ದ-, ಹೊರಡು → ಹೊರಟ್ಟ-, ಮೀ → ಮಿಂದ-) — lives in
//! `data/kan/verbs.tsv`.
//!
//! Endings begin with a dependent vowel sign (a matra) or a consonant
//! and attach to a stem ending in a consonant with its inherent `-ಅ`;
//! the join is ordinary Kannada orthographic composition, which is
//! plain string concatenation (ಮಾಡಿದ + ೆನು → ಮಾಡಿದೆನು). The negative,
//! contingent (dubitative) and analytic (perfect/progressive) forms are
//! outside this synthetic core, as they are for the oracles.

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

/// Gender, distinguished only in the third person: masculine (human
/// male), feminine (human female) and neuter (everything else). Ignored
/// for the first and second person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// A synthetic finite tense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    /// `ಮಾಡಿದೆನು` — the simple past.
    Past,
    /// `ಮಾಡುತ್ತೇನೆ` — the (habitual/progressive) present.
    Present,
    /// `ಮಾಡುವೆನು` — the future.
    Future,
}

/// Why an input cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input is empty, is not a single word, or does not end in a
    /// vowel (Kannada verb roots are vowel-final).
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Kannada verb root")
    }
}

/// The citation vowel most roots end in (Kannada vowel sign U, U+0CC1).
const U: char = 'ು';

/// The ten agreement slots, in the order the ending arrays use:
/// 1sg, 2sg, 3sg-masc, 3sg-fem, 3sg-neut, 1pl, 2pl, 3pl-masc, 3pl-fem,
/// 3pl-neut.
const SLOTS: [(Person, Number, Gender); 10] = [
    (Person::First, Number::Singular, Gender::Neuter),
    (Person::Second, Number::Singular, Gender::Neuter),
    (Person::Third, Number::Singular, Gender::Masculine),
    (Person::Third, Number::Singular, Gender::Feminine),
    (Person::Third, Number::Singular, Gender::Neuter),
    (Person::First, Number::Plural, Gender::Neuter),
    (Person::Second, Number::Plural, Gender::Neuter),
    (Person::Third, Number::Plural, Gender::Masculine),
    (Person::Third, Number::Plural, Gender::Feminine),
    (Person::Third, Number::Plural, Gender::Neuter),
];

/// Endings on the past stem. Index 4 (the neuter 3rd singular) is the
/// stored/derived `-ಇತు` form, built specially and ignored here.
const PAST_ENDINGS: [&str; 10] = ["ೆನು", "ೆ", "ನು", "ಳು", "", "ೆವು", "ಿರಿ", "ರು", "ರు", "ುವು"];
/// Endings on the present stem (which ends in `-ತ್ತ`).
const PRES_ENDINGS: [&str; 10] = ["ೇನೆ", "ೀ", "ಾನೆ", "ಾಳೆ", "ದೆ", "ೇವೆ", "ೀರಿ", "ಾರೆ", "ಾರೆ", "ವೆ"];
/// Endings on the future stem (which ends in `-ವ`).
const FUT_ENDINGS: [&str; 10] = ["ೆನು", "ಿ", "ನು", "ಳು", "ುದು", "ೆವು", "ಿರಿ", "ರು", "ರು", "ುವು"];

/// The compiled-in table of verbs whose past stem is not derivable.
static LEXICON_TSV: &str = include_str!("../data/kan/verbs.tsv");

#[derive(Debug, Clone, Default)]
struct LexEntry {
    past_stem: Option<String>,
    past_3sg_neut: Option<String>,
    prs_stem: Option<String>,
    fut_stem: Option<String>,
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
                    past_3sg_neut: opt(g(2)),
                    prs_stem: opt(g(3)),
                    fut_stem: opt(g(4)),
                },
            );
        }
        m
    })
}

/// Drop the last character of a string.
fn trim_last(s: &str) -> String {
    let mut chars = s.chars();
    chars.next_back();
    chars.as_str().to_string()
}

/// A conjugatable Kannada verb: the three tense stems plus the
/// idiosyncratic neuter 3rd-singular past, resolved once from the root
/// and the stored table.
#[derive(Debug, Clone)]
pub struct Verb {
    root: String,
    past_stem: String,
    past_3sg_neut: String,
    prs_stem: String,
    fut_stem: String,
    imperative_pl: String,
}

impl Verb {
    /// Build a verb from its citation root.
    ///
    /// ```
    /// use ablaut::kan::{Gender, Number, Person, Tense, Verb};
    /// let v = Verb::from_infinitive("ಮಾಡು").unwrap();
    /// assert_eq!(
    ///     v.form(Tense::Past, Person::First, Number::Singular, Gender::Neuter),
    ///     "ಮಾಡಿದೆನು"
    /// );
    /// assert_eq!(
    ///     v.form(Tense::Present, Person::Third, Number::Singular, Gender::Masculine),
    ///     "ಮಾಡುತ್ತಾನೆ"
    /// );
    /// ```
    pub fn from_infinitive(root: &str) -> Result<Self, Error> {
        let root = root.trim().to_string();
        if root.is_empty() || root.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        let u_root = root.ends_with(U);
        // Rule defaults: a -ు root drops its vowel and takes the -ಇದ-/
        // -ುತ್ತ-/-ುವ- markers; any other vowel-final root keeps the
        // vowel and inserts the glide -ಯ- before the present/future.
        let (past_stem, past_3sg_neut, prs_stem, fut_stem) = if u_root {
            let base = trim_last(&root);
            if base.is_empty() {
                return Err(Error::NotAVerb);
            }
            (
                format!("{base}ಿದ"),
                format!("{base}ಿತು"),
                format!("{base}ುತ್ತ"),
                format!("{base}ುವ"),
            )
        } else if root.chars().next_back().is_some_and(is_vowel_sign) {
            (
                format!("{root}ದ"),
                format!("{root}ಯಿತು"),
                format!("{root}ಯುತ್ತ"),
                format!("{root}ಯುವ"),
            )
        } else {
            // Roots must be vowel-final (bare consonants, Latin, etc.
            // are not Kannada verb roots).
            return Err(Error::NotAVerb);
        };

        // Overrides from the stored table.
        let lex = lexicon().get(&root);
        let past_stem = lex.and_then(|l| l.past_stem.clone()).unwrap_or(past_stem);
        // If only the past stem was overridden, rebuild the neuter 3sg
        // from it (ಕೊಟ್ಟ → ಕೊಟ್ಟಿತು); an explicit column wins (ಆಗು → ಆಯಿತು).
        let past_3sg_neut = lex
            .and_then(|l| l.past_3sg_neut.clone())
            .or_else(|| {
                lex.and_then(|l| l.past_stem.clone())
                    .map(|s| format!("{s}ಿತು"))
            })
            .unwrap_or(past_3sg_neut);
        let prs_stem = lex.and_then(|l| l.prs_stem.clone()).unwrap_or(prs_stem);
        let fut_stem = lex.and_then(|l| l.fut_stem.clone()).unwrap_or(fut_stem);

        // Plural imperative: -ಇರಿ on a -ు root's base (ಮಾಡಿರಿ), -ಯಿರಿ on
        // a vowel-final root (ನಡೆಯಿರಿ).
        let imperative_pl = if u_root {
            format!("{}ಿರಿ", trim_last(&root))
        } else {
            format!("{root}ಯಿರಿ")
        };

        Ok(Self {
            root,
            past_stem,
            past_3sg_neut,
            prs_stem,
            fut_stem,
            imperative_pl,
        })
    }

    /// The citation root (`ಮಾಡು`).
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
                Gender::Feminine => 3,
                Gender::Neuter => 4,
            },
            (Person::First, Number::Plural) => 5,
            (Person::Second, Number::Plural) => 6,
            (Person::Third, Number::Plural) => match gender {
                Gender::Masculine => 7,
                Gender::Feminine => 8,
                Gender::Neuter => 9,
            },
        }
    }

    /// A finite form.
    ///
    /// ```
    /// use ablaut::kan::{Gender, Number, Person, Tense, Verb};
    /// let v = Verb::from_infinitive("ಆಗು").unwrap();
    /// let (sg, m, n) = (Number::Singular, Gender::Masculine, Gender::Neuter);
    /// assert_eq!(v.form(Tense::Past, Person::Third, sg, m), "ಆದನು");
    /// assert_eq!(v.form(Tense::Past, Person::Third, sg, n), "ಆಯಿತು");
    /// assert_eq!(v.form(Tense::Future, Person::First, sg, n), "ಆಗುವೆನು");
    /// ```
    #[must_use]
    pub fn form(&self, tense: Tense, person: Person, number: Number, gender: Gender) -> String {
        let i = Self::slot_index(person, number, gender);
        match tense {
            Tense::Past => {
                if i == 4 {
                    self.past_3sg_neut.clone()
                } else {
                    format!("{}{}", self.past_stem, PAST_ENDINGS[i])
                }
            }
            Tense::Present => format!("{}{}", self.prs_stem, PRES_ENDINGS[i]),
            Tense::Future => format!("{}{}", self.fut_stem, FUT_ENDINGS[i]),
        }
    }

    /// The second-person imperative: the bare root in the singular
    /// (ಮಾಡು), the root plus `-ಇರಿ`/`-ಯಿರಿ` in the (polite/plural) form
    /// (ಮಾಡಿರಿ, ನಡೆಯಿರಿ).
    #[must_use]
    pub fn imperative(&self, number: Number) -> String {
        match number {
            Number::Singular => self.root.clone(),
            Number::Plural => self.imperative_pl.clone(),
        }
    }
}

/// Whether a char is a Kannada dependent vowel sign (matra),
/// U+0CBE..U+0CCC — the shape a non-`ు` vowel-final root ends in.
fn is_vowel_sign(c: char) -> bool {
    ('\u{0CBE}'..='\u{0CCC}').contains(&c)
}

/// The full finite conjugation of a Kannada verb — shared by the
/// WebAssembly and Python bindings. Every ten-slot row is
/// `[1sg, 2sg, 3sg-masc, 3sg-fem, 3sg-neut, 1pl, 2pl, 3pl-masc,
/// 3pl-fem, 3pl-neut]`.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// `ಮಾಡಿದೆನು` — the simple past.
    pub past: [String; 10],
    /// `ಮಾಡುತ್ತೇನೆ` — the present.
    pub present: [String; 10],
    /// `ಮಾಡುವೆನು` — the future.
    pub future: [String; 10],
    /// `[ಮಾಡು, ಮಾಡಿರಿ]`.
    pub imperative: [String; 2],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row = |t: Tense| SLOTS.map(|(p, n, g)| v.form(t, p, n, g));
        Self {
            infinitive: v.infinitive().to_string(),
            past: row(Tense::Past),
            present: row(Tense::Present),
            future: row(Tense::Future),
            imperative: [v.imperative(Number::Singular), v.imperative(Number::Plural)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};

    fn v(root: &str) -> Verb {
        Verb::from_infinitive(root).unwrap()
    }

    #[test]
    fn regular_u_root() {
        let a = v("ಮಾಡು");
        assert_eq!(a.form(Tense::Past, P1, SG, N), "ಮಾಡಿದೆನು");
        assert_eq!(a.form(Tense::Past, P2, SG, N), "ಮಾಡಿದೆ");
        assert_eq!(a.form(Tense::Past, P3, SG, M), "ಮಾಡಿದನು");
        assert_eq!(a.form(Tense::Past, P3, SG, F), "ಮಾಡಿದಳು");
        assert_eq!(a.form(Tense::Past, P3, SG, N), "ಮಾಡಿತು");
        assert_eq!(a.form(Tense::Past, P3, PL, M), "ಮಾಡಿದರು");
        assert_eq!(a.form(Tense::Past, P3, PL, N), "ಮಾಡಿದುವು");
        assert_eq!(a.form(Tense::Present, P1, SG, N), "ಮಾಡುತ್ತೇನೆ");
        assert_eq!(a.form(Tense::Present, P3, SG, M), "ಮಾಡುತ್ತಾನೆ");
        assert_eq!(a.form(Tense::Present, P3, SG, F), "ಮಾಡುತ್ತಾಳೆ");
        assert_eq!(a.form(Tense::Present, P3, SG, N), "ಮಾಡುತ್ತದೆ");
        assert_eq!(a.form(Tense::Present, P3, PL, N), "ಮಾಡುತ್ತವೆ");
        assert_eq!(a.form(Tense::Future, P1, SG, N), "ಮಾಡುವೆನು");
        assert_eq!(a.form(Tense::Future, P3, SG, M), "ಮಾಡುವನು");
        assert_eq!(a.form(Tense::Future, P3, SG, N), "ಮಾಡುವುದು");
        assert_eq!(a.form(Tense::Future, P3, PL, N), "ಮಾಡುವುವು");
        assert_eq!(a.imperative(SG), "ಮಾಡು");
        assert_eq!(a.imperative(PL), "ಮಾಡಿರಿ");
    }

    #[test]
    fn vowel_final_root_glide() {
        let n = v("ನಡೆ");
        assert_eq!(n.form(Tense::Past, P1, SG, N), "ನಡೆದೆನು");
        assert_eq!(n.form(Tense::Past, P3, SG, M), "ನಡೆದನು");
        assert_eq!(n.form(Tense::Past, P3, SG, N), "ನಡೆಯಿತು");
        assert_eq!(n.form(Tense::Present, P3, SG, M), "ನಡೆಯುತ್ತಾನೆ");
        assert_eq!(n.form(Tense::Future, P3, SG, M), "ನಡೆಯುವನು");
        assert_eq!(n.imperative(PL), "ನಡೆಯಿರಿ");
        // -ಿ root.
        let k = v("ಕುಡಿ");
        assert_eq!(k.form(Tense::Past, P3, SG, M), "ಕುಡಿದನು");
        assert_eq!(k.form(Tense::Present, P3, SG, M), "ಕುಡಿಯುತ್ತಾನೆ");
    }

    #[test]
    fn stored_irregular_pasts() {
        // ಆಗು "become": past stem ಆದ-, neuter 3sg ಆಯಿತು.
        let a = v("ಆಗು");
        assert_eq!(a.form(Tense::Past, P3, SG, M), "ಆದನು");
        assert_eq!(a.form(Tense::Past, P3, SG, N), "ಆಯಿತು");
        assert_eq!(a.form(Tense::Present, P3, SG, M), "ಆಗುತ್ತಾನೆ");
        // ಕೊಡು "give": past stem ಕೊಟ್ಟ-, neuter 3sg rebuilt ಕೊಟ್ಟಿತು.
        let k = v("ಕೊಡು");
        assert_eq!(k.form(Tense::Past, P1, SG, N), "ಕೊಟ್ಟೆನು");
        assert_eq!(k.form(Tense::Past, P3, SG, N), "ಕೊಟ್ಟಿತು");
        assert_eq!(k.form(Tense::Future, P3, SG, M), "ಕೊಡುವನು");
        // ಕೊಲ್ಲು "kill", ಗೆಲ್ಲು "win": consonant-cluster pasts.
        assert_eq!(v("ಕೊಲ್ಲು").form(Tense::Past, P3, SG, M), "ಕೊಂದನು");
        assert_eq!(v("ಗೆಲ್ಲು").form(Tense::Past, P3, SG, M), "ಗೆದ್ದನು");
        // ಮೀ "bathe": suppletive past ಮಿಂದ-, regular glide present.
        let m = v("ಮೀ");
        assert_eq!(m.form(Tense::Past, P1, SG, N), "ಮಿಂದೆನು");
        assert_eq!(m.form(Tense::Present, P3, SG, M), "ಮೀಯುತ್ತಾನೆ");
    }

    #[test]
    fn table_shape() {
        let t = Table::build(&v("ಓಡು"));
        assert_eq!(t.infinitive, "ಓಡು");
        assert_eq!(t.past[0], "ಓಡಿದೆನು");
        assert_eq!(t.present[2], "ಓಡುತ್ತಾನೆ");
        assert_eq!(t.future[0], "ಓಡುವೆನು");
        assert_eq!(t.imperative, ["ಓಡು", "ಓಡಿರಿ"]);
    }

    #[test]
    fn rejects_non_verbs() {
        assert_eq!(Verb::from_infinitive("run").err(), Some(Error::NotAVerb));
        assert_eq!(Verb::from_infinitive("").err(), Some(Error::NotAVerb));
        assert_eq!(
            Verb::from_infinitive("ಮಾಡು ಓಡು").err(),
            Some(Error::NotAVerb)
        );
    }
}
