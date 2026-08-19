//! Dutch conjugation: the regular (weak) engine with the productive
//! spelling rules, plus a compiled-in lexicon of the strong and
//! irregular verbs.
//!
//! A regular Dutch verb is built from a single stem, derived from the
//! `-en` infinitive by the orthographic rules that keep vowel length
//! and voicing visible:
//!
//! - **Consonant doubling** collapses: `pakken → pak`, `zetten → zet`.
//! - **Vowel doubling** in a now-closed syllable: `maken → maak`,
//!   `lopen → loop`, `studeren → studeer`. A single `e` in the native
//!   schwa suffixes `-elen/-enen/-emen` stays short (`wandelen →
//!   wandel`, `rekenen → reken`) unless the root is monosyllabic
//!   (`delen → deel`, `nemen → neem`).
//! - **Final devoicing spelling**: `v → f`, `z → s` (`leven → leef`,
//!   `reizen → reis`), while the *underlying* voicing still drives the
//!   past and participle suffix.
//!
//! The past and past participle follow *'t kofschip*: a voiceless stem
//! takes `-te/-t` (`werkte`, `gewerkt`), a voiced one `-de/-d`
//! (`woonde`, `gewoond`). The past participle prepends `ge-` unless the
//! verb carries an unstressed inseparable prefix (`bedanken → bedankt`,
//! `verkopen → verkocht`).
//!
//! Strong verbs (ablaut past/participle), the suppletive auxiliaries
//! and modals, the schwa `-eren` verbs and the doubling exceptions all
//! carry mined rows in `data/nld/verbs.tsv`, matched exactly.

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

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Dutch infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Dutch infinitive")
    }
}

/// The compiled-in irregular lexicon.
static LEXICON_TSV: &str = include_str!("../data/nld/verbs.tsv");

/// A stored irregular paradigm. Every column is optional: an empty
/// cell (`-`) falls through to the regular engine built on `stem`
/// (itself derived when absent). `pres` is the six present forms
/// `[sg1, sg2, sg3, pl1, pl2, pl3]`.
#[derive(Debug, Clone, Default)]
struct LexEntry {
    stem: Option<String>,
    pres: Option<[String; 6]>,
    past_sg: Option<String>,
    past_pl: Option<String>,
    pp: Option<String>,
    imp: Option<String>,
    present_participle: Option<String>,
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
                    stem: opt(g(1)),
                    pres: six(g(2)),
                    past_sg: opt(g(3)),
                    past_pl: opt(g(4)),
                    pp: opt(g(5)),
                    imp: opt(g(6)),
                    present_participle: opt(g(7)),
                },
            );
        }
        m
    })
}

const VOWELS: &str = "aeiou";

fn is_vowel(c: char) -> bool {
    VOWELS.contains(c) || "áéíóúàèäëïöü".contains(c)
}

/// The unstressed inseparable prefixes that suppress the participle
/// `ge-`. A prefix only counts when a plausible verb body (ending in
/// `-en`, at least four letters) follows, so `eren`, `verven` and
/// `geven` keep their `ge-`.
const NO_GE_PREFIXES: [&str; 5] = ["be", "ge", "her", "ont", "ver"];

fn takes_ge(inf: &str) -> bool {
    for p in NO_GE_PREFIXES {
        if let Some(rest) = inf.strip_prefix(p) {
            if rest.ends_with("en") && rest.chars().count() >= 4 {
                return false;
            }
        }
    }
    true
}

