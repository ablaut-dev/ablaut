//! Catalan conjugation: the regular engine and the productive
//! orthographic rules, plus a compiled-in irregular lexicon.
//!
//! Three conjugations by infinitive ending:
//!
//! - 1st: `-ar` (parlar) — the open productive class.
//! - 2nd: `-er`/`-re` (témer, perdre, batre) — participle `-ut`,
//!   gerund `-ent`; largely velar-stem and strong-preterite irregulars,
//!   which live in `data/cat/verbs.tsv`.
//! - 3rd: `-ir`, split into pure (dormir: dorm, dorms) and inchoative
//!   (servir: serveixo, serveix), the inchoative `-eix-` infix appearing
//!   in the stressed present and present-subjunctive slots.
//!
//! Sound-preserving spelling before a front vowel (e/i): `c→qu`
//! (tocar → toques, toqui), `ç→c` (començar → comences), `g→gu`
//! (pagar → paguem), `gu→gü`, `qu→qü`, `j→g` (menjar → mengem). Central
//! (IEC) orthography, matching both gold oracles.

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
/// (haver / anar + participle or infinitive) are the compositional
/// layer's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpleTense {
    /// Present d'indicatiu (parlo).
    Present,
    /// Imperfet d'indicatiu (parlava).
    Imperfect,
    /// Passat simple / pretèrit perfet simple (parlí).
    Preterite,
    /// Futur (parlaré).
    Future,
    /// Condicional (parlaria).
    Conditional,
    /// Present de subjuntiu (parli).
    SubjunctivePresent,
    /// Imperfet de subjuntiu (parlés).
    SubjunctiveImperfect,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A verb whose paradigm needs data that has not landed yet.
    Unsupported,
    /// The input does not look like a Catalan infinitive at all.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "not a supported Catalan verb"),
            Self::NotAVerb => write!(f, "not a Catalan infinitive"),
        }
    }
}

/// Inflection class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Group {
    /// -ar.
    Ar,
    /// -er / -re.
    Er,
    /// -ir, pure (dormir).
    Ir,
    /// -ir, inchoative (servir → serveixo).
    IrInch,
}

// Endings, indexed [1sg, 2sg, 3sg, 1pl, 2pl, 3pl].
const PRS_AR: [&str; 6] = ["o", "es", "a", "em", "eu", "en"];
const PRS_ER: [&str; 6] = ["o", "s", "", "em", "eu", "en"];
const PRS_IR: [&str; 6] = ["o", "s", "", "im", "iu", "en"];
const PRS_INCH: [&str; 6] = ["eixo", "eixes", "eix", "im", "iu", "eixen"];

const IPF_AR: [&str; 6] = ["ava", "aves", "ava", "àvem", "àveu", "aven"];
const IPF_ER: [&str; 6] = ["ia", "ies", "ia", "íem", "íeu", "ien"];
const IPF_IR: [&str; 6] = ["ia", "ies", "ia", "íem", "íeu", "ien"];

const PRET_AR: [&str; 6] = ["í", "ares", "à", "àrem", "àreu", "aren"];
const PRET_ER: [&str; 6] = ["í", "eres", "é", "érem", "éreu", "eren"];
const PRET_IR: [&str; 6] = ["í", "ires", "í", "írem", "íreu", "iren"];

const FUT: [&str; 6] = ["é", "às", "à", "em", "eu", "an"];
const COND: [&str; 6] = ["ia", "ies", "ia", "íem", "íeu", "ien"];

const SPRS_AR: [&str; 6] = ["i", "is", "i", "em", "eu", "in"];
const SPRS_ER: [&str; 6] = ["i", "is", "i", "em", "eu", "in"];
const SPRS_IR: [&str; 6] = ["i", "is", "i", "im", "iu", "in"];
const SPRS_INCH: [&str; 6] = ["eixi", "eixis", "eixi", "im", "iu", "eixin"];

