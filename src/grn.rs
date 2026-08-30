//! Paraguayan Guaraní (Jopara) conjugation. Agglutinative: the finite
//! verb is a proclitic person prefix on a (possibly voice-derived) stem,
//! built productively from the bare citation stem — the lemma kaikki.org
//! keys on. Forms are emitted without the optional subject pronoun
//! (`che ajehu` → `ajehu`), matching the golden gold.
//!
//! The engine is a slot template over three things, all read from the
//! verb's inflection class (`data/grn/parts.tsv`, itself lifted verbatim
//! from the kaikki `gug-conj-*` template):
//!
//! * the **person prefix** — `a-/re-/o-` singular, `ja-/ro-/pe-/o-`
//!   plural in the indicative, a `ta-/tere-/to-…` series in the
//!   hortative, `e-/pe-` in the imperative;
//! * **nasal harmony**, which picks the 1st-plural-inclusive prefix
//!   (`ja-` oral vs `ña-` nasal) and the voice allomorphs (`je-/ñe-`
//!   passive, `jo-/ño-` reciprocal, `mbo-/mo-` coactive);
//! * the **class family** — *areal* (`a-jehu`), *aireal* which carries a
//!   person-adjacent `-i-` (`a-i-ke` → `aike`), and the small *h-*
//!   set whose glottal-initial stem takes an `h-` on the vowel-only
//!   person prefixes (`a`+`'a` → `ha'a`, `o`+`'a` → `ho'a`).
//!
//! Voice is derivational: passive `je-/ñe-`, reciprocal `jo-/ño-`
//! (plural only), coactive causative `mbo-/mo-`, and the objective
//! `ro-`/`guero-` comitative (two accepted variants). The one attested
//! nasoral verb (`soro`) mutates its coactive stem irregularly
//! (`mbo+soro` → `mo+ndoro`); that residue lives in
//! `data/grn/overrides.tsv`. The stative (chendal) class is attested by
//! a single verb and is left out of the scored paradigm.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Grammatical number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Singular,
    Plural,
}

/// The subject a finite form agrees with. First-person plural is split
/// by clusivity (inclusive `ñande` vs exclusive `ore`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Person {
    First(Number),
    /// 1st person plural exclusive (`ore`); `First(Plural)` is inclusive.
    FirstExcl,
    Second(Number),
    Third(Number),
}

/// The mood of a finite form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mood {
    Indicative,
    /// The `ta-/tere-/to-` hortative/desiderative series.
    Hortative,
    Imperative,
}

/// Derivational voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Voice {
    Active,
    /// Reflexive/passive `je-/ñe-`.
    Passive,
    /// Reciprocal `jo-/ño-` (plural only).
    Reciprocal,
    /// Coactive causative `mbo-/mo-`.
    Coactive,
    /// Objective comitative `ro-`/`guero-`.
    Objective,
}

/// The inflection-class family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Family {
    Areal,
    Aireal,
    /// The glottal-initial `h-` set.
    H,
}

/// Why an input cannot be conjugated as a Guaraní verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Guaraní verb stem.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Guaraní verb")
    }
}

/// lemma ⇥ class token (`areal-oral`, `aireal-nasal`, `h`, …).
static PARTS_TSV: &str = include_str!("../data/grn/parts.tsv");
/// lemma ⇥ canonical features ⇥ form.
static OVERRIDES_TSV: &str = include_str!("../data/grn/overrides.tsv");

fn parts() -> &'static HashMap<&'static str, (Family, bool)> {
    static MAP: OnceLock<HashMap<&'static str, (Family, bool)>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in PARTS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() || line.starts_with("lemma\t") {
                continue;
            }
            let mut c = line.split('\t');
            let (Some(lemma), Some(class)) = (c.next(), c.next()) else {
                continue;
            };
            m.insert(lemma, parse_class(class));
        }
        m
    })
}

fn overrides() -> &'static HashMap<(&'static str, &'static str), &'static str> {
    static MAP: OnceLock<HashMap<(&'static str, &'static str), &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in OVERRIDES_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            if c.len() >= 3 {
                m.insert((c[0], c[1]), c[2]);
            }
        }
        m
    })
}

/// Split a `gug-conj-*` class token into (family, nasal). `-nasoral`
/// harmonises like an oral stem; only `-nasal` (and the `h` default)
/// flips the nasal flag.
fn parse_class(token: &str) -> (Family, bool) {
    if token == "h" {
        return (Family::H, false);
    }
    let mut it = token.split('-');
    let fam = match it.next() {
        Some("aireal") => Family::Aireal,
        _ => Family::Areal,
    };
    let nasal = it.next() == Some("nasal");
    (fam, nasal)
}

