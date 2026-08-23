//! Norwegian Bokmål conjugation. Like Danish and Swedish: no
//! person/number agreement — a verb is its principal parts, each with an
//! s-form. The productive first conjugation is the default
//! (kaste/kaster/kasta·kastet/kastende); class-2 -te/-de pasts
//! (kjøpe/kjøpte) and strong verbs (skrive/skrev·skreiv/skrevet) carry
//! mined rows in `data/nob/verbs.tsv`, matched exactly. Bokmål admits two
//! standard spellings in the class-1 past and past participle (-a and
//! -et), so the slots here return every accepted variant rather than a
//! single form.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The slots of a Norwegian Bokmål verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Infinitive,
    Present,
    Past,
    Imperative,
    PresentParticiple,
    PastParticiple,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Norwegian infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Norwegian infinitive")
    }
}

/// Mined principal parts: lemma, present, past, past participle,
/// imperative, present participle, present passive. "-" = the productive
/// default; "|" separates accepted variants.
static VERBS_TSV: &str = include_str!("../data/nob/verbs.tsv");

#[derive(Debug, Clone, Default)]
struct Parts {
    present: Option<Vec<String>>,
    past: Option<Vec<String>>,
    past_participle: Option<Vec<String>>,
    imperative: Option<Vec<String>>,
    present_participle: Option<Vec<String>>,
    present_passive: Option<Vec<String>>,
}

fn parts(inf: &str) -> Option<Parts> {
    static MAP: OnceLock<HashMap<&'static str, Parts>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in VERBS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 7 {
                continue;
            }
            let opt = |i: usize| {
                cols.get(i)
                    .filter(|c| **c != "-" && !c.is_empty())
                    .map(|c| c.split('|').map(str::to_string).collect::<Vec<_>>())
            };
            m.insert(
                cols[0],
                Parts {
                    present: opt(1),
                    past: opt(2),
                    past_participle: opt(3),
                    imperative: opt(4),
                    present_participle: opt(5),
                    present_passive: opt(6),
                },
            );
        }
        m
    });
    map.get(inf).cloned()
}

/// A conjugatable Norwegian Bokmål verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    parts: Parts,
}

impl Verb {
    /// Build a verb from its infinitive (the "å" particle is accepted and
    /// stripped).
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let lowered = infinitive.trim().to_lowercase();
        let infinitive = if parts(infinitive.trim()).is_none() && parts(lowered.as_str()).is_some()
        {
            lowered.as_str()
        } else {
            infinitive
        };
        let mut inf = infinitive.trim();
        if let Some(rest) = inf.strip_prefix("å ") {
            inf = rest.trim();
        }
        if inf.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf.chars().all(|c| c.is_alphabetic() || c == '-')
        {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf.to_string(),
            parts: parts(inf).unwrap_or_default(),
        })
    }

    /// The infinitive as normalized.
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// The stem: infinitive minus a final unstressed -e (kaste → kast);
    /// vowel-final verbs (bo, nå) keep everything.
    fn stem(&self) -> &str {
        self.infinitive
            .strip_suffix('e')
            .filter(|s| !s.is_empty() && !s.ends_with(|c| "aeiouyæøå".contains(c)))
            .unwrap_or(&self.infinitive)
    }

    /// True for deponents / s-verbs (lemma in -s: synes, trives): the
    /// s-form is the present, and there is no separate active voice.
    fn deponent(&self) -> bool {
        self.infinitive.ends_with('s')
    }

    /// Every accepted active form of a slot (class-1 past/participle has
    /// two: kasta and kastet). Empty for the s-form-only slots of a
    /// deponent.
    pub fn active_forms(&self, slot: Slot) -> Vec<String> {
        if self.deponent() && slot != Slot::PresentParticiple {
            return Vec::new();
        }
        let s = self.stem();
        match slot {
            Slot::Infinitive => vec![self.infinitive.clone()],
            Slot::Present => match &self.parts.present {
                Some(f) => f.clone(),
                None => vec![format!("{}r", self.infinitive)],
            },
            Slot::Past => match &self.parts.past {
                Some(f) => f.clone(),
                None => vec![format!("{s}et"), format!("{s}a")],
            },
            Slot::PastParticiple => match &self.parts.past_participle {
                Some(f) => f.clone(),
                None => vec![format!("{s}et"), format!("{s}a")],
            },
            Slot::Imperative => match &self.parts.imperative {
                Some(f) => f.clone(),
                None => vec![s.to_string()],
            },
            Slot::PresentParticiple => match &self.parts.present_participle {
                Some(f) => f.clone(),
                None if self.infinitive.ends_with('e') => vec![format!("{}nde", self.infinitive)],
                None => vec![format!("{}ende", self.infinitive)],
            },
        }
    }

    /// The s-form of a slot (kastes; for deponents the lemma itself).
    /// Only the present/infinitive s-form is in scope.
    pub fn passive_forms(&self, slot: Slot) -> Vec<String> {
        if self.deponent() {
            return match slot {
                Slot::Infinitive | Slot::Present => self
                    .parts
                    .present_passive
                    .clone()
                    .unwrap_or_else(|| vec![self.infinitive.clone()]),
                _ => Vec::new(),
            };
        }
        match slot {
            Slot::Infinitive | Slot::Present => match &self.parts.present_passive {
                Some(f) => f.clone(),
                None => vec![format!("{}s", self.infinitive)],
            },
            _ => Vec::new(),
        }
    }

    /// The primary (first) active form of a slot, if any.
    pub fn active(&self, slot: Slot) -> Option<String> {
        self.active_forms(slot).into_iter().next()
    }

    /// The primary s-form of a slot, if any.
    pub fn passive(&self, slot: Slot) -> Option<String> {
        self.passive_forms(slot).into_iter().next()
    }
}