/// Derive the regular stem of an `-en` infinitive and its voicing.
/// Returns `(stem, voiceless)` where `voiceless` follows *'t kofschip*
/// on the underlying (pre-devoicing) final consonant.
fn derive_stem(inf: &str) -> Option<(String, bool)> {
    let root = inf.strip_suffix("en").filter(|r| !r.is_empty())?;
    let chars: Vec<char> = root.chars().collect();
    let n = chars.len();
    let last = chars[n - 1];

    // Underlying voicing is read before any v→f / z→s devoicing.
    let voiceless = if last == 'v' || last == 'z' {
        false
    } else if root.ends_with("ch") {
        true
    } else {
        matches!(last, 't' | 'k' | 'f' | 's' | 'p' | 'x')
    };

    // Doubled final consonant → single (pakken → pak).
    if n >= 2 && chars[n - 1] == chars[n - 2] && !is_vowel(last) {
        return Some((root[..root.len() - last.len_utf8()].to_string(), voiceless));
    }

    let mut stem = root.to_string();
    // A single final consonant preceded by a single short vowel may
    // double that vowel to keep it long in the closed stem.
    if !is_vowel(last) && n >= 2 && is_vowel(chars[n - 2]) {
        let vowel = chars[n - 2];
        // Only a single vowel letter doubles; a spelled digraph
        // (aa/ee/ei/oe/ui/…) is already long. Two vowels that do not
        // form a digraph (accentu·eren, reag·eren) sit in separate
        // syllables, so the last one still doubles.
        let digraph = n >= 3 && is_digraph(chars[n - 3], chars[n - 2]);
        if !digraph && "aeiou".contains(vowel) && should_double(&chars, vowel, last) {
            stem.insert(root.len() - last.len_utf8(), vowel);
        }
    }

    // Final-devoicing spelling.
    if stem.ends_with('v') {
        stem.truncate(stem.len() - 1);
        stem.push('f');
    } else if stem.ends_with('z') {
        stem.truncate(stem.len() - 1);
        stem.push('s');
    }
    Some((stem, voiceless))
}

/// The recognized Dutch vowel digraphs — already long, so the vowel
/// before a final consonant does not double when it closes one of these.
fn is_digraph(a: char, b: char) -> bool {
    matches!(
        (a, b),
        ('a', 'a')
            | ('e', 'e')
            | ('o', 'o')
            | ('u', 'u')
            | ('i', 'e')
            | ('e', 'i')
            | ('i', 'j')
            | ('o', 'e')
            | ('o', 'u')
            | ('a', 'u')
            | ('e', 'u')
            | ('u', 'i')
            | ('a', 'i')
            | ('o', 'i')
    )
}

/// Whether the single short vowel before the final consonant doubles.
/// `a/o/u/i` always do (`maken → maak`); an `e` doubles except in the
/// native schwa suffixes `-el/-en/-em`, and there only when an earlier
/// vowel already carries the stress. `-er` defaults to doubling
/// (`studeren → studeer`); its schwa members live in the lexicon.
fn should_double(root: &[char], vowel: char, last: char) -> bool {
    if vowel != 'e' {
        return true;
    }
    if matches!(last, 'l' | 'n' | 'm') {
        // Schwa unless the root is monosyllabic (no earlier vowel).
        let e_pos = root.len() - 2;
        return !root[..e_pos].iter().any(|&c| is_vowel(c));
    }
    true
}

/// Append the present `-t`, which is absorbed when the stem already
/// ends in `t` (`praat → praat`) but written after `d` (`rijd → rijdt`).
fn add_t(stem: &str) -> String {
    if stem.ends_with('t') {
        stem.to_string()
    } else {
        format!("{stem}t")
    }
}

