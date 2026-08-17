//! Portuguese conjugation: the regular engine, spelling rules, and the
//! layers Portuguese alone has — the personal infinitive and the
//! synthetic pluperfect.
//!
//! Three groups (-ar, -er, -ir; the pôr family lives in the lexicon as a
//! bound base). Sound-preserving spelling before endings: *fiquei /
//! cheguei / comecei* (-ar before e), *conheço / dirijo / sigo / ergo*
//! (-er/-ir before o/a). The productive -ear diphthong (*passeio*) is a
//! rule; the -iar lookalikes (*odeio*) and the -ir metaphony classes
//! (*sirvo*, *durmo*, *sobes*) are mined into `data/por/classes.tsv`.
//! Vowel-final stems accent their i endings (*saímos*, *moído*, *caí*).

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

/// The nine simple tense/mood combinations, including the synthetic
/// pluperfect (falara). Compound tenses (ter + participle) are the
/// compositional layer's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpleTense {
    Present,
    /// Pretérito imperfeito (falava).
    Imperfect,
    /// Pretérito perfeito (falei).
    Preterite,
    /// Pretérito mais-que-perfeito simples (falara).
    Pluperfect,
    Future,
    Conditional,
    SubjunctivePresent,
    /// Pretérito imperfeito do conjuntivo (falasse).
    SubjunctiveImperfect,
    /// Futuro do conjuntivo (falar, falares).
    SubjunctiveFuture,
}

/// Why an infinitive cannot be conjugated (yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A verb whose paradigm needs data that has not landed yet.
    Unsupported,
    /// The input does not look like a Portuguese infinitive at all.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "not a supported Portuguese verb"),
            Self::NotAVerb => write!(f, "not a Portuguese infinitive"),
        }
    }
}

/// Inflection class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Group {
    Ar,
    Er,
    Ir,
}

/// Endings tables, [1sg..3pl].
const PRS_AR: [&str; 6] = ["o", "as", "a", "amos", "ais", "am"];
const PRS_ER: [&str; 6] = ["o", "es", "e", "emos", "eis", "em"];
const PRS_IR: [&str; 6] = ["o", "es", "e", "imos", "is", "em"];
const IPF_AR: [&str; 6] = ["ava", "avas", "ava", "ávamos", "áveis", "avam"];
const IPF_ERIR: [&str; 6] = ["ia", "ias", "ia", "íamos", "íeis", "iam"];
/// The 1pl carries the EU/BR doublet: falámos (variant falamos).
const PRF_AR: [&str; 6] = ["ei", "aste", "ou", "ámos", "astes", "aram"];
const PRF_ER: [&str; 6] = ["i", "este", "eu", "emos", "estes", "eram"];
const PRF_IR: [&str; 6] = ["i", "iste", "iu", "imos", "istes", "iram"];
const PQP_AR: [&str; 6] = ["ara", "aras", "ara", "áramos", "áreis", "aram"];
const PQP_ER: [&str; 6] = ["era", "eras", "era", "êramos", "êreis", "eram"];
const PQP_IR: [&str; 6] = ["ira", "iras", "ira", "íramos", "íreis", "iram"];
const FUT: [&str; 6] = ["ei", "ás", "á", "emos", "eis", "ão"];
const COND: [&str; 6] = ["ia", "ias", "ia", "íamos", "íeis", "iam"];
const SPRS_AR: [&str; 6] = ["e", "es", "e", "emos", "eis", "em"];
const SPRS_ERIR: [&str; 6] = ["a", "as", "a", "amos", "ais", "am"];
const SPST_AR: [&str; 6] = ["asse", "asses", "asse", "ássemos", "ásseis", "assem"];
const SPST_ER: [&str; 6] = ["esse", "esses", "esse", "êssemos", "êsseis", "essem"];
const SPST_IR: [&str; 6] = ["isse", "isses", "isse", "íssemos", "ísseis", "issem"];
const SFUT_AR: [&str; 6] = ["ar", "ares", "ar", "armos", "ardes", "arem"];
const SFUT_ER: [&str; 6] = ["er", "eres", "er", "ermos", "erdes", "erem"];
const SFUT_IR: [&str; 6] = ["ir", "ires", "ir", "irmos", "irdes", "irem"];
/// Personal infinitive endings on the whole infinitive.
const PINF: [&str; 6] = ["", "es", "", "mos", "des", "em"];