/// The full conjugation table of a Norwegian Bokmål verb — shared by the
/// WebAssembly and Python bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub present: Option<String>,
    pub past: Option<String>,
    pub past_participle: Option<String>,
    pub imperative: Option<String>,
    pub present_participle: Option<String>,
    pub present_passive: Option<String>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        // Deponents have no active voice; their s-form fills the present.
        let form = |slot: Slot| v.active(slot).or_else(|| v.passive(slot));
        Self {
            infinitive: v.infinitive().to_string(),
            present: form(Slot::Present),
            past: v.active(Slot::Past),
            past_participle: v.active(Slot::PastParticiple),
            imperative: v.active(Slot::Imperative),
            present_participle: v.active(Slot::PresentParticiple),
            present_passive: v.passive(Slot::Present),
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
    fn weak_one_default() {
        let k = v("kaste");
        assert_eq!(k.active(Slot::Present).unwrap(), "kaster");
        assert_eq!(k.active_forms(Slot::Past), vec!["kastet", "kasta"]);
        assert_eq!(
            k.active_forms(Slot::PastParticiple),
            vec!["kastet", "kasta"]
        );
        assert_eq!(k.active(Slot::Imperative).unwrap(), "kast");
        assert_eq!(k.active(Slot::PresentParticiple).unwrap(), "kastende");
        assert_eq!(k.passive(Slot::Present).unwrap(), "kastes");
        // Vowel-final monosyllables keep the whole stem.
        let b = v("bo");
        assert_eq!(b.active(Slot::Present).unwrap(), "bor");
        assert_eq!(b.active(Slot::Imperative).unwrap(), "bo");
        assert_eq!(b.active(Slot::PresentParticiple).unwrap(), "boende");
    }

    #[test]
    fn weak_two_and_strong_mined() {
        let k = v("kjøpe");
        assert_eq!(k.active(Slot::Past).unwrap(), "kjøpte");
        assert_eq!(k.active(Slot::PastParticiple).unwrap(), "kjøpt");
        let s = v("skrive");
        assert_eq!(s.active_forms(Slot::Past), vec!["skreiv", "skrev"]);
        assert_eq!(s.active(Slot::PastParticiple).unwrap(), "skrevet");
        assert_eq!(s.passive(Slot::Present).unwrap(), "skrives");
    }

    #[test]
    fn particle_and_deponent() {
        assert_eq!(v("å bo").infinitive(), "bo");
        let d = v("trives");
        assert_eq!(d.active(Slot::Present), None);
        assert_eq!(Table::build(&d).present.unwrap(), "trives");
    }
}
