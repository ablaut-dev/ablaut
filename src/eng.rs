//! English conjugation. Four inflected slots beside the infinitive —
//! past, past participle, present participle, third-person singular —
//! with regular -ed/-ing/-s rules (e-drop, y→ie, o/sibilant -es) and
//! a mined table for everything the rules cannot decide: irregular
//! verbs (sing/sang/sung) and final-consonant doubling, which depends
//! on stress the orthography does not record (stop → stopping, but
//! visit → visiting).

use std::collections::HashMap;
use std::sync::OnceLock;

/// The inflected slots of an English verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Infinitive,
    Past,
    PastParticiple,
    PresentParticiple,
    ThirdSingular,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like an English infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an English infinitive")
    }
}

/// Mined rows: lemma, past, past participle, present participle,
/// third singular ("-" = derive).
static VERBS_TSV: &str = include_str!("../data/eng/verbs.tsv");

#[derive(Debug, Clone, Default)]
struct Row {
    past: Option<String>,
    past_participle: Option<String>,
    present_participle: Option<String>,
    third: Option<String>,
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
                    past: opt(1),
                    past_participle: opt(2),
                    present_participle: opt(3),
                    third: opt(4),
                },
            );
        }
        m
    })
}

fn vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// A conjugatable English verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    row: Row,
}

impl Verb {
    /// Build a verb from its infinitive (the "to" particle is
    /// accepted and stripped).
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        // Case-fold only onto known lemmas: Run/BE reach run/be, but
        // Americanize, DJ and Frenchify are legitimately capitalized
        // and regular capitalized input keeps its casing.
        let lowered = infinitive.trim().to_lowercase();
        let infinitive =
            if !rows().contains_key(infinitive.trim()) && rows().contains_key(lowered.as_str()) {
                lowered.as_str()
            } else {
                infinitive
            };
        let mut inf = infinitive.trim();
        if let Some(rest) = inf.strip_prefix("to ") {
            inf = rest.trim();
        }
        if inf.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c == '-' || c == '\'')
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

    /// The regular -ed form (also the default participle): y→ied
    /// after a consonant, bare +d after e.
    fn regular_past(&self) -> String {
        let s = &self.infinitive;
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len();
        if n >= 2 && chars[n - 1] == 'y' && !vowel(chars[n - 2]) {
            return format!("{}ied", &s[..s.len() - 1]);
        }
        if s.ends_with('e') {
            return format!("{s}d");
        }
        format!("{s}ed")
    }

    /// The regular -ing form: e-drop except after ee/oe/ye/nge,
    /// ie→ying (die → dying).
    fn regular_ing(&self) -> String {
        let s = &self.infinitive;
        if let Some(base) = s.strip_suffix("ie") {
            return format!("{base}ying");
        }
        if s.ends_with('e')
            && !s.ends_with("ee")
            && !s.ends_with("oe")
            && !s.ends_with("ye")
            && !s.ends_with("nge")
        {
            return format!("{}ing", &s[..s.len() - 1]);
        }
        format!("{s}ing")
    }

    /// The regular -s form: sibilants and o take -es, consonant+y
    /// takes -ies.
    fn regular_third(&self) -> String {
        let s = &self.infinitive;
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len();
        if n >= 2 && chars[n - 1] == 'y' && !vowel(chars[n - 2]) {
            return format!("{}ies", &s[..s.len() - 1]);
        }
        if s.ends_with('s')
            || s.ends_with('x')
            || s.ends_with('z')
            || s.ends_with("ch")
            || s.ends_with("sh")
            || s.ends_with('o')
        {
            return format!("{s}es");
        }
        format!("{s}s")
    }

    /// A conjugated form.
    #[must_use]
    pub fn form(&self, slot: Slot) -> String {
        match slot {
            Slot::Infinitive => self.infinitive.clone(),
            Slot::Past => self.row.past.clone().unwrap_or_else(|| self.regular_past()),
            Slot::PastParticiple => self
                .row
                .past_participle
                .clone()
                .or_else(|| self.row.past.clone())
                .unwrap_or_else(|| self.regular_past()),
            Slot::PresentParticiple => self
                .row
                .present_participle
                .clone()
                .unwrap_or_else(|| self.regular_ing()),
            Slot::ThirdSingular => self
                .row
                .third
                .clone()
                .unwrap_or_else(|| self.regular_third()),
        }
    }
}

/// The full conjugation table of an English verb — shared by the
/// WebAssembly and Python bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub past: String,
    pub past_participle: String,
    pub present_participle: String,
    pub third_singular: String,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        Self {
            infinitive: v.infinitive().to_string(),
            past: v.form(Slot::Past),
            past_participle: v.form(Slot::PastParticiple),
            present_participle: v.form(Slot::PresentParticiple),
            third_singular: v.form(Slot::ThirdSingular),
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
    fn regular_rules() {
        let w = v("walk");
        assert_eq!(w.form(Slot::Past), "walked");
        assert_eq!(w.form(Slot::PresentParticiple), "walking");
        assert_eq!(w.form(Slot::ThirdSingular), "walks");
        assert_eq!(v("carry").form(Slot::Past), "carried");
        assert_eq!(v("carry").form(Slot::ThirdSingular), "carries");
        assert_eq!(v("make").form(Slot::PresentParticiple), "making");
        assert_eq!(v("die").form(Slot::PresentParticiple), "dying");
        assert_eq!(v("agree").form(Slot::PresentParticiple), "agreeing");
        assert_eq!(v("watch").form(Slot::ThirdSingular), "watches");
        assert_eq!(v("go").form(Slot::ThirdSingular), "goes");
    }

    #[test]
    fn mined_irregulars() {
        let s = v("sing");
        assert_eq!(s.form(Slot::Past), "sang");
        assert_eq!(s.form(Slot::PastParticiple), "sung");
        let b = v("be");
        assert_eq!(b.form(Slot::Past), "was");
        assert_eq!(v("stop").form(Slot::PresentParticiple), "stopping");
        assert_eq!(v("to run").infinitive(), "run");
    }
}
