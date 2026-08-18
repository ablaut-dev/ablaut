//! Romanian conjugation. Five theme classes by infinitive ending
//! (-a, -ea, -e, -i, -î); the productive presents take an augment
//! (lucrez, vorbesc, hotărăsc) and are the rule-side defaults, while
//! bare presents (cânt, dorm, cobor) and the -e/-ea classes carry
//! mined class assignments in `data/ron/classes.tsv` or, where vowel
//! alternations defeat the rules, full mined rows in
//! `data/ron/verbs.tsv`.
//!
//! The past tenses derive: the imperfect from the theme (vorbeam,
//! cântam), the simple perfect from the participle (vorbit → vorbii,
//! mers → mersei), the pluperfect from the perfect base + se
//! (vorbisem). The subjunctive differs from the indicative only in
//! the third person (să vorbească), where e→ea / o→oa
//! diphthongization applies (merg → meargă, dorm → doarmă).

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

/// The synthetic indicative tenses (the future is analytic: voi
/// vorbi).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    Present,
    Imperfect,
    /// The perfect simple (vorbii).
    SimplePerfect,
    /// The synthetic pluperfect (vorbisem).
    Pluperfect,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Romanian infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Romanian infinitive")
    }
}

/// Present-tense shape. The augmented classes are the productive
/// defaults per theme; bare classes are mined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// -a with -ez augment (lucrez, lucrezi, lucrează).
    AEz,
    /// -a bare (cânt, cânți, cântă).
    ABare,
    /// -ea (văd, vezi, vede — presents are mostly mined rows).
    Ea,
    /// -e (merg, mergi, merge).
    E,
    /// -i with -esc augment (vorbesc, vorbești, vorbește).
    IEsc,
    /// -i bare (dorm, dormi, doarme).
    IBare,
    /// -î with -ăsc augment (hotărăsc).
    IhAsc,
    /// -î bare (cobor, cobori, coboară).
    IhBare,
}

/// Mined class assignments for verbs whose bare present the rules
/// cover (lemma \t class).
static CLASSES_TSV: &str = include_str!("../data/ron/classes.tsv");

/// Mined full rows for verbs the class rules cannot reach: lemma,
/// present 1sg…3pl, subjunctive 3, imperative 2sg, participle,
/// gerund, imperfect 1sg, perfect 3sg ("-" = derive).
static VERBS_TSV: &str = include_str!("../data/ron/verbs.tsv");

fn classes() -> &'static HashMap<&'static str, Class> {
    static MAP: OnceLock<HashMap<&'static str, Class>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in CLASSES_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let mut f = line.split('\t');
            let (Some(lemma), Some(class)) = (f.next(), f.next()) else {
                continue;
            };
            let class = match class {
                "a-ez" => Class::AEz,
                "a-bare" => Class::ABare,
                "ea" => Class::Ea,
                "e" => Class::E,
                "i-esc" => Class::IEsc,
                "i-bare" => Class::IBare,
                "ih-asc" => Class::IhAsc,
                "ih-bare" => Class::IhBare,
                _ => continue,
            };
            m.insert(lemma, class);
        }
        m
    })
}

#[derive(Debug, Clone, Default)]
struct Row {
    present: [Option<String>; 6],
    sbjv3: Option<String>,
    imp2: Option<String>,
    participle: Option<String>,
    gerund: Option<String>,
    imperfect1: Option<String>,
    perfect3: Option<String>,
}

fn rows() -> &'static HashMap<&'static str, Row> {
    static MAP: OnceLock<HashMap<&'static str, Row>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in VERBS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            let opt = |i: usize| {
                cols.get(i)
                    .filter(|c| **c != "-" && !c.is_empty())
                    .map(|c| (*c).to_string())
            };
            let row = Row {
                present: [opt(1), opt(2), opt(3), opt(4), opt(5), opt(6)],
                sbjv3: opt(7),
                imp2: opt(8),
                participle: opt(9),
                gerund: opt(10),
                imperfect1: opt(11),
                perfect3: opt(12),
            };
            m.insert(cols[0], row);
        }
        m
    })
}

/// The infinitive theme vowel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {
    A,
    Ea,
    E,
    I,
    Ih,
}

