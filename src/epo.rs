//! Esperanto conjugation. Esperanto is the regular-conjugation limit:
//! every verb is cited by its `-i` infinitive, and the whole paradigm is
//! pure suffixation off the invariant stem (the infinitive minus `-i`).
//! There is not a single irregular verb — even `esti` "to be" is regular
//! (`estas`, `estis`, `estos`, `estus`, `estu`). So there is nothing to
//! mine: `data/epo/parts.tsv` and `data/epo/overrides.tsv` stay empty and
//! every scored form is rule-derived.
//!
//! The finite/non-finite endings are: `-as` present, `-is` past, `-os`
//! future, `-us` conditional, `-u` volitive (jussive/imperative), `-i`
//! infinitive.
//!
//! The participles are a clean cross-product: voice (active `-ant/-int/
//! -ont-`, passive `-at/-it/-ot-`) × tense (present/past/future) × the
//! closing category — adjective `-a`, noun `-o`, adverb `-e`. The
//! adjectival and nominal forms further inflect for number (`-j`) and the
//! accusative (`-n`): amanta, amantaj, amantan, amantajn. That gives
//! 6 finite/non-finite + 6 participle stems × 9 closings = 60 forms, all
//! generated here without exception.

/// A finite / non-finite slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Present,
    Past,
    Future,
    Conditional,
    Volitive,
    Infinitive,
}

/// The voice of a participle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Voice {
    Active,
    Passive,
}

/// The tense of a participle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    Present,
    Past,
    Future,
}

/// Why an input cannot be conjugated as an Esperanto verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like an Esperanto `-i` infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Esperanto infinitive")
    }
}

/// The participle stem for a voice/tense pair: active `-ant/-int/-ont-`,
/// passive `-at/-it/-ot-`.
fn participle_infix(voice: Voice, tense: Tense) -> &'static str {
    match (voice, tense) {
        (Voice::Active, Tense::Present) => "ant",
        (Voice::Active, Tense::Past) => "int",
        (Voice::Active, Tense::Future) => "ont",
        (Voice::Passive, Tense::Present) => "at",
        (Voice::Passive, Tense::Past) => "it",
        (Voice::Passive, Tense::Future) => "ot",
    }
}

/// The nine closings of a participle stem, in a fixed order: the four
/// adjectival (`-a/-aj/-an/-ajn`), the four nominal (`-o/-oj/-on/-ojn`)
/// and the invariant adverb (`-e`).
const CLOSINGS: [&str; 9] = ["a", "aj", "an", "ajn", "o", "oj", "on", "ojn", "e"];

/// A conjugatable Esperanto verb: the infinitive and its invariant stem.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    stem: String,
}

impl Verb {
    /// Build a verb from its `-i` infinitive. The infinitive marker is the
    /// citation form itself, so no stripping is needed; the trailing `-i`
    /// is removed to expose the invariant stem.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim().to_lowercase();
        let Some(stem) = inf.strip_suffix('i') else {
            return Err(Error::NotAVerb);
        };
        if stem.is_empty()
            || inf.contains(char::is_whitespace)
            || !inf.chars().all(|c| c.is_alphabetic() || c == '-')
        {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf.clone(),
            stem: stem.to_string(),
        })
    }

    /// The infinitive (citation form).
    #[must_use]
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// The invariant stem (infinitive minus `-i`).
    #[must_use]
    pub fn stem(&self) -> &str {
        &self.stem
    }

    /// A finite / non-finite form.
    #[must_use]
    pub fn finite(&self, slot: Slot) -> String {
        match slot {
            Slot::Present => format!("{}as", self.stem),
            Slot::Past => format!("{}is", self.stem),
            Slot::Future => format!("{}os", self.stem),
            Slot::Conditional => format!("{}us", self.stem),
            Slot::Volitive => format!("{}u", self.stem),
            Slot::Infinitive => self.infinitive.clone(),
        }
    }

    /// One participle form, addressed by voice, tense and a closing index
    /// into [`CLOSINGS`] (0..9).
    #[must_use]
    fn participle_at(&self, voice: Voice, tense: Tense, closing: usize) -> String {
        format!(
            "{}{}{}",
            self.stem,
            participle_infix(voice, tense),
            CLOSINGS[closing]
        )
    }

    /// The nine forms of one participle stem, in [`CLOSINGS`] order.
    #[must_use]
    fn participles(&self, voice: Voice, tense: Tense) -> Vec<String> {
        (0..CLOSINGS.len())
            .map(|i| self.participle_at(voice, tense, i))
            .collect()
    }

    /// The surface form for a canonical feature bundle, or `None` if the
    /// bundle is not a slot of the paradigm. The bundle vocabulary is the
    /// one the golden harness scores against (see `src/bin/golden_epo.rs`).
    #[must_use]
    pub fn form(&self, feature: &str) -> Option<String> {
        match feature {
            "V;PRS" => Some(self.finite(Slot::Present)),
            "V;PST" => Some(self.finite(Slot::Past)),
            "V;FUT" => Some(self.finite(Slot::Future)),
            "V;COND" => Some(self.finite(Slot::Conditional)),
            "V;VOL" => Some(self.finite(Slot::Volitive)),
            "V;NFIN" => Some(self.finite(Slot::Infinitive)),
            _ => self.participle_form(feature),
        }
    }

    /// Parse a `V.PTCP;<voice>;<tense>;<category>[;<num>;<case>]` bundle
    /// and generate its form.
    fn participle_form(&self, feature: &str) -> Option<String> {
        let rest = feature.strip_prefix("V.PTCP;")?;
        let mut it = rest.split(';');
        let voice = match it.next()? {
            "ACT" => Voice::Active,
            "PASS" => Voice::Passive,
            _ => return None,
        };
        let tense = match it.next()? {
            "PRS" => Tense::Present,
            "PST" => Tense::Past,
            "FUT" => Tense::Future,
            _ => return None,
        };
        let closing = match it.next()? {
            "ADV" => "e",
            cat @ ("ADJ" | "N") => {
                let vowel = if cat == "ADJ" { "a" } else { "o" };
                let plural = matches!(it.next()?, "PL");
                let accusative = matches!(it.next()?, "ACC");
                return Some(format!(
                    "{}{}{}{}{}",
                    self.stem,
                    participle_infix(voice, tense),
                    vowel,
                    if plural { "j" } else { "" },
                    if accusative { "n" } else { "" },
                ));
            }
            _ => return None,
        };
        Some(format!(
            "{}{}{}",
            self.stem,
            participle_infix(voice, tense),
            closing
        ))
    }
}

