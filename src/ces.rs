//! Czech conjugation. Class-driven presents (dělá, mluví, kupuje,
//! tiskne, bělejí) with mined class assignments in
//! `data/ces/classes.tsv` and explicit rows in `data/ces/verbs.tsv`
//! for the e-class and everything with stem alternations (psát →
//! píšu, brát → beru). The l-participle genders derive from the
//! masculine (mluvil → mluvila/mluvilo/mluvili/mluvily), the passive
//! participle likewise (mluven → mluvena/mluveni), imperative plurals
//! from the 2sg (mluv → mluvme, tiskni → tiskněme), and the
//! transgressive plural from the feminine (mluvíc → mluvíce).

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

/// The four genders Czech participles distinguish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    MasculineAnimate,
    MasculineInanimate,
    Feminine,
    Neuter,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Czech infinitive.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Czech infinitive")
    }
}

/// Present-tense class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// -ovat → kupuji/kupuje/kupují.
    Uje,
    /// -at → dělám/dělá/dělají.
    A,
    /// -it → mluvím/mluví/mluví.
    I,
    /// -et/-ět with 3pl -í (trpí).
    EtI,
    /// -et/-ět with 3pl -ějí (bělejí).
    EtEji,
    /// -nout → tisknu/tiskne/tisknou.
    Ne,
}

static CLASSES_TSV: &str = include_str!("../data/ces/classes.tsv");
static ASPECT_TSV: &str = include_str!("../data/ces/aspect.tsv");
static VERBS_TSV: &str = include_str!("../data/ces/verbs.tsv");

fn classes() -> &'static HashMap<&'static str, Class> {
    static MAP: OnceLock<HashMap<&'static str, Class>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in CLASSES_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let mut f = line.split('\t');
            let (Some(lemma), Some(class)) = (f.next(), f.next()) else {
                continue;
            };
            let class = match class {
                "uje" => Class::Uje,
                "a" => Class::A,
                "i" => Class::I,
                "et-i" => Class::EtI,
                "et-eji" => Class::EtEji,
                "ne" => Class::Ne,
                _ => continue,
            };
            m.insert(lemma, class);
        }
        m
    })
}

/// Explicit row: lemma, present 1sg…3pl, imperative 2sg/1pl/2pl,
/// l-participle masc, passive participle masc, transgressive masc,
/// transgressive fem ("-" = derive).
#[derive(Debug, Clone, Default)]
struct Row {
    present: [Option<String>; 6],
    imp: [Option<String>; 3],
    lptcp: Option<String>,
    lptcp_f: Option<String>,
    pass: Option<String>,
    cvb_m: Option<String>,
    cvb_fn: Option<String>,
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
                    present: [opt(1), opt(2), opt(3), opt(4), opt(5), opt(6)],
                    imp: [opt(7), opt(8), opt(9)],
                    lptcp: opt(10),
                    pass: opt(11),
                    cvb_m: opt(12),
                    cvb_fn: opt(13),
                    lptcp_f: opt(14),
                },
            );
        }
        m
    })
}

/// A conjugatable Czech verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    class: Class,
    row: Row,
}

