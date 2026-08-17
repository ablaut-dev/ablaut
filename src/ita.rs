//! Italian conjugation: the regular engine, the -isc- machinery, the
//! strong passato-remoto class, and the irregular lexicon.
//!
//! Three groups (-are, -ere, -ire). Spelling rules: -care/-gare insert
//! h before i/e (cerchi, pagherò); -ciare/-giare drop their i before
//! i/e (cominci, mangerò); plain -iare collapses ii (studi), while the
//! stressed-i verbs keep it (invii — mined class). The productive -ire
//! verbs take -isc- (finisco); the plain ones (dormo) are mined. The
//! strong class carries an irregular passato remoto in exactly the
//! 1sg/3sg/3pl plus an irregular participle (scrissi/scritto) —
//! `data/ita/strong.tsv`, mined from the gold oracles. Everything else
//! irregular lives in `data/ita/verbs.tsv`.

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

/// The seven simple tense/mood combinations. Compound tenses
/// (avere/essere + participle) are the compositional layer's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpleTense {
    Present,
    /// Imperfetto (parlavo).
    Imperfect,
    /// Passato remoto (parlai).
    PastHistoric,
    Future,
    Conditional,
    SubjunctivePresent,
    /// Congiuntivo imperfetto (parlassi).
    SubjunctiveImperfect,
}

/// Why an infinitive cannot be conjugated (yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A verb whose paradigm needs data that has not landed yet.
    Unsupported,
    /// The input does not look like an Italian infinitive at all.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "not a supported Italian verb"),
            Self::NotAVerb => write!(f, "not an Italian infinitive"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Group {
    Are,
    Ere,
    Ire,
}

const PRS_ARE: [&str; 6] = ["o", "i", "a", "iamo", "ate", "ano"];
const PRS_ERE: [&str; 6] = ["o", "i", "e", "iamo", "ete", "ono"];
const PRS_IRE: [&str; 6] = ["o", "i", "e", "iamo", "ite", "ono"];
const PRS_ISC: [&str; 6] = ["isco", "isci", "isce", "iamo", "ite", "iscono"];
const IPF_ARE: [&str; 6] = ["avo", "avi", "ava", "avamo", "avate", "avano"];
const IPF_ERE: [&str; 6] = ["evo", "evi", "eva", "evamo", "evate", "evano"];
const IPF_IRE: [&str; 6] = ["ivo", "ivi", "iva", "ivamo", "ivate", "ivano"];
const PR_ARE: [&str; 6] = ["ai", "asti", "ò", "ammo", "aste", "arono"];
const PR_ERE: [&str; 6] = ["ei", "esti", "é", "emmo", "este", "erono"];
const PR_ERE_ETTI: [&str; 6] = ["etti", "esti", "ette", "emmo", "este", "ettero"];
const PR_IRE: [&str; 6] = ["ii", "isti", "ì", "immo", "iste", "irono"];
const FUT: [&str; 6] = ["ò", "ai", "à", "emo", "ete", "anno"];
const COND: [&str; 6] = ["ei", "esti", "ebbe", "emmo", "este", "ebbero"];
const SPRS_ARE: [&str; 6] = ["i", "i", "i", "iamo", "iate", "ino"];
const SPRS_ERE: [&str; 6] = ["a", "a", "a", "iamo", "iate", "ano"];
const SPRS_ISC: [&str; 6] = ["isca", "isca", "isca", "iamo", "iate", "iscano"];
const SPST_ARE: [&str; 6] = ["assi", "assi", "asse", "assimo", "aste", "assero"];
const SPST_ERE: [&str; 6] = ["essi", "essi", "esse", "essimo", "este", "essero"];
const SPST_IRE: [&str; 6] = ["issi", "issi", "isse", "issimo", "iste", "issero"];

fn is_vowel(c: char) -> bool {
    "aeiouàèéìíòóùú".contains(c)
}

/// The mined class lists.
static CLASSES_TSV: &str = include_str!("../data/ita/classes.tsv");
/// The mined strong passato-remoto / participle pairs.
static STRONG_TSV: &str = include_str!("../data/ita/strong.tsv");
/// The compiled-in irregular lexicon.
static LEXICON_TSV: &str = include_str!("../data/ita/verbs.tsv");

/// Non-default perfect auxiliaries (lemma \t essere|essere/avere).
static AUX_TSV: &str = include_str!("../data/ita/aux.tsv");

fn auxiliary(inf: &str) -> &'static str {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let m = MAP.get_or_init(|| {
        AUX_TSV
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .filter_map(|l| {
                let mut f = l.split('\t');
                Some((f.next()?, f.next()?))
            })
            .collect()
    });
    m.get(inf).copied().unwrap_or("avere")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StemClass {
    /// Plain -ire without -isc- (dormo).
    Plain,
    /// -iare with stressed i keeping the double i (invii).
    Ii,
    /// -ere verbs preferring the -etti passato remoto (dovetti-type,
    /// only as the canonical; -ei rides as a variant everywhere).
    Etti,
}

