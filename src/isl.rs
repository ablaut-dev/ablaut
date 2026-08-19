//! Icelandic conjugation: the productive weak `-a` (ō-class) engine plus
//! a compiled-in principal-parts lexicon for everything else.
//!
//! Icelandic verbs share one infinitive shape (`-a`: kalla, gefa, telja),
//! so the inflection class is not recoverable from the infinitive. One
//! class is productive and rule-generated — the ō-class *kalla*
//! (kalla/kallar/kallaði/kallað), the *tala/borða/vakna* pattern — and
//! every verb that deviates (the weak dental classes with i-mutation and
//! syncope, and the strong ablaut verbs gefa→gaf→gáfu→gefið) carries a
//! mined full-paradigm row in `data/isl/verbs.tsv`.
//!
//! The one productive spelling rule is u-umlaut: the stem's last short
//! `a` fronts to `ö` before an ending whose theme vowel is `u`
//! (köllum, kölluðu, tölum). Verbs the rule mishandles fall to the
//! lexicon, so the engine only has to be right for the plain ō-class.

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

/// The four synthetic active finite tense/mood combinations. Compound
/// tenses (hafa + supine, munu/skulu + infinitive) are the analytic
/// layer's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpleTense {
    /// Framsöguháttur nútíðar — present indicative (kalla, kallar).
    Present,
    /// Framsöguháttur þátíðar — past indicative (kallaði).
    Past,
    /// Viðtengingarháttur nútíðar — present subjunctive (kalli).
    SubjunctivePresent,
    /// Viðtengingarháttur þátíðar — past subjunctive (kallaði, gæfi).
    SubjunctivePast,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like an Icelandic infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Icelandic infinitive")
    }
}

// ō-class active endings, indexed [1sg, 2sg, 3sg, 1pl, 2pl, 3pl]. The
// stem is the infinitive minus its final -a. u-umlaut is applied
// separately for endings that begin with u.
const PRS: [&str; 6] = ["a", "ar", "ar", "um", "ið", "a"];
const PST: [&str; 6] = ["aði", "aðir", "aði", "uðum", "uðuð", "uðu"];
const SPRS: [&str; 6] = ["i", "ir", "i", "um", "ið", "i"];
// Past subjunctive of the ō-class is identical to the past indicative.
const SPST: [&str; 6] = ["aði", "aðir", "aði", "uðum", "uðuð", "uðu"];

/// Apply ō-class u-umlaut: the stem's final vowel, when it is a short
/// `a`, fronts to `ö` before an ending whose theme vowel is `u`
/// (kall + um → köllum). Only the last vowel is a target, so an `a` in an
/// earlier syllable is untouched (gagnrýn + um → gagnrýnum, not
/// *gögnrýnum; af·svert + um → afsvertum).
fn u_umlaut(stem: &str, ending: &str) -> String {
    if !ending.starts_with('u') {
        return stem.to_string();
    }
    match stem.char_indices().rfind(|(_, c)| is_vowel(*c)) {
        Some((i, 'a')) => format!("{}ö{}", &stem[..i], &stem[i + 'a'.len_utf8()..]),
        _ => stem.to_string(),
    }
}

fn is_vowel(c: char) -> bool {
    "aáeéiíoóuúyýæö".contains(c)
}

/// The compiled-in principal-parts lexicon (strong and irregular-weak
/// verbs). Each finite tense is an optional six-form override
/// (comma-separated, [1sg..3pl]); a `-` cell falls through to the
/// ō-class engine, so a verb that is regular in a tense costs nothing.
///
/// Columns: lemma, pres, past, subjpres, subjpast, supine, presp, imp2,
/// imp2pl, exact.
static LEXICON_TSV: &str = include_str!("../data/isl/verbs.tsv");

#[derive(Debug, Clone, Default)]
struct LexEntry {
    pres: Option<[String; 6]>,
    past: Option<[String; 6]>,
    subjpres: Option<[String; 6]>,
    subjpast: Option<[String; 6]>,
    supine: Option<String>,
    presp: Option<String>,
    imp2: Option<String>,
    imp2pl: Option<String>,
}

