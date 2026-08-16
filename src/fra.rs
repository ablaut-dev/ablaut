//! French conjugation: the first- and second-group engine.
//!
//! This is the seed of the French core. The first group (-er) comes with
//! its orthographic alternations:
//!
//! - softening before a/o endings: *commencer → commençons*, *manger → mangeons*
//! - `-yer` verbs: *y → i* before a mute ending (*payer → paie*, *employer →
//!   emploie*)
//! - `e + consonant + er`: *e → è* before a mute ending (*lever → lève*),
//!   except `-eler`/`-eter` verbs, which default to doubling the consonant
//!   (*appeler → appelle*, *jeter → jette*) unless listed in
//!   [`GRAVE_ELER_ETER`] (*geler → gèle*, *acheter → achète*)
//! - `é + consonant + er`: *é → è* before a mute ending (*céder → cède*) but
//!   not in the future/conditional (*céderai*, classical orthography)
//!
//! The second group (-ir with the -iss- infix: *finir → finissons*) is the
//! productive default for -ir infinitives; the closed class of third-group
//! -ir bases ([`THIRD_GROUP_IR`]) is excluded by suffix, with
//! [`SECOND_GROUP_ANYWAY`] overriding false collisions (*asservir*,
//! *assortir*, *répartir*).
//!
//! Third-group verbs (-oir, -re, the -ir bases), and the irregular
//! first-group verbs *aller* and the *envoyer* family (irregular future
//! *enverr-*), return [`Error::Unsupported`] until the lexicon lands.

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

/// The seven synthetic tense/mood combinations of French. Compound tenses
/// (passé composé, plus-que-parfait, …) are the compositional layer's
/// business, as in German.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpleTense {
    Present,
    Imperfect,
    PastHistoric,
    Future,
    Conditional,
    SubjunctivePresent,
    SubjunctiveImperfect,
}

/// Why an infinitive cannot be conjugated (yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Not a first-group verb, or a first-group verb whose paradigm needs
    /// the (future) lexicon: *aller*, the *envoyer* family.
    Unsupported,
    /// The input does not look like a French infinitive at all.
    NotAVerb,
}

/// `-eler`/`-eter` verbs that take a grave accent (*gèle*) instead of the
/// default consonant doubling (*appelle*). Base verbs; prefixed derivatives
/// (*dégeler*, *racheter*) are matched by suffix.
const GRAVE_ELER_ETER: [&str; 17] = [
    "geler",
    "peler",
    "celer",
    "harceler",
    "ciseler",
    "démanteler",
    "écarteler",
    "marteler",
    "modeler",
    "acheter",
    "corseter",
    "crocheter",
    "fureter",
    "fileter",
    "haleter",
    "craqueter",
    "paqueter",
];

/// Prefixes under which a [`GRAVE_ELER_ETER`] base keeps its behavior
/// (*dégeler*, *racheter*). Deliberately closed: *appeler* must not match
/// *peler*.
const GRAVE_PREFIXES: [&str; 7] = ["", "dé", "con", "décon", "sur", "r", "em"];

/// First-group irregulars whose paradigm the rules cannot produce:
/// *aller* and its derivative *raller*.
const IRREGULAR_ER: [&str; 2] = ["aller", "raller"];

/// Third-group `-ir` bases (no *-iss-* infix): a closed class, matched by
/// suffix so prefixed derivatives (*repartir*, *accueillir*, *soutenir*)
/// are caught too. *haïr* is here for its diaeresis alternation (*hais*).
const THIRD_GROUP_IR: [&str; 27] = [
    "partir", "sortir", "dormir", "servir", "mentir", "sentir", "repentir", "tenir", "venir",
    "courir", "mourir", "ouvrir", "couvrir", "offrir", "souffrir", "cueillir", "saillir",
    "faillir", "bouillir", "fuir", "vêtir", "quérir", "gésir", "ouïr", "férir", "haïr", "issir",
];

/// Second-group verbs that would falsely match a [`THIRD_GROUP_IR`] suffix:
/// *asservir* is not *servir*, *assortir* not *sortir*, *répartir* not
/// *partir*.
const SECOND_GROUP_ANYWAY: [&str; 6] = [
    "asservir",
    "assortir",
    "rassortir",
    "réassortir",
    "répartir",
    "impartir",
];