/// A conjugatable Dutch verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    stem: String,
    voiceless: bool,
    lex: Option<LexEntry>,
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
    /// Build a verb from its infinitive (the `te` particle is accepted
    /// and stripped). Regular verbs must end in `-en`; every other
    /// shape has to be in the lexicon.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let lowered = infinitive.trim().to_lowercase();
        let mut inf = lowered.as_str();
        if let Some(rest) = inf.strip_prefix("te ") {
            inf = rest.trim();
        }
        if inf.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf
                .chars()
                .all(|c| c.is_alphabetic() || c == '-' || c == '\'' || c == '\u{2019}')
        {
            return Err(Error::NotAVerb);
        }
        let lex = lexicon().get(inf).cloned();
        let derived = derive_stem(inf);
        if lex.is_none() && derived.is_none() {
            return Err(Error::NotAVerb);
        }
        let (dstem, voiceless) = derived.unwrap_or_else(|| (inf.to_string(), true));
        // A lexicon stem override wins; the derived voicing still drives
        // any regular fall-through slot.
        let stem = lex.as_ref().and_then(|l| l.stem.clone()).unwrap_or(dstem);
        // Re-read the voicing from an overridden stem when possible so
        // the weak fall-through matches it (schwa -eren: reageer/…).
        let voiceless = lex
            .as_ref()
            .and_then(|l| l.stem.as_deref())
            .map_or(voiceless, |s| voicing_of(s).unwrap_or(voiceless));
        Ok(Self {
            infinitive: inf.to_string(),
            stem,
            voiceless,
            lex,
        })
    }

    /// The normalized infinitive.
    #[must_use]
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    fn present_regular(&self, i: usize) -> String {
        match i {
            0 => self.stem.clone(),
            1 | 2 => add_t(&self.stem),
            _ => self.infinitive.clone(),
        }
    }

    /// A present-indicative form.
    #[must_use]
    pub fn present(&self, person: Person, number: Number) -> String {
        let i = person.index(number);
        if let Some(p) = self.lex.as_ref().and_then(|l| l.pres.as_ref()) {
            return p[i].clone();
        }
        self.present_regular(i)
    }

    fn past_sg_form(&self) -> String {
        if let Some(f) = self.lex.as_ref().and_then(|l| l.past_sg.as_ref()) {
            return f.clone();
        }
        let ending = if self.voiceless { "te" } else { "de" };
        format!("{}{ending}", self.stem)
    }

    /// The past indicative, singular or plural (Dutch marks number but
    /// not person in the past).
    #[must_use]
    pub fn past(&self, number: Number) -> String {
        match number {
            Number::Singular => self.past_sg_form(),
            Number::Plural => {
                if let Some(f) = self.lex.as_ref().and_then(|l| l.past_pl.as_ref()) {
                    return f.clone();
                }
                format!("{}n", self.past_sg_form())
            }
        }
    }

    /// The singular imperative (the bare stem).
    #[must_use]
    pub fn imperative(&self) -> String {
        if let Some(f) = self.lex.as_ref().and_then(|l| l.imp.as_ref()) {
            return f.clone();
        }
        self.stem.clone()
    }

    /// The present participle (`werkend`, `gaand`).
    #[must_use]
    pub fn present_participle(&self) -> String {
        if let Some(f) = self
            .lex
            .as_ref()
            .and_then(|l| l.present_participle.as_ref())
        {
            return f.clone();
        }
        format!("{}d", self.infinitive)
    }

    /// The past participle (`gewerkt`, `gemaakt`, `bedankt`).
    #[must_use]
    pub fn past_participle(&self) -> String {
        if let Some(f) = self.lex.as_ref().and_then(|l| l.pp.as_ref()) {
            return f.clone();
        }
        let ge = if takes_ge(&self.infinitive) { "ge" } else { "" };
        let suffix = if self.stem.ends_with('t') || self.stem.ends_with('d') {
            ""
        } else if self.voiceless {
            "t"
        } else {
            "d"
        };
        format!("{ge}{}{suffix}", self.stem)
    }
}

/// The *'t kofschip* voicing of an already-formed stem (used to align
/// the weak fall-through with a lexicon stem override).
fn voicing_of(stem: &str) -> Option<bool> {
    let last = stem.chars().last()?;
    if stem.ends_with("ch") {
        return Some(true);
    }
    Some(matches!(last, 't' | 'k' | 'f' | 's' | 'p' | 'x'))
}

