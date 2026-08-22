//! Swahili (Kiswahili) conjugation. Bantu, agglutinative: the finite
//! verb is a slot template — subject concord, TAM (tense/aspect/mood),
//! ROOT, final vowel — plus a parallel negative pattern. The productive
//! template is the default rule; only the small closed set of
//! irregulars (the monosyllabic `-enda`, the suppletive imperatives of
//! `-ja`/`-enda`) needs stored parts in `data/swa/verbs.tsv`.
//!
//! The one genuinely lexical fact is the **subject/object noun-class
//! agreement matrix**: a Swahili verb agrees with its subject's noun
//! class, of which there are ~15 in the concord system. The `Subject`
//! enum is a person (1/2 sg/pl) or a `Class(n)`, and each carries three
//! prefixes — the plain concord (`a-`, `wa-`, `ki-`, …), the a-tense
//! fusion (`a-`, `wa-`, `cha-`, …) and the negative concord (`ha-`,
//! `hawa-`, `haki-`, …). No orthographic tone is written.
//!
//! Two stem facts are derived from the citation stem: whether it is
//! **monosyllabic** (one vowel — `-la`, `-fa`, `-nywa`), which makes the
//! infinitival `ku-` reappear under the primary TAM markers
//! (`a-na-ku-la`), and whether its final vowel is `-a` (a Bantu stem,
//! whose subjunctive and negative shift the vowel: `some`, `somi`) or
//! not (an Arabic loan in `-i/-e/-u`, which keeps it: `jibu`, `nijibu`).

use std::collections::HashMap;
use std::sync::OnceLock;

/// Grammatical number, for the imperative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Singular,
    Plural,
}

/// Polarity: Swahili negation is a wholly separate prefix pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Polarity {
    Positive,
    Negative,
}

/// The subject the verb agrees with: a grammatical person, or a noun
/// class (1–18). Class 1/2 are the animate m-/wa- pair (also the
/// third-person "he/she/they"); the rest are the inanimate and locative
/// classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Subject {
    First(Number),
    Second(Number),
    Class(u8),
}

/// The tense/aspect/mood of a finite form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    /// -na-, the general present.
    Present,
    /// -li-, the past.
    Past,
    /// -ta-, the future.
    Future,
    /// -me-, the perfect ("have …ed").
    Perfect,
    /// the a-tense (-a-), a gnomic/present, with fused subject concord.
    Gnomic,
    /// final -e, the subjunctive.
    Subjunctive,
    /// hu-, the habitual (no subject concord).
    Habitual,
    /// -ka-, the consecutive/narrative.
    Consecutive,
    /// -ki-, the situative ("if/when …ing").
    Situative,
    /// -nge-, the present conditional.
    ConditionalPresent,
    /// -ngali-, the past conditional.
    ConditionalPast,
}

/// Why an input cannot be conjugated as a Swahili verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Swahili verb stem or infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Swahili verb")
    }
}

/// Mined row: lemma, then the irregular parts (`-` = derive by rule):
/// mono override, infinitive, primary-TAM stem, short stem, subjunctive
/// stem, imperative singular, imperative plural.
static VERBS_TSV: &str = include_str!("../data/swa/verbs.tsv");

#[derive(Debug, Clone, Default)]
struct Row {
    mono: Option<bool>,
    inf: Option<String>,
    long_stem: Option<String>,
    short_stem: Option<String>,
    subj_stem: Option<String>,
    imp_sg: Option<String>,
    imp_pl: Option<String>,
}

fn rows() -> &'static HashMap<&'static str, Row> {
    static MAP: OnceLock<HashMap<&'static str, Row>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in VERBS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            let opt = |i: usize| {
                c.get(i)
                    .filter(|s| **s != "-" && !s.is_empty())
                    .map(|s| (*s).to_string())
            };
            let mono = c.get(1).and_then(|s| match *s {
                "1" => Some(true),
                "0" => Some(false),
                _ => None,
            });
            m.insert(
                c[0],
                Row {
                    mono,
                    inf: opt(2),
                    long_stem: opt(3),
                    short_stem: opt(4),
                    subj_stem: opt(5),
                    imp_sg: opt(6),
                    imp_pl: opt(7),
                },
            );
        }
        m
    })
}

