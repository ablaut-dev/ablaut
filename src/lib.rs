//! # ablaut
//!
//! A fast, correct German verb conjugator.
//!
//! The design follows `docs/design.md`: a small morphological core
//! generates the synthetic forms (Präsens, Präteritum, Konjunktiv I/II,
//! imperative, participles); analytic tenses are composed on top of it.
//!
//! Inflection classes: weak (the productive default), strong (ablaut),
//! mixed (changed stem + weak endings), preterite-present (modals and
//! *wissen*), and three stored suppletives (*sein*, *werden*, *tun*).
//! Everything irregular lives in `data/verbs-deu.tsv`, compiled in.

mod features;
mod lexicon;
mod orthography;
mod prefix;
#[cfg(feature = "python")]
mod python;
mod suppletive;
pub mod table;
#[cfg(feature = "wasm")]
mod wasm;

pub use features::{Mood, Number, Person, Tense};
use lexicon::{DualRuling, LexClass, LexEntry};
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

/// Analytic (periphrastic) tenses (Layer C of `docs/design.md`).
///
/// Composed from an auxiliary conjugated by the same synthetic core plus a
/// participle or infinitive; the morphological heavy lifting is already
/// done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyticTense {
    /// habe gekauft / bin gekommen
    Perfect,
    /// hatte gekauft / war gekommen
    Pluperfect,
    /// werde kaufen (with [`Mood::KonjunktivII`]: würde kaufen)
    FutureI,
    /// werde gekauft haben / werde gekommen sein
    FutureII,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Class {
    Weak,
    Lexical(&'static LexEntry),
    Suppletive(&'static Suppletive),
}

/// A German verb, carrying the lexical facts conjugation needs (Layer A of
/// `docs/design.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verb {
    infinitive: String,
    stem: String,
    /// Infinitive ends in -eln/-ern: stem ends in a schwa syllable.
    schwa_stem: bool,
    /// -ieren verbs take no ge- in the past participle.
    ieren: bool,
    /// Stem-final e merges with e-initial endings (knien: ich knie). False
    /// for archaic -een lemmas whose paradigm keeps both e's (knieen:
    /// ich kniee, du knieest).
    e_merge: bool,
    class: Class,
    /// Derived prefixed verb (aufstehen): the prefix and its behavior…
    prefix: Option<(String, Separability)>,
    /// Lexical participle override (obliegen → oblegen).
    part2_override: Option<&'static str>,
    /// …and the base verb whose paradigm it inherits (stehen).
    base: Option<Box<Self>>,
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
            Self::InvalidInfinitive(s) => write!(f, "not a German infinitive: {s}"),
        }
    }
}

impl std::error::Error for Error {}

