//! # ablaut
//!
//! A fast, correct German verb conjugator.
//!
//! The design follows `docs/ontology.md`: a small morphological core
//! generates the synthetic forms (Präsens, Präteritum, Konjunktiv I/II,
//! imperative, participles); analytic tenses are composed on top of it.
//!
//! Inflection classes: weak (the productive default), strong (ablaut),
//! mixed (changed stem + weak endings), preterite-present (modals and
//! *wissen*), and three stored suppletives (*sein*, *werden*, *tun*).
//! Everything irregular lives in `data/verbs.tsv`, compiled in.

mod features;
mod lexicon;
mod orthography;
mod prefix;
mod suppletive;

pub use features::{Mood, Number, Person, Tense};
use lexicon::{LexClass, LexEntry};
use orthography::attach;
use prefix::Separability;
use suppletive::Suppletive;

/// Six-slot ending rows (1sg, 2sg, 3sg, 1pl, 2pl, 3pl).
const PRESENT: [&str; 6] = ["e", "st", "t", "en", "t", "en"];
const PRETERITE_WEAK: [&str; 6] = ["te", "test", "te", "ten", "tet", "ten"];
/// Strong preterite: zero ending in 1/3sg (ich sang).
const PRETERITE_STRONG: [&str; 6] = ["", "st", "", "en", "t", "en"];
/// Konjunktiv endings; also the row for dental-carrying preterite stems
/// (hatt-e, dacht-est) and all Konjunktiv II stems (käm-e, hätt-e).
const E_ENDINGS: [&str; 6] = ["e", "est", "e", "en", "et", "en"];

/// Perfect auxiliary (Layer A lexical fact; drives the analytic tenses).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Auxiliary {
    Haben,
    Sein,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Class {
    Weak,
    Lexical(&'static LexEntry),
    Suppletive(&'static Suppletive),
}

/// A German verb, carrying the lexical facts conjugation needs (Layer A of
/// the ontology).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verb {
    infinitive: String,
    stem: String,
    /// Infinitive ends in -eln/-ern: stem ends in a schwa syllable.
    schwa_stem: bool,
    /// -ieren verbs take no ge- in the past participle.
    ieren: bool,
    class: Class,
    /// Derived prefixed verb (aufstehen): the prefix and its behavior…
    prefix: Option<(String, Separability)>,
    /// …and the base verb whose paradigm it inherits (stehen).
    base: Option<Box<Verb>>,
}

/// Errors raised when constructing a [`Verb`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// German infinitives end in -en or -n.
    InvalidInfinitive(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidInfinitive(s) => write!(f, "not a German infinitive: {s}"),
        }
    }
}

impl std::error::Error for Error {}