fn stem_class(inf: &str) -> Option<StemClass> {
    for line in CLASSES_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (lemma, class) = line.split_once('\t')?;
        if lemma == inf {
            return match class {
                "plain" => Some(StemClass::Plain),
                "ii" => Some(StemClass::Ii),
                "etti" => Some(StemClass::Etti),
                _ => None,
            };
        }
    }
    None
}

/// Strong entry: passato remoto 1sg and past participle
/// (scrissi, scritto).
fn strong(inf: &str) -> Option<(String, String)> {
    let mut best: Option<(&str, &str, &str)> = None;
    for line in STRONG_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let mut parts = line.split('\t');
        let (Some(lemma), Some(pr1), Some(pp)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        // Longest suffix match: stridere must not stop at ridere.
        if (inf == lemma || inf.ends_with(lemma))
            && !best.is_some_and(|(b, _, _)| b.len() >= lemma.len())
        {
            best = Some((lemma, pr1, pp));
        }
    }
    {
        let (lemma, pr1, pp) = best?;
        {
            let prefix = &inf[..inf.len() - lemma.len()];
            let pr = if pr1 == "-" {
                String::new()
            } else {
                format!("{prefix}{pr1}")
            };
            let pp = if pp == "-" {
                String::new()
            } else {
                format!("{prefix}{pp}")
            };
            Some((pr, pp))
        }
    }
}

/// A stored irregular paradigm from `data/ita/verbs.tsv`. Columns:
/// lemma pres1..6 subj6 pr6 fut_stem pp impf6 ger imp2 [suffix-ok flag]
#[derive(Debug, Clone)]
struct LexEntry {
    pres: [String; 6],
    subj6: [String; 6],
    pr6: [String; 6],
    fut: Option<String>,
    pp: Option<String>,
    impf6: Option<[String; 6]>,
    ger: Option<String>,
    imp2: Option<String>,
}

fn parse_lex(cols: &[&str]) -> LexEntry {
    let opt = |i: usize| cols.get(i).filter(|c| **c != "-").map(|c| (*c).to_string());
    fn six(s: &str) -> [String; 6] {
        let v: Vec<String> = s.split(',').map(str::to_string).collect();
        v.try_into().expect("six comma-separated forms")
    }
    LexEntry {
        pres: std::array::from_fn(|i| cols[1 + i].to_string()),
        subj6: six(cols[7]),
        pr6: six(cols[8]),
        fut: opt(9),
        pp: opt(10),
        impf6: opt(11).map(|s| six(&s)),
        ger: opt(12),
        imp2: opt(13),
    }
}

/// Lexicon lookup: rows ending in "+" match by suffix (porre matches
/// comporre), the rest exactly.
fn lexical(inf: &str) -> Option<(String, LexEntry)> {
    let mut best: Option<(&str, Vec<&str>)> = None;
    for line in LEXICON_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let (lemma, by_suffix) = match cols[0].strip_suffix('+') {
            Some(l) => (l, true),
            None => (cols[0], false),
        };
        let hit = if by_suffix {
            inf.ends_with(lemma)
        } else {
            inf == lemma
        };
        if hit && !best.as_ref().is_some_and(|(b, _)| b.len() >= lemma.len()) {
            best = Some((lemma, cols));
        }
    }
    let (lemma, cols) = best?;
    let prefix = inf[..inf.len() - lemma.len()].to_string();
    Some((prefix, parse_lex(&cols)))
}

/// A conjugatable Italian verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    stem: String,
    group: Group,
    class: Option<StemClass>,
    strong: Option<(String, String)>,
    lex: Option<Box<LexEntry>>,
    prefix: String,
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