const SIMP_AR: [&str; 6] = ["és", "essis", "és", "éssim", "éssiu", "essin"];
const SIMP_ER: [&str; 6] = ["és", "essis", "és", "éssim", "éssiu", "essin"];
const SIMP_IR: [&str; 6] = ["ís", "issis", "ís", "íssim", "íssiu", "issin"];

fn is_vowel(c: char) -> bool {
    "aeiouàéèíïòóúü".contains(c)
}

/// Front-vowel-initial ending: triggers the softening spelling rules.
fn is_front(ending: &str) -> bool {
    matches!(ending.chars().next(), Some('e' | 'i' | 'é' | 'è' | 'í'))
}

/// Sound-preserving spelling of a stem before a front vowel.
fn front_soften(stem: &str) -> String {
    for (from, to) in [("gu", "gü"), ("qu", "qü")] {
        if let Some(body) = stem.strip_suffix(from) {
            return format!("{body}{to}");
        }
    }
    let last = stem.chars().last();
    match last {
        Some('c') => format!("{}qu", &stem[..stem.len() - 1]),
        Some('ç') => format!("{}c", &stem[..stem.len() - 'ç'.len_utf8()]),
        Some('g') => format!("{}gu", &stem[..stem.len() - 1]),
        Some('j') => format!("{}g", &stem[..stem.len() - 1]),
        _ => stem.to_string(),
    }
}

/// The feminine of a participle by surface phonology: a vowel + `t`
/// voices (parlat → parlada, perdut → perduda), everything else takes a
/// plain `-a` (obert → oberta). Athematic vowel + t participles (dit →
/// dita) are stored explicitly and never reach this.
fn voice_feminine(pp: &str) -> String {
    let chars: Vec<char> = pp.chars().collect();
    if chars.last() == Some(&'t') && chars.len() >= 2 && is_vowel(chars[chars.len() - 2]) {
        format!("{}da", &pp[..pp.len() - 1])
    } else {
        format!("{pp}a")
    }
}

/// The compiled-in irregular lexicon.
static LEXICON_TSV: &str = include_str!("../data/cat/verbs.tsv");

// Exact-match-only bases are flagged in the data (column 10 = "1"): a
// base like `dir` is a suffix of many regular verbs (accedir, ascendir),
// so it must match the whole infinitive, and its real compounds
// (contradir, desdir) are stored as their own rows.

/// A stored irregular paradigm. Every finite tense is an optional
/// six-form override (comma-separated, [1sg..3pl]); an empty cell (`-`)
/// falls through to the regular engine, so an irregular that is regular
/// in a tense costs nothing to store. `fut_stem` derives the future and
/// conditional together (tindr → tindré, tindria).
///
/// Columns: lemma, pres, impf, pret, subjpres, subjimpf, fut_stem, pp,
/// ger, imp2.
#[derive(Debug, Clone, Default)]
struct LexEntry {
    pres: Option<[String; 6]>,
    impf: Option<[String; 6]>,
    pret: Option<[String; 6]>,
    subjpres: Option<[String; 6]>,
    subjimpf: Option<[String; 6]>,
    fut_stem: Option<String>,
    pp: Option<String>,
    ger: Option<String>,
    imp2: Option<String>,
    /// Feminine singular participle (athematic ones do not voice: dit →
    /// dita, not *dida); the masculine/feminine plural derive from it.
    ppf: Option<String>,
    /// 2pl imperative override (estar → estigueu, not the indicative
    /// esteu); most velar verbs keep the indicative preneu.
    imp2pl: Option<String>,
}

fn six(s: &str) -> Option<[String; 6]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
}

fn opt(s: &str) -> Option<String> {
    (s != "-").then(|| s.to_string())
}

fn parse_lex(cols: &[&str]) -> LexEntry {
    let g = |i: usize| cols.get(i).copied().unwrap_or("-");
    LexEntry {
        pres: six(g(1)),
        impf: six(g(2)),
        pret: six(g(3)),
        subjpres: six(g(4)),
        subjimpf: six(g(5)),
        fut_stem: opt(g(6)),
        pp: opt(g(7)),
        ger: opt(g(8)),
        imp2: opt(g(9)),
        ppf: opt(g(11)),
        imp2pl: opt(g(12)),
    }
}

