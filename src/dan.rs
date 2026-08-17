//! Danish conjugation. Like Swedish: no person/number agreement — a
//! verb is its principal parts, each with an s-passive. The
//! productive first conjugation is the default (virke/virker/virkede/
//! virket); class-2 -te pasts (tale/talte) and strong verbs
//! (skrive/skrev/skrevet) carry mined rows in `data/dan/parts.tsv`,
//! matched exactly — Danish compounds sit at morpheme boundaries no
//! suffix heuristic can see, the same lesson Swedish taught.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The slots of a Danish verb.
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
    /// The input does not look like a Danish infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Danish infinitive")
    }
}

/// Mined principal parts: lemma, present, past, past participle,
/// imperative, present participle, past passive ("-" = default).
static PARTS_TSV: &str = include_str!("../data/dan/parts.tsv");

#[derive(Debug, Clone, Default)]
struct Parts {
    present: Option<String>,
    past: Option<String>,
    past_participle: Option<String>,
    imperative: Option<String>,
    present_participle: Option<String>,
    past_passive: Option<String>,
}

fn parts(inf: &str) -> Option<Parts> {
    static MAP: OnceLock<HashMap<&'static str, Parts>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in PARTS_TSV.lines() {
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
                Parts {
                    present: opt(1),
                    past: opt(2),
                    past_participle: opt(3),
                    imperative: opt(4),
                    present_participle: opt(5),
                    past_passive: opt(6),
                },
            );
        }
        m
    });
    map.get(inf).cloned()
}

/// A conjugatable Danish verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    parts: Parts,
}

impl Verb {
    /// Build a verb from its infinitive (the "at" particle is
    /// accepted and stripped).
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        // Case-fold only onto known lemmas (VÆRE → være); COR keeps a
        // few legitimately capitalized verbs.
        let lowered = infinitive.trim().to_lowercase();
        let infinitive = if parts(infinitive.trim()).is_none() && parts(lowered.as_str()).is_some()
        {
            lowered.as_str()
        } else {
            infinitive
        };
        let mut inf = infinitive.trim();
        if let Some(rest) = inf.strip_prefix("at ") {
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

    /// The class-1 stem: infinitive minus a final unstressed -e
    /// (virke → virk); vowel-final verbs (bo, nå) keep everything.
    fn stem(&self) -> &str {
        self.infinitive
            .strip_suffix('e')
            .filter(|s| !s.is_empty() && !s.ends_with(|c| "aeiouyæøå".contains(c)))
            .unwrap_or(&self.infinitive)
    }

    /// True for deponents (lemma in -s: findes, lykkes): their forms
    /// are the s-forms and there is no active voice.
    fn deponent(&self) -> bool {
        self.infinitive.ends_with('s')
    }

    /// An active form.
    pub fn active(&self, slot: Slot) -> Option<String> {
        if self.deponent() && slot != Slot::PresentParticiple {
            return None;
        }
        let s = self.stem();
        Some(match slot {
            Slot::Infinitive => self.infinitive.clone(),
            Slot::Present => match &self.parts.present {
                Some(f) => f.clone(),
                None => format!("{}r", self.infinitive),
            },
            Slot::Past => match &self.parts.past {
                Some(f) => f.clone(),
                None if self.infinitive.ends_with('e') => format!("{}de", self.infinitive),
                None => format!("{}ede", self.infinitive),
            },
            Slot::PastParticiple => match &self.parts.past_participle {
                Some(f) => f.clone(),
                None => format!("{s}et"),
            },
            Slot::Imperative => match &self.parts.imperative {
                Some(f) => f.clone(),
                None => s.to_string(),
            },
            Slot::PresentParticiple => match &self.parts.present_participle {
                Some(f) => f.clone(),
                None if self.infinitive.ends_with('e') => format!("{}nde", self.infinitive),
                None => format!("{}ende", self.infinitive),
            },
        })
    }

    /// The s-passive of a slot (medvirkes, skreves); for deponents
    /// the slots themselves are the s-forms.
    pub fn passive(&self, slot: Slot) -> Option<String> {
        if self.deponent() {
            return match slot {
                Slot::Infinitive => Some(self.infinitive.clone()),
                Slot::Present => Some(
                    self.parts
                        .present
                        .clone()
                        .unwrap_or_else(|| self.infinitive.clone()),
                ),
                Slot::Past => self.parts.past.clone(),
                Slot::PastParticiple => self.parts.past_participle.clone(),
                _ => None,
            };
        }
        match slot {
            // The infinitive and present passives coincide (skrives).
            Slot::Infinitive | Slot::Present => Some(format!("{}s", self.infinitive)),
            Slot::Past => {
                if let Some(f) = &self.parts.past_passive {
                    return Some(f.clone());
                }
                let past = self.active(Slot::Past)?;
                // Vowel-final strong pasts take bare -s (så → sås).
                if past.ends_with(|c| "eaiouyæøå".contains(c)) {
                    Some(format!("{past}s"))
                } else {
                    Some(format!("{past}es"))
                }
            }
            _ => None,
        }
    }
}

/// The full conjugation table of a Danish verb — shared by the
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
    pub infinitive_passive: Option<String>,
    pub present_passive: Option<String>,
    pub past_passive: Option<String>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        // Deponents (synes, lykkes) have no active voice; their s-forms
        // are the verb's only forms, so they fill the primary slots
        // rather than leaving the table empty.
        let form = |slot: Slot| v.active(slot).or_else(|| v.passive(slot));
        Self {
            infinitive: v.infinitive().to_string(),
            present: form(Slot::Present),
            past: form(Slot::Past),
            past_participle: form(Slot::PastParticiple),
            imperative: v.active(Slot::Imperative),
            present_participle: v.active(Slot::PresentParticiple),
            infinitive_passive: v.passive(Slot::Infinitive),
            present_passive: v.passive(Slot::Present),
            past_passive: v.passive(Slot::Past),
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
        let m = v("virke");
        assert_eq!(m.active(Slot::Present).unwrap(), "virker");
        assert_eq!(m.active(Slot::Past).unwrap(), "virkede");
        assert_eq!(m.active(Slot::PastParticiple).unwrap(), "virket");
        assert_eq!(m.active(Slot::Imperative).unwrap(), "virk");
        assert_eq!(m.active(Slot::PresentParticiple).unwrap(), "virkende");
        assert_eq!(m.passive(Slot::Present).unwrap(), "virkes");
        assert_eq!(m.passive(Slot::Past).unwrap(), "virkedes");
        let b = v("bo");
        assert_eq!(b.active(Slot::Past).unwrap(), "boede");
        assert_eq!(b.active(Slot::PresentParticiple).unwrap(), "boende");
    }

    #[test]
    fn strong_and_deponent() {
        let s = v("skrive");
        assert_eq!(s.active(Slot::Past).unwrap(), "skrev");
        assert_eq!(s.active(Slot::PastParticiple).unwrap(), "skrevet");
        assert_eq!(s.passive(Slot::Past).unwrap(), "skreves");
        let d = v("lykkes");
        assert_eq!(d.active(Slot::Present), None);
        assert_eq!(Table::build(&d).present.unwrap(), "lykkes");
        assert_eq!(Table::build(&d).past.unwrap(), "lykkedes");
        assert_eq!(v("at bo").infinitive(), "bo");
    }
}