/// A conjugatable Swahili verb.
#[derive(Debug, Clone)]
pub struct Verb {
    /// The citation stem (`soma`, `jibu`, `enda`) — the lemma both
    /// oracles key on, i.e. the infinitive minus `ku-`.
    stem: String,
    row: Row,
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

impl Verb {
    /// Build a verb from its infinitive (`kusoma`) or bare stem
    /// (`soma`). Both are accepted: the `ku-` infinitive marker is
    /// stripped when present (but not from a genuine stem like `kuna`,
    /// which is too short to be `ku-` + a stem).
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim().to_lowercase();
        if inf.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf.chars().all(|c| c.is_ascii_alphabetic())
        {
            return Err(Error::NotAVerb);
        }
        // The lemma is the bare stem (soma, jibu, enda), as both oracles
        // key it. A leading ku-/kw- infinitive marker is stripped when it
        // clearly leaves a stem behind: kwenda → enda and kufa → fa via
        // the lexicon, kusoma → soma by length. Genuine ku-/kw-initial
        // stems (kuna "scratch", kwepa "dodge") are kept: they are short
        // and not lexicon entries, so neither test fires.
        let stem = if let Some(rest) = inf.strip_prefix("kw") {
            if rows().contains_key(rest) {
                rest.to_string() // kwenda → enda
            } else {
                inf.clone()
            }
        } else if let Some(rest) = inf.strip_prefix("ku") {
            if rows().contains_key(rest) || inf.len() >= 5 {
                rest.to_string() // kufa → fa, kusoma → soma
            } else {
                inf.clone() // kuna stays kuna
            }
        } else {
            inf.clone()
        };
        let row = rows().get(stem.as_str()).cloned().unwrap_or_default();
        Ok(Self { stem, row })
    }

    /// The citation stem.
    #[must_use]
    pub fn stem(&self) -> &str {
        &self.stem
    }

    fn vowel_count(&self) -> usize {
        self.stem.chars().filter(|c| is_vowel(*c)).count()
    }

    /// A monosyllabic stem (one vowel) keeps the infinitival `ku-` under
    /// the primary TAM markers. A lexicon row may override the rule.
    fn is_mono(&self) -> bool {
        self.row.mono.unwrap_or_else(|| self.vowel_count() == 1)
    }

    /// The final vowel of the stem decides the Bantu/loan split.
    fn bantu(&self) -> bool {
        self.stem.ends_with('a')
    }

    /// The stem after a primary TAM marker (-na-, -li-, -ta-, -me-,
    /// -nge-, -ngali-): `ku-` reappears for monosyllabic stems.
    fn long_stem(&self) -> String {
        if let Some(s) = &self.row.long_stem {
            return s.clone();
        }
        if self.is_mono() {
            format!("ku{}", self.stem)
        } else {
            self.stem.clone()
        }
    }

    /// The stem after the a-tense/subjunctive/habitual/consecutive/
    /// situative markers: never carries `ku-`.
    fn short_stem(&self) -> String {
        self.row
            .short_stem
            .clone()
            .unwrap_or_else(|| self.stem.clone())
    }

    /// The subjunctive stem: Bantu `-a` → `-e`, loans keep their vowel.
    fn subj_stem(&self) -> String {
        if let Some(s) = &self.row.subj_stem {
            return s.clone();
        }
        if self.bantu() {
            format!("{}e", &self.stem[..self.stem.len() - 1])
        } else {
            self.stem.clone()
        }
    }

    /// The infinitive (`kusoma`, `kufa`, `kwenda`).
    #[must_use]
    pub fn infinitive(&self, polarity: Polarity) -> String {
        match polarity {
            Polarity::Positive => self
                .row
                .inf
                .clone()
                .unwrap_or_else(|| format!("ku{}", self.stem)),
            // Negative infinitive: kuto- + the ku-bearing stem
            // (kutosoma, kutokufa).
            Polarity::Negative => format!("kuto{}", self.long_stem()),
        }
    }