/// The full conjugation table of a Dutch verb — shared by the
/// WebAssembly and Python bindings. Present rows are
/// `[ik, jij, hij, wij, jullie, zij]`.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub present: [String; 6],
    /// Past indicative `[singular, plural]`.
    pub past: [String; 2],
    pub imperative: String,
    pub present_participle: String,
    pub past_participle: String,
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
        Self {
            infinitive: v.infinitive().to_string(),
            present: SLOTS.map(|(p, n)| v.present(p, n)),
            past: [v.past(Number::Singular), v.past(Number::Plural)],
            imperative: v.imperative(),
            present_participle: v.present_participle(),
            past_participle: v.past_participle(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn regular_weak() {
        let w = v("werken");
        assert_eq!(w.present(P1, SG), "werk");
        assert_eq!(w.present(P2, SG), "werkt");
        assert_eq!(w.present(P3, SG), "werkt");
        assert_eq!(w.present(P1, PL), "werken");
        assert_eq!(w.past(SG), "werkte");
        assert_eq!(w.past(PL), "werkten");
        assert_eq!(w.imperative(), "werk");
        assert_eq!(w.present_participle(), "werkend");
        assert_eq!(w.past_participle(), "gewerkt");
    }

    #[test]
    fn spelling_rules() {
        // Vowel doubling.
        assert_eq!(v("maken").present(P1, SG), "maak");
        assert_eq!(v("maken").past_participle(), "gemaakt");
        assert_eq!(v("lopen").present(P1, SG), "loop");
        // Consonant doubling collapse.
        assert_eq!(v("pakken").present(P1, SG), "pak");
        assert_eq!(v("zetten").present(P3, SG), "zet");
        // v→f, z→s with voiced past.
        assert_eq!(v("leven").present(P1, SG), "leef");
        assert_eq!(v("leven").past(SG), "leefde");
        assert_eq!(v("leven").past_participle(), "geleefd");
        assert_eq!(v("reizen").present(P1, SG), "reis");
        assert_eq!(v("reizen").past(SG), "reisde");
        // Stem ending in t/d.
        assert_eq!(v("praten").present(P3, SG), "praat");
        assert_eq!(v("praten").past(SG), "praatte");
        assert_eq!(v("praten").past_participle(), "gepraat");
        assert_eq!(v("antwoorden").past_participle(), "geantwoord");
    }

    #[test]
    fn eren_and_schwa() {
        // -eren defaults to doubling.
        assert_eq!(v("studeren").present(P1, SG), "studeer");
        assert_eq!(v("studeren").past_participle(), "gestudeerd");
        // Native schwa suffixes.
        assert_eq!(v("wandelen").present(P1, SG), "wandel");
        assert_eq!(v("rekenen").present(P1, SG), "reken");
        assert_eq!(v("openen").present(P1, SG), "open");
        // Monosyllabic root still doubles.
        assert_eq!(v("delen").present(P1, SG), "deel");
    }

    #[test]
    fn no_ge_prefix() {
        assert_eq!(v("bedanken").past_participle(), "bedankt");
        assert_eq!(v("veranderen").past_participle(), "veranderd");
        assert_eq!(v("geloven").past_participle(), "geloofd");
        // "ver"/"ge" that are not prefixes keep ge-.
        assert_eq!(v("verven").past_participle(), "geverfd");
    }

    #[test]
    fn irregular_lexicon() {
        // Strong verb (ablaut past, -en participle).
        let s = v("schrijven");
        assert_eq!(s.present(P1, SG), "schrijf");
        assert_eq!(s.past(SG), "schreef");
        assert_eq!(s.past(PL), "schreven");
        assert_eq!(s.past_participle(), "geschreven");
        // Suppletive auxiliaries (hand-curated from kaikki).
        assert_eq!(v("zijn").present(P1, SG), "ben");
        assert_eq!(v("zijn").present(P3, SG), "is");
        assert_eq!(v("zijn").past(SG), "was");
        assert_eq!(v("zijn").past_participle(), "geweest");
        assert_eq!(v("hebben").present(P3, SG), "heeft");
        assert_eq!(v("hebben").past_participle(), "gehad");
        // Schwa -eren verb stored as an exception.
        assert_eq!(v("luisteren").present(P1, SG), "luister");
        assert_eq!(v("luisteren").past_participle(), "geluisterd");
    }
}