/// Longest-base suffix lookup with the derivational prefix returned
/// (comprendre → ("com", prendre)); EXACT_ONLY bases match whole.
fn lexical(inf: &str) -> Option<(String, LexEntry)> {
    let mut best: Option<(&str, Vec<&str>)> = None;
    for line in LEXICON_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let lemma = cols[0];
        let exact = cols.get(10).copied() == Some("1");
        let hit = if exact {
            inf == lemma
        } else {
            inf.ends_with(lemma)
        };
        if hit && !best.as_ref().is_some_and(|(b, _)| b.len() >= lemma.len()) {
            best = Some((lemma, cols));
        }
    }
    let (lemma, cols) = best?;
    let prefix = inf[..inf.len() - lemma.len()].to_string();
    Some((prefix, parse_lex(&cols)))
}

/// A conjugatable Catalan verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    reflexive: bool,
    /// Infinitive minus the class ending.
    stem: String,
    group: Group,
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

/// The inchoative -ir verbs are the productive majority; a short list of
/// common pure -ir verbs is the exception (see `PURE_IR`).
const PURE_IR: [&str; 24] = [
    "dormir",
    "morir",
    "sortir",
    "collir",
    "cosir",
    "cobrir",
    "obrir",
    "omplir",
    "tossir",
    "pudir",
    "fugir",
    "bullir",
    "grunyir",
    "escopir",
    "sentir",
    "consentir",
    "pressentir",
    "mentir",
    "penedir",
    "ajupir",
    "esmunyir",
    "brunzir",
    "retenir",
    "obtenir",
];