impl Verb {
    /// Build a verb from its infinitive, consulting the exception lexicon
    /// and falling back to the productive weak paradigm. This is the main
    /// entry point.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInfinitive`] for strings that cannot be a
    /// German infinitive (no -n/-en ending, empty stem).
    // Internal expect: a prefixed base always carries its inner verb.
    #[allow(clippy::missing_panics_doc)]
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        // Normalize whitespace first: users paste " gehen ", "Rad  fahren",
        // trailing newlines or tabs. Collapses runs to single spaces, trims.
        let normalized = normalize_whitespace(infinitive);
        let mut infinitive = normalized.as_ref();
        // "zu gehen": strip the zu-infinitive particle so the bare verb
        // conjugates (no verb lemma is the two words "zu <x>").
        if let Some(rest) = infinitive
            .strip_prefix("zu ")
            .or_else(|| infinitive.strip_prefix("Zu "))
        {
            infinitive = rest;
        }
        // Multiword lemmas (Rad fahren, Bescheid wissen): the last word
        // conjugates; the rest is a phrasal particle. "sich freuen" is a
        // reflexive lemma, not a phrasal one: the pronoun must agree with
        // the subject. A trailing zu inside the particle ("Rad zu fahren",
        // "sich zu freuen") is the zu-infinitive again; drop it.
        if let Some((particle, verb)) = infinitive.rsplit_once(' ') {
            let particle = particle.strip_suffix(" zu").unwrap_or(particle);
            let (particle, sep) = if particle.eq_ignore_ascii_case("sich") {
                ("sich".to_string(), Separability::Reflexive)
            } else {
                (particle.to_string(), Separability::Phrasal)
            };
            let base = Self::from_infinitive(verb)?;
            let lemma = format!("{particle} {}", base.infinitive());
            return Ok(Self {
                infinitive: lemma,
                stem: String::new(),
                schwa_stem: false,
                ieren: false,
                e_merge: false,
                class: Class::Weak,
                prefix: Some((particle, sep)),
                part2_override: None,
                base: Some(Box::new(base)),
            });
        }
        // German verb infinitives are entirely lowercase; users type
        // "Abbrechen", "GEHEN", "AbbRechen". Lowercase the whole verb token
        // here, after the multiword split so the noun in "Rad fahren" keeps
        // its capital.
        let lowered = lower_all(infinitive);
        let infinitive = lowered.as_ref();
        if let Some(s) = suppletive::lookup(infinitive) {
            return Ok(Self {
                infinitive: infinitive.to_string(),
                stem: infinitive.trim_end_matches('n').to_string(),
                schwa_stem: false,
                ieren: false,
                e_merge: false,
                class: Class::Suppletive(s),
                prefix: None,
                part2_override: None,
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
            let ruling: Option<DualRuling> = lexicon::dual_override(infinitive);
            let split = ruling
                .map(|(p, sep, _, _)| (p, sep, &infinitive[p.len()..]))
                .or_else(|| prefix::split(infinitive, plausible_base));
            if let Some((p, sep, base)) = split {
                // A forced-weak base (umringen: denominal from Ring) must not
                // inherit the strong homograph (ringen).
                let base = if ruling.is_some_and(|(_, _, w, _)| w) {
                    Self::weak(base)?
                } else {
                    Self::from_infinitive(base)?
                };
                // Multi-part particles fuse into one unit (vor+aus·setzen →
                // voraus·setzen: setzte voraus, not *setzte aus vor); an
                // inseparable outer freezes any inner prefix (be+an·spruchen
                // → bean·spruchen: beanspruchte). A separable outer over an
                // inseparable inner stays nested (an+vertrauen: vertraute an).
                let (p, base) = match (sep, &base.prefix) {
                    (
                        Separability::Separable | Separability::Fused,
                        Some((inner, Separability::Separable)),
                    )
                    | (Separability::Inseparable, Some((inner, _))) => {
                        let fused = format!("{p}{inner}");
                        let inner_base = (**base.base.as_ref().expect("prefixed")).clone();
                        (fused, inner_base)
                    }
                    _ => (p.to_string(), base),
                };
                return Ok(Self {
                    infinitive: infinitive.to_string(),
                    stem: String::new(),
                    schwa_stem: false,
                    ieren: false,
                    e_merge: false,
                    class: Class::Weak,
                    prefix: Some((p, sep)),
                    part2_override: ruling.and_then(|(_, _, _, p2)| p2),
                    base: Some(Box::new(base)),
                });
            }
        }
        Self::weak(infinitive)
    }

    /// Build a verb forced into the weak (regular) paradigm, bypassing the
    /// lexicon. Useful for novel coinages; wrong for strong verbs.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInfinitive`] for strings that cannot be a
    /// German infinitive (no -n/-en ending, empty stem).
    pub fn weak(infinitive: &str) -> Result<Self, Error> {
        let schwa_stem = infinitive.ends_with("eln") || infinitive.ends_with("ern");
        // knien-type verbs (consonant + -ien) keep the e in their stem:
        // ich knie, du kniest, gekniet. -eien verbs (schreien) do not.
        let ien_stem = infinitive
            .strip_suffix("ien")
            .and_then(|pre| pre.chars().last())
            .is_some_and(|c| !is_vowel(c));
        let stem = if schwa_stem || ien_stem {
            infinitive.strip_suffix('n')
        } else {
            infinitive.strip_suffix("en")
        };
        let stem = stem.ok_or_else(|| Error::InvalidInfinitive(infinitive.into()))?;
        if stem.is_empty() {
            return Err(Error::InvalidInfinitive(infinitive.into()));
        }
        Ok(Self {
            infinitive: infinitive.to_string(),
            stem: stem.to_string(),
            schwa_stem,
            ieren: latinate_ieren(infinitive),
            e_merge: ien_stem,
            class: Class::Weak,
            prefix: None,
            part2_override: None,
            base: None,
        })
    }

    #[must_use]
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// True if this verb's paradigm is grounded in the exception lexicon or
    /// a suppletive entry (directly, or through its base for prefixed verbs),
    /// i.e. not conjugated by the productive weak fallback.
    #[must_use]
    pub fn is_lexical(&self) -> bool {
        self.base
            .as_ref()
            .map_or(!matches!(self.class, Class::Weak), |base| base.is_lexical())
    }

    /// The perfect auxiliary (*haben* or *sein*): per-lexeme class-"a"
    /// override if present, else inherited from the base for prefixed verbs,
    /// else the lexicon entry, defaulting to *haben*.
    #[must_use]
    pub fn auxiliary(&self) -> Auxiliary {
        if let Some(aux) = lexicon::aux_override(&self.infinitive) {
            return aux;
        }
        match (&self.base, &self.class) {
            (Some(base), _) => base.auxiliary(),
            (None, Class::Weak) => Auxiliary::Haben,
            (None, Class::Lexical(e)) => e.aux,
            (None, Class::Suppletive(s)) => s.aux,
        }
    }

    /// The zu-infinitive: separable prefixes infix zu (aufzustehen);
    /// otherwise it is the particle zu plus the infinitive (zu kaufen).
    #[must_use]
    pub fn zu_infinitive(&self) -> String {
        match &self.prefix {
            Some((p, Separability::Separable | Separability::Fused)) => {
                format!("{p}zu{}", self.base().infinitive())
            }
            // Rad zu fahren, sich zu freuen: the particle stays free,
            // zu stays a particle.
            Some((p, Separability::Phrasal | Separability::Reflexive)) => {
                format!("{p} zu {}", self.base().infinitive())
            }
            _ => format!("zu {}", self.infinitive),
        }
    }

    /// An analytic tense form. The mood conjugates the auxiliary: Perfekt
    /// Konjunktiv is *habe gekauft* (`KonjI`) or *hätte gekauft* (`KonjII`);
    /// `FutureI` + `KonjunktivII` is the *würde*-form.
    // The auxiliaries are literal known infinitives; construction cannot fail.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn analytic(
        &self,
        tense: AnalyticTense,
        mood: Mood,
        person: Person,
        number: Number,
    ) -> String {
        let aux = |name: &str| Self::from_infinitive(name).expect("auxiliary");
        let own_aux = match self.auxiliary() {
            Auxiliary::Haben => "haben",
            Auxiliary::Sein => "sein",
        };
        match tense {
            AnalyticTense::Perfect => format!(
                "{} {}",
                aux(own_aux).conjugate(Tense::Present, mood, person, number),
                self.agree_reflexive(self.past_participle(), person, number)
            ),
            AnalyticTense::Pluperfect => format!(
                "{} {}",
                aux(own_aux).conjugate(Tense::Preterite, mood, person, number),
                self.agree_reflexive(self.past_participle(), person, number)
            ),
            AnalyticTense::FutureI => format!(
                "{} {}",
                aux("werden").conjugate(Tense::Present, mood, person, number),
                self.agree_reflexive(self.infinitive.clone(), person, number)
            ),
            AnalyticTense::FutureII => format!(
                "{} {} {}",
                aux("werden").conjugate(Tense::Present, mood, person, number),
                self.agree_reflexive(self.past_participle(), person, number),
                own_aux
            ),
        }
    }

