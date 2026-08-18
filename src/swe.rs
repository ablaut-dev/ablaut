//! Swedish conjugation. No person/number agreement: a verb is its
//! principal parts — infinitive, present, past, supine, imperative —
//! each with an s-passive, plus the relic subjunctives.
//!
//! The first (weak -ar) conjugation is the productive default
//! (tala/talar/talade/talat); every verb deviating from it carries a
//! mined principal-parts row in `data/swe/parts.tsv` (läser/läste/läst,
//! springer/sprang/sprungit, är/var/varit). The s-passive derives by
//! rule: -ar → -as, -er → -es (or bare s after a long-vowel stem),
//! past/supine/infinitive + s.

/// The five active slots of a Swedish verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Infinitive,
    Present,
    Past,
    /// The supine (har talat).
    Supine,
    Imperative,
    /// Present subjunctive (relic: leve, vare).
    SubjunctivePresent,
    /// Past subjunctive (relic: vore).
    SubjunctivePast,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Swedish infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Swedish infinitive")
    }
}

/// Mined principal parts: lemma, present, past, supine, imperative
/// ("-" derives from the class-1 default; extra columns for the relic
/// subjunctives).
static PARTS_TSV: &str = include_str!("../data/swe/parts.tsv");

#[derive(Debug, Clone, Default)]
struct Parts {
    present: Option<String>,
    past: Option<String>,
    supine: Option<String>,
    imperative: Option<String>,
    subj_pres: Option<String>,
    subj_past: Option<String>,
}

fn parts(inf: &str) -> Option<Parts> {
    // Longest suffix match: Swedish compounds inherit their base verb
    // (skattskriva conjugates like skriva, trakthugga like hugga).
    let mut best: Option<(&str, Vec<&str>)> = None;
    for line in PARTS_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let lemma = cols[0];
        // Exact match only: Swedish compounds sit at morpheme
        // boundaries no suffix heuristic can see (fördärva is not
        // för+därva-like ärva), so every deviant compound gets its own
        // mined row.
        if inf == lemma && best.is_none() {
            best = Some((lemma, cols));
        }
    }
    let (lemma, cols) = best?;
    let prefix = &inf[..inf.len() - lemma.len()];
    let opt = |i: usize| {
        cols.get(i)
            .filter(|c| **c != "-" && !c.is_empty())
            .map(|c| format!("{prefix}{c}"))
    };
    Some(Parts {
        present: opt(1),
        past: opt(2),
        supine: opt(3),
        imperative: opt(4),
        subj_pres: opt(5),
        subj_past: opt(6),
    })
}

/// A conjugatable Swedish verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    parts: Parts,
}

impl Verb {
    /// Build a verb from its infinitive.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        // Case-fold only onto known lemmas (VARA → vara); SALDO has a
        // handful of legitimately capitalized verbs.
        let lowered = infinitive.trim().to_lowercase();
        let infinitive = if parts(infinitive.trim()).is_none() && parts(lowered.as_str()).is_some()
        {
            lowered.as_str()
        } else {
            infinitive
        };
        let inf = infinitive.trim();
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

    /// The class-1 stem: infinitive (tala); non -a verbs (bo, sy) use
    /// the whole word.
    fn stem(&self) -> &str {
        &self.infinitive
    }

    /// An active form.
    pub fn active(&self, slot: Slot) -> Option<String> {
        let s = self.stem();
        Some(match slot {
            Slot::Infinitive => self.infinitive.clone(),
            Slot::Present => match &self.parts.present {
                Some(f) => f.clone(),
                None => format!("{s}r"),
            },
            Slot::Past => match &self.parts.past {
                Some(f) => f.clone(),
                None => format!("{s}de"),
            },
            Slot::Supine => match &self.parts.supine {
                Some(f) => f.clone(),
                None => format!("{s}t"),
            },
            Slot::Imperative => match &self.parts.imperative {
                Some(f) => f.clone(),
                None => self.infinitive.clone(),
            },
            Slot::SubjunctivePresent => match &self.parts.subj_pres {
                Some(f) => f.clone(),
                None => {
                    // stem minus final a + e (tale); relic, mostly
                    // unattested and unscored.
                    let base = s.strip_suffix('a').unwrap_or(s);
                    format!("{base}e")
                }
            },
            Slot::SubjunctivePast => self.parts.subj_past.clone()?,
        })
    }

    /// True for deponents (lemma in -s: förgås, hoppas): their forms
    /// live in the passive slots and there is no active voice.
    fn deponent(&self) -> bool {
        self.infinitive.ends_with('s')
    }

    /// An active-voice form; None for deponents.
    pub fn active_voice(&self, slot: Slot) -> Option<String> {
        if self.deponent() {
            return None;
        }
        self.active(slot)
    }