fn is_vowel(c: char) -> bool {
    "aeiouáéíóúâêôãõü".contains(c)
}

/// The compiled-in irregular lexicon (see the schema comment there).
static LEXICON_TSV: &str = include_str!("../data/por/verbs.tsv");

/// Bases safe to match by suffix (their derivative families are real:
/// desfazer, conduzir, impedir). Everything else matches exact — short
/// bases collide with regular verbs (bater ends in ater, meter in ter).
const SUFFIX_OK: [&str; 18] = [
    "fazer", "dizer", "trazer", "duzir", "luzir", "prazer", "roer", "moer", "doer", "valer",
    "pedir", "medir", "querer", "arguir", "ouvir", "polir", "jazer", "por",
];

/// A stored irregular paradigm from `data/por/verbs.tsv`.
#[derive(Debug, Clone)]
struct LexEntry {
    pres: [String; 6],
    subj6: [String; 6],
    pret6: [String; 6],
    fut: Option<String>,
    pp: Option<String>,
    impf6: Option<[String; 6]>,
    ger: Option<String>,
    imp2: Option<String>,
    /// Overrides the infinitive as the base of the future, conditional,
    /// personal infinitive and future subjunctive (pôr → por).
    inf_base: Option<String>,
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
        pret6: six(cols[8]),
        fut: opt(9),
        pp: opt(10),
        impf6: opt(11).map(|s| six(&s)),
        ger: opt(12),
        imp2: opt(13),
        inf_base: opt(14),
    }
}

/// Longest-base suffix lookup with the derivative prefix returned
/// (desfazer → ("des", fazer)); EXACT_ONLY bases match whole.
fn lexical(inf: &str) -> Option<(String, LexEntry)> {
    let mut best: Option<(&str, Vec<&str>)> = None;
    for line in LEXICON_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let lemma = cols[0];
        let hit = if SUFFIX_OK.contains(&lemma) {
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

/// The mined stem-changing classes (see the file header).
static CLASSES_TSV: &str = include_str!("../data/por/classes.tsv");

/// A stem-changing class from `data/por/classes.tsv`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StemClass {
    /// -iar verbs that diphthongize like -ear (odeio).
    Ei,
    /// -ir metaphony: e→i in 1sg present and the whole present
    /// subjunctive (sirvo, sirva).
    I,
    /// -ir metaphony: o→u in the same slots (durmo, durma).
    U,
    /// -ir u→o in the 2sg/3sg/3pl present (sobes, sobe, sobem).
    Uo,
    /// -ir e→i everywhere the -iss- slots would be (rare; frigir).
    IWide,
    /// Hiatus i stressed í (coibir → coíbo, ajuizar → ajuízo).
    AccI,
    /// Hiatus u stressed ú (reunir → reúno, saudar → saúdo).
    AccU,
}

fn stem_class(inf: &str) -> Option<StemClass> {
    for line in CLASSES_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (lemma, class) = line.split_once('\t')?;
        if lemma == inf {
            return match class {
                "ei" => Some(StemClass::Ei),
                "i" => Some(StemClass::I),
                "u" => Some(StemClass::U),
                "uo" => Some(StemClass::Uo),
                "iw" => Some(StemClass::IWide),
                "acc-i" => Some(StemClass::AccI),
                "acc-u" => Some(StemClass::AccU),
                _ => None,
            };
        }
    }
    None
}

/// Replace the last occurrence of any of `targets` with `to`.
fn replace_last(stem: &str, targets: &[char], to: &str) -> String {
    if let Some(i) = stem.rfind(|c| targets.contains(&c)) {
        let c = stem[i..].chars().next().unwrap();
        return format!("{}{to}{}", &stem[..i], &stem[i + c.len_utf8()..]);
    }
    stem.to_string()
}