const PRESENT: [&str; 6] = ["e", "es", "e", "ons", "ez", "ent"];
const IMPERFECT: [&str; 6] = ["ais", "ais", "ait", "ions", "iez", "aient"];
const PAST_HISTORIC: [&str; 6] = ["ai", "as", "a", "âmes", "âtes", "èrent"];
const FUTURE: [&str; 6] = ["ai", "as", "a", "ons", "ez", "ont"];
const CONDITIONAL: [&str; 6] = ["ais", "ais", "ait", "ions", "iez", "aient"];
const SUBJ_PRESENT: [&str; 6] = ["e", "es", "e", "ions", "iez", "ent"];
const SUBJ_IMPERFECT: [&str; 6] = ["asse", "asses", "ât", "assions", "assiez", "assent"];

const PRESENT_IR: [&str; 6] = ["is", "is", "it", "issons", "issez", "issent"];
const IMPERFECT_IR: [&str; 6] = [
    "issais", "issais", "issait", "issions", "issiez", "issaient",
];
const PAST_HISTORIC_IR: [&str; 6] = ["is", "is", "it", "îmes", "îtes", "irent"];
const SUBJ_PRESENT_IR: [&str; 6] = ["isse", "isses", "isse", "issions", "issiez", "issent"];
const SUBJ_IMPERFECT_IR: [&str; 6] = ["isse", "isses", "ît", "issions", "issiez", "issent"];

/// Inflection class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Group {
    /// First group: -er.
    Er,
    /// Second group: -ir with the -iss- infix.
    Ir,
}

/// A conjugatable French verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    /// Infinitive minus the group ending.
    stem: String,
    group: Group,
}

impl Person {
    const fn index(self, number: Number) -> usize {
        let p = match self {
            Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
        };
        match number {
            Number::Singular => p,
            Number::Plural => p + 3,
        }
    }
}

/// True if `ending` starts with a mute *e* (so the stem's own vowel carries
/// the syllable and may need adjusting: *lève*, *appelle*, *paie*).
fn is_mute(ending: &str) -> bool {
    matches!(ending.as_bytes().first(), Some(b'e')) && !ending.starts_with("ez")
}

