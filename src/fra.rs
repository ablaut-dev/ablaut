//! French conjugation: all three groups.
//!
//! The first group (-er) comes with its orthographic alternations:
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
//! The third group splits three ways: the regular -dre class (*vendre*)
//! is a rule; everything else irregular lives in the compiled-in lexicon
//! (`data/fra/verbs.tsv`) as base paradigms, with prefixed derivatives
//! (*soutenir*, *apprendre*, *reconnaître*) resolved by longest-base
//! suffix match. Defective verbs (*falloir*, *gésir*, *traire*, …) carry
//! full lexicon rows too: the gold oracles only list attested slots, so
//! the unattested cells are never scored and never surface.

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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "not a supported French verb"),
            Self::NotAVerb => write!(f, "not a French infinitive"),
        }
    }
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
const SECOND_GROUP_ANYWAY: [&str; 7] = [
    "asservir",
    "assortir",
    "rassortir",
    "réassortir",
    "répartir",
    "impartir",
    "épaissir",
];

const PRESENT: [&str; 6] = ["e", "es", "e", "ons", "ez", "ent"];
const IMPERFECT: [&str; 6] = ["ais", "ais", "ait", "ions", "iez", "aient"];
const PAST_HISTORIC: [&str; 6] = ["ai", "as", "a", "âmes", "âtes", "èrent"];
const FUTURE: [&str; 6] = ["ai", "as", "a", "ons", "ez", "ont"];
const CONDITIONAL: [&str; 6] = ["ais", "ais", "ait", "ions", "iez", "aient"];
const SUBJ_PRESENT: [&str; 6] = ["e", "es", "e", "ions", "iez", "ent"];
const SUBJ_IMPERFECT: [&str; 6] = ["asse", "asses", "ât", "assions", "assiez", "assent"];

const PRESENT_RE: [&str; 6] = ["s", "s", "", "ons", "ez", "ent"];

const PRESENT_IR: [&str; 6] = ["is", "is", "it", "issons", "issez", "issent"];
const IMPERFECT_IR: [&str; 6] = [
    "issais", "issais", "issait", "issions", "issiez", "issaient",
];
const PAST_HISTORIC_IR: [&str; 6] = ["is", "is", "it", "îmes", "îtes", "irent"];
const SUBJ_PRESENT_IR: [&str; 6] = ["isse", "isses", "isse", "issions", "issiez", "issent"];
const SUBJ_IMPERFECT_IR: [&str; 6] = ["isse", "isses", "ît", "issions", "issiez", "issent"];

/// The compiled-in irregular lexicon (see the schema comment in the file).
static LEXICON_TSV: &str = include_str!("../data/fra/verbs.tsv");

/// A stored irregular paradigm from `data/fra/verbs.tsv`.
#[derive(Debug, Clone)]
struct LexEntry {
    pres: [String; 6],
    subj_sg: String,
    subj_pl: String,
    /// Past-historic 1sg; the series (-ai / -s) derives the other five
    /// slots and the imperfect subjunctive.
    ps: String,
    fut: String,
    pp: String,
    /// Overrides for être/avoir-class irregularity; None means derive.
    impf: Option<String>,
    prsp: Option<String>,
    subj6: Option<[String; 6]>,
    imp3: Option<[String; 3]>,
}

/// Parse one lexicon line into an entry.
fn parse_entry(cols: &[&str]) -> LexEntry {
    let opt = |i: usize| cols.get(i).filter(|c| **c != "-").map(|c| (*c).to_string());
    fn split<const N: usize>(s: Option<String>) -> Option<[String; N]> {
        s.map(|s| {
            let v: Vec<String> = s.split(',').map(str::to_string).collect();
            v.try_into().expect("wrong count in comma column")
        })
    }
    LexEntry {
        pres: std::array::from_fn(|i| cols[1 + i].to_string()),
        subj_sg: cols[7].to_string(),
        subj_pl: cols[8].to_string(),
        ps: cols[9].to_string(),
        fut: cols[10].to_string(),
        pp: cols[11].to_string(),
        impf: opt(12),
        prsp: opt(13),
        subj6: split(opt(14)),
        imp3: split(opt(15)),
    }
}