impl Verb {
    /// Build a verb from its infinitive.
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
        let class = classes().get(inf).copied().unwrap_or({
            if inf.ends_with("ovat") {
                Class::Uje
            } else if inf.ends_with("nout") {
                Class::Ne
            } else if inf.ends_with("it") {
                Class::I
            } else if inf.ends_with("et") || inf.ends_with("ět") {
                Class::EtEji
            } else if inf.ends_with("at") {
                Class::A
            } else {
                Class::I
            }
        });
        Ok(Self {
            infinitive: inf.to_string(),
            class,
            row: rows().get(inf).cloned().unwrap_or_default(),
        })
    }

    /// The infinitive as normalized.
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    fn stem(&self) -> &str {
        let inf = &self.infinitive;
        match self.class {
            Class::Uje => inf.strip_suffix("ovat").unwrap_or(inf),
            Class::Ne => inf.strip_suffix("nout").unwrap_or(inf),
            Class::A => inf.strip_suffix("at").unwrap_or(inf),
            Class::I => inf.strip_suffix("it").unwrap_or(inf),
            Class::EtI | Class::EtEji => inf
                .strip_suffix("ět")
                .or_else(|| inf.strip_suffix("et"))
                .unwrap_or(inf),
        }
    }

    /// Whether the infinitive actually carries the class's theme
    /// (mluv-it, kupov-at). Athematic infinitives (být, vzít, moct,
    /// nést) fall through `stem()` unstripped, so class defaults
    /// would glue endings onto the whole infinitive (býten, vzítě,
    /// moctme); those slots must come from the lexicon row or stay
    /// empty.
    fn thematic(&self) -> bool {
        self.stem().len() < self.infinitive.len()
    }

    fn idx(person: Person, number: Number) -> usize {
        let p = match person {
            Person::First => 0,
            Person::Second => 1,
            Person::Third => 2,
        };
        p + if number == Number::Plural { 3 } else { 0 }
    }

    /// The present indicative (future meaning for perfective verbs).
    pub fn present(&self, person: Person, number: Number) -> String {
        if let Some(f) = &self.row.present[Self::idx(person, number)] {
            return f.clone();
        }
        let s = self.stem();
        let endings: [&str; 6] = match self.class {
            Class::Uje => ["uji", "uješ", "uje", "ujeme", "ujete", "ují"],
            Class::A => ["ám", "áš", "á", "áme", "áte", "ají"],
            Class::I => ["ím", "íš", "í", "íme", "íte", "í"],
            Class::EtI => ["ím", "íš", "í", "íme", "íte", "í"],
            Class::EtEji => ["ím", "íš", "í", "íme", "íte", "ějí"],
            Class::Ne => ["nu", "neš", "ne", "neme", "nete", "nou"],
        };
        format!("{s}{}", endings[Self::idx(person, number)])
    }

    /// The imperative 2sg by class (mluv, dělej, kupuj, tiskni), or
    /// None when the infinitive is athematic and the row is silent.
    fn imp2(&self) -> Option<String> {
        if let Some(f) = &self.row.imp[0] {
            return Some(f.clone());
        }
        if !self.thematic() {
            return None;
        }
        let s = self.stem();
        Some(match self.class {
            Class::Uje => format!("{s}uj"),
            Class::A => format!("{s}ej"),
            Class::Ne => format!("{s}ni"),
            Class::EtEji => format!("{s}ej"),
            Class::I | Class::EtI => imp_bare(s),
        })
    }

    /// An imperative form (2sg, 1pl, 2pl).
    pub fn imperative(&self, person: Person, number: Number) -> Option<String> {
        let (slot, suffix) = match (number, person) {
            (Number::Singular, Person::Second) => (0, ""),
            (Number::Plural, Person::First) => (1, "me"),
            (Number::Plural, Person::Second) => (2, "te"),
            _ => return None,
        };
        if let Some(f) = &self.row.imp[slot] {
            return Some(f.clone());
        }
        let base = self.imp2()?;
        if suffix.is_empty() {
            return Some(base);
        }
        // tiskni → tiskněme: final -i softens to -ě before the plural
        // endings; vowel-final bases otherwise keep their shape.
        let base = if let Some(b) = base.strip_suffix('i') {
            if b.ends_with(|c: char| !"aeiouyáéíóúůý".contains(c)) {
                format!("{b}ě")
            } else {
                base
            }
        } else {
            base
        };
        Some(format!("{base}{suffix}"))
    }

    /// The masculine singular l-participle (mluvil).
    fn lptcp_masc(&self) -> String {
        if let Some(f) = &self.row.lptcp {
            return f.clone();
        }
        let inf = &self.infinitive;
        if let Some(b) = inf.strip_suffix("nout") {
            // tisknout → tiskl after a consonant, minout → minul.
            if b.ends_with(|c: char| !"aeiouyáéíóúůý".contains(c)) {
                format!("{b}l")
            } else {
                format!("{b}nul")
            }
        } else {
            let b = &inf[..inf.len() - "t".len()];
            format!("{b}l")
        }
    }

    /// The masculine singular passive participle (mluven, dělán).
    fn pass_masc(&self) -> Option<String> {
        if let Some(f) = &self.row.pass {
            return Some(f.clone());
        }
        if !self.thematic() {
            return None;
        }
        let inf = &self.infinitive;
        let s = self.stem();
        Some(if inf.ends_with("ovat") {
            format!("{s}ován")
        } else if let Some(b) = inf.strip_suffix("nout") {
            format!("{b}nut")
        } else if inf.ends_with("at") {
            format!("{s}án")
        } else {
            // -it/-et: -en with softening the row must carry when the
            // stem mutates (prosit → prošen).
            format!("{s}en")
        })
    }

    /// Inflect a participle base across gender and number: mluvil →
    /// mluvila/mluvilo/mluvili/mluvily; mluven → mluvena/mluveni.
    fn inflect_ptcp(base: &str, gender: Gender, number: Number) -> String {
        match (number, gender) {
            (Number::Singular, Gender::MasculineAnimate | Gender::MasculineInanimate) => {
                base.to_string()
            }
            (Number::Singular, Gender::Feminine) | (Number::Plural, Gender::Neuter) => {
                format!("{base}a")
            }
            (Number::Singular, Gender::Neuter) => format!("{base}o"),
            (Number::Plural, Gender::MasculineAnimate) => format!("{base}i"),
            (Number::Plural, Gender::MasculineInanimate | Gender::Feminine) => {
                format!("{base}y")
            }
        }
    }

    /// The past (l-) participle. The jít family drops its fleeting
    /// -e- outside the masculine singular (šel → šla), carried by a
    /// mined feminine base.
    pub fn past_participle(&self, gender: Gender, number: Number) -> String {
        if let Some(fem) = &self.row.lptcp_f {
            let masc = matches!(
                (gender, number),
                (
                    Gender::MasculineAnimate | Gender::MasculineInanimate,
                    Number::Singular
                )
            );
            if !masc {
                let base = fem.strip_suffix('a').unwrap_or(fem);
                return Self::inflect_ptcp(&format!("{base}#"), gender, number).replace('#', "");
            }
        }
        Self::inflect_ptcp(&self.lptcp_masc(), gender, number)
    }

    /// The passive participle.
    pub fn passive_participle(&self, gender: Gender, number: Number) -> Option<String> {
        Some(Self::inflect_ptcp(&self.pass_masc()?, gender, number))
    }

    /// The present transgressive: masculine (mluvě), feminine/neuter
    /// (mluvíc), plural (mluvíce).
    pub fn transgressive(&self, slot: TransgressiveSlot) -> Option<String> {
        let s = self.stem();
        let default = |mk: fn(&str) -> String| self.thematic().then(|| mk(s));
        let m = self.row.cvb_m.clone().or_else(|| {
            default(match self.class {
                Class::Uje => |s| format!("{s}uje"),
                Class::A => |s| format!("{s}aje"),
                Class::I | Class::EtI => |s| format!("{}ě", soften(s)),
                Class::EtEji => |s| format!("{s}ěje"),
                Class::Ne => |s| format!("{s}na"),
            })
        });
        let f = self.row.cvb_fn.clone().or_else(|| {
            default(match self.class {
                Class::Uje => |s| format!("{s}ujíc"),
                Class::A => |s| format!("{s}ajíc"),
                Class::I | Class::EtI => |s| format!("{s}íc"),
                Class::EtEji => |s| format!("{s}ějíc"),
                Class::Ne => |s| format!("{s}nouc"),
            })
        });
        match slot {
            TransgressiveSlot::Masculine => m,
            TransgressiveSlot::FeminineNeuter => f,
            TransgressiveSlot::Plural => f.map(|f| format!("{f}e")),
        }
    }
}