impl Verb {
    /// Build a verb from its infinitive.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim();
        if inf.is_empty() || inf.contains(char::is_whitespace) || inf.contains('\'') {
            return Err(Error::NotAVerb);
        }
        let (prefix, lex) = match lexical(inf) {
            Some((p, e)) => (p, Some(Box::new(e))),
            None => (String::new(), None),
        };
        let (stem, group) = if let Some(stem) = inf.strip_suffix("are") {
            (stem, Group::Are)
        } else if let Some(stem) = inf.strip_suffix("ere") {
            (stem, Group::Ere)
        } else if let Some(stem) = inf.strip_suffix("ire") {
            (stem, Group::Ire)
        } else if lex.is_some() {
            // porre, trarre, -durre: lexicon-only shapes.
            ("", Group::Ere)
        } else {
            return Err(Error::Unsupported);
        };
        if lex.is_none() && (stem.is_empty() || !stem.chars().any(is_vowel)) {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf.to_string(),
            stem: stem.to_string(),
            group,
            class: if lex.is_some() { None } else { stem_class(inf) },
            strong: if lex.is_some() { None } else { strong(inf) },
            lex,
            prefix,
        })
    }

    /// The infinitive as normalized.
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// True for -ire verbs taking the productive -isc- infix (the
    /// default; the plain ones are the mined class).
    fn isc(&self) -> bool {
        self.group == Group::Ire && self.class != Some(StemClass::Plain)
    }

    /// Attach an ending with the spelling rules: -c/-g take h before
    /// front vowels (cerchi, paghe­rò); -ci/-gi and plain -i drop the i
    /// before another i (cominci, studi).
    fn attach(&self, stem: &str, ending: &str) -> String {
        let mut s = stem.to_string();
        let front = ending.starts_with(['i', 'e', 'é', 'è']);
        if self.group == Group::Are
            && front
            && (s.ends_with('c') || s.ends_with('g'))
            && !(s.ends_with("ci") || s.ends_with("gi"))
        {
            // cercare → cerchi, cascare → caschi, pagare → paghi.
            s.push('h');
        }
        let mut e = ending.to_string();
        let keep_ii = self.class == Some(StemClass::Ii) && matches!(ending, "i" | "ino");
        if ending.starts_with('i') && s.ends_with('i') && !keep_ii {
            // comincio + i → cominci; studio + iamo → studiamo; the
            // stressed-i verbs keep ii only where stressed (invii,
            // inviino — but inviamo).
            e.remove(0);
        }
        format!("{s}{e}")
    }

    /// The future/conditional stem: -are/-ere → -er, -ire → -ir, with
    /// the -are spelling rules applied (cercherò, comincerò).
    fn future_stem(&self) -> String {
        match self.group {
            Group::Are => {
                let mut s = self.stem.clone();
                if (s.ends_with('c') || s.ends_with('g'))
                    && !(s.ends_with("ci") || s.ends_with("gi"))
                {
                    s.push('h');
                }
                if s.ends_with("ci") || s.ends_with("gi") {
                    s.pop();
                }
                format!("{s}er")
            }
            Group::Ere => format!("{}er", self.stem),
            Group::Ire => format!("{}ir", self.stem),
        }
    }

    /// A finite form.
    pub fn conjugate(&self, tense: SimpleTense, person: Person, number: Number) -> String {
        let i = person.index(number);
        if let Some(e) = &self.lex {
            return self.conjugate_lex(&e.clone(), tense, i);
        }
        match tense {
            SimpleTense::Present => {
                let endings: &[&str; 6] = match self.group {
                    Group::Are => &PRS_ARE,
                    Group::Ere => &PRS_ERE,
                    Group::Ire if self.isc() => &PRS_ISC,
                    Group::Ire => &PRS_IRE,
                };
                self.attach(&self.stem, endings[i])
            }
            SimpleTense::Imperfect => {
                let endings = match self.group {
                    Group::Are => &IPF_ARE,
                    Group::Ere => &IPF_ERE,
                    Group::Ire => &IPF_IRE,
                };
                self.attach(&self.stem, endings[i])
            }
            SimpleTense::PastHistoric => {
                if let Some((pr1, _)) = self.strong.as_ref().filter(|(p, _)| !p.is_empty()) {
                    // Irregular in exactly 1sg/3sg/3pl (scrissi,
                    // scrisse, scrissero); the rest regular.
                    let strong_stem = pr1.strip_suffix('i').unwrap_or(pr1);
                    match i {
                        0 => return pr1.clone(),
                        2 => return format!("{strong_stem}e"),
                        5 => return format!("{strong_stem}ero"),
                        _ => {}
                    }
                }
                let endings: &[&str; 6] = match self.group {
                    Group::Are => &PR_ARE,
                    Group::Ere if self.class == Some(StemClass::Etti) => &PR_ERE_ETTI,
                    Group::Ere => &PR_ERE,
                    Group::Ire => &PR_IRE,
                };
                self.attach(&self.stem, endings[i])
            }
            SimpleTense::Future => format!("{}{}", self.future_stem(), FUT[i]),
            SimpleTense::Conditional => format!("{}{}", self.future_stem(), COND[i]),
            SimpleTense::SubjunctivePresent => {
                let endings: &[&str; 6] = match self.group {
                    Group::Are => &SPRS_ARE,
                    Group::Ire if self.isc() => &SPRS_ISC,
                    _ => &SPRS_ERE,
                };
                self.attach(&self.stem, endings[i])
            }
            SimpleTense::SubjunctiveImperfect => {
                let endings = match self.group {
                    Group::Are => &SPST_ARE,
                    Group::Ere => &SPST_ERE,
                    Group::Ire => &SPST_IRE,
                };
                self.attach(&self.stem, endings[i])
            }
        }
    }

    fn conjugate_lex(&self, e: &LexEntry, tense: SimpleTense, i: usize) -> String {
        let form = match tense {
            SimpleTense::Present => e.pres[i].clone(),
            SimpleTense::Imperfect => match &e.impf6 {
                Some(f) => f[i].clone(),
                None => {
                    let endings = match self.group {
                        Group::Are => &IPF_ARE,
                        Group::Ere => &IPF_ERE,
                        Group::Ire => &IPF_IRE,
                    };
                    return self.attach(&self.stem, endings[i]);
                }
            },
            SimpleTense::PastHistoric => e.pr6[i].clone(),
            SimpleTense::Future | SimpleTense::Conditional => {
                let endings = if tense == SimpleTense::Future {
                    &FUT
                } else {
                    &COND
                };
                let base = match &e.fut {
                    Some(f) => format!("{}{f}", self.prefix),
                    None => self.future_stem(),
                };
                return format!("{base}{}", endings[i]);
            }
            SimpleTense::SubjunctivePresent => e.subj6[i].clone(),
            SimpleTense::SubjunctiveImperfect => {
                // From the passato remoto 2sg (facesti → facessi).
                let base = e.pr6[1].strip_suffix("sti").unwrap_or(&e.pr6[1]);
                match i {
                    0 | 1 => format!("{base}ssi"),
                    2 => format!("{base}sse"),
                    3 => format!("{base}ssimo"),
                    4 => format!("{base}ste"),
                    _ => format!("{base}ssero"),
                }
            }
        };
        format!("{}{form}", self.prefix)
    }

    /// Variants: the -ei/-etti doublet of the -ere passato remoto.
    pub fn variants(&self, tense: SimpleTense, person: Person, number: Number) -> Vec<String> {
        let mut out = vec![self.conjugate(tense, person, number)];
        if tense == SimpleTense::PastHistoric
            && self.group == Group::Ere
            && self.lex.is_none()
            && self.strong.is_none()
        {
            let i = person.index(number);
            let alt: &[&str; 6] = if self.class == Some(StemClass::Etti) {
                &PR_ERE
            } else {
                &PR_ERE_ETTI
            };
            if alt[i] != out[0].strip_prefix(&self.stem).unwrap_or("") {
                out.push(self.attach(&self.stem, alt[i]));
            }
        }
        out
    }

    /// The imperative: tu, Lei (subjunctive), noi, voi, Loro
    /// (subjunctive).
    pub fn imperative(&self, person: Person, number: Number) -> Option<String> {
        match (person, number) {
            (Person::First, Number::Singular) => None,
            (Person::Second, Number::Singular) => {
                if let Some(e) = &self.lex {
                    if let Some(f) = &e.imp2 {
                        return Some(format!("{}{f}", self.prefix));
                    }
                }
                Some(match self.group {
                    Group::Are => self.attach(&self.stem, "a"),
                    _ => self.conjugate(SimpleTense::Present, Person::Second, Number::Singular),
                })
            }
            (Person::Second, Number::Plural) => {
                // essere/avere/sapere use the subjunctive (siate,
                // abbiate, sappiate); everyone else the present.
                if let Some(e) = &self.lex {
                    if ["essere", "avere", "sapere"]
                        .iter()
                        .any(|b| self.infinitive.ends_with(b))
                    {
                        return Some(format!("{}{}", self.prefix, e.subj6[4]));
                    }
                }
                Some(self.conjugate(SimpleTense::Present, Person::Second, Number::Plural))
            }
            (Person::First, Number::Plural) => {
                Some(self.conjugate(SimpleTense::Present, Person::First, Number::Plural))
            }
            (p, n) => Some(self.conjugate(SimpleTense::SubjunctivePresent, p, n)),
        }
    }

    /// Gerund: parlando, credendo, dormendo.
    pub fn gerund(&self) -> String {
        if let Some(e) = &self.lex {
            if let Some(g) = &e.ger {
                return format!("{}{g}", self.prefix);
            }
        }
        let ending = match self.group {
            Group::Are => "ando",
            _ => "endo",
        };
        format!("{}{ending}", self.stem)
    }

    /// Present participle: parlante, credente.
    pub fn present_participle(&self) -> String {
        // A gerund override carries the participle stem (facendo →
        // facente, ponendo → ponente); otherwise the infinitive stem
        // does (piacente, sciogliente, obbediente — -ire takes -iente).
        if let Some(e) = &self.lex {
            if let Some(g) = &e.ger {
                let stem = g.strip_suffix("endo").unwrap_or(g);
                return format!("{}{stem}ente", self.prefix);
            }
        }
        // -ire defaults to -ente (svilente); a small lexical set keeps
        // the full -iente (obbediente, dormiente, nutriente).
        const IENTE: [&str; 9] = [
            "obbedire",
            "ubbidire",
            "disobbedire",
            "disubbidire",
            "dormire",
            "nutrire",
            "esordire",
            "partorire",
            "venire",
        ];
        let ending = match self.group {
            Group::Are => "ante",
            Group::Ire if IENTE.iter().any(|b| self.infinitive.ends_with(b)) => "iente",
            _ => "ente",
        };
        format!("{}{ending}", self.stem)
    }

    /// Past participle, masculine singular.
    pub fn past_participle(&self) -> String {
        if let Some(e) = &self.lex {
            if let Some(pp) = &e.pp {
                return format!("{}{}", self.prefix, pp);
            }
        }
        if let Some((_, pp)) = self.strong.as_ref().filter(|(_, p)| !p.is_empty()) {
            return pp.clone();
        }
        let ending = match self.group {
            Group::Are => "ato",
            Group::Ere => "uto",
            Group::Ire => "ito",
        };
        self.attach(&self.stem, ending)
    }

    /// A gender/number-inflected past participle.
    pub fn past_participle_inflected(&self, feminine: bool, plural: bool) -> String {
        let mut pp = self.past_participle();
        pp.pop();
        pp.push(match (feminine, plural) {
            (false, false) => 'o',
            (true, false) => 'a',
            (false, true) => 'i',
            (true, true) => 'e',
        });
        pp
    }
}