impl Verb {
    /// Build a verb from its infinitive, consulting the exception lexicon
    /// and falling back to the productive weak paradigm. This is the main
    /// entry point.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        // Multiword lemmas (Rad fahren, Bescheid wissen): the last word
        // conjugates; the rest is a phrasal particle.
        if let Some((particle, verb)) = infinitive.rsplit_once(' ') {
            let base = Self::from_infinitive(verb)?;
            return Ok(Verb {
                infinitive: infinitive.to_string(),
                stem: String::new(),
                schwa_stem: false,
                ieren: false,
                class: Class::Weak,
                prefix: Some((particle.to_string(), Separability::Phrasal)),
                base: Some(Box::new(base)),
            });
        }
        if let Some(s) = suppletive::lookup(infinitive) {
            return Ok(Verb {
                infinitive: infinitive.to_string(),
                stem: infinitive.trim_end_matches('n').to_string(),
                schwa_stem: false,
                ieren: false,
                class: Class::Suppletive(s),
                prefix: None,
                base: None,
            });
        }
        if let Some(entry) = lexicon::lookup(infinitive) {
            let mut v = Self::weak(infinitive)?;
            v.class = Class::Lexical(entry);
            return Ok(v);
        }
        // Prefixed verbs are derived: aufstehen inherits stehen's paradigm.
        if !lexicon::is_forced_weak(infinitive) {
            // Per-lexeme dual-prefix rulings (umarmen: um- inseparable)
            // outrank the prefix's default behavior.
            let split = lexicon::dual_override(infinitive)
                .map(|(p, sep)| (p, sep, &infinitive[p.len()..]))
                .or_else(|| prefix::split(infinitive, plausible_base));
            if let Some((p, sep, base)) = split {
                let base = Self::from_infinitive(base)?;
                // Multi-part particles fuse into one unit (vor+aus·setzen →
                // voraus·setzen: setzte voraus, not *setzte aus vor); an
                // inseparable outer freezes any inner prefix (be+an·spruchen
                // → bean·spruchen: beanspruchte). A separable outer over an
                // inseparable inner stays nested (an+vertrauen: vertraute an).
                let (p, base) = match (sep, &base.prefix) {
                    (Separability::Separable, Some((inner, Separability::Separable)))
                    | (Separability::Inseparable, Some((inner, _))) => {
                        let fused = format!("{p}{inner}");
                        let inner_base = (**base.base.as_ref().expect("prefixed")).clone();
                        (fused, inner_base)
                    }
                    _ => (p.to_string(), base),
                };
                return Ok(Verb {
                    infinitive: infinitive.to_string(),
                    stem: String::new(),
                    schwa_stem: false,
                    ieren: false,
                    class: Class::Weak,
                    prefix: Some((p, sep)),
                    base: Some(Box::new(base)),
                });
            }
        }
        Self::weak(infinitive)
    }

    /// Build a verb forced into the weak (regular) paradigm, bypassing the
    /// lexicon. Useful for novel coinages; wrong for strong verbs.
    pub fn weak(infinitive: &str) -> Result<Self, Error> {
        let schwa_stem = infinitive.ends_with("eln") || infinitive.ends_with("ern");
        let stem = if schwa_stem {
            infinitive.strip_suffix('n')
        } else {
            infinitive.strip_suffix("en")
        };
        let stem = stem.ok_or_else(|| Error::InvalidInfinitive(infinitive.into()))?;
        if stem.is_empty() {
            return Err(Error::InvalidInfinitive(infinitive.into()));
        }
        Ok(Verb {
            infinitive: infinitive.to_string(),
            stem: stem.to_string(),
            schwa_stem,
            ieren: infinitive.ends_with("ieren"),
            class: Class::Weak,
            prefix: None,
            base: None,
        })
    }

    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// True if this verb's paradigm is grounded in the exception lexicon or
    /// a suppletive entry (directly, or through its base for prefixed verbs),
    /// i.e. not conjugated by the productive weak fallback.
    pub fn is_lexical(&self) -> bool {
        match &self.base {
            Some(base) => base.is_lexical(),
            None => !matches!(self.class, Class::Weak),
        }
    }

    /// The perfect auxiliary (*haben* or *sein*). Prefixed verbs currently
    /// inherit their base's auxiliary — wrong for cases like aufstehen
    /// (sein) vs stehen (haben); per-lexeme auxiliary flags are future
    /// lexicon work.
    pub fn auxiliary(&self) -> Auxiliary {
        match (&self.base, &self.class) {
            (Some(base), _) => base.auxiliary(),
            (None, Class::Weak) => Auxiliary::Haben,
            (None, Class::Lexical(e)) => e.aux,
            (None, Class::Suppletive(s)) => s.aux,
        }
    }

    /// The zu-infinitive: separable prefixes infix zu (aufzustehen);
    /// otherwise it is the particle zu plus the infinitive (zu kaufen).
    pub fn zu_infinitive(&self) -> String {
        match &self.prefix {
            Some((p, Separability::Separable)) => {
                format!("{p}zu{}", self.base().infinitive())
            }
            // Rad zu fahren: the particle stays free, zu stays a particle.
            Some((p, Separability::Phrasal)) => {
                format!("{p} zu {}", self.base().infinitive())
            }
            _ => format!("zu {}", self.infinitive),
        }
    }

    fn base(&self) -> &Verb {
        self.base.as_ref().expect("prefixed verb has a base")
    }

    /// A finite synthetic form. Separable prefixes split off, verb-second
    /// style (*stehe auf*); the syntax of where each word lands is out of
    /// scope (see docs/ontology.md).
    pub fn conjugate(&self, tense: Tense, mood: Mood, person: Person, number: Number) -> String {
        if let Some((prefix, sep)) = &self.prefix {
            let f = self.base().conjugate(tense, mood, person, number);
            return match sep {
                Separability::Separable | Separability::Phrasal => format!("{f} {prefix}"),
                Separability::Inseparable => format!("{prefix}{f}"),
            };
        }
        let i = person.index(number);
        if let Class::Suppletive(s) = &self.class {
            return match (tense, mood) {
                (Tense::Present, Mood::Indicative) => s.present[i],
                (Tense::Preterite, Mood::Indicative) => s.preterite[i],
                (_, Mood::KonjunktivI) => s.konjunktiv1[i],
                (_, Mood::KonjunktivII) => s.konjunktiv2[i],
            }
            .to_string();
        }
        match (tense, mood) {
            (Tense::Present, Mood::Indicative) => self.present_indicative(i),
            (Tense::Preterite, Mood::Indicative) => self.preterite_indicative(i),
            // Konjunktiv I is built on the present stem; Konjunktiv II on the
            // lexical Konjunktiv II stem (weak: = preterite). Both are
            // tense-independent, so `tense` is ignored.
            (_, Mood::KonjunktivI) if i == 3 || i == 5 => self.infinitive.clone(),
            (_, Mood::KonjunktivI) => attach(&self.stem, E_ENDINGS[i], self.schwa_stem),
            (_, Mood::KonjunktivII) => match &self.class {
                Class::Weak => self.preterite_indicative(i),
                Class::Lexical(e) => attach(&e.konj2, E_ENDINGS[i], false),
                Class::Suppletive(_) => unreachable!(),
            },
        }
    }

    fn present_indicative(&self, i: usize) -> String {
        if let Class::Lexical(e) = &self.class {
            if let Some(pres) = &e.pres {
                match (e.class, i) {
                    // Präteritopräsentia: zero ending in 1sg and 3sg
                    // (ich kann, er weiß).
                    (LexClass::PreteritePresent, 0 | 2) => return pres.clone(),
                    // Changed-stem 2sg: raw attachment — no epenthesis
                    // (du hältst, du rätst), only s-coalescence (du lässt).
                    (_, 1) => return second_sg_changed(pres),
                    // Changed-stem 3sg: stems already ending in a dental take
                    // no further -t (er hält, er rät).
                    (LexClass::Strong | LexClass::Mixed, 2) => {
                        return if pres.ends_with('t') || pres.ends_with('d') {
                            pres.clone()
                        } else {
                            format!("{pres}t")
                        };
                    }
                    _ => {}
                }
            }
        }
        // Regular path: 1pl/3pl are the infinitive (wir sammeln, wir kaufen).
        if i == 3 || i == 5 {
            self.infinitive.clone()
        } else {
            attach(&self.stem, PRESENT[i], self.schwa_stem)
        }
    }

    fn preterite_indicative(&self, i: usize) -> String {
        match &self.class {
            Class::Weak => attach(&self.stem, PRETERITE_WEAK[i], self.schwa_stem),
            Class::Lexical(e) => match e.class {
                LexClass::Strong => {
                    // Sibilant stems take -est in 2sg (du saßest, du lasest);
                    // s-coalescence is a present-tense rule only.
                    if i == 1 && matches!(e.pret.chars().last(), Some('s' | 'ß' | 'x' | 'z')) {
                        format!("{}est", e.pret)
                    } else {
                        attach(&e.pret, PRETERITE_STRONG[i], false)
                    }
                }
                // Mixed and preterite-present stems carry their dental
                // (dacht-, hatt-, konnt-), so plain e-endings complete them.
                LexClass::Mixed | LexClass::PreteritePresent => {
                    attach(&e.pret, E_ENDINGS[i], false)
                }
            },
            Class::Suppletive(_) => unreachable!(),
        }
    }

    /// Imperative — exists only in the 2nd person. Returns `None` for verbs
    /// without one (the modals).
    pub fn imperative(&self, number: Number) -> Option<String> {
        if let Some((prefix, sep)) = &self.prefix {
            let imp = self.base().imperative(number)?;
            return Some(match sep {
                Separability::Separable | Separability::Phrasal => format!("{imp} {prefix}"),
                Separability::Inseparable => format!("{prefix}{imp}"),
            });
        }
        match (&self.class, number) {
            (Class::Suppletive(s), Number::Singular) => Some(s.imp_sg.to_string()),
            (Class::Suppletive(s), Number::Plural) => Some(s.imp_pl.to_string()),
            (Class::Lexical(e), Number::Singular) => match (&e.imp, e.class) {
                // e/i-alternating verbs store their bare-stem imperative
                // (sprich!); modals have none.
                (Some(imp), _) => Some(imp.clone()),
                (None, LexClass::PreteritePresent) => None,
                (None, _) => Some(self.imperative_sg_default()),
            },
            (Class::Lexical(e), Number::Plural) => {
                if e.class == LexClass::PreteritePresent && e.imp.is_none() {
                    None
                } else {
                    Some(self.conjugate(
                        Tense::Present,
                        Mood::Indicative,
                        Person::Second,
                        Number::Plural,
                    ))
                }
            }
            (Class::Weak, Number::Singular) => Some(self.imperative_sg_default()),
            (Class::Weak, Number::Plural) => Some(self.conjugate(
                Tense::Present,
                Mood::Indicative,
                Person::Second,
                Number::Plural,
            )),
        }
    }

    /// Default 2sg imperative: the bare stem (fahr!, kauf!, lass!) — the
    /// modern canonical form per Duden — with -e where it is mandatory:
    /// epenthesis stems (arbeite!, atme!), schwa stems (sammle!), and
    /// -ieren/-igen verbs (studiere!, entschuldige!).
    fn imperative_sg_default(&self) -> String {
        let mandatory_e = self.schwa_stem
            || self.ieren
            || self.infinitive.ends_with("igen")
            || orthography::needs_epenthesis(&self.stem);
        if mandatory_e {
            attach(&self.stem, "e", self.schwa_stem)
        } else {
            self.stem.clone()
        }
    }

    /// Partizip II (gekauft, gesungen, studiert, vergessen, aufgestanden).
    pub fn past_participle(&self) -> String {
        if let Some((prefix, sep)) = &self.prefix {
            let base = self.base().past_participle();
            return match sep {
                // Separable prefixes infix ge- (auf·ge·standen).
                Separability::Separable => format!("{prefix}{base}"),
                // Inseparable prefixes suppress it (verstanden).
                Separability::Inseparable => {
                    format!("{prefix}{}", base.strip_prefix("ge").unwrap_or(&base))
                }
                // Phrasal particles stay free words (Rad gefahren).
                Separability::Phrasal => format!("{prefix} {base}"),
            };
        }
        match &self.class {
            Class::Weak => {
                let base = attach(&self.stem, "t", self.schwa_stem);
                if self.ieren {
                    base
                } else {
                    format!("ge{base}")
                }
            }
            Class::Lexical(e) => e.part2.clone(),
            Class::Suppletive(s) => s.part2.to_string(),
        }
    }

    /// Partizip I (kaufend, seiend).
    pub fn present_participle(&self) -> String {
        match &self.class {
            Class::Suppletive(s) => s.part1.to_string(),
            _ => format!("{}d", self.infinitive),
        }
    }
}

