//! Zulu (isiZulu) conjugation. Bantu, agglutinative: the finite verb is
//! a slot template — subject concord + tense/aspect marker + ROOT + final
//! vowel — built productively from the bare verb stem, which is the
//! lemma both oracles (UniMorph `zul`, kaikki) key on.
//!
//! The engine is a set of morphophonemic rules over four things derived
//! from the stem:
//!
//! * the **subject concord**, of which there are four series — the plain
//!   concord (`ngi-`, `u-`, `ba-`, `zi-`, …), the subjunctive concord
//!   (plain, but class 1 is `a-`), the participial/relative concord (the
//!   `a`-vowel concords front to `e-`: `ba-`→`be-`, class 1 `u-`→`e-`),
//!   and the remote-past concord, which fuses the concord with the past
//!   `-a-` (`ngi+a`→`nga-`, `u+a`→`wa-`, `lu+a`→`lwa-`);
//! * the **final vowel**, which is `-a` in the present/past indicative
//!   but fronts to `-e` in the subjunctive (`fika`→`fike`);
//! * whether the stem is **monosyllabic** (one vowel: `fa`, `dla`,
//!   `hlwa`), which makes the infinitival `ku-` reappear under the future
//!   (`-zokudla`) and takes the `yi-` imperative (`yifa`);
//! * whether the stem is **vowel-initial** (`enza`, `akha`, `ona`),
//!   which triggers concord–stem coalescence (`ngi+enza`→`ngenza`,
//!   `u+enza`→`wenza`, `uku+enza`→`ukwenza`) and the `y-` imperative
//!   (`yenza`).
//!
//! The one genuinely suppletive verb is `iza` "come", whose citation
//! `i-` augment drops in the conjugation (`ukuza`, not \*`ukwiza`); its
//! real root `za` is supplied by `data/zul/parts.tsv`. Per-cell residue
//! lives in `data/zul/overrides.tsv`.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Grammatical number, for persons and the imperative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Singular,
    Plural,
}

/// The subject the verb agrees with: a grammatical person, or a noun
/// class. Classes 1/2 are the animate person pairs (also 3rd person);
/// 1a/2a and the rest are inanimate/locative. The engine covers the
/// classes UniMorph attests: 1–11, 14, 15, 17.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Subject {
    First(Number),
    Second(Number),
    Class(u8),
}

/// The tense/aspect/mood of a finite form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    /// -ya-/short present.
    Present,
    /// -zo- future.
    Future,
    /// -ile recent past (perfect).
    RecentPast,
    /// be- recent past continuous.
    RecentPastProgressive,
    /// -a- remote past.
    RemotePast,
    /// remote past continuous.
    RemotePastProgressive,
    /// final -e subjunctive.
    Subjunctive,
    /// the participial/relative.
    Participle,
}

/// Why an input cannot be conjugated as a Zulu verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Zulu verb stem or infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Zulu verb")
    }
}

/// The parts row: lemma, then overrides (`-` = derive by rule) for the
/// conjugation root, monosyllabicity and vowel-initiality.
static PARTS_TSV: &str = include_str!("../data/zul/parts.tsv");
/// Per-cell overrides: lemma, canonical features, form.
static OVERRIDES_TSV: &str = include_str!("../data/zul/overrides.tsv");

#[derive(Debug, Clone, Default)]
struct Row {
    root: Option<String>,
    mono: Option<bool>,
    vowel_initial: Option<bool>,
}

fn parts() -> &'static HashMap<&'static str, Row> {
    static MAP: OnceLock<HashMap<&'static str, Row>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in PARTS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() || line.starts_with("lemma\t") {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            let opt = |i: usize| {
                c.get(i)
                    .filter(|s| **s != "-" && !s.is_empty())
                    .map(|s| (*s).to_string())
            };
            let flag = |i: usize| {
                c.get(i).and_then(|s| match *s {
                    "1" => Some(true),
                    "0" => Some(false),
                    _ => None,
                })
            };
            m.insert(
                c[0],
                Row {
                    root: opt(1),
                    mono: flag(2),
                    vowel_initial: flag(3),
                },
            );
        }
        m
    })
}