/// A conjugatable Portuguese verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    stem: String,
    group: Group,
    class: Option<StemClass>,
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
        // Case-fold: lemmas are lowercase in every oracle of this
        // language (the Abbrechen lesson, generalized).
        let lowered = infinitive.to_lowercase();
        let infinitive = lowered.as_str();
        let inf = infinitive.trim();
        if inf.is_empty() || inf.contains(char::is_whitespace) || inf.contains('\'') {
            return Err(Error::NotAVerb);
        }
        let (stem, group) = if let Some(stem) = inf.strip_suffix("ar") {
            (stem, Group::Ar)
        } else if let Some(stem) = inf.strip_suffix("er") {
            (stem, Group::Er)
        } else if let Some(stem) = inf.strip_suffix("ir") {
            (stem, Group::Ir)
        } else if inf.ends_with("or") || inf.ends_with("\u{f4}r") {
            // The pôr family (compor, supor) lives in the lexicon and
            // conjugates on -er endings where anything derives.
            ("", Group::Er)
        } else {
            return Err(Error::Unsupported);
        };
        let (prefix, lex) = match lexical(inf) {
            Some((p, e)) => (p, Some(Box::new(e))),
            None => (String::new(), None),
        };
        if lex.is_none() && (stem.is_empty() || !stem.chars().any(is_vowel)) {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf.to_string(),
            stem: stem.to_string(),
            group,
            class: if lex.is_some() { None } else { stem_class(inf) },
            lex,
            prefix,
        })
    }

    /// The infinitive as normalized.
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// True in the slots where the stem syllable is stressed in the
    /// present: singular and 3pl.
    fn stressed(i: usize) -> bool {
        matches!(i, 0 | 1 | 2 | 5)
    }

    /// The stem for a present-system slot, with -ear/-iar diphthongs
    /// and -ir metaphony applied.
    fn present_stem(&self, i: usize, subjunctive: bool) -> String {
        let s = &self.stem;
        // Productive -ear diphthong: passear → passei- when stressed.
        if self.group == Group::Ar && s.ends_with('e') && Self::stressed(i) {
            return format!("{s}i");
        }
        match self.class {
            Some(StemClass::Ei) if Self::stressed(i) => {
                // odiar → odei- (the stem ends in i; the e slips in
                // before it).
                if let Some(body) = s.strip_suffix('i') {
                    return format!("{body}ei");
                }
                s.clone()
            }
            Some(StemClass::I) if i == 0 || subjunctive => replace_last(s, &['e'], "i"),
            Some(StemClass::U) if i == 0 || subjunctive => replace_last(s, &['o'], "u"),
            Some(StemClass::Uo) if matches!(i, 1 | 2 | 5) && !subjunctive => {
                replace_last(s, &['u'], "o")
            }
            Some(StemClass::IWide) if Self::stressed(i) || subjunctive => {
                replace_last(s, &['e'], "i")
            }
            Some(StemClass::AccI) if Self::stressed(i) => replace_last(s, &['i'], "\u{ed}"),
            Some(StemClass::AccU) if Self::stressed(i) => replace_last(s, &['u'], "\u{fa}"),
            _ => s.clone(),
        }
    }

    /// Attach an ending, applying the sound-preserving spelling rules
    /// and the vowel-stem accent/glide alternations.
    fn attach(&self, stem: &str, ending: &str) -> String {
        let first = ending.chars().next().unwrap_or('x');
        let soft = matches!(first, 'e' | 'é' | 'ê'); // -ar before e
        let hard = matches!(first, 'o' | 'a' | 'á' | 'â'); // -er/-ir before o/a
        let mut s = stem.to_string();
        let mut e = ending.to_string();

        match self.group {
            Group::Ar if soft => {
                if let Some(body) = s.strip_suffix('c') {
                    s = format!("{body}qu"); // ficar → fiquei
                } else if let Some(body) = s.strip_suffix('g') {
                    s = format!("{body}gu"); // chegar → cheguei
                } else if let Some(body) = s.strip_suffix('ç') {
                    s = format!("{body}c"); // começar → comecei
                }
            }
            Group::Er | Group::Ir if hard => {
                if let Some(body) = s.strip_suffix("gu") {
                    s = format!("{body}g"); // seguir → sigo, erguer → ergo
                } else if let Some(body) = s.strip_suffix("qu") {
                    s = format!("{body}c"); // delinquir → delinco
                } else if let Some(body) = s.strip_suffix('c') {
                    s = format!("{body}ç"); // conhecer → conheço
                } else if let Some(body) = s.strip_suffix('g') {
                    s = format!("{body}j"); // dirigir → dirijo
                }
            }
            _ => {}
        }

        if matches!(self.group, Group::Er | Group::Ir) {
            // gu/qu-final stems are digraphs (seguimos, not seguímos);
            // a real u vowel keeps the accent rules (possuímos).
            let vowel_stem =
                s.ends_with(['a', 'e', 'o', 'u']) && !(s.ends_with("gu") || s.ends_with("qu"));
            if vowel_stem {
                // A stressed i after a vowel is written í: saímos, caí,
                // moído, saísse; the 3sg preterite keeps the diphthong
                // (saiu, caiu).
                for (from, to) in [
                    ("iste", "íste"),
                    ("istes", "ístes"),
                    ("imos", "ímos"),
                    ("ido", "ído"),
                    ("i", "í"),
                    ("is", "ís"),
                    ("ia", "ía"),
                    ("ias", "ías"),
                    ("iam", "íam"),
                    ("isse", "ísse"),
                    ("isses", "ísses"),
                    ("issem", "íssem"),
                    ("ira", "íra"),
                    ("iras", "íras"),
                    ("iram", "íram"),
                    ("ires", "íres"),
                    ("irem", "írem"),
                ] {
                    if e == from {
                        e = to.to_string();
                        break;
                    }
                }
            }
        }
        format!("{s}{e}")
    }

    /// The base for infinitive-built tenses (future, conditional,
    /// personal infinitive, future subjunctive).
    fn inf_base(&self) -> String {
        match &self.lex {
            Some(e) => match &e.inf_base {
                Some(b) => format!("{}{b}", self.prefix),
                None => self.infinitive.clone(),
            },
            None => self.infinitive.clone(),
        }
    }

    /// A stored-paradigm finite form (prefix attached).
    fn conjugate_lex(&self, e: &LexEntry, tense: SimpleTense, i: usize) -> String {
        // The preterite 3pl minus -am is the shared stem of the
        // pluperfect and both subjunctive pasts (tiveram → tiver-).
        let pret_stem = e.pret6[5]
            .strip_suffix("am")
            .unwrap_or(&e.pret6[5])
            .to_string();
        // Regular preterites (1sg in -i: li, vali) take the circumflex
        // in the derived pasts (lêramos); strong ones the acute
        // (tivéramos).
        // Only -er regular preterites circumflex (lêramos); -ar ones
        // (dar: déramos) and strong preterites take the acute.
        let e_acc = if e.pret6[0].ends_with(['i', '\u{ed}']) && self.group == Group::Er {
            '\u{ea}'
        } else {
            '\u{e9}'
        };
        let acc = |st: &str| -> String {
            // Accent the last e/a of the stem for the 1pl/2pl
            // (tivéramos, puséssemos, fôramos keeps ô).
            match st.rfind(['e', 'a', 'o', 'i', 'u']) {
                Some(ix) => {
                    let c = st[ix..].chars().next().unwrap();
                    let a = match c {
                        'e' => e_acc,
                        'a' => '\u{e1}',
                        'i' => '\u{ed}',
                        'u' => '\u{fa}',
                        _ => '\u{f4}',
                    };
                    format!("{}{a}{}", &st[..ix], &st[ix + c.len_utf8()..])
                }
                None => st.to_string(),
            }
        };
        let form = match tense {
            SimpleTense::Present => e.pres[i].clone(),
            SimpleTense::Imperfect => match &e.impf6 {
                Some(f) => f[i].clone(),
                None => {
                    let endings = if self.group == Group::Ar {
                        &IPF_AR
                    } else {
                        &IPF_ERIR
                    };
                    return self.attach(&self.stem, endings[i]);
                }
            },
            SimpleTense::Preterite => e.pret6[i].clone(),
            SimpleTense::Pluperfect => {
                let st = &pret_stem;
                match i {
                    3 => format!("{}amos", acc(st)),
                    4 => format!("{}eis", acc(st)),
                    _ => format!("{st}{}", ["a", "as", "a", "", "", "am"][i]),
                }
            }
            SimpleTense::Future | SimpleTense::Conditional => {
                let endings = if tense == SimpleTense::Future {
                    &FUT
                } else {
                    &COND
                };
                let base = match &e.fut {
                    Some(f) => format!("{}{f}", self.prefix),
                    None => self.inf_base(),
                };
                return format!("{base}{}", endings[i]);
            }
            SimpleTense::SubjunctivePresent => e.subj6[i].clone(),
            SimpleTense::SubjunctiveImperfect => {
                let base = pret_stem
                    .strip_suffix('r')
                    .unwrap_or(&pret_stem)
                    .to_string();
                match i {
                    3 => format!("{}ssemos", acc(&base)),
                    4 => format!("{}sseis", acc(&base)),
                    _ => format!("{base}{}", ["sse", "sses", "sse", "", "", "ssem"][i]),
                }
            }
            SimpleTense::SubjunctiveFuture => {
                let st = &pret_stem;
                format!("{st}{}", ["", "es", "", "mos", "des", "em"][i])
            }
        };
        format!("{}{form}", self.prefix)
    }

    /// A finite form (the EU-orthography canonical; BR doublets are in
    /// variants()).
    pub fn conjugate(&self, tense: SimpleTense, person: Person, number: Number) -> String {
        let i = person.index(number);
        if let Some(e) = &self.lex {
            return self.conjugate_lex(&e.clone(), tense, i);
        }
        let ar = self.group == Group::Ar;
        match tense {
            SimpleTense::Present => {
                let endings: &[&str; 6] = match self.group {
                    Group::Ar => &PRS_AR,
                    Group::Er => &PRS_ER,
                    Group::Ir => &PRS_IR,
                };
                let stem = self.present_stem(i, false);
                // -air and vowel -uir verbs: sais/sai, possuis/possui;
                // -air additionally inserts i in the 1sg (saio, caio).
                if self.group == Group::Ir
                    && stem.ends_with(['a', 'u'])
                    && !(stem.ends_with("gu") || stem.ends_with("qu"))
                {
                    let e = match i {
                        0 if stem.ends_with('a') => "io",
                        0 => "o",
                        1 => "is",
                        2 => "i",
                        3 => "ímos",
                        4 => "ís",
                        _ => "em",
                    };
                    return format!("{stem}{e}");
                }
                self.attach(&stem, endings[i])
            }
            SimpleTense::Imperfect => {
                let endings = if ar { &IPF_AR } else { &IPF_ERIR };
                self.attach(&self.stem, endings[i])
            }
            SimpleTense::Preterite => {
                let endings = match self.group {
                    Group::Ar => &PRF_AR,
                    Group::Er => &PRF_ER,
                    Group::Ir => &PRF_IR,
                };
                self.attach(&self.stem, endings[i])
            }
            SimpleTense::Pluperfect => {
                let endings = match self.group {
                    Group::Ar => &PQP_AR,
                    Group::Er => &PQP_ER,
                    Group::Ir => &PQP_IR,
                };
                self.attach(&self.stem, endings[i])
            }
            SimpleTense::Future => format!("{}{}", self.infinitive, FUT[i]),
            SimpleTense::Conditional => format!("{}{}", self.infinitive, COND[i]),
            SimpleTense::SubjunctivePresent => {
                let endings = if ar { &SPRS_AR } else { &SPRS_ERIR };
                let stem = self.present_stem(i, true);
                // -air verbs build the subjunctive on the 1sg stem
                // (saia, caia, traia).
                if self.group == Group::Ir && stem.ends_with('a') {
                    return format!("{stem}i{}", SPRS_ERIR[i]);
                }
                self.attach(&stem, endings[i])
            }
            SimpleTense::SubjunctiveImperfect => {
                let endings = match self.group {
                    Group::Ar => &SPST_AR,
                    Group::Er => &SPST_ER,
                    Group::Ir => &SPST_IR,
                };
                self.attach(&self.stem, endings[i])
            }
            SimpleTense::SubjunctiveFuture => {
                let endings = match self.group {
                    Group::Ar => &SFUT_AR,
                    Group::Er => &SFUT_ER,
                    Group::Ir => &SFUT_IR,
                };
                self.attach(&self.stem, endings[i])
            }
        }
    }

    /// Every standard spelling, canonical first: the BR preterite 1pl
    /// (falamos beside falámos).
    pub fn variants(&self, tense: SimpleTense, person: Person, number: Number) -> Vec<String> {
        let mut out = vec![self.conjugate(tense, person, number)];
        if tense == SimpleTense::Preterite
            && self.group == Group::Ar
            && (person, number) == (Person::First, Number::Plural)
        {
            out.push(self.attach(&self.stem, "amos"));
        }
        out
    }

    /// The personal infinitive: falares, falarmos. The 1sg/3sg equal
    /// the impersonal infinitive.
    pub fn personal_infinitive(&self, person: Person, number: Number) -> String {
        let base = self.inf_base();
        let i = person.index(number);
        // The bare slots are the written infinitive itself (pôr, not
        // por).
        if PINF[i].is_empty() {
            return self.infinitive.clone();
        }
        // Hiatus verbs accent the stressed i of the 2sg/3pl endings:
        // concluíres, caírem (but concluirmos).
        if matches!(i, 1 | 5)
            && (base.ends_with("air") || base.ends_with("uir"))
            && !(base.ends_with("guir") || base.ends_with("quir"))
        {
            let head = &base[..base.len() - 2];
            let e = if i == 1 { "\u{ed}res" } else { "\u{ed}rem" };
            return format!("{head}{e}");
        }
        format!("{base}{}", PINF[i])
    }

    /// The imperative: tu (3sg present), você (subjunctive), nós
    /// (subjunctive), vós (stem + ai/ei/i), vocês (subjunctive).
    pub fn imperative(&self, person: Person, number: Number) -> Option<String> {
        match (person, number) {
            (Person::First, Number::Singular) => None,
            (Person::Second, Number::Singular) => Some(match &self.lex {
                Some(e) => match &e.imp2 {
                    Some(f) => format!("{}{f}", self.prefix),
                    None => self.conjugate(SimpleTense::Present, Person::Third, Number::Singular),
                },
                None => self.conjugate(SimpleTense::Present, Person::Third, Number::Singular),
            }),
            (Person::Second, Number::Plural) => {
                if let Some(e) = &self.lex {
                    // The vós imperative is the 2pl present minus its s:
                    // lede, dizei, tende, vinde, ponde. ser is sede.
                    if self.infinitive == "ser" {
                        return Some("sede".to_string());
                    }
                    let p5 = &e.pres[4];
                    let f = p5.strip_suffix('s').unwrap_or(p5);
                    return Some(format!("{}{f}", self.prefix));
                }
                let ending = match self.group {
                    Group::Ar => "ai",
                    Group::Er => "ei",
                    Group::Ir => "i",
                };
                Some(self.attach(&self.stem, ending))
            }
            (p, n) => Some(self.conjugate(SimpleTense::SubjunctivePresent, p, n)),
        }
    }

    /// Gerund: falando, comendo, partindo, saindo.
    pub fn gerund(&self) -> String {
        if let Some(e) = &self.lex {
            if let Some(g) = &e.ger {
                return format!("{}{g}", self.prefix);
            }
        }
        let ending = match self.group {
            Group::Ar => "ando",
            Group::Er => "endo",
            Group::Ir => "indo",
        };
        format!("{}{ending}", self.stem)
    }

    /// Past participle, masculine singular: falado, comido, partido,
    /// saído.
    pub fn past_participle(&self) -> String {
        if let Some(e) = &self.lex {
            if let Some(pp) = &e.pp {
                return format!("{}{}", self.prefix, pp);
            }
        }
        // Productive irregular participles: aberto, coberto, escrito.
        for (suffix, repl) in [
            ("abrir", "aberto"),
            ("cobrir", "coberto"),
            ("crever", "crito"),
        ] {
            if let Some(head) = self.infinitive.strip_suffix(suffix) {
                return format!("{head}{repl}");
            }
        }
        match self.group {
            Group::Ar => self.attach(&self.stem, "ado"),
            Group::Er | Group::Ir => self.attach(&self.stem, "ido"),
        }
    }

    /// A gender/number-inflected past participle.
    pub fn past_participle_inflected(&self, feminine: bool, plural: bool) -> String {
        let mut pp = self.past_participle();
        if feminine {
            pp.pop();
            pp.push('a');
        }
        if plural {
            pp.push('s');
        }
        pp
    }
}