    /// Swap the citation *sich* of a reflexive lemma's phrase for the
    /// pronoun agreeing with the subject (habe **mich** gefreut).
    fn agree_reflexive(&self, phrase: String, person: Person, number: Number) -> String {
        if matches!(self.prefix, Some((_, Separability::Reflexive))) {
            if let Some(rest) = phrase.strip_prefix("sich ") {
                return format!("{} {rest}", reflexive_pronoun(person, number));
            }
        }
        phrase
    }

    /// Processual passive (Vorgangspassiv): *wird gekauft*, *wurde gekauft*.
    // werden is a literal known infinitive; construction cannot fail.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn passive(&self, tense: Tense, mood: Mood, person: Person, number: Number) -> String {
        let werden = Self::from_infinitive("werden").expect("werden");
        format!(
            "{} {}",
            werden.conjugate(tense, mood, person, number),
            self.past_participle()
        )
    }

    /// Statal passive (Zustandspassiv): *ist gekauft*, *war gekauft*.
    // sein is a literal known infinitive; construction cannot fail.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn statal_passive(
        &self,
        tense: Tense,
        mood: Mood,
        person: Person,
        number: Number,
    ) -> String {
        let sein = Self::from_infinitive("sein").expect("sein");
        format!(
            "{} {}",
            sein.conjugate(tense, mood, person, number),
            self.past_participle()
        )
    }

    /// The perfect infinitive (Infinitiv II): *gekauft haben*,
    /// *aufgestanden sein*.
    #[must_use]
    pub fn perfect_infinitive(&self) -> String {
        let aux = match self.auxiliary() {
            Auxiliary::Haben => "haben",
            Auxiliary::Sein => "sein",
        };
        format!("{} {aux}", self.past_participle())
    }

    /// The adhortative (1pl) imperative: *stehen wir auf!*, *seien wir!*.
    /// Built on Konjunktiv I, which is why *sein* comes out right.
    #[must_use]
    pub fn imperative_first_plural(&self) -> String {
        self.imperative_with_pronoun("wir", Person::First)
    }

    /// The polite (Sie) imperative: *stehen Sie auf!*, *seien Sie!*.
    #[must_use]
    pub fn imperative_polite(&self) -> String {
        self.imperative_with_pronoun("Sie", Person::Third)
    }

    /// Verb-first Konjunktiv I plural with the pronoun inserted after the
    /// finite verb (before any separable particle: *stehen wir auf*).
    fn imperative_with_pronoun(&self, pronoun: &str, person: Person) -> String {
        let form = self.conjugate(Tense::Present, Mood::KonjunktivI, person, Number::Plural);
        match form.split_once(' ') {
            Some((finite, rest)) => format!("{finite} {pronoun} {rest}"),
            None => format!("{form} {pronoun}"),
        }
    }

    fn base(&self) -> &Self {
        self.base.as_ref().expect("prefixed verb has a base")
    }

    /// A finite synthetic form. Separable prefixes split off, verb-second
    /// style (*stehe auf*); the syntax of where each word lands is out of
    /// scope (see docs/design.md).
    #[must_use]
    pub fn conjugate(&self, tense: Tense, mood: Mood, person: Person, number: Number) -> String {
        if let Some((prefix, sep)) = &self.prefix {
            let f = self.base().conjugate(tense, mood, person, number);
            return match sep {
                Separability::Separable | Separability::Phrasal => format!("{f} {prefix}"),
                Separability::Inseparable | Separability::Fused => format!("{prefix}{f}"),
                // The pronoun lands right after the finite verb, before any
                // separable particle (stellt sich vor).
                Separability::Reflexive => {
                    insert_after_finite(&f, reflexive_pronoun(person, number))
                }
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
            (_, Mood::KonjunktivI) => {
                attach(&self.stem, E_ENDINGS[i], self.schwa_stem, self.e_merge)
            }
            (_, Mood::KonjunktivII) => match &self.class {
                Class::Weak => self.preterite_indicative(i),
                Class::Lexical(e) => attach(&e.konj2, E_ENDINGS[i], false, true),
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
                        return if pres.ends_with('t') || pres.ends_with("th") {
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
            attach(&self.stem, PRESENT[i], self.schwa_stem, self.e_merge)
        }
    }

    fn preterite_indicative(&self, i: usize) -> String {
        match &self.class {
            Class::Weak => attach(&self.stem, PRETERITE_WEAK[i], self.schwa_stem, self.e_merge),
            Class::Lexical(e) => match e.class {
                LexClass::Strong => {
                    // Sibilant stems take -est in 2sg (du saßest, du lasest);
                    // s-coalescence is a present-tense rule only.
                    if i == 1 && matches!(e.pret.chars().last(), Some('s' | 'ß' | 'x' | 'z')) {
                        format!("{}est", e.pret)
                    } else {
                        attach(&e.pret, PRETERITE_STRONG[i], false, true)
                    }
                }
                // Mixed and preterite-present stems carry their dental
                // (dacht-, hatt-, konnt-), so plain e-endings complete them.
                LexClass::Mixed | LexClass::PreteritePresent => {
                    attach(&e.pret, E_ENDINGS[i], false, true)
                }
            },
            Class::Suppletive(_) => unreachable!(),
        }
    }

    /// Imperative — exists only in the 2nd person. Returns `None` for verbs
    /// without one (the modals).
    #[must_use]
    pub fn imperative(&self, number: Number) -> Option<String> {
        if let Some((prefix, sep)) = &self.prefix {
            let imp = self.base().imperative(number)?;
            return Some(match sep {
                Separability::Separable | Separability::Phrasal => format!("{imp} {prefix}"),
                Separability::Inseparable | Separability::Fused => format!("{prefix}{imp}"),
                // The imperative addresses du/ihr, pronoun before any
                // separable particle: freu dich!, stell dich vor!
                Separability::Reflexive => {
                    let pron = match number {
                        Number::Singular => "dich",
                        Number::Plural => "euch",
                    };
                    insert_after_finite(&imp, pron)
                }
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
            attach(&self.stem, "e", self.schwa_stem, self.e_merge)
        } else {
            self.stem.clone()
        }
    }

    /// Partizip II (gekauft, gesungen, studiert, vergessen, aufgestanden).
    #[must_use]
    pub fn past_participle(&self) -> String {
        if let Some(p2) = self.part2_override {
            return p2.to_string();
        }
        if let Some((prefix, sep)) = &self.prefix {
            let base = self.base().past_participle();
            return match sep {
                // Separable and fused prefixes infix ge- (auf·ge·standen,
                // lob·ge·sungen).
                Separability::Separable | Separability::Fused => format!("{prefix}{base}"),
                // Inseparable prefixes suppress it (verstanden).
                Separability::Inseparable => {
                    format!("{prefix}{}", base.strip_prefix("ge").unwrap_or(&base))
                }
                // Phrasal particles stay free words (Rad gefahren); the
                // citation participle keeps sich (sich gefreut).
                Separability::Phrasal | Separability::Reflexive => format!("{prefix} {base}"),
            };
        }
        match &self.class {
            Class::Weak => {
                let base = attach(&self.stem, "t", self.schwa_stem, self.e_merge);
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
    #[must_use]
    pub fn present_participle(&self) -> String {
        match &self.class {
            Class::Suppletive(s) => s.part1.to_string(),
            _ => format!("{}d", self.infinitive),
        }
    }
}

/// Latinate -ieren verbs (studieren, probieren) take no ge- in the past
/// participle. Native verbs that merely end in -ieren (schmieren, zieren)
/// do: their stem has no vowel before the -ier- (schm-, z-), while Latinate
/// stems always do (stud-, prob-).
fn latinate_ieren(infinitive: &str) -> bool {
    infinitive
        .strip_suffix("ieren")
        .or_else(|| infinitive.strip_suffix("iren"))
        .is_some_and(|pre| pre.chars().any(is_vowel))
}

/// Lowercase every character (German infinitives are entirely lowercase).
/// Borrows unchanged when already lowercase, so the common path allocates
/// nothing.
fn lower_all(s: &str) -> std::borrow::Cow<'_, str> {
    if s.chars().any(char::is_uppercase) {
        std::borrow::Cow::Owned(s.to_lowercase())
    } else {
        std::borrow::Cow::Borrowed(s)
    }
}

/// Trim surrounding whitespace and collapse internal runs to single spaces.
/// Borrows unchanged when the input is already clean.
fn normalize_whitespace(s: &str) -> std::borrow::Cow<'_, str> {
    let clean = s.trim();
    if clean.len() == s.len()
        && !clean.contains(|c: char| c.is_whitespace() && c != ' ')
        && !clean.contains("  ")
    {
        std::borrow::Cow::Borrowed(clean)
    } else {
        std::borrow::Cow::Owned(clean.split_whitespace().collect::<Vec<_>>().join(" "))
    }
}

/// Vowels for plausibility checks, including accented loan-word vowels
/// (crèmen must not look vowel-less).
const fn is_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'e'
            | 'i'
            | 'o'
            | 'u'
            | 'ä'
            | 'ö'
            | 'ü'
            | 'y'
            | 'à'
            | 'â'
            | 'è'
            | 'é'
            | 'ê'
            | 'î'
            | 'ô'
            | 'û'
    )
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
        // Latinate -ieren remainders are never trusted (stand+ardisieren,
        // an+odisieren — full of prefix-lookalikes); real separable compounds
        // (einstudieren) get explicit x rulings in the lexicon. Native -ieren
        // remainders (an+schmieren) are fine but need a substantial stem
        // (da+tieren is not a split).
        if v.ieren {
            return false;
        }
        let min = if v.infinitive.ends_with("ieren") {
            5
        } else if v.schwa_stem {
            4
        } else {
            3
        };
        let stem: Vec<char> = v.stem.chars().collect();
        // Schwa stems must have a vowel before their -el/-er syllable, or the
        // "vowel" is just the schwa itself (ge+ndern, da+ckeln are not verbs).
        let core = if v.schwa_stem {
            &stem[..stem.len().saturating_sub(2)]
        } else {
            &stem[..]
        };
        stem.len() >= min && core.iter().copied().any(is_vowel)
    })
}

/// 2sg present on a changed stem: raw attachment with s-coalescence only
/// (du sprichst, du lässt, du weißt, du hältst).
fn second_sg_changed(stem: &str) -> String {
    if stem.ends_with("st") {
        // A changed stem already ending in -st absorbs the whole ending
        // (du birst).
        stem.to_string()
    } else if matches!(stem.chars().last(), Some('s' | 'ß' | 'x' | 'z')) {
        format!("{stem}t")
    } else {
        format!("{stem}st")
    }
}

/// Insert a word after the finite verb of a possibly multiword form
/// (stellt vor + sich → stellt sich vor).
fn insert_after_finite(form: &str, word: &str) -> String {
    match form.split_once(' ') {
        Some((finite, rest)) => format!("{finite} {word} {rest}"),
        None => format!("{form} {word}"),
    }
}

/// The accusative reflexive pronoun agreeing with the subject.
const fn reflexive_pronoun(person: Person, number: Number) -> &'static str {
    match (person, number) {
        (Person::First, Number::Singular) => "mich",
        (Person::Second, Number::Singular) => "dich",
        (Person::First, Number::Plural) => "uns",
        (Person::Second, Number::Plural) => "euch",
        (Person::Third, _) => "sich",
    }
}

#[cfg(test)]
mod tests;