fn overrides() -> &'static HashMap<(&'static str, &'static str), &'static str> {
    static MAP: OnceLock<HashMap<(&'static str, &'static str), &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in OVERRIDES_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            if c.len() >= 3 {
                m.insert((c[0], c[1]), c[2]);
            }
        }
        m
    })
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// A conjugatable Zulu verb.
#[derive(Debug, Clone)]
pub struct Verb {
    /// The citation stem (`fika`, `enza`, `iza`) — the lemma both oracles
    /// key on.
    lemma: String,
    /// The conjugation root (usually the lemma; `za` for `iza`).
    root: String,
    mono: bool,
    vowel_initial: bool,
}

/// Glue a prefix (concord, TAM marker) to a stem, applying Zulu
/// concord–stem coalescence when the stem is vowel-initial: a prefix-final
/// `-u` glides to `-w` (`uku+enza`→`ukwenza`, `u+enza`→`wenza`); a
/// prefix-final `-a/-e/-i/-o` elides (`ngi+enza`→`ngenza`,
/// `nga+akha`→`ngakha`), except a lone `-i` glides to `y-` (class 4/9
/// `i+enza`→`yenza`). A consonant-initial stem just concatenates.
fn glue(prefix: &str, stem: &str) -> String {
    let Some(first) = stem.chars().next() else {
        return prefix.to_string();
    };
    if !is_vowel(first) {
        return format!("{prefix}{stem}");
    }
    let mut chars = prefix.chars();
    let Some(last) = chars.next_back() else {
        return format!("{prefix}{stem}");
    };
    let head: String = chars.collect();
    match last {
        'u' => format!("{head}w{stem}"),
        'i' if head.is_empty() => format!("y{stem}"),
        'a' | 'e' | 'i' | 'o' => format!("{head}{stem}"),
        _ => format!("{prefix}{stem}"),
    }
}