/// The full conjugation table of a Portuguese verb as one plain struct —
/// shared by the WebAssembly and Python bindings. Rows are
/// [eu, tu, ele/ela, nós, vós, eles/elas].
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub gerund: String,
    pub past_participle: String,
    /// [tu, você, nós, vós, vocês].
    pub imperative: [Option<String>; 5],
    pub personal_infinitive: [String; 6],
    pub present: [String; 6],
    pub imperfect: [String; 6],
    pub preterite: [String; 6],
    pub pluperfect: [String; 6],
    pub future: [String; 6],
    pub conditional: [String; 6],
    pub subjunctive_present: [String; 6],
    pub subjunctive_imperfect: [String; 6],
    pub subjunctive_future: [String; 6],
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
            gerund: v.gerund(),
            past_participle: v.past_participle(),
            imperative: [
                v.imperative(Person::Second, Number::Singular),
                v.imperative(Person::Third, Number::Singular),
                v.imperative(Person::First, Number::Plural),
                v.imperative(Person::Second, Number::Plural),
                v.imperative(Person::Third, Number::Plural),
            ],
            personal_infinitive: SLOTS.map(|(p, n)| v.personal_infinitive(p, n)),
            present: row(SimpleTense::Present),
            imperfect: row(SimpleTense::Imperfect),
            preterite: row(SimpleTense::Preterite),
            pluperfect: row(SimpleTense::Pluperfect),
            future: row(SimpleTense::Future),
            conditional: row(SimpleTense::Conditional),
            subjunctive_present: row(SimpleTense::SubjunctivePresent),
            subjunctive_imperfect: row(SimpleTense::SubjunctiveImperfect),
            subjunctive_future: row(SimpleTense::SubjunctiveFuture),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};
    use SimpleTense::{
        Future, Imperfect, Pluperfect, Present, Preterite, SubjunctiveFuture, SubjunctiveImperfect,
        SubjunctivePresent,
    };

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn irregulars_and_classes() {
        assert_eq!(v("ser").conjugate(Present, P1, SG), "sou");
        assert_eq!(v("ser").conjugate(Imperfect, P1, SG), "era");
        assert_eq!(v("ser").conjugate(Preterite, P3, SG), "foi");
        assert_eq!(v("ser").conjugate(Pluperfect, P1, PL), "fôramos");
        assert_eq!(v("ter").conjugate(Present, P3, PL), "têm");
        assert_eq!(v("conter").conjugate(Present, P3, SG), "contém");
        assert_eq!(v("ter").conjugate(SubjunctiveImperfect, P1, SG), "tivesse");
        assert_eq!(v("ter").conjugate(SubjunctiveFuture, P1, SG), "tiver");
        assert_eq!(v("pôr").conjugate(Present, P3, SG), "põe");
        assert_eq!(v("compor").conjugate(Preterite, P3, SG), "compôs");
        assert_eq!(v("pôr").personal_infinitive(P1, PL), "pormos");
        assert_eq!(v("pôr").personal_infinitive(P1, SG), "pôr");
        assert_eq!(v("fazer").conjugate(Future, P1, SG), "farei");
        assert_eq!(v("desfazer").conjugate(Preterite, P3, SG), "desfez");
        assert_eq!(v("dizer").conjugate(Present, P3, SG), "diz");
        assert_eq!(v("conduzir").conjugate(Present, P3, SG), "conduz");
        assert_eq!(v("ir").gerund(), "indo");
        assert_eq!(v("vir").past_participle(), "vindo");
        assert_eq!(v("ler").conjugate(Pluperfect, P1, PL), "lêramos");
        assert_eq!(v("dar").conjugate(SubjunctiveImperfect, P1, PL), "déssemos");
        assert_eq!(v("servir").conjugate(Present, P1, SG), "sirvo");
        assert_eq!(v("dormir").conjugate(Present, P1, SG), "durmo");
        assert_eq!(v("subir").conjugate(Present, P3, SG), "sobe");
        assert_eq!(v("agredir").conjugate(Present, P3, SG), "agride");
        assert_eq!(v("odiar").conjugate(Present, P1, SG), "odeio");
        assert_eq!(v("reunir").conjugate(Present, P1, SG), "reúno");
        assert_eq!(v("impedir").conjugate(Present, P1, SG), "impeço");
        assert_eq!(v("abrir").past_participle(), "aberto");
        assert_eq!(v("escrever").past_participle(), "escrito");
        assert_eq!(v("possuir").conjugate(Present, P3, SG), "possui");
        assert_eq!(v("possuir").conjugate(Present, P1, PL), "possuímos");
        assert_eq!(v("atrair").conjugate(Present, P1, SG), "atraio");
        assert_eq!(v("cair").conjugate(SubjunctivePresent, P1, SG), "caia");
        assert_eq!(v("concluir").personal_infinitive(P2, SG), "concluíres");
        assert_eq!(v("bater").conjugate(Present, P1, SG), "bato");
        assert_eq!(v("meter").conjugate(Preterite, P1, SG), "meti");
    }

    #[test]
    fn regular_groups() {
        let f = v("falar");
        assert_eq!(f.conjugate(Present, P1, SG), "falo");
        assert_eq!(f.conjugate(Imperfect, P1, PL), "falávamos");
        assert_eq!(f.conjugate(Preterite, P1, SG), "falei");
        assert_eq!(f.conjugate(Preterite, P1, PL), "falámos");
        assert_eq!(f.variants(Preterite, P1, PL), vec!["falámos", "falamos"]);
        assert_eq!(f.conjugate(Pluperfect, P1, SG), "falara");
        assert_eq!(f.conjugate(Future, P3, PL), "falarão");
        assert_eq!(f.conjugate(SubjunctiveImperfect, P1, PL), "falássemos");
        assert_eq!(f.conjugate(SubjunctiveFuture, P2, SG), "falares");
        assert_eq!(f.personal_infinitive(P1, PL), "falarmos");
        assert_eq!(f.personal_infinitive(P2, SG), "falares");
        assert_eq!(f.imperative(P2, SG).unwrap(), "fala");
        assert_eq!(f.imperative(P2, PL).unwrap(), "falai");
        assert_eq!(f.gerund(), "falando");
        let c = v("comer");
        assert_eq!(c.conjugate(Preterite, P3, SG), "comeu");
        assert_eq!(c.conjugate(Pluperfect, P1, PL), "comêramos");
        assert_eq!(c.imperative(P2, PL).unwrap(), "comei");
        let p = v("partir");
        assert_eq!(p.conjugate(Preterite, P3, SG), "partiu");
        assert_eq!(p.conjugate(Present, P2, PL), "partis");
        assert_eq!(p.imperative(P2, PL).unwrap(), "parti");
    }

    #[test]
    fn spelling_and_vowel_stems() {
        assert_eq!(v("ficar").conjugate(Preterite, P1, SG), "fiquei");
        assert_eq!(v("chegar").conjugate(Preterite, P1, SG), "cheguei");
        assert_eq!(v("começar").conjugate(Preterite, P1, SG), "comecei");
        assert_eq!(v("conhecer").conjugate(Present, P1, SG), "conheço");
        assert_eq!(v("dirigir").conjugate(Present, P1, SG), "dirijo");
        assert_eq!(v("erguer").conjugate(Present, P1, SG), "ergo");
        assert_eq!(v("sair").conjugate(Preterite, P1, SG), "saí");
        assert_eq!(v("sair").conjugate(Preterite, P3, SG), "saiu");
        assert_eq!(v("sair").conjugate(Present, P1, PL), "saímos");
        assert_eq!(v("sair").past_participle(), "saído");
        assert_eq!(v("cair").conjugate(Imperfect, P1, SG), "caía");
        assert_eq!(v("passear").conjugate(Present, P1, SG), "passeio");
        assert_eq!(v("passear").conjugate(Present, P1, PL), "passeamos");
        assert_eq!(
            v("passear").conjugate(SubjunctivePresent, P3, PL),
            "passeiem"
        );
    }
}