    /// An imperative (`soma`/`someni`, `kula`/`kuleni`, `nenda`).
    #[must_use]
    pub fn imperative(&self, number: Number, polarity: Polarity) -> String {
        if polarity == Polarity::Negative {
            // usisome / msisomeni — the negative imperative is the
            // subjunctive with -si-.
            let sm = match number {
                Number::Singular => "u",
                Number::Plural => "m",
            };
            let stem = if number == Number::Plural {
                format!("{}ni", self.subj_stem())
            } else {
                self.subj_stem()
            };
            return format!("{sm}si{stem}");
        }
        match number {
            Number::Singular => self.row.imp_sg.clone().unwrap_or_else(|| {
                if self.is_mono() {
                    format!("ku{}", self.stem)
                } else {
                    self.stem.clone()
                }
            }),
            Number::Plural => self.row.imp_pl.clone().unwrap_or_else(|| {
                let ku = if self.is_mono() { "ku" } else { "" };
                format!("{ku}{}ni", self.subj_stem())
            }),
        }
    }

    /// A finite form for a (tense, subject, polarity) cell.
    #[must_use]
    pub fn form(&self, tense: Tense, subject: Subject, polarity: Polarity) -> String {
        if polarity == Polarity::Negative {
            return self.negative(tense, subject);
        }
        match tense {
            Tense::Habitual => format!("hu{}", self.short_stem()),
            Tense::Gnomic => concord(a_tense_sm(subject), &self.short_stem()),
            Tense::Subjunctive => concord(positive_sm(subject), &self.subj_stem()),
            Tense::Present => {
                format!("{}na{}", positive_sm(subject), self.long_stem())
            }
            Tense::Past => format!("{}li{}", positive_sm(subject), self.long_stem()),
            Tense::Future => format!("{}ta{}", positive_sm(subject), self.long_stem()),
            Tense::Perfect => format!("{}me{}", positive_sm(subject), self.long_stem()),
            Tense::Consecutive => format!("{}ka{}", positive_sm(subject), self.short_stem()),
            Tense::Situative => format!("{}ki{}", positive_sm(subject), self.short_stem()),
            Tense::ConditionalPresent => {
                format!("{}nge{}", positive_sm(subject), self.long_stem())
            }
            Tense::ConditionalPast => {
                format!("{}ngali{}", positive_sm(subject), self.long_stem())
            }
        }
    }

    fn negative(&self, tense: Tense, subject: Subject) -> String {
        let neg = negative_sm(subject);
        match tense {
            // Negative present: -a → -i for Bantu stems (sisomi), loans
            // keep the vowel (sijibu); short stem, no ku.
            Tense::Present => {
                let stem = if self.bantu() {
                    format!("{}i", &self.short_stem()[..self.short_stem().len() - 1])
                } else {
                    self.short_stem()
                };
                concord(neg, &stem)
            }
            // Negative past: -ku- + stem (sikusoma).
            Tense::Past => format!("{neg}ku{}", self.short_stem()),
            Tense::Future => format!("{neg}ta{}", self.long_stem()),
            Tense::Perfect => format!("{neg}ja{}", self.short_stem()),
            Tense::Subjunctive => concord(positive_sm(subject), &format!("si{}", self.subj_stem())),
            Tense::ConditionalPresent => format!("{neg}singe{}", self.long_stem()),
            Tense::ConditionalPast => format!("{neg}singali{}", self.long_stem()),
            // The remaining aspects negate periphrastically or rarely;
            // fall back to the positive concord with -si-.
            _ => concord(positive_sm(subject), &format!("si{}", self.short_stem())),
        }
    }
}