impl Verb {
    /// Build a verb from its bare stem (`fika`, `enza`) or infinitive
    /// (`ukufika`, `ukwenza`). The `uku-`/`ukw-` infinitive marker is
    /// stripped when it clearly leaves a stem behind.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim().to_lowercase();
        if inf.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf.chars().all(|c| c.is_ascii_alphabetic())
        {
            return Err(Error::NotAVerb);
        }
        // Strip a leading uku-/ukw- infinitive marker: ukwenza → enza,
        // ukufika → fika. A genuine stem is left when the residue is
        // known in parts.tsv or long enough not to be a bare marker.
        let lemma = if let Some(rest) = inf.strip_prefix("ukw") {
            rest.to_string() // ukwenza → enza (the vowel-initial stem)
        } else if let Some(rest) = inf.strip_prefix("uku") {
            if parts().contains_key(rest) || inf.len() >= 6 {
                rest.to_string()
            } else {
                inf.clone()
            }
        } else {
            inf.clone()
        };
        Ok(Self::from_lemma_str(&lemma))
    }

    /// Build directly from the bare lemma (the oracle key).
    pub fn from_lemma(lemma: &str) -> Result<Self, Error> {
        let l = lemma.trim().to_lowercase();
        if l.is_empty()
            || l.contains(char::is_whitespace)
            || !l.chars().all(|c| c.is_ascii_alphabetic())
        {
            return Err(Error::NotAVerb);
        }
        Ok(Self::from_lemma_str(&l))
    }

    fn from_lemma_str(lemma: &str) -> Self {
        let row = parts().get(lemma).cloned().unwrap_or_default();
        let root = row.root.unwrap_or_else(|| lemma.to_string());
        let mono = row
            .mono
            .unwrap_or_else(|| root.chars().filter(|c| is_vowel(*c)).count() == 1);
        let vowel_initial = row
            .vowel_initial
            .unwrap_or_else(|| root.chars().next().is_some_and(is_vowel));
        Self {
            lemma: lemma.to_string(),
            root,
            mono,
            vowel_initial,
        }
    }

    /// The citation stem (lemma).
    #[must_use]
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    /// The subjunctive stem: final `-a` fronts to `-e` (`fika`→`fike`).
    fn subj_stem(&self) -> String {
        if self.root.ends_with('a') {
            format!("{}e", &self.root[..self.root.len() - 1])
        } else {
            self.root.clone()
        }
    }

    /// The infinitive (`ukufika`, `ukwenza`, `ukuza`).
    #[must_use]
    pub fn infinitive(&self) -> String {
        glue("uku", &self.root)
    }

    /// The imperative singular (`fika`, `yenza`, `yifa`).
    #[must_use]
    pub fn imperative(&self, number: Number) -> String {
        let sg = if self.vowel_initial {
            format!("y{}", self.root)
        } else if self.mono {
            format!("yi{}", self.root)
        } else {
            self.root.clone()
        };
        match number {
            Number::Singular => sg,
            Number::Plural => format!("{sg}ni"),
        }
    }

    /// A finite form for a (tense, subject) cell.
    #[must_use]
    pub fn form(&self, tense: Tense, subject: Subject) -> String {
        match tense {
            Tense::Present => glue(plain(subject), &self.root),
            Tense::Participle => glue(participial(subject), &self.root),
            Tense::Subjunctive => glue(subjunctive(subject), &self.subj_stem()),
            Tense::RemotePast => glue(remote(subject), &self.root),
            Tense::Future => {
                let long = if self.vowel_initial || self.mono {
                    glue("ku", &self.root)
                } else {
                    self.root.clone()
                };
                format!("{}zo{long}", plain(subject))
            }
            // The perfect and the two continuous pasts are carried for the
            // Table; the -ile perfect uses the productive rule (imbricated
            // residue is not part of the scored agreement gold).
            Tense::RecentPast => glue(plain(subject), &self.perfect_stem()),
            Tense::RecentPastProgressive => glue(&recent_prog(subject), &self.root),
            Tense::RemotePastProgressive => {
                format!(
                    "{}{}",
                    remote(subject),
                    glue(resumptive(subject), &self.root)
                )
            }
        }
    }

    /// The -ile perfect stem (`fika`→`fikile`), by the productive rule.
    fn perfect_stem(&self) -> String {
        if self.root.ends_with('a') {
            format!("{}ile", &self.root[..self.root.len() - 1])
        } else {
            format!("{}ile", self.root)
        }
    }

    /// The long ("disjoint") present with the -ya- focus marker
    /// (`ngiyafika`, `ngiyenza`).
    #[must_use]
    pub fn present_long(&self, subject: Subject) -> String {
        format!("{}{}", plain(subject), glue("ya", &self.root))
    }

    /// Resolve a canonical feature bundle to its form(s), consulting the
    /// override table first. Returns every accepted variant.
    pub fn generate(&self, features: &str) -> Option<Vec<String>> {
        if let Some(f) = overrides().get(&(self.lemma.as_str(), features)) {
            return Some(vec![(*f).to_string()]);
        }
        let f: Vec<&str> = features.split(';').collect();
        match f.as_slice() {
            ["V", "NFIN"] => Some(vec![self.infinitive()]),
            ["V", "IMP", "2SG"] => Some(vec![self.imperative(Number::Singular)]),
            ["V", "IMP", "2PL"] => Some(vec![self.imperative(Number::Plural)]),
            ["V", subj, tam] => {
                let s = subject(subj)?;
                let t = tense(tam)?;
                if t == Tense::Present {
                    // Both the short and the -ya- long present are attested.
                    Some(vec![self.form(t, s), self.present_long(s)])
                } else {
                    Some(vec![self.form(t, s)])
                }
            }
            _ => None,
        }
    }
}

/// Parse a canonical subject token (1SG, 2PL, CL7).
fn subject(tag: &str) -> Option<Subject> {
    match tag {
        "1SG" => Some(Subject::First(Number::Singular)),
        "2SG" => Some(Subject::Second(Number::Singular)),
        "1PL" => Some(Subject::First(Number::Plural)),
        "2PL" => Some(Subject::Second(Number::Plural)),
        _ => tag
            .strip_prefix("CL")
            .and_then(|n| n.parse::<u8>().ok())
            .map(Subject::Class),
    }
}

fn tense(tag: &str) -> Option<Tense> {
    Some(match tag {
        "PRS" => Tense::Present,
        "FUT" => Tense::Future,
        "RCT_PST" => Tense::RecentPast,
        "RCT_PST_PROG" => Tense::RecentPastProgressive,
        "RMT_PST" => Tense::RemotePast,
        "RMT_PST_PROG" => Tense::RemotePastProgressive,
        "SBJV" => Tense::Subjunctive,
        "PTCP" => Tense::Participle,
        _ => return None,
    })
}