/// The full conjugation table of an Italian verb as one plain struct —
/// shared by the WebAssembly and Python bindings. Rows are
/// [io, tu, lui/lei, noi, voi, loro].
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// Perfect auxiliary: "avere" (default), "essere", or
    /// "essere/avere" for dual-auxiliary verbs, mined from kaikki.
    pub auxiliary: String,
    pub gerund: String,
    pub present_participle: String,
    pub past_participle: String,
    /// [tu, Lei, noi, voi, Loro].
    pub imperative: [Option<String>; 5],
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
            auxiliary: auxiliary(v.infinitive()).to_string(),
            gerund: v.gerund(),
            present_participle: v.present_participle(),
            past_participle: v.past_participle(),
            imperative: [
                v.imperative(Person::Second, Number::Singular),
                v.imperative(Person::Third, Number::Singular),
                v.imperative(Person::First, Number::Plural),
                v.imperative(Person::Second, Number::Plural),
                v.imperative(Person::Third, Number::Plural),
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
        Future, Imperfect, PastHistoric, Present, SubjunctiveImperfect, SubjunctivePresent,
    };

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn irregulars() {
        assert_eq!(v("essere").conjugate(Present, P1, SG), "sono");
        assert_eq!(v("essere").conjugate(Imperfect, P1, SG), "ero");
        assert_eq!(v("essere").conjugate(PastHistoric, P3, SG), "fu");
        assert_eq!(v("avere").conjugate(Present, P3, PL), "hanno");
        assert_eq!(v("fare").conjugate(PastHistoric, P3, SG), "fece");
        assert_eq!(v("rifare").conjugate(Future, P1, SG), "rifarò");
        assert_eq!(v("andare").conjugate(Future, P1, SG), "andrò");
        assert_eq!(v("dire").gerund(), "dicendo");
        assert_eq!(v("comporre").conjugate(Present, P1, SG), "compongo");
        assert_eq!(v("tradurre").conjugate(PastHistoric, P1, SG), "tradussi");
        assert_eq!(v("scrivere").conjugate(PastHistoric, P1, SG), "scrissi");
        assert_eq!(v("scrivere").conjugate(PastHistoric, P2, SG), "scrivesti");
        assert_eq!(v("descrivere").past_participle(), "descritto");
        assert_eq!(v("mettere").conjugate(PastHistoric, P3, PL), "misero");
        assert_eq!(v("venire").conjugate(Present, P1, SG), "vengo");
        assert_eq!(v("volere").conjugate(Present, P2, SG), "vuoi");
        assert_eq!(v("bere").conjugate(Future, P1, SG), "berrò");
        assert_eq!(v("dormire").conjugate(Present, P1, SG), "dormo");
        assert_eq!(v("tuffare").conjugate(Present, P1, PL), "tuffiamo");
        assert_eq!(v("stridere").past_participle(), "striduto");
        assert_eq!(v("piacere").present_participle(), "piacente");
        assert_eq!(v("sapere").imperative(P2, PL).unwrap(), "sappiate");
        assert_eq!(v("inviare").conjugate(Present, P2, SG), "invii");
        assert_eq!(v("inviare").conjugate(Present, P1, PL), "inviamo");
    }

    #[test]
    fn regular_groups() {
        let p = v("parlare");
        assert_eq!(p.conjugate(Present, P1, SG), "parlo");
        assert_eq!(p.conjugate(Present, P1, PL), "parliamo");
        assert_eq!(p.conjugate(PastHistoric, P3, SG), "parlò");
        assert_eq!(p.conjugate(Future, P1, SG), "parlerò");
        assert_eq!(p.conjugate(SubjunctivePresent, P3, SG), "parli");
        assert_eq!(p.conjugate(SubjunctiveImperfect, P1, SG), "parlassi");
        assert_eq!(p.imperative(P2, SG).unwrap(), "parla");
        assert_eq!(p.gerund(), "parlando");
        assert_eq!(p.past_participle(), "parlato");
        let c = v("credere");
        assert_eq!(c.conjugate(PastHistoric, P1, SG), "credei");
        assert_eq!(c.conjugate(PastHistoric, P3, SG), "credé");
        assert_eq!(c.past_participle(), "creduto");
        let f = v("finire");
        assert_eq!(f.conjugate(Present, P1, SG), "finisco");
        assert_eq!(f.conjugate(Present, P1, PL), "finiamo");
        assert_eq!(f.conjugate(SubjunctivePresent, P3, PL), "finiscano");
        assert_eq!(f.conjugate(PastHistoric, P3, SG), "finì");
    }

    #[test]
    fn spelling() {
        assert_eq!(v("cercare").conjugate(Present, P2, SG), "cerchi");
        assert_eq!(v("cercare").conjugate(Future, P1, SG), "cercherò");
        assert_eq!(v("pagare").conjugate(SubjunctivePresent, P3, SG), "paghi");
        assert_eq!(v("cominciare").conjugate(Present, P2, SG), "cominci");
        assert_eq!(v("cominciare").conjugate(Future, P1, SG), "comincerò");
        assert_eq!(v("mangiare").conjugate(Future, P3, SG), "mangerà");
        assert_eq!(v("studiare").conjugate(Present, P2, SG), "studi");
        assert_eq!(v("studiare").conjugate(Present, P1, PL), "studiamo");
    }
}