fn six(s: &str) -> Option<[String; 6]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
}

fn opt(s: &str) -> Option<String> {
    (s != "-").then(|| s.to_string())
}

fn parse_lex(cols: &[&str]) -> LexEntry {
    let g = |i: usize| cols.get(i).copied().unwrap_or("-");
    LexEntry {
        pres: six(g(1)),
        past: six(g(2)),
        subjpres: six(g(3)),
        subjpast: six(g(4)),
        supine: opt(g(5)),
        presp: opt(g(6)),
        imp2: opt(g(7)),
        imp2pl: opt(g(8)),
    }
}

/// Longest-base suffix lookup with the derivational prefix returned
/// (endurtaka → ("endur", taka)); rows flagged exact-only (col 9 = "1")
/// match the whole infinitive so a base that is a suffix of an
/// otherwise-regular verb cannot hijack it.
fn lexical(inf: &str) -> Option<(usize, LexEntry)> {
    let mut best: Option<(&str, Vec<&str>)> = None;
    for line in LEXICON_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let lemma = cols[0];
        let exact = cols.get(9).copied() == Some("1");
        // Case-insensitive: the lemma keeps the gold's case (DNA-greina,
        // Brasilíuvæða) while the query has been lowercased. Icelandic
        // and ASCII letters keep their byte length under lowercasing.
        let lemma_lc = lemma.to_lowercase();
        let hit = if exact {
            inf == lemma_lc.as_str()
        } else {
            inf.ends_with(lemma_lc.as_str())
        };
        if hit && !best.as_ref().is_some_and(|(b, _)| b.len() >= lemma.len()) {
            best = Some((lemma, cols));
        }
    }
    let (lemma, cols) = best?;
    Some((lemma.len(), parse_lex(&cols)))
}