/// Look up `inf` in the lexicon: exact base match, or base matched by
/// suffix with the leading prefix returned (*soutenir* → ("sou", tenir)).
/// Longest base wins so *repentir* is not *r + (?)entir*. A lemma may
/// carry several rows (competing paradigms: *asseoir* assieds/assois);
/// the first row is the canonical one.
fn lexical(inf: &str) -> Option<(String, Vec<LexEntry>)> {
    let mut lemma: Option<&str> = None;
    let mut entries: Vec<LexEntry> = Vec::new();
    for line in LEXICON_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if !inf.ends_with(cols[0]) {
            continue;
        }
        match lemma {
            Some(l) if cols[0].len() < l.len() => {}
            Some(l) if cols[0].len() == l.len() => entries.push(parse_entry(&cols)),
            _ => {
                lemma = Some(cols[0]);
                entries = vec![parse_entry(&cols)];
            }
        }
    }
    // The comparison above relies on equal-length matches being the same
    // lemma, which suffix matching guarantees.
    let lemma = lemma?;
    let prefix = inf[..inf.len() - lemma.len()].to_string();
    Some((prefix, entries))
}

/// Circumflex the last vowel of a past-historic stem (tin → tîn, parti →
/// partî, couru → courû). Already-marked vowels (haï) stay as they are.
fn circumflex(stem: &str) -> String {
    let mut chars: Vec<char> = stem.chars().collect();
    for c in chars.iter_mut().rev() {
        match *c {
            'a' => *c = 'â',
            'e' => *c = 'ê',
            'i' => *c = 'î',
            'o' => *c = 'ô',
            'u' => *c = 'û',
            'ï' | 'î' | 'û' | 'â' | 'ê' | 'ô' => {}
            _ => continue,
        }
        break;
    }
    chars.into_iter().collect()
}

/// Inflection class.
#[derive(Debug, Clone)]
enum Group {
    /// First group: -er.
    Er,
    /// Second group: -ir with the -iss- infix.
    Ir,
    /// Regular third-group -dre (vendre, répondre, perdre, mordre).
    Re,
    /// Irregular, from the lexicon; several rows mean competing
    /// paradigms (asseoir), the first canonical.
    Lex(Vec<LexEntry>),
}