/// t→ț, d→z, s→ș (and st→șt) before the 2sg -i: cânt → cânți,
/// cred → crezi, las → lași, gust → guști.
fn palatalize(stem: &str) -> String {
    if let Some(base) = stem.strip_suffix("st") {
        return format!("{base}șt");
    }
    if let Some(base) = stem.strip_suffix('t') {
        return format!("{base}ț");
    }
    if let Some(base) = stem.strip_suffix('d') {
        return format!("{base}z");
    }
    if let Some(base) = stem.strip_suffix('s') {
        return format!("{base}ș");
    }
    stem.to_string()
}

/// e→ea / o→oa on the stem's last vowel, the stressed-syllable
/// diphthongization before -ă/-e endings: merg → meargă, dorm →
/// doarmă, plec → pleacă.
fn diphthongize(stem: &str) -> String {
    let chars: Vec<char> = stem.chars().collect();
    for i in (0..chars.len()).rev() {
        match chars[i] {
            'e' => {
                let mut out: String = chars[..i].iter().collect();
                out.push_str("ea");
                out.extend(&chars[i + 1..]);
                return out;
            }
            'o' => {
                let mut out: String = chars[..i].iter().collect();
                out.push_str("oa");
                out.extend(&chars[i + 1..]);
                return out;
            }
            c if "aăâîiu".contains(c) => return stem.to_string(),
            _ => {}
        }
    }
    stem.to_string()
}

/// A conjugatable Romanian verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    theme: Theme,
    class: Class,
    row: Row,
}