/// The three transgressive forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransgressiveSlot {
    Masculine,
    FeminineNeuter,
    Plural,
}

/// The bare imperative of the í-classes: drop the theme, soften a
/// final dental (mluvit → mluv, platit → plať is mined; the default
/// keeps the consonant).
fn imp_bare(stem: &str) -> String {
    stem.to_string()
}

/// d/t/n soften to ď/ť/ň before -ě (transgressive masc).
fn soften(stem: &str) -> String {
    stem.to_string()
}

/// The aspect map mined from the gold data: lemma → (aspect,
/// counterpart lemmas). Display metadata only; the harness never
/// reads it.
fn aspect_map() -> &'static std::collections::HashMap<&'static str, (&'static str, &'static str)> {
    static MAP: std::sync::OnceLock<
        std::collections::HashMap<&'static str, (&'static str, &'static str)>,
    > = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = std::collections::HashMap::new();
        for line in ASPECT_TSV.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut f = line.split('\t');
            let (Some(lemma), Some(aspect), Some(partners)) = (f.next(), f.next(), f.next()) else {
                continue;
            };
            let aspect = if aspect == "impf" {
                "imperfective"
            } else {
                "perfective"
            };
            m.insert(lemma, (aspect, partners.split(',').next().unwrap_or("")));
        }
        m
    })
}