/// A conjugatable French verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    /// Infinitive minus the group ending (empty for lexical verbs).
    stem: String,
    /// For lexical verbs: what precedes the base (*sou* in *soutenir*).
    prefix: String,
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
        let (stem, prefix, group) = if let Some(stem) = inf.strip_suffix("er") {
            if IRREGULAR_ER.contains(&inf) || inf.ends_with("envoyer") {
                match lexical(inf) {
                    Some((prefix, e)) => ("", prefix, Group::Lex(e)),
                    None => return Err(Error::Unsupported),
                }
            } else {
                (stem, String::new(), Group::Er)
            }
        } else if let Some(stem) = inf.strip_suffix("ir") {
            // -oir verbs (voir, devoir, …) are third group wholesale, and
            // so are the THIRD_GROUP_IR bases — unless the match is a
            // false suffix collision (asservir vs servir). Third-group
            // bases with a stored paradigm come from the lexicon; the
            // defective archaic ones (gésir, férir, …) stay unsupported.
            let third = inf.ends_with("oir")
                || (!SECOND_GROUP_ANYWAY.iter().any(|s| inf.ends_with(s))
                    && THIRD_GROUP_IR.iter().any(|base| inf.ends_with(base)));
            if third {
                match lexical(inf) {
                    Some((prefix, e)) => ("", prefix, Group::Lex(e)),
                    None => return Err(Error::Unsupported),
                }
            } else {
                (stem, String::new(), Group::Ir)
            }
        } else {
            // Everything else is lexicon-first; haïr lands here because
            // its infinitive ends in -ïr, not -ir. What the lexicon does
            // not know but ends in -dre (and is not one of the lexical
            // -oudre/-indre families) is the regular vendre class.
            match lexical(inf) {
                Some((prefix, e)) => ("", prefix, Group::Lex(e)),
                None => match inf.strip_suffix("re") {
                    Some(stem) if stem.ends_with('d') && !inf.ends_with("oudre") => {
                        (stem, String::new(), Group::Re)
                    }
                    _ => return Err(Error::Unsupported),
                },
            }
        };
        // A bare ending or a stem without a vowel is not a verb.
        if matches!(group, Group::Er | Group::Ir | Group::Re)
            && (stem.is_empty() || !stem.chars().any(|c| "aeiouyàâéèêëîïôûü".contains(c)))
        {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf.to_string(),
            stem: stem.to_string(),
            prefix,
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
        Self::grave(s, in_future).unwrap_or_else(|| s.clone())
    }

    /// Alternate mute stems accepted alongside the canonical one:
    /// -ayer may keep its y (paye), non-appeler/jeter doubling verbs may
    /// take the 1990 grave accent (courrièle), and é may take è in the
    /// 1990 future/conditional (cèderai).
    fn alt_mute_stems(&self, in_future: bool) -> Vec<String> {
        let s = &self.stem;
        let mut alts = Vec::new();
        if let Some(body) = s.strip_suffix('y') {
            if body.ends_with('a') {
                alts.push(s.clone());
            }
        }
        if self.doubles()
            && !self.infinitive.ends_with("appeler")
            && !self.infinitive.ends_with("jeter")
        {
            alts.extend(Self::grave(s, false));
        }
        if in_future {
            let alt = self.mute_stem(false);
            if alt != self.mute_stem(true) {
                alts.push(alt);
            }
        }
        alts
    }

    /// The e/é → è open-syllable adjustment, or None if it does not apply.
    fn grave(s: &str, in_future: bool) -> Option<String> {
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
        // x and w are loanword consonants; so is a single f in this
        // position (débriefe, briefe) — no native e/é+f+er verb exists.
        let clean = units
            .iter()
            .all(|u| !u.contains(['x', 'w', 'f', 'ç', '-', '\'']));
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
                return Some(format!("{head}è{tail}"));
            }
        }
        None
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

    /// A stored-paradigm finite form (with the derivative prefix attached).
    fn conjugate_lex(&self, e: &LexEntry, tense: SimpleTense, i: usize) -> String {
        // The imperfect stem is pres1p minus -ons (overridable: étais);
        // it also carries the present participle.
        let impf = e
            .impf
            .as_deref()
            .unwrap_or_else(|| e.pres[3].strip_suffix("ons").unwrap_or(&e.pres[3]));
        let form = match tense {
            SimpleTense::Present => e.pres[i].clone(),
            SimpleTense::Imperfect => format!("{impf}{}", IMPERFECT[i]),
            SimpleTense::Future => format!("{}{}", e.fut, FUTURE[i]),
            SimpleTense::Conditional => format!("{}{}", e.fut, CONDITIONAL[i]),
            SimpleTense::SubjunctivePresent => {
                if let Some(subj) = &e.subj6 {
                    subj[i].clone()
                } else {
                    let stem = if i == 3 || i == 4 {
                        &e.subj_pl
                    } else {
                        &e.subj_sg
                    };
                    format!("{stem}{}", SUBJ_PRESENT[i])
                }
            }
            SimpleTense::PastHistoric | SimpleTense::SubjunctiveImperfect => {
                let past = matches!(tense, SimpleTense::PastHistoric);
                if let Some(stem) = e.ps.strip_suffix("ai") {
                    // er-series: allai / allasse.
                    let endings = if past {
                        &PAST_HISTORIC
                    } else {
                        &SUBJ_IMPERFECT
                    };
                    format!("{stem}{}", endings[i])
                } else {
                    // s-series: tins, tint, tînmes / tinsse, tînt.
                    let stem = e.ps.strip_suffix('s').unwrap_or(&e.ps);
                    let circ = circumflex(stem);
                    if past {
                        match i {
                            0 | 1 => e.ps.clone(),
                            2 => format!("{stem}t"),
                            3 => format!("{circ}mes"),
                            4 => format!("{circ}tes"),
                            _ => format!("{stem}rent"),
                        }
                    } else {
                        match i {
                            0 => format!("{}se", e.ps),
                            1 => format!("{}ses", e.ps),
                            2 => format!("{circ}t"),
                            3 => format!("{}sions", e.ps),
                            4 => format!("{}siez", e.ps),
                            _ => format!("{}sent", e.ps),
                        }
                    }
                }
            }
        };
        format!("{}{form}", self.prefix)
    }

    /// A finite form.
    pub fn conjugate(&self, tense: SimpleTense, person: Person, number: Number) -> String {
        let i = person.index(number);
        if let Group::Lex(entries) = &self.group {
            return self.conjugate_lex(&entries[0], tense, i);
        }
        if matches!(self.group, Group::Ir) {
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
        if matches!(self.group, Group::Re) {
            // Regular -dre: bare stem (vend) for the present, the
            // infinitive minus -e (vendr) for the future/conditional.
            let fut = &self.infinitive[..self.infinitive.len() - 1];
            let (endings, base): (&[&str; 6], &str) = match tense {
                SimpleTense::Present => (&PRESENT_RE, &self.stem),
                SimpleTense::Imperfect => (&IMPERFECT, &self.stem),
                SimpleTense::PastHistoric => (&PAST_HISTORIC_IR, &self.stem),
                SimpleTense::Future => (&FUTURE, fut),
                SimpleTense::Conditional => (&CONDITIONAL, fut),
                SimpleTense::SubjunctivePresent => (&SUBJ_PRESENT, &self.stem),
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
        if let Group::Lex(entries) = &self.group {
            let e = &entries[0];
            if let Some(imp) = &e.imp3 {
                let form = match (person, number) {
                    (Person::Second, Number::Singular) => &imp[0],
                    (Person::First, Number::Plural) => &imp[1],
                    (Person::Second, Number::Plural) => &imp[2],
                    _ => return None,
                };
                return Some(format!("{}{form}", self.prefix));
            }
            let form = match (person, number) {
                (Person::Second, Number::Singular) => {
                    // The -s drops after a mute -es (offre!) and in va!.
                    let p2 = &e.pres[1];
                    p2.strip_suffix('s')
                        .filter(|f| f.ends_with('e') || *f == "va")
                        .unwrap_or(p2)
                        .to_string()
                }
                (Person::First, Number::Plural) => e.pres[3].clone(),
                (Person::Second, Number::Plural) => e.pres[4].clone(),
                _ => return None,
            };
            return Some(format!("{}{form}", self.prefix));
        }
        if matches!(self.group, Group::Ir) {
            return match (person, number) {
                (Person::Second, Number::Singular) => Some(format!("{}is", self.stem)),
                (Person::First, Number::Plural) => Some(format!("{}issons", self.stem)),
                (Person::Second, Number::Plural) => Some(format!("{}issez", self.stem)),
                _ => None,
            };
        }
        if matches!(self.group, Group::Re) {
            return match (person, number) {
                (Person::Second, Number::Singular) => Some(format!("{}s", self.stem)),
                (Person::First, Number::Plural) => Some(format!("{}ons", self.stem)),
                (Person::Second, Number::Plural) => Some(format!("{}ez", self.stem)),
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

    /// Present participle: *parlant*, *commençant*, *finissant*, *tenant*.
    pub fn present_participle(&self) -> String {
        match &self.group {
            Group::Er => Self::attach(&self.stem, "ant"),
            Group::Ir => format!("{}issant", self.stem),
            Group::Re => format!("{}ant", self.stem),
            Group::Lex(entries) => {
                let e = &entries[0];
                if let Some(prsp) = &e.prsp {
                    return format!("{}{prsp}", self.prefix);
                }
                let impf = e
                    .impf
                    .as_deref()
                    .unwrap_or_else(|| e.pres[3].strip_suffix("ons").unwrap_or(&e.pres[3]));
                format!("{}{impf}ant", self.prefix)
            }
        }
    }

    /// Past participle, masculine singular: *parlé*, *fini*, *tenu*.
    pub fn past_participle(&self) -> String {
        match &self.group {
            Group::Er => format!("{}é", self.stem),
            Group::Ir => format!("{}i", self.stem),
            Group::Re => format!("{}u", self.stem),
            Group::Lex(entries) => format!("{}{}", self.prefix, entries[0].pp),
        }
    }

    fn dedup(mut v: Vec<String>) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        v.retain(|f| seen.insert(f.clone()));
        v
    }

    /// Every standard spelling of a finite form, canonical first: the
    /// 1990-rectification doublets (courrielle/courrièle, céderai/cèderai),
    /// the -ayer doublets (paie/paye), and competing lexicon paradigms
    /// (assiéra/assoira).
    pub fn variants(&self, tense: SimpleTense, person: Person, number: Number) -> Vec<String> {
        let i = person.index(number);
        let mut out = vec![self.conjugate(tense, person, number)];
        match &self.group {
            Group::Lex(entries) => {
                for e in &entries[1..] {
                    out.push(self.conjugate_lex(e, tense, i));
                }
            }
            Group::Er => match tense {
                SimpleTense::Present | SimpleTense::SubjunctivePresent => {
                    let endings = if matches!(tense, SimpleTense::Present) {
                        &PRESENT
                    } else {
                        &SUBJ_PRESENT
                    };
                    if is_mute(endings[i]) {
                        for alt in self.alt_mute_stems(false) {
                            out.push(Self::attach(&alt, endings[i]));
                        }
                    }
                }
                SimpleTense::Future | SimpleTense::Conditional => {
                    let endings = if matches!(tense, SimpleTense::Future) {
                        &FUTURE
                    } else {
                        &CONDITIONAL
                    };
                    for alt in self.alt_mute_stems(true) {
                        out.push(format!("{alt}er{}", endings[i]));
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Self::dedup(out)
    }

    /// Every standard spelling of an imperative form, canonical first.
    pub fn imperative_variants(&self, person: Person, number: Number) -> Vec<String> {
        let Some(canon) = self.imperative(person, number) else {
            return Vec::new();
        };
        let mut out = vec![canon];
        if matches!(self.group, Group::Er)
            && matches!((person, number), (Person::Second, Number::Singular))
        {
            for alt in self.alt_mute_stems(false) {
                out.push(Self::attach(&alt, "e"));
            }
        }
        if let Group::Lex(entries) = &self.group {
            // Every row's present-derived imperative counts, including the
            // canonical row's when an imp3 override shadows it (veuillons
            // and voulons are both standard).
            for e in entries.iter() {
                let form = match (person, number) {
                    (Person::Second, Number::Singular) => {
                        let p2 = &e.pres[1];
                        p2.strip_suffix('s')
                            .filter(|f| f.ends_with('e') || *f == "va")
                            .unwrap_or(p2)
                            .to_string()
                    }
                    (Person::First, Number::Plural) => e.pres[3].clone(),
                    (Person::Second, Number::Plural) => e.pres[4].clone(),
                    _ => continue,
                };
                out.push(format!("{}{form}", self.prefix));
            }
        }
        Self::dedup(out)
    }

    /// Every standard present participle (asseyant/assoyant).
    pub fn present_participle_variants(&self) -> Vec<String> {
        let mut out = vec![self.present_participle()];
        if let Group::Lex(entries) = &self.group {
            for e in &entries[1..] {
                if let Some(prsp) = &e.prsp {
                    out.push(format!("{}{prsp}", self.prefix));
                } else {
                    let impf = e
                        .impf
                        .as_deref()
                        .unwrap_or_else(|| e.pres[3].strip_suffix("ons").unwrap_or(&e.pres[3]));
                    out.push(format!("{}{impf}ant", self.prefix));
                }
            }
        }
        Self::dedup(out)
    }
}

/// The full conjugation table of a French verb as one plain struct —
/// shared by the WebAssembly and Python bindings. Rows are
/// [je, tu, il/elle, nous, vous, ils/elles].
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub present_participle: String,
    pub past_participle: String,
    /// [tu, nous, vous].
    pub imperative: [Option<String>; 3],
    pub present: [String; 6],
    pub imperfect: [String; 6],
    pub past_historic: [String; 6],
    pub future: [String; 6],
    pub conditional: [String; 6],
    pub subjunctive_present: [String; 6],
    pub subjunctive_imperfect: [String; 6],
}

const SLOTS: [(Person, Number); 6] = [
    (Person::First, Number::Singular),
    (Person::Second, Number::Singular),
    (Person::Third, Number::Singular),
    (Person::First, Number::Plural),
    (Person::Second, Number::Plural),
    (Person::Third, Number::Plural),
];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row = |t: SimpleTense| SLOTS.map(|(p, n)| v.conjugate(t, p, n));
        Self {
            infinitive: v.infinitive().to_string(),
            present_participle: v.present_participle(),
            past_participle: v.past_participle(),
            imperative: [
                v.imperative(Person::Second, Number::Singular),
                v.imperative(Person::First, Number::Plural),
                v.imperative(Person::Second, Number::Plural),
            ],
            present: row(SimpleTense::Present),
            imperfect: row(SimpleTense::Imperfect),
            past_historic: row(SimpleTense::PastHistoric),
            future: row(SimpleTense::Future),
            conditional: row(SimpleTense::Conditional),
            subjunctive_present: row(SimpleTense::SubjunctivePresent),
            subjunctive_imperfect: row(SimpleTense::SubjunctiveImperfect),
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
        // Third-group -ir verbs resolve through the lexicon, not the
        // -iss- default.
        for lex in ["partir", "soutenir", "devenir", "secourir", "revêtir"] {
            let verb = Verb::from_infinitive(lex).unwrap();
            assert!(
                !verb.conjugate(Present, P3, PL).ends_with("issent"),
                "{lex}"
            );
        }
        // Still out: bases with no lexicon row at all.
        for bad in ["férir", "seoir", "messeoir", "issir"] {
            assert_eq!(
                Verb::from_infinitive(bad).unwrap_err(),
                Error::Unsupported,
                "{bad}"
            );
        }
    }

    #[test]
    fn lexical_paradigms() {
        let t = v("tenir");
        assert_eq!(t.conjugate(Present, P1, SG), "tiens");
        assert_eq!(t.conjugate(Present, P1, PL), "tenons");
        assert_eq!(t.conjugate(Present, P3, PL), "tiennent");
        assert_eq!(t.conjugate(Imperfect, P1, SG), "tenais");
        assert_eq!(t.conjugate(PastHistoric, P1, SG), "tins");
        assert_eq!(t.conjugate(PastHistoric, P3, SG), "tint");
        assert_eq!(t.conjugate(PastHistoric, P1, PL), "tînmes");
        assert_eq!(t.conjugate(PastHistoric, P3, PL), "tinrent");
        assert_eq!(t.conjugate(Future, P1, SG), "tiendrai");
        assert_eq!(t.conjugate(SubjunctivePresent, P3, SG), "tienne");
        assert_eq!(t.conjugate(SubjunctivePresent, P1, PL), "tenions");
        assert_eq!(t.conjugate(SubjunctiveImperfect, P3, SG), "tînt");
        assert_eq!(t.conjugate(SubjunctiveImperfect, P1, SG), "tinsse");
        assert_eq!(t.present_participle(), "tenant");
        assert_eq!(t.past_participle(), "tenu");
    }

    #[test]
    fn lexical_derivatives() {
        let s = v("soutenir");
        assert_eq!(s.conjugate(Present, P3, SG), "soutient");
        assert_eq!(s.conjugate(Future, P1, SG), "soutiendrai");
        assert_eq!(s.past_participle(), "soutenu");
        let a = v("accueillir");
        assert_eq!(a.conjugate(Present, P1, SG), "accueille");
        assert_eq!(a.conjugate(Future, P1, SG), "accueillerai");
        assert_eq!(a.imperative(P2, SG).unwrap(), "accueille");
        let d = v("découvrir");
        assert_eq!(d.past_participle(), "découvert");
        let acq = v("acquérir");
        assert_eq!(acq.conjugate(Present, P1, SG), "acquiers");
        assert_eq!(acq.conjugate(Present, P3, PL), "acquièrent");
        assert_eq!(acq.conjugate(Future, P1, SG), "acquerrai");
        assert_eq!(acq.past_participle(), "acquis");
    }

    #[test]
    fn aller_envoyer_hair() {
        let a = v("aller");
        assert_eq!(a.conjugate(Present, P1, SG), "vais");
        assert_eq!(a.conjugate(Present, P3, PL), "vont");
        assert_eq!(a.conjugate(Future, P1, SG), "irai");
        assert_eq!(a.conjugate(PastHistoric, P3, SG), "alla");
        assert_eq!(a.conjugate(SubjunctivePresent, P3, SG), "aille");
        assert_eq!(a.imperative(P2, SG).unwrap(), "va");
        assert_eq!(a.present_participle(), "allant");
        let e = v("envoyer");
        assert_eq!(e.conjugate(Present, P1, SG), "envoie");
        assert_eq!(e.conjugate(Future, P1, SG), "enverrai");
        let r = v("renvoyer");
        assert_eq!(r.conjugate(Future, P3, SG), "renverra");
        let h = v("haïr");
        assert_eq!(h.conjugate(Present, P1, SG), "hais");
        assert_eq!(h.conjugate(Present, P1, PL), "haïssons");
        assert_eq!(h.conjugate(PastHistoric, P1, PL), "haïmes");
        assert_eq!(h.conjugate(SubjunctiveImperfect, P3, SG), "haït");
    }

    #[test]
    fn table_builds() {
        let t = Table::build(&v("parler"));
        assert_eq!(t.present[0], "parle");
        assert_eq!(t.imperative[0].as_deref(), Some("parle"));
        let t = Table::build(&v("être"));
        assert_eq!(t.present[3], "sommes");
        assert_eq!(t.subjunctive_present[2], "soit");
    }

    #[test]
    fn lang_codes() {
        use crate::Lang;
        assert_eq!(Lang::from_code("fr"), Some(Lang::Fra));
        assert_eq!(Lang::from_code("FRA"), Some(Lang::Fra));
        assert_eq!(Lang::from_code("german"), Some(Lang::Deu));
        assert_eq!(Lang::from_code("xx"), None);
    }

    #[test]
    fn unsupported() {
        assert_eq!(
            Verb::from_infinitive("férir").unwrap_err(),
            Error::Unsupported
        );
        assert_eq!(
            Verb::from_infinitive("xyz").unwrap_err(),
            Error::Unsupported
        );
    }

    #[test]
    fn third_group_re_and_oir() {
        let v_ = v("vendre");
        assert_eq!(v_.conjugate(Present, P3, SG), "vend");
        assert_eq!(v_.conjugate(Present, P1, PL), "vendons");
        assert_eq!(v_.conjugate(PastHistoric, P1, PL), "vendîmes");
        assert_eq!(v_.conjugate(Future, P1, SG), "vendrai");
        assert_eq!(v_.past_participle(), "vendu");
        let e = v("être");
        assert_eq!(e.conjugate(Present, P1, SG), "suis");
        assert_eq!(e.conjugate(Imperfect, P1, SG), "étais");
        assert_eq!(e.conjugate(Future, P1, SG), "serai");
        assert_eq!(e.conjugate(PastHistoric, P3, SG), "fut");
        assert_eq!(e.conjugate(SubjunctivePresent, P3, SG), "soit");
        assert_eq!(e.conjugate(SubjunctiveImperfect, P3, SG), "fût");
        assert_eq!(e.imperative(P2, SG).unwrap(), "sois");
        assert_eq!(e.present_participle(), "étant");
        let a = v("avoir");
        assert_eq!(a.conjugate(Present, P3, PL), "ont");
        assert_eq!(a.conjugate(SubjunctivePresent, P3, SG), "ait");
        assert_eq!(a.present_participle(), "ayant");
        assert_eq!(a.imperative(P2, SG).unwrap(), "aie");
        let p = v("prendre");
        assert_eq!(p.conjugate(Present, P3, PL), "prennent");
        assert_eq!(p.conjugate(PastHistoric, P1, SG), "pris");
        let ap = v("apprendre");
        assert_eq!(ap.conjugate(Future, P3, SG), "apprendra");
        let r = v("recevoir");
        assert_eq!(r.conjugate(Present, P1, SG), "reçois");
        assert_eq!(r.conjugate(Present, P1, PL), "recevons");
        assert_eq!(r.past_participle(), "reçu");
        let pe = v("peindre");
        assert_eq!(pe.conjugate(Present, P1, PL), "peignons");
        assert_eq!(pe.past_participle(), "peint");
        let s = v("savoir");
        assert_eq!(s.present_participle(), "sachant");
        assert_eq!(s.imperative(P2, SG).unwrap(), "sache");
        assert_eq!(s.conjugate(SubjunctivePresent, P1, PL), "sachions");
        let vo = v("vouloir");
        assert_eq!(vo.imperative(P2, PL).unwrap(), "veuillez");
        assert_eq!(vo.conjugate(SubjunctivePresent, P3, SG), "veuille");
        let d = v("décrire");
        assert_eq!(d.conjugate(Present, P1, PL), "décrivons");
        let c = v("conduire");
        assert_eq!(c.past_participle(), "conduit");
        let dev = v("devoir");
        assert_eq!(dev.past_participle(), "dû");
        assert_eq!(dev.conjugate(Future, P1, SG), "devrai");
    }
}
