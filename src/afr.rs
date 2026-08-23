//! Afrikaans conjugation. Afrikaans lost almost all verb inflection:
//! there is no person or number agreement, the present is the bare
//! infinitive (ek/jy/hy loop), the past is periphrastic (het + past
//! participle) for everything but a closed set of preterites (wees ->
//! was, kan -> kon), and the only productive synthetic form is the ge-
//! past participle. So a verb is its infinitive plus a few principal
//! parts: the productive rules — present = infinitive, past participle
//! = ge- + stem (geë- before a stem-initial e), present participle =
//! stem + -ende — carry the regular verbs, and `data/afr/parts.tsv`
//! stores what deviates: inseparable-prefix verbs that take no ge-
//! (verstaan -> verstaan), separable-prefix verbs whose ge- goes inside
//! (aankom -> aangekom), the spelling alternations of the -ende
//! participle (loop -> lopende, sit -> sittende) and the preterites.
//! Compounds sit at morpheme boundaries no suffix heuristic can see —
//! the lesson Danish and Swedish taught — so they are matched exactly.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The slots of an Afrikaans verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Infinitive,
    Present,
    /// Synthetic preterite: only the closed modal/copula set has one;
    /// every other verb forms its past periphrastically (het + PTCP).
    Past,
    Imperative,
    PresentParticiple,
    PastParticiple,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like an Afrikaans infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Afrikaans infinitive")
    }
}

/// Mined principal parts: lemma, present, past, past participle,
/// present participle, imperative ("-" = productive default).
static PARTS_TSV: &str = include_str!("../data/afr/parts.tsv");

#[derive(Debug, Clone, Default)]
struct Parts {
    present: Option<String>,
    past: Option<String>,
    past_participle: Option<String>,
    present_participle: Option<String>,
    imperative: Option<String>,
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
                    present_participle: opt(4),
                    imperative: opt(5),
                },
            );
        }
        m
    });
    map.get(inf).cloned()
}

/// The productive ge- past participle: geë- before a stem-initial e
/// (eet -> geëet), ge- otherwise (loop -> geloop).
fn ge_participle(inf: &str) -> String {
    if let Some(rest) = inf.strip_prefix('e') {
        format!("geë{rest}")
    } else {
        format!("ge{inf}")
    }
}

/// A conjugatable Afrikaans verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    parts: Parts,
}

impl Verb {
    /// Build a verb from its infinitive. The infinitive marker
    /// ("om te loop", "te loop") is accepted and stripped.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let lowered = infinitive.trim().to_lowercase();
        let base = if parts(infinitive.trim()).is_none() && parts(lowered.as_str()).is_some() {
            lowered.as_str()
        } else {
            infinitive
        };
        let mut inf = base.trim();
        if let Some(rest) = inf.strip_prefix("om te ") {
            inf = rest.trim();
        } else if let Some(rest) = inf.strip_prefix("te ") {
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

    /// A conjugated form. `Past` is `None` for the vast majority of
    /// verbs, whose past is periphrastic (het + past participle).
    pub fn form(&self, slot: Slot) -> Option<String> {
        Some(match slot {
            Slot::Infinitive => self.infinitive.clone(),
            Slot::Present => self
                .parts
                .present
                .clone()
                .unwrap_or_else(|| self.infinitive.clone()),
            Slot::Past => self.parts.past.clone()?,
            Slot::Imperative => self
                .parts
                .imperative
                .clone()
                .unwrap_or_else(|| self.infinitive.clone()),
            Slot::PresentParticiple => self
                .parts
                .present_participle
                .clone()
                .unwrap_or_else(|| format!("{}ende", self.infinitive)),
            Slot::PastParticiple => self
                .parts
                .past_participle
                .clone()
                .unwrap_or_else(|| ge_participle(&self.infinitive)),
        })
    }
}

/// The full conjugation table of an Afrikaans verb — shared by the
/// WebAssembly and Python bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub present: Option<String>,
    /// Synthetic preterite where one exists (modals, copula); `None`
    /// for verbs whose past is periphrastic.
    pub past: Option<String>,
    /// The periphrastic perfect: "het " + past participle, the everyday
    /// Afrikaans past.
    pub perfect: Option<String>,
    pub past_participle: Option<String>,
    pub present_participle: Option<String>,
    pub imperative: Option<String>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let past_participle = v.form(Slot::PastParticiple);
        let perfect = past_participle.as_ref().map(|p| format!("het {p}"));
        Self {
            infinitive: v.infinitive().to_string(),
            present: v.form(Slot::Present),
            past: v.form(Slot::Past),
            perfect,
            past_participle,
            present_participle: v.form(Slot::PresentParticiple),
            imperative: v.form(Slot::Imperative),
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
    fn regular_default() {
        let m = v("loop");
        assert_eq!(m.form(Slot::Present).unwrap(), "loop");
        assert_eq!(m.form(Slot::PastParticiple).unwrap(), "geloop");
        assert_eq!(m.form(Slot::Past), None); // periphrastic: het geloop
        assert_eq!(m.form(Slot::Imperative).unwrap(), "loop");
        assert_eq!(Table::build(&m).perfect.unwrap(), "het geloop");
    }

    #[test]
    fn e_initial_gets_diaeresis() {
        assert_eq!(v("eet").form(Slot::PastParticiple).unwrap(), "geëet");
        assert_eq!(v("eis").form(Slot::PastParticiple).unwrap(), "geëis");
    }

    #[test]
    fn mined_exceptions() {
        // Inseparable prefix: no ge-.
        assert_eq!(v("verstaan").form(Slot::PastParticiple).unwrap(), "verstaan");
        // Separable prefix: ge- goes inside.
        assert_eq!(v("aankom").form(Slot::PastParticiple).unwrap(), "aangekom");
        // -ende spelling alternation.
        assert_eq!(v("sit").form(Slot::PresentParticiple).unwrap(), "sittende");
        // Closed-set preterite and copula present.
        assert_eq!(v("het").form(Slot::Past).unwrap(), "had");
        assert_eq!(v("wees").form(Slot::Present).unwrap(), "is");
        assert_eq!(v("wees").form(Slot::Past).unwrap(), "was");
    }

    #[test]
    fn infinitive_marker_stripped() {
        assert_eq!(v("om te loop").infinitive(), "loop");
        assert_eq!(v("te werk").infinitive(), "werk");
    }
}