/// Index into the class concord arrays for the attested classes.
fn class_index(c: u8) -> Option<usize> {
    Some(match c {
        1..=11 => (c - 1) as usize,
        14 => 11,
        15 => 12,
        17 => 13,
        _ => return None,
    })
}

/// (plain, subjunctive, participial, remote) concords for the classes,
/// indexed by `class_index`: 1–11, 14, 15, 17.
const CLASS_CONCORD: [(&str, &str, &str, &str); 14] = [
    ("u", "a", "e", "wa"),     // 1
    ("ba", "ba", "be", "ba"),  // 2
    ("u", "u", "u", "wa"),     // 3
    ("i", "i", "i", "ya"),     // 4
    ("li", "li", "li", "la"),  // 5
    ("a", "a", "e", "a"),      // 6
    ("si", "si", "si", "sa"),  // 7
    ("zi", "zi", "zi", "za"),  // 8
    ("i", "i", "i", "ya"),     // 9
    ("zi", "zi", "zi", "za"),  // 10
    ("lu", "lu", "lu", "lwa"), // 11
    ("bu", "bu", "bu", "ba"),  // 14
    ("ku", "ku", "ku", "kwa"), // 15
    ("ku", "ku", "ku", "kwa"), // 17
];

fn person_concord(s: Subject, which: usize) -> Option<&'static str> {
    // (plain, subjunctive, participial, remote) for the four persons.
    let row = match s {
        Subject::First(Number::Singular) => ["ngi", "ngi", "ngi", "nga"],
        Subject::Second(Number::Singular) => ["u", "u", "u", "wa"],
        Subject::First(Number::Plural) => ["si", "si", "si", "sa"],
        Subject::Second(Number::Plural) => ["ni", "ni", "ni", "na"],
        Subject::Class(_) => return None,
    };
    Some(row[which])
}

fn concord(s: Subject, which: usize) -> &'static str {
    if let Some(p) = person_concord(s, which) {
        return p;
    }
    if let Subject::Class(c) = s {
        if let Some(i) = class_index(c) {
            let t = CLASS_CONCORD[i];
            return [t.0, t.1, t.2, t.3][which];
        }
    }
    ""
}

fn plain(s: Subject) -> &'static str {
    concord(s, 0)
}
fn subjunctive(s: Subject) -> &'static str {
    concord(s, 1)
}
fn participial(s: Subject) -> &'static str {
    concord(s, 2)
}
fn remote(s: Subject) -> &'static str {
    concord(s, 3)
}

/// The recent-past continuous prefix (`be-` + concord, but the light
/// concords precede: `bengi-`, `ube-`, `sibe-`, `beku-`).
fn recent_prog(s: Subject) -> String {
    match s {
        Subject::First(Number::Singular) => "bengi".into(),
        Subject::Second(Number::Singular) => "ube".into(),
        Subject::First(Number::Plural) => "sibe".into(),
        Subject::Second(Number::Plural) => "nibe".into(),
        Subject::Class(1) => "ube".into(),
        Subject::Class(6) => "abe".into(),
        Subject::Class(_) => format!("be{}", resumptive(s)),
    }
}

/// The resumptive (full) subject concord used after the remote-past
/// copula in the continuous (`ngangi-`, `wawu-`, `waye-`).
fn resumptive(s: Subject) -> &'static str {
    match s {
        Subject::First(Number::Singular) => "ngi",
        Subject::Second(Number::Singular) => "wu",
        Subject::First(Number::Plural) => "si",
        Subject::Second(Number::Plural) => "ni",
        Subject::Class(c) => match c {
            1 => "ye",
            2 => "be",
            3 => "wu",
            4 | 9 => "yi",
            5 => "li",
            6 => "ye",
            7 => "si",
            8 | 10 => "zi",
            11 => "lu",
            14 => "bu",
            15 | 17 => "ku",
            _ => "",
        },
    }
}

