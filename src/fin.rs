//! Finnish conjugation. Type-I vowel verbs without gradation run on
//! rules (puhua → puhun/puhuu/puhui/puhunut/puhutaan); consonant
//! gradation and the other infinitive types carry mined rows in
//! `data/fin/verbs.tsv`. Everything else derives: the conditional
//! from the strong stem with e/i-drop (lukee → lukisi), the potential
//! and the imperative koon-set from the nut-participle (tullut →
//! tullee, tulkoon), the passive family from the two passive bases
//! (puhutaan, puhuttiin → puhuttaisiin/puhuttaneen/puhuttakoon/
//! puhuttu), with vowel harmony throughout.

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

/// The synthetic moods/tenses of the active voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    Present,
    Past,
    Conditional,
    Potential,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Finnish infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Finnish infinitive")
    }
}

/// Mined row: lemma, present 1sg, present 3sg, past 1sg, past 3sg,
/// nut-participle, passive present, passive past, imperative 2sg
/// ("-" = type-I default).
static VERBS_TSV: &str = include_str!("../data/fin/verbs.tsv");

#[derive(Debug, Clone, Default)]
struct Row {
    p1sg: Option<String>,
    p3sg: Option<String>,
    pst1sg: Option<String>,
    pst3sg: Option<String>,
    nut: Option<String>,
    pass_prs: Option<String>,
    pass_pst: Option<String>,
    imp2: Option<String>,
    p3pl: Option<String>,
    imp_pl2: Option<String>,
    cond3: Option<String>,
    pot3: Option<String>,
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
            m.insert(
                cols[0],
                Row {
                    p1sg: opt(1),
                    p3sg: opt(2),
                    pst1sg: opt(3),
                    pst3sg: opt(4),
                    nut: opt(5),
                    pass_prs: opt(6),
                    pass_pst: opt(7),
                    imp2: opt(8),
                    p3pl: opt(9),
                    imp_pl2: opt(10),
                    cond3: opt(11),
                    pot3: opt(12),
                },
            );
        }
        m
    })
}

/// A conjugatable Finnish verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    row: Row,
}