/// A conjugatable Guaraní verb: its citation stem plus its class.
#[derive(Debug, Clone)]
pub struct Verb {
    lemma: String,
    family: Family,
    nasal: bool,
}

impl Verb {
    /// Build from the bare citation stem (`jehu`, `ke`, `'a`) — the
    /// kaikki lemma. Verbs absent from `parts.tsv` default to the
    /// regular areal-oral class.
    pub fn from_lemma(lemma: &str) -> Result<Self, Error> {
        let l = lemma.trim().to_lowercase();
        if l.is_empty() || l.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        let (family, nasal) = parts()
            .get(l.as_str())
            .copied()
            .unwrap_or((Family::Areal, false));
        Ok(Self {
            lemma: l,
            family,
            nasal,
        })
    }

    /// The citation stem (lemma).
    #[must_use]
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    /// The class-adjusted stem a voice prefix attaches to: the `h-` set
    /// carries its leading glottal (`y'u` → `'y'u`).
    fn stem(&self) -> String {
        if self.family == Family::H && !self.lemma.starts_with('\'') {
            format!("'{}", self.lemma)
        } else {
            self.lemma.clone()
        }
    }

    /// One conjugated form for a (voice, mood, person) cell, or `None`
    /// where the cell is not part of the paradigm (e.g. a reciprocal
    /// singular, or an imperative outside 2nd person).
    #[must_use]
    pub fn form(&self, voice: Voice, mood: Mood, person: Person) -> Option<String> {
        Some(self.forms(voice, mood, person)?.0)
    }

    /// Every accepted variant for a cell (the objective has two; the
    /// 3rd-plural adds an optional ` hikuái` postclitic). The first
    /// element is the primary form.
    fn forms(&self, voice: Voice, mood: Mood, person: Person) -> Option<(String, Vec<String>)> {
        // Reciprocal is plural-only.
        if voice == Voice::Reciprocal
            && matches!(
                person,
                Person::First(Number::Singular)
                    | Person::Second(Number::Singular)
                    | Person::Third(Number::Singular)
            )
        {
            return None;
        }
        let i = if self.family == Family::Aireal {
            "i"
        } else {
            ""
        };
        let stem = self.stem();
        // (person-harmony nasal, whether the h- prefix may apply, bodies)
        let (pnasal, h_applies, bodies): (bool, bool, Vec<String>) = match voice {
            Voice::Active => (
                self.nasal,
                self.family == Family::H,
                vec![format!("{i}{stem}")],
            ),
            Voice::Passive => {
                let p = if self.nasal { "ñe" } else { "je" };
                (self.nasal, false, vec![format!("{p}{stem}")])
            }
            Voice::Reciprocal => {
                let p = if self.nasal { "ño" } else { "jo" };
                (
                    self.nasal,
                    self.family == Family::H,
                    vec![format!("{p}{i}{stem}")],
                )
            }
            Voice::Coactive => {
                let p = if self.nasal { "mo" } else { "mbo" };
                (
                    true,
                    self.family == Family::H,
                    vec![format!("{i}{p}{stem}")],
                )
            }
            Voice::Objective => (
                self.nasal,
                self.family == Family::H,
                vec![format!("{i}ro{stem}"), format!("{i}guero{stem}")],
            ),
        };
        let prefix = person_prefix(mood, person, pnasal)?;
        let mut out = Vec::new();
        for body in &bodies {
            let p = if h_applies && h_prefix_cell(mood, person) {
                format!("h{prefix}")
            } else {
                prefix.to_string()
            };
            let form = format!("{p}{body}");
            if matches!(person, Person::Third(Number::Plural))
                && matches!(mood, Mood::Indicative | Mood::Hortative)
            {
                out.push(form.clone());
                out.push(format!("{form} hikuái"));
            } else {
                out.push(form);
            }
        }
        let primary = out.first()?.clone();
        Some((primary, out))
    }

    /// Resolve a canonical `V;VOICE;MOOD;PERSON` bundle to its accepted
    /// form(s), consulting the override table first.
    pub fn generate(&self, features: &str) -> Option<Vec<String>> {
        if let Some(f) = overrides().get(&(self.lemma.as_str(), features)) {
            return Some(vec![(*f).to_string()]);
        }
        let mut it = features.split(';');
        if it.next()? != "V" {
            return None;
        }
        let voice = parse_voice(it.next()?)?;
        let mood = parse_mood(it.next()?)?;
        let person = parse_person(it.next()?)?;
        if it.next().is_some() {
            return None;
        }
        self.forms(voice, mood, person).map(|(_, all)| all)
    }
}

/// Whether the `h-` prefix surfaces on the person marker for this cell:
/// the vowel-only prefixes `a-` (1sg ind), `o-` (3rd ind), `e-` (2sg imp).
fn h_prefix_cell(mood: Mood, person: Person) -> bool {
    match mood {
        Mood::Indicative => matches!(person, Person::First(Number::Singular) | Person::Third(_)),
        Mood::Imperative => matches!(person, Person::Second(Number::Singular)),
        Mood::Hortative => false,
    }
}