/// The full conjugation table of a Czech verb — shared by the
/// WebAssembly and Python bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// "imperfective" or "perfective", where the lexicon knows it.
    pub aspect: Option<&'static str>,
    /// The other half of the aspect pair (dělat → udělat).
    pub aspect_counterpart: Option<&'static str>,
    pub present: [String; 6],
    /// [2sg, 1pl, 2pl].
    pub imperative: [Option<String>; 3],
    /// [masc-sg, fem-sg, neut-sg, masc-anim-pl, fem-pl, neut-pl].
    pub past_participle: [String; 6],
    pub passive_participle: Option<[String; 6]>,
    /// [masc, fem/neut, plural].
    pub transgressive: [Option<String>; 3],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row6 = |f: &dyn Fn(Gender, Number) -> String| {
            [
                f(Gender::MasculineAnimate, Number::Singular),
                f(Gender::Feminine, Number::Singular),
                f(Gender::Neuter, Number::Singular),
                f(Gender::MasculineAnimate, Number::Plural),
                f(Gender::Feminine, Number::Plural),
                f(Gender::Neuter, Number::Plural),
            ]
        };
        Self {
            infinitive: v.infinitive().to_string(),
            aspect: aspect_map().get(v.infinitive()).map(|(a, _)| *a),
            aspect_counterpart: aspect_map()
                .get(v.infinitive())
                .map(|(_, p)| *p)
                .filter(|p| !p.is_empty()),
            present: [
                v.present(Person::First, Number::Singular),
                v.present(Person::Second, Number::Singular),
                v.present(Person::Third, Number::Singular),
                v.present(Person::First, Number::Plural),
                v.present(Person::Second, Number::Plural),
                v.present(Person::Third, Number::Plural),
            ],
            imperative: [
                v.imperative(Person::Second, Number::Singular),
                v.imperative(Person::First, Number::Plural),
                v.imperative(Person::Second, Number::Plural),
            ],
            past_participle: row6(&|g, n| v.past_participle(g, n)),
            passive_participle: v
                .passive_participle(Gender::MasculineAnimate, Number::Singular)
                .map(|_| row6(&|g, n| v.passive_participle(g, n).unwrap_or_default())),
            transgressive: [
                v.transgressive(TransgressiveSlot::Masculine),
                v.transgressive(TransgressiveSlot::FeminineNeuter),
                v.transgressive(TransgressiveSlot::Plural),
            ],
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
    fn class_defaults() {
        let m = v("mluvit");
        assert_eq!(m.present(Person::First, Number::Singular), "mluvím");
        assert_eq!(m.present(Person::Third, Number::Plural), "mluví");
        assert_eq!(
            m.past_participle(Gender::Feminine, Number::Singular),
            "mluvila"
        );
        let k = v("kupovat");
        assert_eq!(k.present(Person::Third, Number::Singular), "kupuje");
        assert_eq!(
            k.imperative(Person::Second, Number::Singular).unwrap(),
            "kupuj"
        );
        let d = v("dělat");
        assert_eq!(d.present(Person::Third, Number::Plural), "dělají");
        assert_eq!(
            d.imperative(Person::Second, Number::Singular).unwrap(),
            "dělej"
        );
        let t = v("tisknout");
        assert_eq!(t.present(Person::First, Number::Singular), "tisknu");
        assert_eq!(
            t.past_participle(Gender::MasculineAnimate, Number::Singular),
            "tiskl"
        );
        assert_eq!(
            t.imperative(Person::First, Number::Plural).unwrap(),
            "tiskněme"
        );
    }

    /// Athematic infinitives (být, moct, vzít) must not overgenerate:
    /// class defaults would glue endings onto the whole infinitive
    /// (býten, moctme, vzítě), so underived slots stay empty (#88).
    #[test]
    fn athematic_no_overgeneration() {
        let byt = v("být");
        assert_eq!(
            byt.passive_participle(Gender::MasculineAnimate, Number::Singular),
            None,
            "být has no passive participle (was: býten)"
        );
        assert_eq!(
            byt.imperative(Person::Second, Number::Singular).unwrap(),
            "buď"
        );
        assert_eq!(
            byt.transgressive(TransgressiveSlot::Masculine).unwrap(),
            "jsa"
        );

        let moct = v("moct");
        assert_eq!(moct.imperative(Person::Second, Number::Singular), None);
        assert_eq!(moct.imperative(Person::First, Number::Plural), None);
        assert_eq!(moct.imperative(Person::Second, Number::Plural), None);
        assert_eq!(
            moct.passive_participle(Gender::MasculineAnimate, Number::Singular),
            None,
            "moct has no passive participle (was: mocten)"
        );
        assert_eq!(
            moct.transgressive(TransgressiveSlot::Masculine).unwrap(),
            "moha"
        );

        let vzit = v("vzít");
        assert_eq!(
            vzit.transgressive(TransgressiveSlot::Masculine),
            None,
            "vzít has no present transgressive (was: vzítě)"
        );
        assert_eq!(vzit.transgressive(TransgressiveSlot::Plural), None);
        assert_eq!(
            vzit.passive_participle(Gender::MasculineAnimate, Number::Singular)
                .unwrap(),
            "vzat"
        );
        assert_eq!(
            vzit.imperative(Person::First, Number::Plural).unwrap(),
            "vezměme"
        );

        for lemma in ["jít", "mít"] {
            assert_eq!(
                v(lemma).passive_participle(Gender::MasculineAnimate, Number::Singular),
                None,
                "{lemma} has no passive participle"
            );
        }
        assert_eq!(
            v("znát")
                .passive_participle(Gender::MasculineAnimate, Number::Plural)
                .unwrap(),
            "znáni"
        );
    }
}