impl Verb {
    /// Build a verb from its (first) infinitive.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        // Case-fold: lemmas are lowercase in every oracle of this
        // language (the Abbrechen lesson, generalized).
        let lowered = infinitive.to_lowercase();
        let infinitive = lowered.as_str();
        let inf = infinitive.trim();
        if inf.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf.chars().all(|c| c.is_alphabetic() || c == '-')
        {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf.to_string(),
            row: rows().get(inf).cloned().unwrap_or_default(),
        })
    }

    /// The infinitive as normalized.
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// Front-vowel harmony. A mined nut-participle settles it
    /// outright (-nut back, -nyt front) — compounds with neutral
    /// final stems (aivopestä → aivopessyt → -vät) need that;
    /// otherwise the last non-neutral vowel of the lemma decides.
    fn front(&self) -> bool {
        if let Some(nut) = &self.row.nut {
            if nut.ends_with("yt") {
                return true;
            }
            if nut.ends_with("ut") {
                return false;
            }
        }
        for c in self.infinitive.chars().rev() {
            match c {
                'a' | 'o' | 'u' => return false,
                'ä' | 'ö' | 'y' => return true,
                _ => {}
            }
        }
        true
    }

    fn h(&self, back: &str, front: &str) -> String {
        if self.front() { front } else { back }.to_string()
    }

    /// The type-I stem: infinitive minus the final vowel.
    fn stem(&self) -> &str {
        let inf = &self.infinitive;
        let n = inf.chars().last().map_or(0, char::len_utf8);
        &inf[..inf.len() - n]
    }

    /// The weak present stem (p1sg minus -n).
    fn weak(&self) -> String {
        match &self.row.p1sg {
            Some(f) => f.strip_suffix('n').unwrap_or(f).to_string(),
            None => self.stem().to_string(),
        }
    }

    /// The strong present stem: 3pl minus -vat/-vät when mined
    /// (nyhjäävät → nyhjää), else 3sg minus its final vowel.
    fn strong(&self) -> String {
        if let Some(p) = &self.row.p3pl {
            let b = p
                .strip_suffix("vat")
                .or_else(|| p.strip_suffix("vät"))
                .unwrap_or(p);
            return b.to_string();
        }
        let p3 = self.active(Tense::Present, Person::Third, Number::Singular);
        let n = p3.chars().last().map_or(0, char::len_utf8);
        p3[..p3.len() - n].to_string()
    }

    /// The nut-participle (puhunut, tullut).
    pub fn nut(&self) -> String {
        self.row
            .nut
            .clone()
            .unwrap_or_else(|| format!("{}{}", self.stem(), self.h("nut", "nyt")))
    }

    /// The passive bases: present (puhutaan) and past (puhuttiin).
    fn pass_prs(&self) -> String {
        self.row
            .pass_prs
            .clone()
            .unwrap_or_else(|| format!("{}{}", self.stem(), self.h("taan", "tään")))
    }

    fn pass_pst(&self) -> String {
        self.row
            .pass_pst
            .clone()
            .unwrap_or_else(|| format!("{}ttiin", self.stem()))
    }

    /// The t-base of the past passive (puhutt-), which drives the
    /// passive conditional, potential, imperative and tu-participle.
    fn pass_base(&self) -> String {
        let p = self.pass_pst();
        p.strip_suffix("iin").unwrap_or(&p).to_string()
    }

    /// An active indicative/conditional/potential form.
    pub fn active(&self, tense: Tense, person: Person, number: Number) -> String {
        let (base, endings): (String, [&str; 6]) = match tense {
            Tense::Present => {
                let w = self.weak();
                match (number, person) {
                    (Number::Singular, Person::First) => return format!("{w}n"),
                    (Number::Singular, Person::Second) => return format!("{w}t"),
                    (Number::Singular, Person::Third) => {
                        return self.row.p3sg.clone().unwrap_or_else(|| {
                            let s = self.stem();
                            let last = s.chars().last().unwrap_or('a');
                            format!("{s}{last}")
                        })
                    }
                    (Number::Plural, Person::First) => return format!("{w}mme"),
                    (Number::Plural, Person::Second) => return format!("{w}tte"),
                    (Number::Plural, Person::Third) => {
                        let s = self.strong();
                        return format!("{s}{}", self.h("vat", "vät"));
                    }
                }
            }
            Tense::Past => {
                let b1 = match &self.row.pst1sg {
                    Some(f) => f.strip_suffix('n').unwrap_or(f).to_string(),
                    None => format!("{}i", self.stem()),
                };
                let b3 = self.row.pst3sg.clone().unwrap_or_else(|| b1.clone());
                return match (number, person) {
                    (Number::Singular, Person::First) => format!("{b1}n"),
                    (Number::Singular, Person::Second) => format!("{b1}t"),
                    (Number::Singular, Person::Third) => b3,
                    (Number::Plural, Person::First) => format!("{b1}mme"),
                    (Number::Plural, Person::Second) => format!("{b1}tte"),
                    (Number::Plural, Person::Third) => {
                        format!("{b3}{}", self.h("vat", "vät"))
                    }
                };
            }
            Tense::Conditional => {
                let base = match &self.row.cond3 {
                    Some(f) => f.clone(),
                    None => {
                        // Strong stem: e/i and half of a long vowel
                        // drop, uo/yö/ie contract (juo → joisi).
                        let mut s = self.strong();
                        for (d, r) in [("uo", "o"), ("yö", "ö"), ("ie", "e")] {
                            if let Some(b) = s.strip_suffix(d) {
                                s = format!("{b}{r}");
                            }
                        }
                        let chars: Vec<char> = s.chars().collect();
                        let n = chars.len();
                        if n >= 1
                            && (matches!(chars[n - 1], 'e' | 'i')
                                || (n >= 2 && chars[n - 1] == chars[n - 2]))
                        {
                            s.pop();
                        }
                        format!("{s}isi")
                    }
                };
                (base, ["n", "t", "", "mme", "tte", ""])
            }
            Tense::Potential => {
                // Base puhune-: 3sg doubles the e (puhunee).
                let base = match &self.row.pot3 {
                    Some(f) => f.strip_suffix('e').unwrap_or(f).to_string(),
                    None => {
                        let nut = self.nut();
                        format!("{}e", &nut[..nut.len() - 2])
                    }
                };
                if number == Number::Plural && person == Person::Third && self.row.pot3.is_some() {
                    // Neutral-only bases are front (liene-); else a
                    // mined nut-participle decides (aivopessyt →
                    // -vät despite the back prefix); else the base's
                    // last non-neutral vowel.
                    let scan = base.chars().rev().find_map(|c| match c {
                        'a' | 'o' | 'u' => Some(false),
                        'ä' | 'ö' | 'y' => Some(true),
                        _ => None,
                    });
                    let front = match (scan, &self.row.nut) {
                        (None, _) => true,
                        (Some(_), Some(nut)) if nut.ends_with("yt") => true,
                        (Some(_), Some(nut)) if nut.ends_with("ut") => false,
                        (Some(s), _) => s,
                    };
                    let suffix = if front { "vät" } else { "vat" };
                    return format!("{base}{suffix}");
                }
                (base, ["n", "t", "e", "mme", "tte", ""])
            }
        };
        let idx = match (number, person) {
            (Number::Singular, Person::First) => 0,
            (Number::Singular, Person::Second) => 1,
            (Number::Singular, Person::Third) => 2,
            (Number::Plural, Person::First) => 3,
            (Number::Plural, Person::Second) => 4,
            (Number::Plural, Person::Third) => 5,
        };
        if idx == 5 {
            let suffix = self.h("vat", "vät");
            format!("{base}{suffix}")
        } else {
            format!("{base}{}", endings[idx])
        }
    }

    /// A passive form.
    pub fn passive(&self, tense: Tense) -> String {
        let b = self.pass_base();
        match tense {
            Tense::Present => self.pass_prs(),
            Tense::Past => self.pass_pst(),
            Tense::Conditional => format!("{b}{}", self.h("aisiin", "äisiin")),
            Tense::Potential => format!("{b}{}", self.h("aneen", "äneen")),
        }
    }

    /// An imperative form (puhu, puhukoon, puhukaamme, puhukaa,
    /// puhukoot; passive puhuttakoon).
    pub fn imperative(&self, person: Person, number: Number) -> Option<String> {
        if person == Person::Second && number == Number::Singular {
            return Some(self.row.imp2.clone().unwrap_or_else(|| self.weak()));
        }
        // koon-stem: the mined 2pl minus -kaa/-kää, else the nut base
        // with final n dropped or a geminate simplified (tull- → tul-).
        let k = if let Some(p) = &self.row.imp_pl2 {
            p.strip_suffix("kaa")
                .or_else(|| p.strip_suffix("kää"))
                .unwrap_or(p)
                .to_string()
        } else {
            let nut = self.nut();
            let mut k = nut[..nut.len() - 2].to_string();
            if k.ends_with('n') {
                k.pop();
            } else {
                let chars: Vec<char> = k.chars().collect();
                if chars.len() >= 2 && chars[chars.len() - 1] == chars[chars.len() - 2] {
                    k.pop();
                }
            }
            k
        };
        let suffix = match (number, person) {
            (Number::Singular, Person::Third) => self.h("koon", "köön"),
            (Number::Plural, Person::First) => self.h("kaamme", "käämme"),
            (Number::Plural, Person::Second) => self.h("kaa", "kää"),
            (Number::Plural, Person::Third) => self.h("koot", "kööt"),
            _ => return None,
        };
        Some(format!("{k}{suffix}"))
    }

    /// The passive imperative (puhuttakoon).
    pub fn imperative_passive(&self) -> String {
        format!("{}{}", self.pass_base(), self.h("akoon", "äköön"))
    }

    /// A participle.
    pub fn participle(&self, past: bool, passive: bool) -> String {
        match (past, passive) {
            (false, false) => format!("{}{}", self.strong(), self.h("va", "vä")),
            (false, true) => {
                format!("{}{}", self.pass_base(), self.h("ava", "ävä"))
            }
            (true, false) => self.nut(),
            (true, true) => {
                format!("{}{}", self.pass_base(), self.h("u", "y"))
            }
        }
    }
}