/// A conjugatable Icelandic verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    /// Infinitive minus the final -a (the ō-class stem).
    stem: String,
    lex: Option<Box<LexEntry>>,
    prefix: String,
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
    /// Build a verb from its infinitive. Case is preserved (the
    /// proper-noun-derived verbs keep their capital: *Brasilíuvæða*,
    /// *DNA-greina*); the lexicon is consulted case-insensitively.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim().to_string();
        if inf.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf.chars().all(|c| c.is_alphabetic() || c == '-')
        {
            return Err(Error::NotAVerb);
        }
        // Case-fold only for the suffix lookup; the prefix is sliced back
        // out of the original so its capitalization survives. Icelandic
        // (and ASCII) letters keep their UTF-8 length under lowercasing,
        // so the byte offset is shared.
        let lowered = inf.to_lowercase();
        let (prefix, lex) = match lexical(&lowered) {
            Some((base_len, e)) => (inf[..inf.len() - base_len].to_string(), Some(Box::new(e))),
            None => (String::new(), None),
        };
        // The ō-class stem is the infinitive minus -a; a lexicon verb
        // whose infinitive does not end in -a (þvo, ná) still needs a
        // stem placeholder, but its forms come from stored parts.
        let stem = inf.strip_suffix('a').unwrap_or(&inf).to_string();
        // Icelandic infinitives end in a vowel (-a, the contracted -á/-o
        // strong verbs) or in the middle-voice -st; anything else is not
        // a verb. Non-ō shapes rely on a lexicon row for real forms.
        let shape_ok = inf.ends_with(is_vowel) || inf.ends_with("st");
        if lex.is_none() && !shape_ok {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf,
            stem,
            lex,
            prefix,
        })
    }

    /// The infinitive as normalized.
    #[must_use]
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// The ō-class rule form for a slot.
    fn regular_form(&self, tense: SimpleTense, i: usize) -> String {
        let endings = match tense {
            SimpleTense::Present => &PRS,
            SimpleTense::Past => &PST,
            SimpleTense::SubjunctivePresent => &SPRS,
            SimpleTense::SubjunctivePast => &SPST,
        };
        let ending = endings[i];
        format!("{}{ending}", u_umlaut(&self.stem, ending))
    }

    /// A stored-paradigm form, falling through to the ō-class engine for
    /// any tense the lexicon leaves blank.
    fn lex_form(&self, l: &LexEntry, tense: SimpleTense, i: usize) -> String {
        let stored = match tense {
            SimpleTense::Present => l.pres.as_ref(),
            SimpleTense::Past => l.past.as_ref(),
            SimpleTense::SubjunctivePresent => l.subjpres.as_ref(),
            SimpleTense::SubjunctivePast => l.subjpast.as_ref(),
        };
        match stored {
            Some(f) => format!("{}{}", self.prefix, f[i]),
            None => self.regular_form(tense, i),
        }
    }

    /// A finite form.
    #[must_use]
    pub fn conjugate(&self, tense: SimpleTense, person: Person, number: Number) -> String {
        let i = person.index(number);
        match &self.lex {
            Some(l) => self.lex_form(&l.clone(), tense, i),
            None => self.regular_form(tense, i),
        }
    }

    /// Every standard spelling of a slot, canonical first. Icelandic has
    /// no synthetic doublets in the core active paradigm, so this is a
    /// single form; the method mirrors the other languages' contract.
    #[must_use]
    pub fn variants(&self, tense: SimpleTense, person: Person, number: Number) -> Vec<String> {
        vec![self.conjugate(tense, person, number)]
    }

    /// The supine (sagnbót): the indeclinable past participle used with
    /// hafa (hef kallað, hef gefið).
    #[must_use]
    pub fn supine(&self) -> String {
        if let Some(l) = &self.lex {
            if let Some(s) = &l.supine {
                return format!("{}{s}", self.prefix);
            }
        }
        format!("{}að", self.stem)
    }

    /// The present participle (lýsingarháttur nútíðar): kallandi,
    /// gefandi.
    #[must_use]
    pub fn present_participle(&self) -> String {
        if let Some(l) = &self.lex {
            if let Some(p) = &l.presp {
                return format!("{}{p}", self.prefix);
            }
        }
        format!("{}ndi", self.infinitive)
    }

    /// The imperative: the clipped singular (kalla, gef, far) and the
    /// plural in -ið (kallið). No first person.
    #[must_use]
    pub fn imperative(&self, person: Person, number: Number) -> Option<String> {
        match (person, number) {
            (Person::Second, Number::Singular) => {
                Some(self.lex.as_ref().and_then(|l| l.imp2.as_ref()).map_or_else(
                    || self.infinitive.clone(),
                    |f| format!("{}{f}", self.prefix),
                ))
            }
            (Person::Second, Number::Plural) => Some(
                self.lex
                    .as_ref()
                    .and_then(|l| l.imp2pl.as_ref())
                    .map_or_else(
                        || self.conjugate(SimpleTense::Present, Person::Second, Number::Plural),
                        |f| format!("{}{f}", self.prefix),
                    ),
            ),
            _ => None,
        }
    }
}