impl Verb {
    /// Build a verb from its infinitive. Regular first-group (-er) and
    /// second-group (-ir/-iss-) verbs are accepted.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim();
        if inf.is_empty() || inf.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        let (stem, group) = if let Some(stem) = inf.strip_suffix("er") {
            if IRREGULAR_ER.contains(&inf) || inf.ends_with("envoyer") {
                return Err(Error::Unsupported);
            }
            (stem, Group::Er)
        } else if let Some(stem) = inf.strip_suffix("ir") {
            // -oir verbs (voir, devoir, …) are third group wholesale, and
            // so are the THIRD_GROUP_IR bases — unless the match is a
            // false suffix collision (asservir vs servir).
            if inf.ends_with("oir") {
                return Err(Error::Unsupported);
            }
            if !SECOND_GROUP_ANYWAY.contains(&inf)
                && THIRD_GROUP_IR.iter().any(|base| inf.ends_with(base))
            {
                return Err(Error::Unsupported);
            }
            (stem, Group::Ir)
        } else {
            return Err(Error::Unsupported);
        };
        // A bare ending or a stem without a vowel is not a verb.
        if stem.is_empty() || !stem.chars().any(|c| "aeiouyàâéèêëîïôûü".contains(c)) {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf.to_string(),
            stem: stem.to_string(),
            group,
        })
    }

    /// The infinitive as normalized.
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// Whether this `-eler`/`-eter` verb doubles its consonant (*appelle*)
    /// rather than taking a grave accent (*gèle*).
    fn doubles(&self) -> bool {
        (self.stem.ends_with("el") || self.stem.ends_with("et"))
            && !GRAVE_ELER_ETER.iter().any(|base| {
                self.infinitive.strip_suffix(base).is_some_and(|prefix| {
                    GRAVE_PREFIXES.contains(&prefix) || prefix == "re" || prefix == "ré"
                })
            })
    }

    /// The stem adjusted for a mute ending: *y → i*, doubling, or *e/é → è*.
    /// `in_future` selects the future/conditional stem, where classical
    /// orthography keeps *é* (*céderai*).
    fn mute_stem(&self, in_future: bool) -> String {
        let s = &self.stem;
        // -yer: employer → emploi-, payer → pai- (the -ayer verbs allow both
        // paie/paye; the i-form is the canonical output).
        if let Some(body) = s.strip_suffix('y') {
            if body.ends_with(['a', 'o', 'u']) {
                return format!("{body}i");
            }
        }
        if self.doubles() {
            let last = s.chars().last().unwrap();
            return format!("{s}{last}");
        }
        // Last e/é in an open syllable takes a grave accent: lève, cède,
        // sèvre (consonant + liquid), sèche/règne/lègue (digraphs count as
        // one consonant sound). A closed syllable (accepte, ferme), a
        // doubled consonant (regrette, interpelle), a glide (grasseye), or
        // a loanword consonant (vexe, interviewe) leaves the e alone.
        let chars: Vec<char> = s.chars().collect();
        // Collect consonant units from the end until the last vowel,
        // nearest-the-end first. "gu"/"qu" and ch/gn/ph/th are one unit.
        let mut units: Vec<String> = Vec::new();
        let mut i = chars.len();
        while i > 0 {
            let c = chars[i - 1];
            if "aeiouyàâéèêëîïôûü".contains(c) {
                // A stem-final "gu"/"qu" is one consonant sound (lègue,
                // remarquer); anywhere else the u is a real vowel (régule).
                if c == 'u' && units.is_empty() && i >= 2 && matches!(chars[i - 2], 'g' | 'q') {
                    units.push(chars[i - 2..i].iter().collect());
                    i -= 2;
                    continue;
                }
                break;
            }
            if i >= 2 {
                let d: String = chars[i - 2..i].iter().collect();
                if ["ch", "gn", "ph", "th"].contains(&d.as_str()) {
                    units.push(d);
                    i -= 2;
                    continue;
                }
            }
            units.push(c.to_string());
            i -= 1;
        }
        let liquid = |u: &str| u == "r" || u == "l";
        let clean = units
            .iter()
            .all(|u| !u.contains(['x', 'w', 'ç', '-', '\'']));
        let open = clean
            && match units.as_slice() {
                [_] => true,
                [last, first] => liquid(last) && !liquid(first),
                _ => false,
            };
        if i > 0 && open {
            let grave = match chars[i - 1] {
                'e' => true,
                'é' => !in_future,
                _ => false,
            };
            if grave {
                let head: String = chars[..i - 1].iter().collect();
                let tail: String = chars[i..].iter().collect();
                return format!("{head}è{tail}");
            }
        }
        s.clone()
    }

    /// Attach an ending, softening a stem-final *c*/*g* before *a*/*â*/*o*.
    fn attach(stem: &str, ending: &str) -> String {
        let soft =
            matches!(ending.as_bytes().first(), Some(b'a' | b'o')) || ending.starts_with('â');
        if soft && stem.ends_with('c') {
            format!("{}ç{ending}", &stem[..stem.len() - 1])
        } else if soft && stem.ends_with('g') {
            format!("{stem}e{ending}")
        } else {
            format!("{stem}{ending}")
        }
    }

    /// The future/conditional stem: the (adjusted) infinitive.
    fn future_stem(&self) -> String {
        format!("{}er", self.mute_stem(true))
    }

    /// A finite form.
    pub fn conjugate(&self, tense: SimpleTense, person: Person, number: Number) -> String {
        let i = person.index(number);
        if self.group == Group::Ir {
            // The second group is agglutinative: bare stem + ending, the
            // future/conditional on the whole infinitive. No orthographic
            // adjustments apply.
            let (endings, base): (&[&str; 6], &str) = match tense {
                SimpleTense::Present => (&PRESENT_IR, &self.stem),
                SimpleTense::Imperfect => (&IMPERFECT_IR, &self.stem),
                SimpleTense::PastHistoric => (&PAST_HISTORIC_IR, &self.stem),
                SimpleTense::Future => (&FUTURE, &self.infinitive),
                SimpleTense::Conditional => (&CONDITIONAL, &self.infinitive),
                SimpleTense::SubjunctivePresent => (&SUBJ_PRESENT_IR, &self.stem),
                SimpleTense::SubjunctiveImperfect => (&SUBJ_IMPERFECT_IR, &self.stem),
            };
            return format!("{base}{}", endings[i]);
        }
        let (endings, base): (&[&str; 6], String) = match tense {
            SimpleTense::Present => (&PRESENT, self.stem.clone()),
            SimpleTense::Imperfect => (&IMPERFECT, self.stem.clone()),
            SimpleTense::PastHistoric => (&PAST_HISTORIC, self.stem.clone()),
            SimpleTense::Future => (&FUTURE, self.future_stem()),
            SimpleTense::Conditional => (&CONDITIONAL, self.future_stem()),
            SimpleTense::SubjunctivePresent => (&SUBJ_PRESENT, self.stem.clone()),
            SimpleTense::SubjunctiveImperfect => (&SUBJ_IMPERFECT, self.stem.clone()),
        };
        let ending = endings[i];
        let stem = if is_mute(ending)
            && matches!(
                tense,
                SimpleTense::Present | SimpleTense::SubjunctivePresent
            ) {
            self.mute_stem(false)
        } else {
            base
        };
        Self::attach(&stem, ending)
    }

    /// The imperative: 2sg (*parle*, *finis*), 1pl (*parlons*,
    /// *finissons*), 2pl (*parlez*, *finissez*). Other bundles have no
    /// imperative.
    pub fn imperative(&self, person: Person, number: Number) -> Option<String> {
        if self.group == Group::Ir {
            return match (person, number) {
                (Person::Second, Number::Singular) => Some(format!("{}is", self.stem)),
                (Person::First, Number::Plural) => Some(format!("{}issons", self.stem)),
                (Person::Second, Number::Plural) => Some(format!("{}issez", self.stem)),
                _ => None,
            };
        }
        match (person, number) {
            (Person::Second, Number::Singular) => Some(Self::attach(&self.mute_stem(false), "e")),
            (Person::First, Number::Plural) => Some(Self::attach(&self.stem, "ons")),
            (Person::Second, Number::Plural) => Some(Self::attach(&self.stem, "ez")),
            _ => None,
        }
    }

    /// Present participle: *parlant*, *commençant*, *finissant*.
    pub fn present_participle(&self) -> String {
        match self.group {
            Group::Er => Self::attach(&self.stem, "ant"),
            Group::Ir => format!("{}issant", self.stem),
        }
    }

    /// Past participle, masculine singular: *parlé*, *fini*.
    pub fn past_participle(&self) -> String {
        match self.group {
            Group::Er => format!("{}é", self.stem),
            Group::Ir => format!("{}i", self.stem),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};
    use SimpleTense::{
        Conditional, Future, Imperfect, PastHistoric, Present, SubjunctiveImperfect,
        SubjunctivePresent,
    };

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn regular_er_paradigm() {
        let p = v("parler");
        assert_eq!(p.conjugate(Present, P1, SG), "parle");
        assert_eq!(p.conjugate(Present, P1, PL), "parlons");
        assert_eq!(p.conjugate(Imperfect, P3, PL), "parlaient");
        assert_eq!(p.conjugate(PastHistoric, P1, PL), "parlâmes");
        assert_eq!(p.conjugate(PastHistoric, P3, PL), "parlèrent");
        assert_eq!(p.conjugate(Future, P1, SG), "parlerai");
        assert_eq!(p.conjugate(Conditional, P3, SG), "parlerait");
        assert_eq!(p.conjugate(SubjunctivePresent, P1, PL), "parlions");
        assert_eq!(p.conjugate(SubjunctiveImperfect, P3, SG), "parlât");
        assert_eq!(p.imperative(P2, SG).unwrap(), "parle");
        assert_eq!(p.present_participle(), "parlant");
        assert_eq!(p.past_participle(), "parlé");
    }

    #[test]
    fn softening_cer_ger() {
        let c = v("commencer");
        assert_eq!(c.conjugate(Present, P1, PL), "commençons");
        assert_eq!(c.conjugate(Imperfect, P1, SG), "commençais");
        assert_eq!(c.conjugate(Imperfect, P1, PL), "commencions");
        assert_eq!(c.conjugate(PastHistoric, P3, SG), "commença");
        assert_eq!(c.conjugate(PastHistoric, P1, PL), "commençâmes");
        assert_eq!(c.conjugate(PastHistoric, P3, PL), "commencèrent");
        assert_eq!(c.conjugate(SubjunctiveImperfect, P3, SG), "commençât");
        assert_eq!(c.present_participle(), "commençant");
        let m = v("manger");
        assert_eq!(m.conjugate(Present, P1, PL), "mangeons");
        assert_eq!(m.conjugate(Imperfect, P1, SG), "mangeais");
        assert_eq!(m.conjugate(PastHistoric, P3, SG), "mangea");
        assert_eq!(m.conjugate(PastHistoric, P3, PL), "mangèrent");
        assert_eq!(m.present_participle(), "mangeant");
    }

    #[test]
    fn yer_verbs() {
        let p = v("payer");
        assert_eq!(p.conjugate(Present, P1, SG), "paie");
        assert_eq!(p.conjugate(Future, P1, SG), "paierai");
        assert_eq!(p.conjugate(Present, P1, PL), "payons");
        let e = v("employer");
        assert_eq!(e.conjugate(Present, P3, SG), "emploie");
        assert_eq!(e.conjugate(Future, P3, PL), "emploieront");
        assert_eq!(e.conjugate(Imperfect, P2, PL), "employiez");
        let g = v("grasseyer");
        assert_eq!(g.conjugate(Present, P1, SG), "grasseye");
    }

    #[test]
    fn mute_e_stems() {
        let l = v("lever");
        assert_eq!(l.conjugate(Present, P1, SG), "lève");
        assert_eq!(l.conjugate(Present, P1, PL), "levons");
        assert_eq!(l.conjugate(Future, P1, SG), "lèverai");
        let s = v("sevrer");
        assert_eq!(s.conjugate(Present, P3, SG), "sèvre");
        let c = v("céder");
        assert_eq!(c.conjugate(Present, P1, SG), "cède");
        assert_eq!(c.conjugate(Future, P1, SG), "céderai");
        let i = v("interpeller");
        assert_eq!(i.conjugate(Present, P1, SG), "interpelle");
        let r = v("regretter");
        assert_eq!(r.conjugate(Present, P1, SG), "regrette");
    }

    #[test]
    fn eler_eter_doubling_and_graves() {
        assert_eq!(v("appeler").conjugate(Present, P1, SG), "appelle");
        assert_eq!(v("jeter").conjugate(Present, P3, SG), "jette");
        assert_eq!(v("jeter").conjugate(Future, P1, SG), "jetterai");
        assert_eq!(v("geler").conjugate(Present, P1, SG), "gèle");
        assert_eq!(v("congeler").conjugate(Present, P1, SG), "congèle");
        assert_eq!(v("acheter").conjugate(Present, P1, SG), "achète");
        assert_eq!(v("racheter").conjugate(Future, P1, SG), "rachèterai");
    }

    #[test]
    fn second_group_paradigm() {
        let f = v("finir");
        assert_eq!(f.conjugate(Present, P1, SG), "finis");
        assert_eq!(f.conjugate(Present, P3, SG), "finit");
        assert_eq!(f.conjugate(Present, P1, PL), "finissons");
        assert_eq!(f.conjugate(Present, P3, PL), "finissent");
        assert_eq!(f.conjugate(Imperfect, P1, SG), "finissais");
        assert_eq!(f.conjugate(PastHistoric, P1, PL), "finîmes");
        assert_eq!(f.conjugate(PastHistoric, P3, PL), "finirent");
        assert_eq!(f.conjugate(Future, P1, SG), "finirai");
        assert_eq!(f.conjugate(Conditional, P3, PL), "finiraient");
        assert_eq!(f.conjugate(SubjunctivePresent, P1, SG), "finisse");
        assert_eq!(f.conjugate(SubjunctiveImperfect, P3, SG), "finît");
        assert_eq!(f.imperative(P2, SG).unwrap(), "finis");
        assert_eq!(f.imperative(P1, PL).unwrap(), "finissons");
        assert_eq!(f.present_participle(), "finissant");
        assert_eq!(f.past_participle(), "fini");
    }

    #[test]
    fn second_group_classification() {
        // Productive second group, including collision-prone lemmas.
        for ok in [
            "atterrir",
            "jouir",
            "nourrir",
            "jaillir",
            "asservir",
            "assortir",
            "répartir",
        ] {
            assert!(Verb::from_infinitive(ok).is_ok(), "{ok}");
        }
        // Third group: base verbs and prefixed derivatives.
        for bad in [
            "partir",
            "repartir",
            "dormir",
            "endormir",
            "tenir",
            "soutenir",
            "venir",
            "devenir",
            "courir",
            "secourir",
            "ouvrir",
            "découvrir",
            "cueillir",
            "accueillir",
            "fuir",
            "vêtir",
            "revêtir",
            "acquérir",
            "bouillir",
            "haïr",
            "voir",
            "devoir",
            "pouvoir",
        ] {
            assert_eq!(
                Verb::from_infinitive(bad).unwrap_err(),
                Error::Unsupported,
                "{bad}"
            );
        }
    }

    #[test]
    fn unsupported() {
        assert_eq!(
            Verb::from_infinitive("aller").unwrap_err(),
            Error::Unsupported
        );
        assert_eq!(
            Verb::from_infinitive("envoyer").unwrap_err(),
            Error::Unsupported
        );
        assert_eq!(
            Verb::from_infinitive("vendre").unwrap_err(),
            Error::Unsupported
        );
        assert_eq!(
            Verb::from_infinitive("xyz").unwrap_err(),
            Error::Unsupported
        );
    }
}