/// Glue a subject marker to a stem. The only concord that reshapes
/// before a vowel-initial stem is the plural `m-`, which becomes `mw-`
/// (or `mu-` before `u`): `m + ona → mwona`, `m + umba → muumba`. The
/// vowel-final person and class markers (`tu-`, `ni-`, `wa-`, `ki-`, …)
/// keep their hiatus in the written standard both oracles agree on
/// (`tuandike`, not \*twandike), so everything else concatenates.
fn concord(sm: &str, stem: &str) -> String {
    if sm == "m" {
        if let Some(v) = stem.chars().next() {
            if is_vowel(v) {
                let g = if v == 'u' { "mu" } else { "mw" };
                return format!("{g}{stem}");
            }
        }
    }
    format!("{sm}{stem}")
}

/// The positive subject concord for the finite tenses.
fn positive_sm(s: Subject) -> &'static str {
    match s {
        Subject::First(Number::Singular) => "ni",
        Subject::Second(Number::Singular) => "u",
        Subject::First(Number::Plural) => "tu",
        Subject::Second(Number::Plural) => "m",
        Subject::Class(c) => CLASS_SM[class_index(c)].0,
    }
}

/// The a-tense subject concord (the fused `-a-` present).
fn a_tense_sm(s: Subject) -> &'static str {
    match s {
        Subject::First(Number::Singular) => "na",
        Subject::Second(Number::Singular) => "wa",
        Subject::First(Number::Plural) => "twa",
        Subject::Second(Number::Plural) => "mwa",
        Subject::Class(c) => CLASS_SM[class_index(c)].1,
    }
}

/// The negative subject concord.
fn negative_sm(s: Subject) -> &'static str {
    match s {
        Subject::First(Number::Singular) => "si",
        Subject::Second(Number::Singular) => "hu",
        Subject::First(Number::Plural) => "hatu",
        Subject::Second(Number::Plural) => "ham",
        Subject::Class(c) => CLASS_SM[class_index(c)].2,
    }
}

/// (plain, a-tense, negative) concords for the noun classes, indexed by
/// `class_index`. Classes 1–11 and the locatives 15–18.
const CLASS_SM: [(&str, &str, &str); 15] = [
    ("a", "a", "ha"),      // 1
    ("wa", "wa", "hawa"),  // 2
    ("u", "wa", "hau"),    // 3
    ("i", "ya", "hai"),    // 4
    ("li", "la", "hali"),  // 5
    ("ya", "ya", "haya"),  // 6
    ("ki", "cha", "haki"), // 7
    ("vi", "vya", "havi"), // 8
    ("i", "ya", "hai"),    // 9
    ("zi", "za", "hazi"),  // 10
    ("u", "wa", "hau"),    // 11
    ("ku", "kwa", "haku"), // 15
    ("pa", "pa", "hapa"),  // 16
    ("ku", "kwa", "haku"), // 17
    ("mu", "mwa", "hamu"), // 18
];

fn class_index(c: u8) -> usize {
    match c {
        1..=11 => (c - 1) as usize,
        15 => 11,
        16 => 12,
        17 => 13,
        18 => 14,
        _ => 0,
    }
}

/// The conjugation table of a Swahili verb — the person-based core, for
/// the WebAssembly and Python bindings. Six-slot rows run
/// [1sg, 2sg, 3sg (class 1), 1pl, 2pl, 3pl (class 2)]; the full
/// noun-class matrix is reached through `Verb::form`.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub infinitive_negative: String,
    /// [singular, plural].
    pub imperative: [String; 2],
    pub habitual: String,
    pub present: [String; 6],
    pub present_negative: [String; 6],
    pub past: [String; 6],
    pub future: [String; 6],
    pub perfect: [String; 6],
    pub subjunctive: [String; 6],
    pub gnomic: [String; 6],
}