/// The full conjugation table of a Finnish verb — shared by the
/// WebAssembly and Python bindings. Six-slot rows run sg1…pl3.
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub present: [String; 6],
    pub past: [String; 6],
    pub conditional: [String; 6],
    pub potential: [String; 6],
    /// [2sg, 3sg, 1pl, 2pl, 3pl].
    pub imperative: [Option<String>; 5],
    pub present_passive: String,
    pub past_passive: String,
    pub conditional_passive: String,
    pub potential_passive: String,
    pub imperative_passive: String,
    pub present_participle: String,
    pub past_participle: String,
    pub present_passive_participle: String,
    pub past_passive_participle: String,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let ps = [
            (Person::First, Number::Singular),
            (Person::Second, Number::Singular),
            (Person::Third, Number::Singular),
            (Person::First, Number::Plural),
            (Person::Second, Number::Plural),
            (Person::Third, Number::Plural),
        ];
        Self {
            infinitive: v.infinitive().to_string(),
            present: ps.map(|(p, n)| v.active(Tense::Present, p, n)),
            past: ps.map(|(p, n)| v.active(Tense::Past, p, n)),
            conditional: ps.map(|(p, n)| v.active(Tense::Conditional, p, n)),
            potential: ps.map(|(p, n)| v.active(Tense::Potential, p, n)),
            imperative: [
                v.imperative(Person::Second, Number::Singular),
                v.imperative(Person::Third, Number::Singular),
                v.imperative(Person::First, Number::Plural),
                v.imperative(Person::Second, Number::Plural),
                v.imperative(Person::Third, Number::Plural),
            ],
            present_passive: v.passive(Tense::Present),
            past_passive: v.passive(Tense::Past),
            conditional_passive: v.passive(Tense::Conditional),
            potential_passive: v.passive(Tense::Potential),
            imperative_passive: v.imperative_passive(),
            present_participle: v.participle(false, false),
            past_participle: v.participle(true, false),
            present_passive_participle: v.participle(false, true),
            past_passive_participle: v.participle(true, true),
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
    fn type_one_defaults() {
        let p = v("puhua");
        assert_eq!(
            p.active(Tense::Present, Person::First, Number::Singular),
            "puhun"
        );
        assert_eq!(
            p.active(Tense::Present, Person::Third, Number::Singular),
            "puhuu"
        );
        assert_eq!(
            p.active(Tense::Past, Person::Third, Number::Singular),
            "puhui"
        );
        assert_eq!(
            p.active(Tense::Conditional, Person::Third, Number::Singular),
            "puhuisi"
        );
        assert_eq!(
            p.active(Tense::Potential, Person::Third, Number::Singular),
            "puhunee"
        );
        assert_eq!(p.passive(Tense::Present), "puhutaan");
        assert_eq!(p.passive(Tense::Conditional), "puhuttaisiin");
        assert_eq!(
            p.imperative(Person::Third, Number::Singular).unwrap(),
            "puhukoon"
        );
        assert_eq!(p.participle(true, true), "puhuttu");
    }

    #[test]
    fn harmony() {
        let s = v("kysyä");
        assert_eq!(s.nut(), "kysynyt");
        assert_eq!(s.passive(Tense::Present), "kysytään");
        assert_eq!(
            s.imperative(Person::Second, Number::Plural).unwrap(),
            "kysykää"
        );
    }
}