/// The full conjugation table of an Icelandic verb as one plain struct —
/// shared by the WebAssembly and Python bindings. Rows are
/// [ég, þú, hann/hún, við, þið, þeir].
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub supine: String,
    pub present_participle: String,
    /// [þú, þið].
    pub imperative: [Option<String>; 2],
    pub present: [String; 6],
    pub past: [String; 6],
    pub subjunctive_present: [String; 6],
    pub subjunctive_past: [String; 6],
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
        let row = |t: SimpleTense| SLOTS.map(|(p, n)| v.conjugate(t, p, n));
        Self {
            infinitive: v.infinitive().to_string(),
            supine: v.supine(),
            present_participle: v.present_participle(),
            imperative: [
                v.imperative(Person::Second, Number::Singular),
                v.imperative(Person::Second, Number::Plural),
            ],
            present: row(SimpleTense::Present),
            past: row(SimpleTense::Past),
            subjunctive_present: row(SimpleTense::SubjunctivePresent),
            subjunctive_past: row(SimpleTense::SubjunctivePast),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};
    use SimpleTense::{Past, Present, SubjunctivePast, SubjunctivePresent};

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn regular_o_class() {
        let k = v("kalla");
        assert_eq!(k.conjugate(Present, P1, SG), "kalla");
        assert_eq!(k.conjugate(Present, P2, SG), "kallar");
        assert_eq!(k.conjugate(Present, P3, SG), "kallar");
        assert_eq!(k.conjugate(Present, P1, PL), "köllum");
        assert_eq!(k.conjugate(Present, P2, PL), "kallið");
        assert_eq!(k.conjugate(Present, P3, PL), "kalla");
        assert_eq!(k.conjugate(Past, P1, SG), "kallaði");
        assert_eq!(k.conjugate(Past, P1, PL), "kölluðum");
        assert_eq!(k.conjugate(Past, P3, PL), "kölluðu");
        assert_eq!(k.conjugate(SubjunctivePresent, P1, SG), "kalli");
        assert_eq!(k.conjugate(SubjunctivePresent, P1, PL), "köllum");
        assert_eq!(k.conjugate(SubjunctivePast, P3, SG), "kallaði");
        assert_eq!(k.supine(), "kallað");
        assert_eq!(k.present_participle(), "kallandi");
        assert_eq!(k.imperative(P2, SG).unwrap(), "kalla");
        assert_eq!(k.imperative(P2, PL).unwrap(), "kallið");
    }

    #[test]
    fn u_umlaut_rule() {
        assert_eq!(v("tala").conjugate(Present, P1, PL), "tölum");
        assert_eq!(v("vakna").conjugate(Past, P3, PL), "vöknuðu");
        assert_eq!(v("kasta").conjugate(Present, P1, PL), "köstum");
        // u-umlaut targets only the stem's final vowel.
        assert_eq!(
            v("gagnrýna").conjugate(SubjunctivePresent, P1, PL),
            "gagnrýnum"
        );
    }

    #[test]
    fn strong_verbs_from_lexicon() {
        let g = v("gefa");
        assert_eq!(g.conjugate(Present, P1, SG), "gef");
        assert_eq!(g.conjugate(Present, P3, SG), "gefur");
        assert_eq!(g.conjugate(Past, P3, SG), "gaf");
        assert_eq!(g.conjugate(Past, P3, PL), "gáfu");
        assert_eq!(g.conjugate(SubjunctivePast, P3, SG), "gæfi");
        assert_eq!(g.supine(), "gefið");
        assert_eq!(g.imperative(P2, SG).unwrap(), "gef");
        let f = v("fara");
        assert_eq!(f.conjugate(Present, P3, SG), "fer");
        assert_eq!(f.conjugate(Past, P3, SG), "fór");
        assert_eq!(f.supine(), "farið");
        let t = v("telja");
        assert_eq!(t.conjugate(Past, P3, SG), "taldi");
        assert_eq!(t.supine(), "talið");
    }

    #[test]
    fn prefixed_derivative_inherits_base() {
        // endurtaka conjugates like taka with the endur- prefix.
        let e = v("endurtaka");
        assert_eq!(e.conjugate(Present, P3, SG), "endurtekur");
        assert_eq!(e.conjugate(Past, P3, SG), "endurtók");
        assert_eq!(e.supine(), "endurtekið");
    }

    #[test]
    fn non_a_and_case() {
        // A contracted strong verb (þvo) comes wholly from the lexicon.
        assert_eq!(v("þvo").supine(), "þvegið");
        // Capital-initial proper-derived verbs keep their case.
        let b = Verb::from_infinitive("Brasilíuvæða").unwrap();
        assert!(b.conjugate(Present, P3, SG).starts_with("Brasilíu"));
        assert!(Verb::from_infinitive("xyz").is_err());
    }
}