const PERSON_ROW: [Subject; 6] = [
    Subject::First(Number::Singular),
    Subject::Second(Number::Singular),
    Subject::Class(1),
    Subject::First(Number::Plural),
    Subject::Second(Number::Plural),
    Subject::Class(2),
];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row = |t: Tense, p: Polarity| PERSON_ROW.map(|s| v.form(t, s, p));
        Self {
            infinitive: v.infinitive(Polarity::Positive),
            infinitive_negative: v.infinitive(Polarity::Negative),
            imperative: [
                v.imperative(Number::Singular, Polarity::Positive),
                v.imperative(Number::Plural, Polarity::Positive),
            ],
            habitual: v.form(Tense::Habitual, Subject::Class(1), Polarity::Positive),
            present: row(Tense::Present, Polarity::Positive),
            present_negative: row(Tense::Present, Polarity::Negative),
            past: row(Tense::Past, Polarity::Positive),
            future: row(Tense::Future, Polarity::Positive),
            perfect: row(Tense::Perfect, Polarity::Positive),
            subjunctive: row(Tense::Subjunctive, Polarity::Positive),
            gnomic: row(Tense::Gnomic, Polarity::Positive),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Verb {
        Verb::from_infinitive(s).unwrap()
    }

    #[test]
    fn regular_soma() {
        let s = v("soma");
        assert_eq!(s.infinitive(Polarity::Positive), "kusoma");
        assert_eq!(
            s.form(
                Tense::Present,
                Subject::First(Number::Singular),
                Polarity::Positive
            ),
            "ninasoma"
        );
        assert_eq!(
            s.form(Tense::Present, Subject::Class(1), Polarity::Positive),
            "anasoma"
        );
        assert_eq!(
            s.form(
                Tense::Subjunctive,
                Subject::First(Number::Plural),
                Polarity::Positive
            ),
            "tusome"
        );
        assert_eq!(
            s.form(Tense::Gnomic, Subject::Class(7), Polarity::Positive),
            "chasoma"
        );
        assert_eq!(
            s.form(Tense::Habitual, Subject::Class(1), Polarity::Positive),
            "husoma"
        );
        assert_eq!(
            s.form(
                Tense::Present,
                Subject::First(Number::Singular),
                Polarity::Negative
            ),
            "sisomi"
        );
    }

    #[test]
    fn monosyllabic_keeps_ku() {
        let f = v("fa");
        assert_eq!(f.infinitive(Polarity::Positive), "kufa");
        assert_eq!(
            f.form(
                Tense::Present,
                Subject::First(Number::Singular),
                Polarity::Positive
            ),
            "ninakufa"
        );
        assert_eq!(
            f.form(Tense::Present, Subject::Class(2), Polarity::Positive),
            "wanakufa"
        );
        // The a-tense and subjunctive drop the ku again.
        assert_eq!(
            f.form(Tense::Gnomic, Subject::Class(1), Polarity::Positive),
            "afa"
        );
        assert_eq!(
            f.form(Tense::Habitual, Subject::Class(1), Polarity::Positive),
            "hufa"
        );
    }

    #[test]
    fn arabic_loan_keeps_final_vowel() {
        let j = v("jibu");
        assert_eq!(
            j.form(
                Tense::Subjunctive,
                Subject::First(Number::Plural),
                Polarity::Positive
            ),
            "tujibu"
        );
        assert_eq!(
            j.form(
                Tense::Present,
                Subject::First(Number::Singular),
                Polarity::Negative
            ),
            "sijibu"
        );
    }

    #[test]
    fn vowel_initial_no_ku() {
        let o = v("ona");
        assert_eq!(o.infinitive(Polarity::Positive), "kuona");
        assert_eq!(
            o.form(Tense::Present, Subject::Class(1), Polarity::Positive),
            "anaona"
        );
        // 2pl m- glides to mw- before a vowel stem.
        assert_eq!(
            o.form(
                Tense::Subjunctive,
                Subject::Second(Number::Plural),
                Polarity::Positive
            ),
            "mwone"
        );
    }

    #[test]
    fn suppletive_enda() {
        let e = v("enda");
        assert_eq!(e.infinitive(Polarity::Positive), "kwenda");
        assert_eq!(
            e.form(Tense::Present, Subject::Class(1), Polarity::Positive),
            "anakwenda"
        );
        // tuende (a kaikki-listed variant of the more common twende):
        // the engine keeps the tu- hiatus, matching the no-glide rule the
        // two oracles agree on for the other vowel stems (tuandike).
        assert_eq!(
            e.form(
                Tense::Subjunctive,
                Subject::First(Number::Plural),
                Polarity::Positive
            ),
            "tuende"
        );
    }
}