    /// All standard passive spellings of a slot (läses and läss-style
    /// doublets: kvädes/kväds), canonical first.
    pub fn passive_variants(&self, slot: Slot) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(f) = self.passive(slot) {
            out.push(f);
        }
        if slot == Slot::Present && !self.deponent() {
            if let Some(active) = self.active(slot) {
                if let Some(base) = active.strip_suffix("er") {
                    let alt = format!("{base}s");
                    if !out.contains(&alt) {
                        out.push(alt);
                    }
                }
            }
        }
        out
    }

    fn parts_slot(&self, slot: Slot) -> Option<String> {
        match slot {
            Slot::Past => self.parts.past.clone(),
            Slot::Supine => self.parts.supine.clone(),
            _ => None,
        }
    }

    /// Derive a deponent's s-form from the base verb obtained by
    /// stripping the final s (bitas → bita), falling back to the
    /// weak-1 default when no base exists.
    fn deponent_via_base(&self, slot: Slot) -> Option<String> {
        self.infinitive
            .strip_suffix('s')
            .filter(|b| !b.ends_with('s'))
            .and_then(|b| Verb::from_infinitive(b).ok())
            .and_then(|base| base.passive(slot))
    }

    /// The s-passive of a slot: talas, talades, läses, görs. For
    /// deponents the parts themselves are the s-forms.
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
                // An s-lemma without stored parts derives from its
                // base verb's s-passive (bitas → bita → bet → bets,
                // hoppas → hoppa → hoppades); irregular deponents
                // carry parts entries instead.
                Slot::Past | Slot::Supine => self
                    .parts_slot(slot)
                    .or_else(|| self.deponent_via_base(slot)),
                _ => None,
            };
        }
        let active = self.active(slot)?;
        Some(match slot {
            Slot::Present => {
                // Suppletive vara: the s-form rides the infinitive
                // (varas), not the present (är).
                if self.infinitive == "vara" {
                    return Some("varas".to_string());
                }
                if active == format!("{}r", self.infinitive) {
                    // Weak-1 and vowel-stem presents drop their r:
                    // talas, strös, begås.
                    format!("{}s", self.infinitive)
                } else if let Some(base) = active.strip_suffix("er") {
                    format!("{base}es") // läser → läses
                } else {
                    format!("{active}s") // gör → görs, erfar → erfars
                }
            }
            Slot::Imperative | Slot::SubjunctivePresent | Slot::SubjunctivePast => return None,
            _ => format!("{active}s"),
        })
    }
}

/// The full conjugation table of a Swedish verb — shared by the
/// WebAssembly and Python bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub present: Option<String>,
    pub past: Option<String>,
    pub supine: Option<String>,
    pub imperative: Option<String>,
    pub infinitive_passive: Option<String>,
    pub present_passive: Option<String>,
    pub past_passive: Option<String>,
    pub supine_passive: Option<String>,
    pub subjunctive_past: Option<String>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        // Deponents (hoppas, andas) have no active voice; their s-forms
        // are the verb's only forms, so they fill the primary slots
        // rather than leaving the table empty.
        let form = |slot: Slot| v.active_voice(slot).or_else(|| v.passive(slot));
        Self {
            infinitive: v.infinitive().to_string(),
            present: form(Slot::Present),
            past: form(Slot::Past),
            supine: form(Slot::Supine),
            imperative: v.active_voice(Slot::Imperative),
            infinitive_passive: v.passive(Slot::Infinitive),
            present_passive: v.passive(Slot::Present),
            past_passive: v.passive(Slot::Past),
            supine_passive: v.passive(Slot::Supine),
            subjunctive_past: v.active_voice(Slot::SubjunctivePast),
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
    fn parts_and_passives() {
        let s = v("skriva");
        assert_eq!(s.active(Slot::Present).unwrap(), "skriver");
        assert_eq!(s.active(Slot::Past).unwrap(), "skrev");
        assert_eq!(s.active(Slot::Supine).unwrap(), "skrivit");
        assert_eq!(s.passive(Slot::Past).unwrap(), "skrevs");
        let vara = v("vara");
        assert_eq!(vara.active(Slot::Present).unwrap(), "är");
        assert_eq!(vara.active(Slot::Past).unwrap(), "var");
        assert_eq!(vara.active(Slot::SubjunctivePast).unwrap(), "vore");
        let g = v("göra");
        assert_eq!(g.passive(Slot::Present).unwrap(), "görs");
        let h = v("hoppas");
        assert_eq!(h.active_voice(Slot::Present), None);
        assert_eq!(h.passive(Slot::Past).unwrap(), "hoppades");
        assert_eq!(h.passive(Slot::Supine).unwrap(), "hoppats");
        // Irregular deponents come from the parts table, not weak 1.
        assert_eq!(v("trivas").passive(Slot::Past).unwrap(), "trivdes");
        assert_eq!(Table::build(&h).present.unwrap(), "hoppas");
        assert_eq!(Table::build(&h).past.unwrap(), "hoppades");
    }

    #[test]
    fn deponent_presents_are_modern() {
        // Issue #84: the archaic -es presents (minnes, finnes) gave way
        // to the modern -s forms wherever gold attests them.
        assert_eq!(v("minnas").passive(Slot::Present).unwrap(), "minns");
        assert_eq!(v("finnas").passive(Slot::Present).unwrap(), "finns");
        assert_eq!(v("kännas").passive(Slot::Present).unwrap(), "känns");
        assert_eq!(v("synas").passive(Slot::Present).unwrap(), "syns");
    }

    #[test]
    fn weak_one_default() {
        let t = v("tala");
        assert_eq!(t.active(Slot::Present).unwrap(), "talar");
        assert_eq!(t.active(Slot::Past).unwrap(), "talade");
        assert_eq!(t.active(Slot::Supine).unwrap(), "talat");
        assert_eq!(t.active(Slot::Imperative).unwrap(), "tala");
        assert_eq!(t.passive(Slot::Present).unwrap(), "talas");
        assert_eq!(t.passive(Slot::Past).unwrap(), "talades");
    }
}