/// The conjugation table of a Zulu verb — the person-based core, for the
/// WebAssembly and Python bindings. Six-slot rows run
/// [1sg, 2sg, 3sg (class 1), 1pl, 2pl, 3pl (class 2)]; the full noun-class
/// matrix is reached through `Verb::form`.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// [singular, plural].
    pub imperative: [String; 2],
    pub present: [String; 6],
    pub present_long: [String; 6],
    pub future: [String; 6],
    pub recent_past: [String; 6],
    pub remote_past: [String; 6],
    pub subjunctive: [String; 6],
    pub participle: [String; 6],
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
        let row = |t: Tense| PERSON_ROW.map(|s| v.form(t, s));
        Self {
            infinitive: v.infinitive(),
            imperative: [v.imperative(Number::Singular), v.imperative(Number::Plural)],
            present: row(Tense::Present),
            present_long: PERSON_ROW.map(|s| v.present_long(s)),
            future: row(Tense::Future),
            recent_past: row(Tense::RecentPast),
            remote_past: row(Tense::RemotePast),
            subjunctive: row(Tense::Subjunctive),
            participle: row(Tense::Participle),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Verb {
        Verb::from_lemma(s).unwrap()
    }

    #[test]
    fn regular_fika() {
        let f = v("fika");
        assert_eq!(f.infinitive(), "ukufika");
        assert_eq!(f.imperative(Number::Singular), "fika");
        assert_eq!(f.imperative(Number::Plural), "fikani");
        let sg = Subject::First(Number::Singular);
        assert_eq!(f.form(Tense::Subjunctive, sg), "ngifike");
        assert_eq!(f.form(Tense::RemotePast, sg), "ngafika");
        assert_eq!(f.form(Tense::Present, sg), "ngifika");
        assert_eq!(f.present_long(sg), "ngiyafika");
        assert_eq!(f.form(Tense::Subjunctive, Subject::Class(1)), "afike");
        assert_eq!(f.form(Tense::Participle, Subject::Class(2)), "befika");
        assert_eq!(f.form(Tense::RemotePast, Subject::Class(11)), "lwafika");
    }

    #[test]
    fn vowel_initial_enza() {
        let e = v("enza");
        assert_eq!(e.infinitive(), "ukwenza");
        assert_eq!(e.imperative(Number::Singular), "yenza");
        assert_eq!(e.imperative(Number::Plural), "yenzani");
        let sg = Subject::First(Number::Singular);
        assert_eq!(e.form(Tense::Subjunctive, sg), "ngenze");
        assert_eq!(e.form(Tense::RemotePast, sg), "ngenza");
        assert_eq!(e.form(Tense::Present, sg), "ngenza");
        assert_eq!(
            e.form(Tense::Subjunctive, Subject::Second(Number::Singular)),
            "wenze"
        );
        assert_eq!(e.form(Tense::Future, sg), "ngizokwenza");
    }

    #[test]
    fn monosyllabic_fa() {
        let f = v("fa");
        assert_eq!(f.infinitive(), "ukufa");
        assert_eq!(f.imperative(Number::Singular), "yifa");
        assert_eq!(f.imperative(Number::Plural), "yifani");
        let sg = Subject::First(Number::Singular);
        assert_eq!(f.form(Tense::Subjunctive, sg), "ngife");
        assert_eq!(f.form(Tense::RemotePast, sg), "ngafa");
        assert_eq!(f.form(Tense::Future, sg), "ngizokufa");
    }

    #[test]
    fn suppletive_iza() {
        let i = v("iza");
        assert_eq!(i.infinitive(), "ukuza");
        assert_eq!(i.imperative(Number::Singular), "yiza");
        let sg = Subject::First(Number::Singular);
        assert_eq!(i.form(Tense::Subjunctive, sg), "ngize");
        assert_eq!(
            i.form(Tense::Subjunctive, Subject::Second(Number::Singular)),
            "uze"
        );
        assert_eq!(i.form(Tense::RemotePast, sg), "ngaza");
    }

    #[test]
    fn infinitive_round_trip() {
        assert_eq!(Verb::from_infinitive("ukufika").unwrap().lemma(), "fika");
        assert_eq!(Verb::from_infinitive("ukwenza").unwrap().lemma(), "enza");
        assert_eq!(Verb::from_infinitive("fika").unwrap().lemma(), "fika");
        assert!(Verb::from_infinitive("").is_err());
        assert!(Verb::from_infinitive("two words").is_err());
    }
}