impl Verb {
    /// Build a verb from its infinitive (without the particle: vorbi,
    /// not a vorbi — the particle is accepted and stripped).
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        // Case-fold: lemmas are lowercase in every oracle of this
        // language (the Abbrechen lesson, generalized).
        let lowered = infinitive.to_lowercase();
        let infinitive = lowered.as_str();
        let mut inf = infinitive.trim();
        if let Some(rest) = inf.strip_prefix("a ") {
            inf = rest.trim();
        }
        if inf.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf.chars().all(|c| c.is_alphabetic() || c == '-')
        {
            return Err(Error::NotAVerb);
        }
        let theme = if inf.ends_with("ea") {
            Theme::Ea
        } else if inf.ends_with('a') {
            Theme::A
        } else if inf.ends_with('e') {
            Theme::E
        } else if inf.ends_with('i') {
            Theme::I
        } else if inf.ends_with('î') {
            Theme::Ih
        } else {
            return Err(Error::NotAVerb);
        };
        let class = classes().get(inf).copied().unwrap_or(match theme {
            Theme::A => Class::AEz,
            Theme::Ea => Class::Ea,
            Theme::E => Class::E,
            Theme::I => Class::IEsc,
            Theme::Ih => Class::IhAsc,
        });
        let row = rows().get(inf).cloned().unwrap_or_default();
        Ok(Self {
            infinitive: inf.to_string(),
            theme,
            class,
            row,
        })
    }

    /// The infinitive as normalized (particle stripped).
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// The stem: infinitive minus the theme vowel(s).
    fn stem(&self) -> &str {
        let n = match self.theme {
            Theme::Ea => 2,
            Theme::A | Theme::E | Theme::I => 1,
            Theme::Ih => "î".len(),
        };
        &self.infinitive[..self.infinitive.len() - n]
    }

    fn idx(person: Person, number: Number) -> usize {
        let p = match person {
            Person::First => 0,
            Person::Second => 1,
            Person::Third => 2,
        };
        p + if number == Number::Plural { 3 } else { 0 }
    }

    /// The present indicative.
    fn present(&self, person: Person, number: Number) -> String {
        if let Some(f) = &self.row.present[Self::idx(person, number)] {
            return f.clone();
        }
        let s = self.stem();
        let vowel_stem = s.ends_with(|c| "aeiou".contains(c));
        match (self.class, number, person) {
            // The plural endings are class-independent per theme.
            (Class::AEz | Class::ABare, Number::Plural, Person::First) => {
                if vowel_stem {
                    format!("{s}em")
                } else {
                    format!("{s}ăm")
                }
            }
            (Class::AEz | Class::ABare, Number::Plural, Person::Second) => format!("{s}ați"),
            (Class::Ea | Class::E, Number::Plural, Person::First) => format!("{s}em"),
            (Class::Ea | Class::E, Number::Plural, Person::Second) => format!("{s}eți"),
            (Class::IEsc | Class::IBare, Number::Plural, Person::First) => format!("{s}im"),
            (Class::IEsc | Class::IBare, Number::Plural, Person::Second) => format!("{s}iți"),
            (Class::IhAsc | Class::IhBare, Number::Plural, Person::First) => format!("{s}âm"),
            (Class::IhAsc | Class::IhBare, Number::Plural, Person::Second) => format!("{s}âți"),
            // -ez: c/g take the orthographic h (marchez).
            (Class::AEz, _, _) => {
                let aug = if s.ends_with('c') || s.ends_with('g') {
                    format!("{s}h")
                } else {
                    s.to_string()
                };
                match (number, person) {
                    (Number::Singular, Person::First) => format!("{aug}ez"),
                    (Number::Singular, Person::Second) => format!("{aug}ezi"),
                    _ => format!("{aug}ează"),
                }
            }
            (Class::IEsc, _, _) => match (number, person) {
                (Number::Singular, Person::First) | (Number::Plural, Person::Third) => {
                    format!("{s}esc")
                }
                (Number::Singular, Person::Second) => format!("{s}ești"),
                _ => format!("{s}ește"),
            },
            (Class::IhAsc, _, _) => match (number, person) {
                (Number::Singular, Person::First) | (Number::Plural, Person::Third) => {
                    format!("{s}ăsc")
                }
                (Number::Singular, Person::Second) => format!("{s}ăști"),
                _ => format!("{s}ăște"),
            },
            (Class::ABare, Number::Singular, Person::First) => s.to_string(),
            (Class::ABare, Number::Singular, Person::Second) => format!("{}i", palatalize(s)),
            (Class::ABare, _, Person::Third) => format!("{}ă", diphthongize(s)),
            (Class::Ea | Class::E, Number::Singular, Person::First)
            | (Class::Ea | Class::E, Number::Plural, Person::Third) => s.to_string(),
            (Class::Ea | Class::E, Number::Singular, Person::Second) => {
                format!("{}i", palatalize(s))
            }
            (Class::Ea | Class::E, Number::Singular, Person::Third) => format!("{s}e"),
            (Class::IBare, Number::Singular, Person::First)
            | (Class::IBare, Number::Plural, Person::Third) => s.to_string(),
            (Class::IBare, Number::Singular, Person::Second) => format!("{}i", palatalize(s)),
            (Class::IBare, Number::Singular, Person::Third) => format!("{}e", diphthongize(s)),
            (Class::IhBare, Number::Singular, Person::First) => s.to_string(),
            (Class::IhBare, Number::Singular, Person::Second) => format!("{}i", palatalize(s)),
            (Class::IhBare, _, Person::Third) => format!("{}ă", diphthongize(s)),
        }
    }

    /// The imperfect: theme vowel + am/ai/a/am/ați/au (vorbeam,
    /// cântam, eram).
    fn imperfect(&self, person: Person, number: Number) -> String {
        let base = match &self.row.imperfect1 {
            Some(f) => f.strip_suffix('m').unwrap_or(f).to_string(),
            None => {
                let s = self.stem();
                match self.theme {
                    Theme::A | Theme::Ea => format!("{}a", self.infinitive.trim_end_matches('a')),
                    // Vowel stems keep the glide: bubuia, suia, scria
                    // — but -ii collapses (prii → pria).
                    Theme::E | Theme::I => {
                        if s.ends_with(|c| "aeiouăâî".contains(c)) {
                            let base = self.infinitive.trim_end_matches('e');
                            format!(
                                "{}a",
                                base.strip_suffix("ii")
                                    .map_or(base, |b| { &self.infinitive[..b.len() + 1] })
                            )
                        } else {
                            format!("{s}ea")
                        }
                    }
                    Theme::Ih => format!("{s}a"),
                }
            }
        };
        match (number, person) {
            (Number::Singular, Person::First) => format!("{base}m"),
            (Number::Singular, Person::Second) => format!("{base}i"),
            (Number::Singular, Person::Third) => base,
            (Number::Plural, Person::First) => format!("{base}m"),
            (Number::Plural, Person::Second) => format!("{base}ți"),
            (Number::Plural, Person::Third) => format!("{base}u"),
        }
    }

    /// The perfect base: the 3sg minus a final -ă normalized back to
    /// -a (cântă → cânta), otherwise the 3sg itself (vorbi, merse).
    fn perfect_base(&self) -> String {
        let p3 = match &self.row.perfect3 {
            Some(f) => f.clone(),
            None => {
                let ptcp = self.participle();
                if let Some(base) = ptcp.strip_suffix('t') {
                    match self.theme {
                        Theme::A => format!("{}ă", &base[..base.len() - 1]),
                        _ => base.to_string(),
                    }
                } else {
                    // Strong -s participles: mers → merse.
                    format!("{ptcp}e")
                }
            }
        };
        if let Some(base) = p3.strip_suffix('ă') {
            format!("{base}a")
        } else if p3.ends_with("se") {
            // Strong perfects keep their base (merse, rămase, vruse).
            p3
        } else if p3.ends_with("che") || p3.ends_with("ghe") {
            // -chea/-ghea verbs: 3sg veghe, base veghea- (veghearăm).
            format!("{p3}a")
        } else if matches!(self.theme, Theme::A | Theme::Ea) && p3.ends_with('e') {
            // -ia verbs: 3sg detalie, but the base restores the theme
            // (detaliai).
            format!("{}a", &p3[..p3.len() - 1])
        } else if let Some(base) = p3.strip_suffix('î') {
            // 3sg coborî, base coborâ- (coborârăm).
            format!("{base}â")
        } else {
            p3
        }
    }

    /// Strong -se perfects reduce their stressed vowel off the 3sg:
    /// trase → trăsei, coapse → copsei, rămase → rămăsesem. Weak
    /// bases pass through unchanged.
    fn reduce(base: &str) -> String {
        let Some(head) = base.strip_suffix("se") else {
            return base.to_string();
        };
        // Only the final vowel group of the head reduces (trase,
        // coapse — but aduse keeps its earlier a untouched).
        let vowels = "aăâeiîou";
        let chars: Vec<char> = head.chars().collect();
        let mut end = chars.len();
        while end > 0 && !vowels.contains(chars[end - 1]) {
            end -= 1;
        }
        let mut start = end;
        while start > 0 && vowels.contains(chars[start - 1]) {
            start -= 1;
        }
        if start == 0 {
            // Word-initial vowels stay stressed: arse → arsei.
            return base.to_string();
        }
        let group: String = chars[start..end].iter().collect();
        let reduced = match group.as_str() {
            "a" => "ă",
            "oa" => "o",
            _ => return base.to_string(),
        };
        let pre: String = chars[..start].iter().collect();
        let post: String = chars[end..].iter().collect();
        format!("{pre}{reduced}{post}se")
    }

    /// The perfect simple (vorbii, cântai, mersei).
    fn simple_perfect(&self, person: Person, number: Number) -> String {
        let base = self.perfect_base();
        match (number, person) {
            (Number::Singular, Person::First) => format!("{}i", Self::reduce(&base)),
            (Number::Singular, Person::Second) => format!("{}și", Self::reduce(&base)),
            (Number::Singular, Person::Third) => match &self.row.perfect3 {
                Some(f) => f.clone(),
                None if self.theme == Theme::A => {
                    format!("{}ă", &base[..base.len() - 1])
                }
                // -î verbs restore the theme vowel: hotărâi but hotărî.
                None if self.theme == Theme::Ih => format!("{}î", self.stem()),
                None => base,
            },
            (Number::Plural, Person::First) => format!("{base}răm"),
            (Number::Plural, Person::Second) => format!("{base}răți"),
            (Number::Plural, Person::Third) => format!("{base}ră"),
        }
    }

    /// The pluperfect (vorbisem): perfect base + se + endings.
    fn pluperfect(&self, person: Person, number: Number) -> String {
        // fi doubles the marker: fusesem, not *fusem.
        let base = if self.infinitive == "fi" {
            "fusese".to_string()
        } else {
            format!("{}se", Self::reduce(&self.perfect_base()))
        };
        match (number, person) {
            (Number::Singular, Person::First) => format!("{base}m"),
            (Number::Singular, Person::Second) => format!("{base}și"),
            (Number::Singular, Person::Third) => base,
            (Number::Plural, Person::First) => format!("{base}răm"),
            (Number::Plural, Person::Second) => format!("{base}răți"),
            (Number::Plural, Person::Third) => format!("{base}ră"),
        }
    }

    /// An indicative form.
    pub fn indicative(&self, tense: Tense, person: Person, number: Number) -> String {
        match tense {
            Tense::Present => self.present(person, number),
            Tense::Imperfect => self.imperfect(person, number),
            Tense::SimplePerfect => self.simple_perfect(person, number),
            Tense::Pluperfect => self.pluperfect(person, number),
        }
    }

    /// The present subjunctive (să vorbesc … să vorbească): identical
    /// to the indicative outside the third person.
    pub fn subjunctive(&self, person: Person, number: Number) -> String {
        // fi is suppletive: the subjunctive leaves the sunt- stem
        // entirely (să fiu, să fii, să fim, să fiți).
        if self.infinitive == "fi" {
            return match (number, person) {
                (Number::Singular, Person::First) => "fiu",
                (Number::Singular, Person::Second) => "fii",
                (Number::Plural, Person::First) => "fim",
                (Number::Plural, Person::Second) => "fiți",
                (_, Person::Third) => "fie",
            }
            .to_string();
        }
        if person != Person::Third {
            return self.present(person, number);
        }
        if let Some(f) = &self.row.sbjv3 {
            return f.clone();
        }
        let s = self.stem();
        match self.class {
            Class::AEz => {
                let aug = if s.ends_with('c') || s.ends_with('g') {
                    format!("{s}h")
                } else {
                    s.to_string()
                };
                format!("{aug}eze")
            }
            Class::IEsc => format!("{s}ească"),
            Class::IhAsc => format!("{s}ască"),
            Class::ABare => format!("{}e", diphthongize(s)),
            Class::Ea | Class::E | Class::IBare | Class::IhBare => {
                format!("{}ă", diphthongize(s))
            }
        }
    }

    /// The imperative: 2pl = present 2pl; 2sg defaults to the present
    /// 3sg (vorbește!, lucrează!), deviants are mined (mergi!, du!).
    pub fn imperative(&self, number: Number) -> String {
        if self.infinitive == "fi" {
            return match number {
                Number::Singular => "fii".to_string(),
                Number::Plural => "fiți".to_string(),
            };
        }
        match number {
            Number::Plural => self.present(Person::Second, Number::Plural),
            Number::Singular => match &self.row.imp2 {
                Some(f) => f.clone(),
                None => self.present(Person::Third, Number::Singular),
            },
        }
    }

    /// The past participle, masculine singular (vorbit, cântat, mers).
    pub fn participle(&self) -> String {
        if let Some(f) = &self.row.participle {
            return f.clone();
        }
        match self.theme {
            Theme::A | Theme::I => format!("{}t", self.infinitive),
            Theme::Ih => format!("{}ât", self.stem()),
            Theme::Ea | Theme::E => format!("{}ut", self.stem()),
        }
    }

    /// The gerund (vorbind, cântând, mergând).
    pub fn gerund(&self) -> String {
        if let Some(f) = &self.row.gerund {
            return f.clone();
        }
        let s = self.stem();
        match self.theme {
            Theme::I => format!("{}nd", self.infinitive),
            _ if s.ends_with('i') => format!("{s}ind"),
            _ => format!("{s}ând"),
        }
    }
}