/// The full conjugation table of an Esperanto verb — shared by the
/// WebAssembly and Python bindings. The finite/non-finite cells are
/// scalars; each participle group carries its nine closings in
/// [`CLOSINGS`] order (adjective nom/acc × sg/pl, noun nom/acc × sg/pl,
/// adverb).
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub present: String,
    pub past: String,
    pub future: String,
    pub conditional: String,
    pub volitive: String,
    pub active_present: Vec<String>,
    pub active_past: Vec<String>,
    pub active_future: Vec<String>,
    pub passive_present: Vec<String>,
    pub passive_past: Vec<String>,
    pub passive_future: Vec<String>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        Self {
            infinitive: v.infinitive().to_string(),
            present: v.finite(Slot::Present),
            past: v.finite(Slot::Past),
            future: v.finite(Slot::Future),
            conditional: v.finite(Slot::Conditional),
            volitive: v.finite(Slot::Volitive),
            active_present: v.participles(Voice::Active, Tense::Present),
            active_past: v.participles(Voice::Active, Tense::Past),
            active_future: v.participles(Voice::Active, Tense::Future),
            passive_present: v.participles(Voice::Passive, Tense::Present),
            passive_past: v.participles(Voice::Passive, Tense::Past),
            passive_future: v.participles(Voice::Passive, Tense::Future),
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
    fn finite_endings() {
        let a = v("ami");
        assert_eq!(a.finite(Slot::Present), "amas");
        assert_eq!(a.finite(Slot::Past), "amis");
        assert_eq!(a.finite(Slot::Future), "amos");
        assert_eq!(a.finite(Slot::Conditional), "amus");
        assert_eq!(a.finite(Slot::Volitive), "amu");
        assert_eq!(a.finite(Slot::Infinitive), "ami");
    }

    #[test]
    fn esti_is_regular() {
        let e = v("esti");
        assert_eq!(e.finite(Slot::Present), "estas");
        assert_eq!(e.finite(Slot::Past), "estis");
        assert_eq!(e.finite(Slot::Future), "estos");
        assert_eq!(e.finite(Slot::Volitive), "estu");
    }

    #[test]
    fn active_participles() {
        let a = v("ami");
        assert_eq!(a.form("V.PTCP;ACT;PRS;ADJ;SG;NOM").unwrap(), "amanta");
        assert_eq!(a.form("V.PTCP;ACT;PRS;ADJ;PL;NOM").unwrap(), "amantaj");
        assert_eq!(a.form("V.PTCP;ACT;PRS;ADJ;SG;ACC").unwrap(), "amantan");
        assert_eq!(a.form("V.PTCP;ACT;PRS;ADJ;PL;ACC").unwrap(), "amantajn");
        assert_eq!(a.form("V.PTCP;ACT;PST;ADJ;SG;NOM").unwrap(), "aminta");
        assert_eq!(a.form("V.PTCP;ACT;FUT;ADJ;SG;NOM").unwrap(), "amonta");
        assert_eq!(a.form("V.PTCP;ACT;PRS;N;SG;NOM").unwrap(), "amanto");
        assert_eq!(a.form("V.PTCP;ACT;PRS;ADV").unwrap(), "amante");
    }

    #[test]
    fn passive_participles() {
        let a = v("ami");
        assert_eq!(a.form("V.PTCP;PASS;PRS;ADJ;SG;NOM").unwrap(), "amata");
        assert_eq!(a.form("V.PTCP;PASS;PST;ADJ;SG;NOM").unwrap(), "amita");
        assert_eq!(a.form("V.PTCP;PASS;FUT;ADJ;SG;NOM").unwrap(), "amota");
        assert_eq!(a.form("V.PTCP;PASS;PRS;N;PL;ACC").unwrap(), "amatojn");
        assert_eq!(a.form("V.PTCP;PASS;PST;ADV").unwrap(), "amite");
    }

    #[test]
    fn table_groups() {
        let t = Table::build(&v("ami"));
        assert_eq!(t.present, "amas");
        // CLOSINGS order: a, aj, an, ajn, o, oj, on, ojn, e
        assert_eq!(
            t.active_present,
            vec![
                "amanta", "amantaj", "amantan", "amantajn", "amanto", "amantoj", "amanton",
                "amantojn", "amante"
            ]
        );
        assert_eq!(t.passive_future[0], "amota");
    }

    #[test]
    fn non_verb_rejected() {
        assert!(Verb::from_infinitive("").is_err());
        assert!(Verb::from_infinitive("amo").is_err()); // not an -i infinitive
        assert!(Verb::from_infinitive("i").is_err()); // empty stem
        assert!(Verb::from_infinitive("du vortoj").is_err());
    }
}