/// The person proclitic for a (mood, person), with `nasal` selecting the
/// 1st-plural-inclusive allomorph (`ja-`/`ña-`, `taja-`/`taña-`).
fn person_prefix(mood: Mood, person: Person, nasal: bool) -> Option<&'static str> {
    Some(match mood {
        Mood::Indicative => match person {
            Person::First(Number::Singular) => "a",
            Person::Second(Number::Singular) => "re",
            Person::Third(Number::Singular) => "o",
            Person::First(Number::Plural) => {
                if nasal {
                    "ña"
                } else {
                    "ja"
                }
            }
            Person::FirstExcl => "ro",
            Person::Second(Number::Plural) => "pe",
            Person::Third(Number::Plural) => "o",
        },
        Mood::Hortative => match person {
            Person::First(Number::Singular) => "ta",
            Person::Second(Number::Singular) => "tere",
            Person::Third(Number::Singular) => "to",
            Person::First(Number::Plural) => {
                if nasal {
                    "taña"
                } else {
                    "taja"
                }
            }
            Person::FirstExcl => "toro",
            Person::Second(Number::Plural) => "tape",
            Person::Third(Number::Plural) => "to",
        },
        Mood::Imperative => match person {
            Person::Second(Number::Singular) => "e",
            Person::Second(Number::Plural) => "pe",
            _ => return None,
        },
    })
}

fn parse_voice(tag: &str) -> Option<Voice> {
    Some(match tag {
        "ACT" => Voice::Active,
        "PASSIVE" => Voice::Passive,
        "RECIPROCAL" => Voice::Reciprocal,
        "COACTIVE" => Voice::Coactive,
        "OBJECTIVE" => Voice::Objective,
        _ => return None,
    })
}

fn parse_mood(tag: &str) -> Option<Mood> {
    Some(match tag {
        "INDICATIVE" => Mood::Indicative,
        "HORTATIVE" => Mood::Hortative,
        "IMPERATIVE" => Mood::Imperative,
        _ => return None,
    })
}

fn parse_person(tag: &str) -> Option<Person> {
    Some(match tag {
        "1SG" => Person::First(Number::Singular),
        "2SG" => Person::Second(Number::Singular),
        "3SG" => Person::Third(Number::Singular),
        "1PL.INCL" => Person::First(Number::Plural),
        "1PL.EXCL" => Person::FirstExcl,
        "2PL" => Person::Second(Number::Plural),
        "3PL" => Person::Third(Number::Plural),
        _ => return None,
    })
}

/// The person row of a conjugation table:
/// [1sg, 2sg, 3sg, 1pl.incl, 1pl.excl, 2pl, 3pl].
const PERSON_ROW: [Person; 7] = [
    Person::First(Number::Singular),
    Person::Second(Number::Singular),
    Person::Third(Number::Singular),
    Person::First(Number::Plural),
    Person::FirstExcl,
    Person::Second(Number::Plural),
    Person::Third(Number::Plural),
];