/// Is this remainder of a prefix split a plausible verb on its own? Lexicon
/// and suppletive hits always qualify; otherwise require a weak-parsable
/// infinitive whose stem has a vowel and at least three characters — this
/// rejects false splits like be+ten, zu+cken, ge+igen, fest+igen, while
/// keeping auf+räumen and mit+teilen.
fn plausible_base(base: &str) -> bool {
    if suppletive::lookup(base).is_some()
        || lexicon::lookup(base).is_some()
        || lexicon::is_forced_weak(base)
    {
        return true;
    }
    Verb::weak(base).is_ok_and(|v| {
        // -ieren bases need a substantial stem (ein+studieren, not
        // da+tieren); schwa bases likewise (aus+wandern, not rum+peln).
        let min = if v.ieren {
            5
        } else if v.schwa_stem {
            4
        } else {
            3
        };
        v.stem.chars().count() >= min
            && v.stem
                .chars()
                .any(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'ä' | 'ö' | 'ü' | 'y'))
    })
}

/// 2sg present on a changed stem: raw attachment with s-coalescence only
/// (du sprichst, du lässt, du weißt, du hältst).
fn second_sg_changed(stem: &str) -> String {
    if matches!(stem.chars().last(), Some('s' | 'ß' | 'x' | 'z')) {
        format!("{stem}t")
    } else {
        format!("{stem}st")
    }
}

#[cfg(test)]
mod tests;