/// The full conjugation table of a Romanian verb — shared by the
/// WebAssembly and Python bindings. Six-slot rows run 1sg…3pl.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub gerund: String,
    pub participle: String,
    /// [2sg, 2pl].
    pub imperative: [String; 2],
    pub present: [String; 6],
    pub imperfect: [String; 6],
    pub simple_perfect: [String; 6],
    pub pluperfect: [String; 6],
    pub subjunctive: [String; 6],
    pub perfect_compus: [String; 6],
    pub viitor: [String; 6],
    pub conditional_prezent: [String; 6],
    pub subjunctive_perfect: [String; 6],
    pub conditional_perfect: [String; 6],
    pub viitor_anterior: [String; 6],
}

/// Auxiliary rows for the analytic tenses: a avea (am vorbit),
/// a vrea (voi vorbi) and the conditional clitics (aș vorbi).
const AVEA: [&str; 6] = ["am", "ai", "a", "am", "ați", "au"];
const VREA: [&str; 6] = ["voi", "vei", "va", "vom", "veți", "vor"];
const COND: [&str; 6] = ["aș", "ai", "ar", "am", "ați", "ar"];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row = |f: &dyn Fn(Person, Number) -> String| {
            [
                f(Person::First, Number::Singular),
                f(Person::Second, Number::Singular),
                f(Person::Third, Number::Singular),
                f(Person::First, Number::Plural),
                f(Person::Second, Number::Plural),
                f(Person::Third, Number::Plural),
            ]
        };
        Self {
            infinitive: v.infinitive().to_string(),
            gerund: v.gerund(),
            participle: v.participle(),
            imperative: [v.imperative(Number::Singular), v.imperative(Number::Plural)],
            present: row(&|p, n| v.indicative(Tense::Present, p, n)),
            imperfect: row(&|p, n| v.indicative(Tense::Imperfect, p, n)),
            simple_perfect: row(&|p, n| v.indicative(Tense::SimplePerfect, p, n)),
            pluperfect: row(&|p, n| v.indicative(Tense::Pluperfect, p, n)),
            subjunctive: row(&|p, n| v.subjunctive(p, n)),
            perfect_compus: AVEA.map(|a| format!("{a} {}", v.participle())),
            viitor: VREA.map(|a| format!("{a} {}", v.infinitive())),
            conditional_prezent: COND.map(|a| format!("{a} {}", v.infinitive())),
            subjunctive_perfect: std::array::from_fn(|_| format!("fi {}", v.participle())),
            conditional_perfect: COND.map(|a| format!("{a} fi {}", v.participle())),
            viitor_anterior: VREA.map(|a| format!("{a} fi {}", v.participle())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn analytic_tenses() {
        let t = Table::build(&v("vorbi"));
        assert_eq!(t.perfect_compus[0], "am vorbit");
        assert_eq!(t.perfect_compus[4], "ați vorbit");
        assert_eq!(t.viitor[0], "voi vorbi");
        assert_eq!(t.viitor[5], "vor vorbi");
        assert_eq!(t.conditional_prezent[0], "aș vorbi");
        assert_eq!(t.conditional_prezent[2], "ar vorbi");
        assert_eq!(t.subjunctive_perfect[3], "fi vorbit");
        assert_eq!(t.conditional_perfect[0], "aș fi vorbit");
        assert_eq!(t.viitor_anterior[2], "va fi vorbit");
    }

    #[test]
    fn productive_defaults() {
        let l = v("lucra");
        assert_eq!(l.present(Person::First, Number::Singular), "lucrez");
        assert_eq!(l.present(Person::Third, Number::Singular), "lucrează");
        assert_eq!(l.subjunctive(Person::Third, Number::Singular), "lucreze");
        let w = v("vorbi");
        assert_eq!(w.present(Person::First, Number::Singular), "vorbesc");
        assert_eq!(w.present(Person::Second, Number::Singular), "vorbești");
        assert_eq!(w.subjunctive(Person::Third, Number::Plural), "vorbească");
        assert_eq!(w.imperative(Number::Singular), "vorbește");
        assert_eq!(w.participle(), "vorbit");
        assert_eq!(w.gerund(), "vorbind");
    }

    #[test]
    fn derived_tenses() {
        let w = v("vorbi");
        assert_eq!(
            w.indicative(Tense::Imperfect, Person::First, Number::Singular),
            "vorbeam"
        );
        assert_eq!(
            w.indicative(Tense::SimplePerfect, Person::First, Number::Singular),
            "vorbii"
        );
        assert_eq!(
            w.indicative(Tense::SimplePerfect, Person::Third, Number::Singular),
            "vorbi"
        );
        assert_eq!(
            w.indicative(Tense::Pluperfect, Person::First, Number::Singular),
            "vorbisem"
        );
        let h = v("hotărî");
        assert_eq!(h.present(Person::First, Number::Singular), "hotărăsc");
        assert_eq!(h.participle(), "hotărât");
        assert_eq!(h.gerund(), "hotărând");
        assert_eq!(
            h.indicative(Tense::Imperfect, Person::Third, Number::Singular),
            "hotăra"
        );
    }

    #[test]
    fn particle_stripped() {
        assert_eq!(v("a vorbi").infinitive(), "vorbi");
        assert!(Verb::from_infinitive("xyz123").is_err());
    }
}