/// The conjugation table of a Guaraní verb, for the WebAssembly and
/// Python bindings. Each seven-slot row runs
/// [1sg, 2sg, 3sg, 1pl.incl, 1pl.excl, 2pl, 3pl]; empty strings mark
/// cells outside a voice's paradigm (reciprocal singulars). The full
/// mood/voice matrix is reached through `Verb::form`.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// Active indicative.
    pub indicative: [String; 7],
    /// Active hortative (`ta-` series).
    pub hortative: [String; 7],
    /// Active imperative ([_, 2sg, _, _, _, 2pl, _]).
    pub imperative: [String; 7],
    /// Passive/reflexive `je-/ñe-` indicative.
    pub passive: [String; 7],
    /// Reciprocal `jo-/ño-` indicative (plural cells only).
    pub reciprocal: [String; 7],
    /// Coactive causative `mbo-/mo-` indicative.
    pub coactive: [String; 7],
    /// Objective comitative `ro-` indicative (primary variant).
    pub objective: [String; 7],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row = |voice: Voice, mood: Mood| {
            PERSON_ROW.map(|p| v.form(voice, mood, p).unwrap_or_default())
        };
        Self {
            indicative: row(Voice::Active, Mood::Indicative),
            hortative: row(Voice::Active, Mood::Hortative),
            imperative: row(Voice::Active, Mood::Imperative),
            passive: row(Voice::Passive, Mood::Indicative),
            reciprocal: row(Voice::Reciprocal, Mood::Indicative),
            coactive: row(Voice::Coactive, Mood::Indicative),
            objective: row(Voice::Objective, Mood::Indicative),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Verb {
        Verb::from_lemma(s).unwrap()
    }

    fn ind(v: &Verb, p: Person) -> String {
        v.form(Voice::Active, Mood::Indicative, p).unwrap()
    }

    #[test]
    fn areal_oral_jehu() {
        let j = v("jehu");
        assert_eq!(ind(&j, Person::First(Number::Singular)), "ajehu");
        assert_eq!(ind(&j, Person::Second(Number::Singular)), "rejehu");
        assert_eq!(ind(&j, Person::Third(Number::Singular)), "ojehu");
        assert_eq!(ind(&j, Person::First(Number::Plural)), "jajehu"); // inclusive: oral ja-
        assert_eq!(ind(&j, Person::FirstExcl), "rojehu");
        assert_eq!(ind(&j, Person::Second(Number::Plural)), "pejehu");
        assert_eq!(
            j.form(
                Voice::Active,
                Mood::Imperative,
                Person::Second(Number::Singular)
            )
            .unwrap(),
            "ejehu"
        );
        // Coactive nasalises the inclusive prefix (mbo- is prenasal).
        assert_eq!(
            j.form(
                Voice::Coactive,
                Mood::Indicative,
                Person::First(Number::Plural)
            )
            .unwrap(),
            "ñambojehu"
        );
    }

    #[test]
    fn areal_nasal_mano() {
        let m = v("mano");
        assert_eq!(ind(&m, Person::First(Number::Singular)), "amano");
        // The nasal class picks ña- for the inclusive.
        assert_eq!(ind(&m, Person::First(Number::Plural)), "ñamano");
        assert_eq!(ind(&m, Person::FirstExcl), "romano");
        // Passive/reciprocal harmonise to ñe-/ño-.
        assert_eq!(
            m.form(
                Voice::Passive,
                Mood::Indicative,
                Person::First(Number::Singular)
            )
            .unwrap(),
            "añemano"
        );
        assert_eq!(
            m.form(
                Voice::Reciprocal,
                Mood::Indicative,
                Person::First(Number::Plural)
            )
            .unwrap(),
            "ñañomano"
        );
    }

    #[test]
    fn aireal_oral_ke() {
        let k = v("ke");
        assert_eq!(ind(&k, Person::First(Number::Singular)), "aike"); // a-i-ke
        assert_eq!(ind(&k, Person::Second(Number::Singular)), "reike");
        assert_eq!(ind(&k, Person::Third(Number::Singular)), "oike");
        assert_eq!(ind(&k, Person::First(Number::Plural)), "jaike");
        // The aireal -i- survives the coactive but drops under the passive.
        assert_eq!(
            k.form(
                Voice::Coactive,
                Mood::Indicative,
                Person::First(Number::Singular)
            )
            .unwrap(),
            "aimboke"
        );
        assert_eq!(
            k.form(
                Voice::Passive,
                Mood::Indicative,
                Person::First(Number::Singular)
            )
            .unwrap(),
            "ajeke"
        );
    }

    #[test]
    fn h_class_glottal() {
        let a = v("'a");
        // Vowel-only prefixes take h-: a→ha, o→ho, e→he.
        assert_eq!(ind(&a, Person::First(Number::Singular)), "ha'a");
        assert_eq!(ind(&a, Person::Third(Number::Singular)), "ho'a");
        assert_eq!(ind(&a, Person::First(Number::Plural)), "ja'a");
        assert_eq!(
            a.form(
                Voice::Active,
                Mood::Imperative,
                Person::Second(Number::Singular)
            )
            .unwrap(),
            "he'a"
        );
        // Passive drops the h-.
        assert_eq!(
            a.form(
                Voice::Passive,
                Mood::Indicative,
                Person::First(Number::Singular)
            )
            .unwrap(),
            "aje'a"
        );
    }

    #[test]
    fn objective_two_variants() {
        let j = v("jehu");
        let forms = j.generate("V;OBJECTIVE;INDICATIVE;1SG").unwrap();
        assert!(forms.contains(&"arojehu".to_string()));
        assert!(forms.contains(&"aguerojehu".to_string()));
    }

    #[test]
    fn reciprocal_singular_absent() {
        let j = v("jehu");
        assert!(j
            .form(
                Voice::Reciprocal,
                Mood::Indicative,
                Person::First(Number::Singular)
            )
            .is_none());
        assert!(j.generate("V;RECIPROCAL;INDICATIVE;1SG").is_none());
    }

    #[test]
    fn third_plural_hikuai_variant() {
        let j = v("jehu");
        let forms = j.generate("V;ACT;INDICATIVE;3PL").unwrap();
        assert!(forms.contains(&"ojehu".to_string()));
        assert!(forms.contains(&"ojehu hikuái".to_string()));
    }
}