impl Verb {
    /// Build a verb from its infinitive. A pronominal `-se`/`-'s`
    /// infinitive conjugates with its clitic.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let lowered = infinitive.to_lowercase();
        let mut inf = lowered.trim();
        let mut reflexive = false;
        // pronominal: rentar-se, penedir-se, moure's.
        if let Some(bare) = inf.strip_suffix("-se").or_else(|| inf.strip_suffix("-s")) {
            inf = bare;
            reflexive = true;
        } else if let Some(bare) = inf.strip_suffix("'s") {
            inf = bare;
            reflexive = true;
        }
        if inf.is_empty() || inf.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        let (stem, group) = if let Some(stem) = inf.strip_suffix("ar") {
            (stem, Group::Ar)
        } else if let Some(stem) = inf.strip_suffix("re") {
            (stem, Group::Er)
        } else if let Some(stem) = inf.strip_suffix("er") {
            (stem, Group::Er)
        } else if let Some(stem) = inf.strip_suffix("ir") {
            let g = if PURE_IR.iter().any(|p| inf.ends_with(p)) {
                Group::Ir
            } else {
                Group::IrInch
            };
            (stem, g)
        } else {
            return Err(Error::Unsupported);
        };
        let (prefix, lex) = match lexical(inf) {
            Some((p, e)) => (p, Some(Box::new(e))),
            None => (String::new(), None),
        };
        if lex.is_none() && stem.is_empty() {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf.to_string(),
            reflexive,
            stem: stem.to_string(),
            group,
            lex,
            prefix,
        })
    }

    /// The infinitive as normalized (rentar-se for pronominal verbs).
    pub fn infinitive(&self) -> String {
        if self.reflexive {
            // -s after a vowel-final infinitive (moure's), -se elsewhere.
            if self.infinitive.ends_with('e') && self.infinitive.ends_with("re") {
                format!("{}'s", self.infinitive)
            } else {
                format!("{}-se", self.infinitive)
            }
        } else {
            self.infinitive.clone()
        }
    }

    /// Prepend the reflexive clitic (proclitic before finite forms).
    fn cliticize(&self, form: String, person: Person, number: Number) -> String {
        if !self.reflexive {
            return form;
        }
        let vowel = form.starts_with(|c: char| is_vowel(c) || c == 'h');
        let clitic = match (person, number) {
            (Person::First, Number::Singular) => {
                if vowel {
                    "m'"
                } else {
                    "em "
                }
            }
            (Person::Second, Number::Singular) => {
                if vowel {
                    "t'"
                } else {
                    "et "
                }
            }
            (Person::Third, _) => {
                if vowel {
                    "s'"
                } else {
                    "es "
                }
            }
            (Person::First, Number::Plural) => "ens ",
            (Person::Second, Number::Plural) => "us ",
        };
        format!("{clitic}{form}")
    }

    /// The regular future/conditional stem.
    fn future_stem(&self) -> String {
        if let Some(l) = &self.lex {
            if let Some(fs) = &l.fut_stem {
                return format!("{}{fs}", self.prefix);
            }
        }
        match self.group {
            Group::Ar | Group::Ir | Group::IrInch => self.infinitive.clone(),
            // -re: drop the final e (perdre → perdr); -er: deaccent
            // (témer → temer).
            Group::Er => {
                if let Some(body) = self.infinitive.strip_suffix('e') {
                    body.to_string()
                } else {
                    // -er ending: deaccent è/é in the stem.
                    self.infinitive.replace(['è', 'é'], "e")
                }
            }
        }
    }

    /// Attach an ending to a stem with the front-vowel spelling rules and
    /// the syllabic-i diaeresis.
    fn attach(&self, stem: &str, ending: &str) -> String {
        let s = if is_front(ending) {
            front_soften(stem)
        } else {
            stem.to_string()
        };
        // An i/í-initial ending after a vowel stem takes a diaeresis so it
        // reads as its own syllable, not a diphthong: crear → creï,
        // continuar → continuï, agrair → agraïm. The silent u of gu/qu
        // (adequar → adeqüi) does not count as a preceding vowel.
        let mut e = ending.to_string();
        let vowel_stem = s.ends_with(|c: char| is_vowel(c))
            && !s.ends_with("gu")
            && !s.ends_with("qu")
            && !s.ends_with("gü")
            && !s.ends_with("qü");
        // Only the unstressed i takes the diaeresis; a stressed í keeps
        // its accent (parlí, agraíem), which already marks the hiatus.
        if vowel_stem {
            if let Some(rest) = e.strip_prefix('i') {
                e = format!("ï{rest}");
            }
        }
        format!("{s}{e}")
    }

    /// The rule-path form for a slot (regular + spelling).
    fn regular_form(&self, tense: SimpleTense, i: usize) -> String {
        match tense {
            SimpleTense::Present => {
                let e = match self.group {
                    Group::Ar => &PRS_AR,
                    Group::Er => &PRS_ER,
                    Group::Ir => &PRS_IR,
                    Group::IrInch => &PRS_INCH,
                };
                self.attach(&self.stem, e[i])
            }
            SimpleTense::Imperfect => {
                let e = match self.group {
                    Group::Ar => &IPF_AR,
                    Group::Er => &IPF_ER,
                    Group::Ir | Group::IrInch => &IPF_IR,
                };
                self.attach(&self.stem, e[i])
            }
            SimpleTense::Preterite => {
                let e = match self.group {
                    Group::Ar => &PRET_AR,
                    Group::Er => &PRET_ER,
                    Group::Ir | Group::IrInch => &PRET_IR,
                };
                self.attach(&self.stem, e[i])
            }
            SimpleTense::Future => format!("{}{}", self.future_stem(), FUT[i]),
            SimpleTense::Conditional => format!("{}{}", self.future_stem(), COND[i]),
            SimpleTense::SubjunctivePresent => {
                let e = match self.group {
                    Group::Ar => &SPRS_AR,
                    Group::Er => &SPRS_ER,
                    Group::Ir => &SPRS_IR,
                    Group::IrInch => &SPRS_INCH,
                };
                self.attach(&self.stem, e[i])
            }
            SimpleTense::SubjunctiveImperfect => {
                let e = match self.group {
                    Group::Ar => &SIMP_AR,
                    Group::Er => &SIMP_ER,
                    Group::Ir | Group::IrInch => &SIMP_IR,
                };
                self.attach(&self.stem, e[i])
            }
        }
    }

    /// A stored-paradigm form, falling through to the regular engine for
    /// any tense the lexicon leaves blank.
    fn lex_form(&self, l: &LexEntry, tense: SimpleTense, i: usize) -> String {
        let stored = match tense {
            SimpleTense::Present => l.pres.as_ref().map(|f| f[i].clone()),
            SimpleTense::Preterite => l.pret.as_ref().map(|f| f[i].clone()),
            SimpleTense::Imperfect => l.impf.as_ref().map(|f| f[i].clone()),
            SimpleTense::SubjunctivePresent => l.subjpres.as_ref().map(|f| f[i].clone()),
            SimpleTense::SubjunctiveImperfect => l.subjimpf.as_ref().map(|f| f[i].clone()),
            SimpleTense::Future | SimpleTense::Conditional => None,
        };
        match stored {
            Some(f) => format!("{}{f}", self.prefix),
            None => self.regular_form(tense, i),
        }
    }

    fn plain(&self, tense: SimpleTense, person: Person, number: Number) -> String {
        let i = person.index(number);
        if let Some(l) = &self.lex {
            return self.lex_form(&l.clone(), tense, i);
        }
        self.regular_form(tense, i)
    }

    /// A finite form; pronominal verbs carry their clitic (em rento).
    pub fn conjugate(&self, tense: SimpleTense, person: Person, number: Number) -> String {
        self.cliticize(self.plain(tense, person, number), person, number)
    }

    /// Every standard spelling, canonical first. Catalan has no synthetic
    /// doublets in the core paradigm, so this is a single form; the method
    /// mirrors the other languages' harness contract.
    pub fn variants(&self, tense: SimpleTense, person: Person, number: Number) -> Vec<String> {
        vec![self.conjugate(tense, person, number)]
    }

    /// The imperative: tu, vostè (subjunctive), nosaltres, vosaltres,
    /// vostès. No first-person singular.
    pub fn imperative(&self, person: Person, number: Number) -> Option<String> {
        match (person, number) {
            (Person::First, Number::Singular) => None,
            // tu: the 3sg present indicative (parla, perd, dorm, serveix),
            // or a stored short imperative (pren, digues, estigues).
            (Person::Second, Number::Singular) => {
                if let Some(l) = &self.lex {
                    if let Some(imp) = &l.imp2 {
                        return Some(format!("{}{imp}", self.prefix));
                    }
                }
                Some(self.plain(SimpleTense::Present, Person::Third, Number::Singular))
            }
            // nosaltres: the present subjunctive (parlem, prenguem).
            (Person::First, Number::Plural) => Some(self.plain(
                SimpleTense::SubjunctivePresent,
                Person::First,
                Number::Plural,
            )),
            // vosaltres: the present indicative (preneu), unless stored
            // (estar → estigueu).
            (Person::Second, Number::Plural) => {
                if let Some(l) = &self.lex {
                    if let Some(imp) = &l.imp2pl {
                        return Some(format!("{}{imp}", self.prefix));
                    }
                }
                Some(self.plain(SimpleTense::Present, Person::Second, Number::Plural))
            }
            // vostè, vostès: the present subjunctive.
            (p, n) => Some(self.plain(SimpleTense::SubjunctivePresent, p, n)),
        }
    }

    /// Gerund: parlant, perdent, dormint, servint.
    pub fn gerund(&self) -> String {
        if let Some(l) = &self.lex {
            if let Some(g) = &l.ger {
                return format!("{}{g}", self.prefix);
            }
        }
        let ending = match self.group {
            Group::Ar => "ant",
            Group::Er => "ent",
            Group::Ir | Group::IrInch => "int",
        };
        format!("{}{ending}", self.stem)
    }

    /// Past participle, masculine singular (citation form).
    pub fn past_participle(&self) -> String {
        if let Some(l) = &self.lex {
            if let Some(pp) = &l.pp {
                return format!("{}{pp}", self.prefix);
            }
        }
        let ending = match self.group {
            Group::Ar => "at",
            Group::Er => "ut",
            Group::Ir | Group::IrInch => "it",
        };
        format!("{}{ending}", self.stem)
    }

    /// A gender/number-inflected past participle.
    ///
    /// Regular participles end in a vowel + `t` and voice it in the
    /// feminine: parlat → parlada, perdut → perduda, dormit → dormida.
    /// Irregular (stored) participles are athematic: the feminine is a
    /// plain `-a` (obert → oberta, dit → dita, escrit → escrita) and an
    /// `-s` participle takes `-sa` (pres → presa). The feminine plural is
    /// the feminine with `-a → -es`; the masculine plural adds `-s`
    /// (`-os` after a sibilant).
    pub fn past_participle_inflected(&self, feminine: bool, plural: bool) -> String {
        let pp = self.past_participle();
        // The feminine singular carries the productive stem for every
        // inflected form: a stored athematic form (dit → dita), else a
        // regular vowel + t participle voices (parlat → parlada), else a
        // plain -a. A sibilant participle deaccents (comès → comesa), so
        // the feminine stem is what the plurals build on too.
        let fem = self
            .lex
            .as_ref()
            .and_then(|l| l.ppf.as_ref())
            .map_or_else(|| voice_feminine(&pp), |f| format!("{}{f}", self.prefix));
        let fem_stem = &fem[..fem.len() - 1];
        if feminine {
            return if plural { format!("{fem_stem}es") } else { fem };
        }
        if !plural {
            return pp;
        }
        // Masculine plural: sibilant participles take -os on the
        // deaccented stem (comès → comesos); the rest add -s (parlats,
        // oberts).
        if pp.ends_with('s') {
            format!("{fem_stem}os")
        } else {
            format!("{pp}s")
        }
    }
}

/// The full conjugation table of a Catalan verb as one plain struct —
/// shared by the WebAssembly and Python bindings. Rows are
/// [jo, tu, ell/ella, nosaltres, vosaltres, ells/elles].
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    pub gerund: String,
    pub past_participle: String,
    /// [tu, vostè, nosaltres, vosaltres, vostès].
    pub imperative: [Option<String>; 5],
    pub present: [String; 6],
    pub imperfect: [String; 6],
    pub preterite: [String; 6],
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
            infinitive: v.infinitive(),
            gerund: v.gerund(),
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
            preterite: row(SimpleTense::Preterite),
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
    use SimpleTense::*;

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn regular_ar() {
        let p = v("parlar");
        assert_eq!(p.conjugate(Present, P1, SG), "parlo");
        assert_eq!(p.conjugate(Present, P2, SG), "parles");
        assert_eq!(p.conjugate(Present, P3, SG), "parla");
        assert_eq!(p.conjugate(Present, P1, PL), "parlem");
        assert_eq!(p.conjugate(Present, P2, PL), "parleu");
        assert_eq!(p.conjugate(Present, P3, PL), "parlen");
        assert_eq!(p.conjugate(Imperfect, P1, SG), "parlava");
        assert_eq!(p.conjugate(Imperfect, P1, PL), "parlàvem");
        assert_eq!(p.conjugate(Preterite, P1, SG), "parlí");
        assert_eq!(p.conjugate(Preterite, P3, SG), "parlà");
        assert_eq!(p.conjugate(Future, P1, SG), "parlaré");
        assert_eq!(p.conjugate(Future, P3, PL), "parlaran");
        assert_eq!(p.conjugate(Conditional, P1, SG), "parlaria");
        assert_eq!(p.conjugate(SubjunctivePresent, P1, SG), "parli");
        assert_eq!(p.conjugate(SubjunctivePresent, P3, PL), "parlin");
        assert_eq!(p.conjugate(SubjunctiveImperfect, P1, SG), "parlés");
        assert_eq!(p.gerund(), "parlant");
        assert_eq!(p.past_participle(), "parlat");
        assert_eq!(p.past_participle_inflected(true, false), "parlada");
        assert_eq!(p.past_participle_inflected(true, true), "parlades");
        assert_eq!(p.past_participle_inflected(false, true), "parlats");
        assert_eq!(p.imperative(P2, SG).unwrap(), "parla");
    }

    #[test]
    fn spelling_rules() {
        assert_eq!(v("tocar").conjugate(Present, P2, SG), "toques");
        assert_eq!(v("tocar").conjugate(SubjunctivePresent, P1, SG), "toqui");
        assert_eq!(v("pagar").conjugate(SubjunctivePresent, P1, PL), "paguem");
        assert_eq!(v("menjar").conjugate(Present, P2, SG), "menges");
        assert_eq!(v("començar").conjugate(Present, P2, SG), "comences");
        assert_eq!(v("començar").conjugate(Present, P1, SG), "començo");
    }

    #[test]
    fn regular_er_ir() {
        assert_eq!(v("perdre").conjugate(Present, P1, SG), "perdo");
        assert_eq!(v("perdre").conjugate(Present, P3, SG), "perd");
        assert_eq!(v("perdre").conjugate(Present, P1, PL), "perdem");
        assert_eq!(v("perdre").conjugate(Future, P1, SG), "perdré");
        assert_eq!(v("perdre").gerund(), "perdent");
        assert_eq!(v("perdre").past_participle(), "perdut");
        assert_eq!(v("dormir").conjugate(Present, P1, SG), "dormo");
        assert_eq!(v("dormir").conjugate(Present, P1, PL), "dormim");
        assert_eq!(v("dormir").gerund(), "dormint");
        assert_eq!(v("servir").conjugate(Present, P1, SG), "serveixo");
        assert_eq!(v("servir").conjugate(Present, P3, SG), "serveix");
        assert_eq!(v("servir").conjugate(Present, P1, PL), "servim");
        assert_eq!(
            v("servir").conjugate(SubjunctivePresent, P1, SG),
            "serveixi"
        );
        assert_eq!(v("servir").imperative(P2, SG).unwrap(), "serveix");
    }

    #[test]
    fn irregular_lexicon() {
        assert_eq!(v("anar").conjugate(Present, P1, SG), "vaig");
        assert_eq!(v("anar").conjugate(Present, P3, SG), "va");
        assert_eq!(v("anar").conjugate(Future, P1, SG), "aniré");
        assert_eq!(v("anar").past_participle(), "anat");
        assert_eq!(v("fer").conjugate(Present, P1, SG), "faig");
        assert_eq!(v("fer").conjugate(Preterite, P3, SG), "féu");
        assert_eq!(v("fer").past_participle(), "fet");
        assert_eq!(v("fer").past_participle_inflected(true, false), "feta");
        assert_eq!(v("dir").conjugate(Present, P3, SG), "diu");
        assert_eq!(v("dir").gerund(), "dient");
        assert_eq!(v("dir").past_participle(), "dit");
        assert_eq!(v("dir").past_participle_inflected(true, false), "dita");
        assert_eq!(v("tenir").conjugate(Present, P1, SG), "tinc");
        assert_eq!(v("tenir").conjugate(Future, P1, SG), "tindré");
        // Prefixed derivatives come free by suffix match.
        assert_eq!(v("obtenir").conjugate(Present, P1, SG), "obtinc");
        assert_eq!(v("comprendre").past_participle(), "comprès");
        assert_eq!(v("recórrer").conjugate(Preterite, P3, SG), "recorregué");
        // A regular verb ending in an irregular base is not hijacked.
        assert_eq!(v("accedir").conjugate(Present, P3, SG), "accedeix");
    }

    #[test]
    fn irregular_participle_plurals() {
        // Sibilant participles deaccent and take -os / -es.
        assert_eq!(
            v("comprendre").past_participle_inflected(false, true),
            "compresos"
        );
        assert_eq!(
            v("comprendre").past_participle_inflected(true, true),
            "compreses"
        );
    }
}
