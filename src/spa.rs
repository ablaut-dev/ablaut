//! Spanish conjugation: the regular engine and the productive
//! orthographic rules.
//!
//! Three groups (-ar, -er, -ir) with the spelling alternations that
//! preserve the stem's sound in writing:
//!
//! - before e: *buscar → busqué*, *llegar → llegué*, *empezar → empecé*,
//!   *averiguar → averigüé*
//! - before o/a: *coger → cojo*, *vencer → venzo*, *conocer → conozco*
//!   (zc after a vowel), *distinguir → distingo*, *delinquir → delinco*
//! - y-insertion in -uir verbs: *construir → construyo, construyó*
//! - vowel-final stems: *leer → leyó, leyendo, leído* (i → y between
//!   vowels, accented í in i-initial endings)
//!
//! Stem-changing classes (e→ie *pienso*, o→ue *cuento*, e→i *pido*, …)
//! come from `data/spa/classes.tsv`, mined from the gold oracles; true
//! irregulars live in `data/spa/verbs.tsv` (both in later commits).

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

/// The eight simple tense/mood combinations. Compound tenses
/// (haber + participle) are the compositional layer's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpleTense {
    Present,
    /// Imperfecto (hablaba).
    Imperfect,
    /// Pretérito indefinido (hablé).
    Preterite,
    Future,
    Conditional,
    SubjunctivePresent,
    /// The -ra imperfect subjunctive (hablara); the -se doublet is a
    /// variant.
    SubjunctiveImperfect,
    /// The archaic future subjunctive (hablare).
    SubjunctiveFuture,
}

/// Why an infinitive cannot be conjugated (yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A verb whose paradigm needs data that has not landed yet.
    Unsupported,
    /// The input does not look like a Spanish infinitive at all.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "not a supported Spanish verb"),
            Self::NotAVerb => write!(f, "not a Spanish infinitive"),
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

/// Endings tables, [1sg, 2sg, 3sg, 1pl, 2pl, 3pl]. The empty string in
/// the imperative marks a slot that does not exist (1sg).
const PRS_AR: [&str; 6] = ["o", "as", "a", "amos", "áis", "an"];
const PRS_ER: [&str; 6] = ["o", "es", "e", "emos", "éis", "en"];
const PRS_IR: [&str; 6] = ["o", "es", "e", "imos", "ís", "en"];
const IPF_AR: [&str; 6] = ["aba", "abas", "aba", "ábamos", "abais", "aban"];
const IPF_ERIR: [&str; 6] = ["ía", "ías", "ía", "íamos", "íais", "ían"];
const PRET_AR: [&str; 6] = ["é", "aste", "ó", "amos", "asteis", "aron"];
const PRET_ERIR: [&str; 6] = ["í", "iste", "ió", "imos", "isteis", "ieron"];
const FUT: [&str; 6] = ["é", "ás", "á", "emos", "éis", "án"];
const COND: [&str; 6] = ["ía", "ías", "ía", "íamos", "íais", "ían"];
const SPRS_AR: [&str; 6] = ["e", "es", "e", "emos", "éis", "en"];
const SPRS_ERIR: [&str; 6] = ["a", "as", "a", "amos", "áis", "an"];
const SIMP_RA_AR: [&str; 6] = ["ara", "aras", "ara", "áramos", "arais", "aran"];
const SIMP_SE_AR: [&str; 6] = ["ase", "ases", "ase", "ásemos", "aseis", "asen"];
const SIMP_RA_ERIR: [&str; 6] = ["iera", "ieras", "iera", "iéramos", "ierais", "ieran"];
const SIMP_SE_ERIR: [&str; 6] = ["iese", "ieses", "iese", "iésemos", "ieseis", "iesen"];
const SFUT_AR: [&str; 6] = ["are", "ares", "are", "áremos", "areis", "aren"];
const SFUT_ERIR: [&str; 6] = ["iere", "ieres", "iere", "iéremos", "iereis", "ieren"];

fn is_vowel(c: char) -> bool {
    "aeiouáéíóúü".contains(c)
}

/// The compiled-in irregular lexicon (see the schema comment there).
static LEXICON_TSV: &str = include_str!("../data/spa/verbs.tsv");

/// Bases whose suffix collides with regular verbs (toser is not to+ser,
/// mandar not m+andar, mover not mo+ver): exact match only.
const EXACT_ONLY: [&str; 9] = [
    "ser", "estar", "ir", "dar", "ver", "rever", "haber", "andar", "asir",
];

/// A stored irregular paradigm from `data/spa/verbs.tsv`.
#[derive(Debug, Clone)]
struct LexEntry {
    pres: [String; 6],
    /// Either a stem (+ a,as,a,amos,áis,an) or six full forms.
    subj: String,
    /// Strong preterite 1sg (puse); pret6 overrides it wholesale.
    pret1: Option<String>,
    fut: Option<String>,
    pp: String,
    impf6: Option<[String; 6]>,
    ger: Option<String>,
    imp2: Option<String>,
    pret6: Option<[String; 6]>,
}

fn parse_lex(cols: &[&str]) -> LexEntry {
    let opt = |i: usize| cols.get(i).filter(|c| **c != "-").map(|c| (*c).to_string());
    let six = |s: Option<String>| {
        s.map(|s| {
            let v: Vec<String> = s.split(',').map(str::to_string).collect();
            v.try_into().expect("six comma-separated forms")
        })
    };
    LexEntry {
        pres: std::array::from_fn(|i| cols[1 + i].to_string()),
        subj: cols[7].to_string(),
        pret1: opt(8),
        fut: opt(9),
        pp: cols[10].to_string(),
        impf6: six(opt(11)),
        ger: opt(12),
        imp2: opt(13),
        pret6: six(opt(14)),
    }
}

/// Longest-base suffix lookup with the derivative prefix returned
/// (proponer → ("pro", poner)); EXACT_ONLY bases match whole.
fn lexical(inf: &str) -> Option<(String, LexEntry)> {
    let mut best: Option<(&str, Vec<&str>)> = None;
    for line in LEXICON_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let lemma = cols[0];
        let hit = if EXACT_ONLY.contains(&lemma) {
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

/// The mined stem-changing classes (see the file header).
static CLASSES_TSV: &str = include_str!("../data/spa/classes.tsv");

/// A stem-changing class from `data/spa/classes.tsv`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StemClass {
    /// e→ie when stressed (pienso); -ir verbs raise to i when the
    /// ending carries an unstressed high vowel (sintió). Also i→ie
    /// (adquiero).
    Ie,
    /// o→ue when stressed (cuento, juego, huele); -ir raises to u
    /// (durmió).
    Ue,
    /// -ir e→i in every stressed or raised slot (pido, pidió).
    Ei,
    /// -iar verbs that stress the i (envío).
    AccI,
    /// -uar (and aullar-type) verbs that stress the u (actúo, aúllo).
    AccU,
    /// e→ie when stressed but no raising (discernir: discierne,
    /// discernió).
    IeNoRaise,
}

fn stem_class(inf: &str) -> Option<StemClass> {
    for line in CLASSES_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (lemma, class) = line.split_once('\t')?;
        if lemma == inf {
            return match class {
                "ie" => Some(StemClass::Ie),
                "ue" => Some(StemClass::Ue),
                "ei" => Some(StemClass::Ei),
                "acc-i" => Some(StemClass::AccI),
                "acc-u" => Some(StemClass::AccU),
                "ien" => Some(StemClass::IeNoRaise),
                _ => None,
            };
        }
    }
    None
}

/// Replace the last occurrence of any of `targets` in `stem` with `to`.
fn replace_last(stem: &str, targets: &[char], to: &str) -> String {
    if let Some(i) = stem.rfind(|c| targets.contains(&c)) {
        let mut out = String::with_capacity(stem.len() + 2);
        out.push_str(&stem[..i]);
        out.push_str(to);
        out.push_str(&stem[i + stem[i..].chars().next().unwrap().len_utf8()..]);
        return out;
    }
    stem.to_string()
}

/// A conjugatable Spanish verb.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    /// Infinitive minus the group ending.
    stem: String,
    group: Group,
    class: Option<StemClass>,
    /// For irregulars: the stored base paradigm and what precedes it
    /// (pro in proponer).
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
        let (stem, group) = if let Some(stem) = inf.strip_suffix("ar") {
            (stem, Group::Ar)
        } else if let Some(stem) = inf.strip_suffix("er") {
            (stem, Group::Er)
        } else if let Some(stem) = inf.strip_suffix("ir") {
            (stem, Group::Ir)
        } else if let Some(stem) = inf.strip_suffix("ír") {
            // oír-family spelling; treated as -ir with a vowel stem.
            (stem, Group::Ir)
        } else {
            return Err(Error::Unsupported);
        };
        let (prefix, lex) = match lexical(inf) {
            Some((p, e)) => (p, Some(Box::new(e))),
            None => (String::new(), None),
        };
        // ser and ir leave no usable stem; the lexicon carries them.
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

    /// The stem with the class change applied for a stressed slot:
    /// piens-, cuent-, pid-, enví-, actú-. Word-initial diphthongs take
    /// their spelling (hue-, ye-).
    fn stressed_stem(&self) -> String {
        let s = &self.stem;
        let out = match self.class {
            Some(StemClass::Ie | StemClass::IeNoRaise) => replace_last(s, &['e', 'i'], "ie"),
            Some(StemClass::Ue) => {
                // ue after g is written üe: avergüenzo, degüella.
                match s.rfind(['o', 'u']) {
                    Some(i) => {
                        let head = &s[..i];
                        let tail = &s[i + 1..];
                        let diph = if head.ends_with('g') { "üe" } else { "ue" };
                        format!("{head}{diph}{tail}")
                    }
                    None => s.clone(),
                }
            }
            Some(StemClass::Ei) => replace_last(s, &['e'], "i"),
            Some(StemClass::AccI) => replace_last(s, &['i'], "í"),
            Some(StemClass::AccU) => replace_last(s, &['u'], "ú"),
            None => s.clone(),
        };
        if let Some(rest) = out.strip_prefix("ie") {
            return format!("ye{rest}");
        }
        if let Some(rest) = out.strip_prefix("ue") {
            return format!("hue{rest}");
        }
        out
    }

    /// The raised stem for -ir verbs in unstressed high slots (sintió,
    /// durmió, pidiendo); other groups keep the base stem.
    fn raised_stem(&self) -> String {
        if self.group != Group::Ir {
            return self.stem.clone();
        }
        // Only the last stem vowel raises (sintió, durmió); readquirir's
        // initial e is not the changing vowel and stays put.
        // The u of gu/qu is a digraph, not the stem vowel (seguir raises
        // its e: siguió).
        let chars: Vec<char> = self.stem.chars().collect();
        let mut last_vowel = None;
        for i in (0..chars.len()).rev() {
            if is_vowel(chars[i]) {
                if chars[i] == 'u' && i > 0 && matches!(chars[i - 1], 'g' | 'q') {
                    continue;
                }
                last_vowel = Some(chars[i]);
                break;
            }
        }
        match (self.class, last_vowel) {
            (Some(StemClass::Ie | StemClass::Ei), Some('e')) => {
                replace_last(&self.stem, &['e'], "i")
            }
            (Some(StemClass::Ue), Some('o')) => replace_last(&self.stem, &['o'], "u"),
            _ => self.stem.clone(),
        }
    }

    /// True where the stem syllable is stressed: present and present
    /// subjunctive, singular and 3pl.
    fn stressed(tense: SimpleTense, i: usize) -> bool {
        matches!(
            tense,
            SimpleTense::Present | SimpleTense::SubjunctivePresent
        ) && matches!(i, 0 | 1 | 2 | 5)
    }

    /// True where -ir verbs raise the stem vowel: preterite 3sg/3pl,
    /// both imperfect subjunctives, future subjunctive, and the 1pl/2pl
    /// present subjunctive (sintamos).
    fn raised(tense: SimpleTense, i: usize) -> bool {
        match tense {
            SimpleTense::Preterite => matches!(i, 2 | 5),
            SimpleTense::SubjunctiveImperfect | SimpleTense::SubjunctiveFuture => true,
            SimpleTense::SubjunctivePresent => matches!(i, 3 | 4),
            _ => false,
        }
    }

    /// The stem for a given slot.
    fn slot_stem(&self, tense: SimpleTense, i: usize) -> String {
        if Self::stressed(tense, i) {
            self.stressed_stem()
        } else if Self::raised(tense, i) {
            self.raised_stem()
        } else {
            self.stem.clone()
        }
    }

    /// Attach an ending to the stem, applying the sound-preserving
    /// spelling rules and the vowel-stem i/y/í alternations.
    fn attach_stem(&self, stem: &str, ending: &str) -> String {
        let first = ending.chars().next().unwrap_or('x');
        let soft = matches!(first, 'e' | 'é'); // -ar verbs before e
        let hard = matches!(first, 'o' | 'a' | 'á'); // -er/-ir before o/a
        let mut s: String = stem.to_string();
        let mut e: String = ending.to_string();

        match self.group {
            Group::Ar if soft => {
                if let Some(body) = s.strip_suffix("gu") {
                    s = format!("{body}gü"); // averiguar → averigüé
                } else if let Some(body) = s.strip_suffix('c') {
                    s = format!("{body}qu"); // buscar → busqué
                } else if let Some(body) = s.strip_suffix('g') {
                    s = format!("{body}gu"); // llegar → llegué
                } else if let Some(body) = s.strip_suffix('z') {
                    s = format!("{body}c"); // empezar → empecé
                }
            }
            Group::Er | Group::Ir if hard => {
                if let Some(body) = s.strip_suffix("gu") {
                    s = format!("{body}g"); // distinguir → distingo
                } else if let Some(body) = s.strip_suffix("qu") {
                    s = format!("{body}c"); // delinquir → delinco
                } else if let Some(body) = s.strip_suffix('c') {
                    // zc after a vowel (conozco), z after a consonant
                    // (venzo). mecer and cocer take plain z (meza,
                    // cuezo); hacer/decir live in the lexicon.
                    let mecer_like = matches!(self.infinitive.as_str(), "mecer" | "remecer")
                        || self.infinitive.ends_with("cocer");
                    if body.ends_with(is_vowel) && !mecer_like {
                        s = format!("{body}zc");
                    } else {
                        s = format!("{body}z");
                    }
                } else if let Some(body) = s.strip_suffix('g') {
                    s = format!("{body}j"); // coger → cojo
                }
            }
            _ => {}
        }

        if matches!(self.group, Group::Er | Group::Ir) {
            let uir = self.is_uir() && s.ends_with(['u', 'ü']);
            let vowel_stem = s.ends_with(['a', 'e', 'o']);
            if uir || vowel_stem {
                // Unstressed i between vowels is written y: leyó,
                // construyeron, cayendo, leyera.
                for (from, to) in [
                    ("ió", "yó"),
                    ("ieron", "yeron"),
                    ("iendo", "yendo"),
                    ("iera", "yera"),
                    ("iéra", "yéra"),
                    ("iese", "yese"),
                    ("iése", "yése"),
                    ("iere", "yere"),
                    ("iére", "yére"),
                ] {
                    if let Some(rest) = e.strip_prefix(from) {
                        e = format!("{to}{rest}");
                        break;
                    }
                }
            }
            // After ñ and ll the unstressed i of the ending is absorbed:
            // bulló, tañendo, riñera.
            if s.ends_with('ñ') || s.ends_with("ll") {
                for (from, to) in [
                    ("ió", "ó"),
                    ("ieron", "eron"),
                    ("iendo", "endo"),
                    ("iera", "era"),
                    ("iéra", "éra"),
                    ("iese", "ese"),
                    ("iése", "ése"),
                    ("iere", "ere"),
                    ("iére", "ére"),
                ] {
                    if let Some(rest) = e.strip_prefix(from) {
                        e = format!("{to}{rest}");
                        break;
                    }
                }
            }
            if uir && e.starts_with('y') {
                // The rewritten y endings also drop the diaeresis:
                // arguyera, arguyendo.
                if let Some(body) = s.strip_suffix('ü') {
                    s = format!("{body}u");
                }
            }
            if uir {
                // y before every vowel-initial ending except i:
                // construyo, construye, construya; construimos and
                // construido keep the bare stem.
                if matches!(first, 'o' | 'e' | 'a' | 'é' | 'á') {
                    // The diaeresis is only needed before e/i: arguyo.
                    if let Some(body) = s.strip_suffix('ü') {
                        s = format!("{body}u");
                    }
                    s.push('y');
                }
            } else if vowel_stem {
                // A stressed i after a strong vowel is written í:
                // leíste, leímos, leído, caído.
                for (from, to) in [
                    ("iste", "íste"),
                    ("isteis", "ísteis"),
                    ("imos", "ímos"),
                    ("ido", "ído"),
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

    /// True for the y-inserting -uir verbs (construir), excluding
    /// -guir/-quir where u is part of the consonant.
    fn is_uir(&self) -> bool {
        self.group == Group::Ir
            && (self.stem.ends_with('ü')
                || (self.stem.ends_with('u')
                    && !self.stem.ends_with("gu")
                    && !self.stem.ends_with("qu")))
    }

    fn attach(&self, ending: &str) -> String {
        self.attach_stem(&self.stem.clone(), ending)
    }

    /// Attach with the class-adjusted stem for this slot.
    fn attach_slot(&self, tense: SimpleTense, i: usize, ending: &str) -> String {
        self.attach_stem(&self.slot_stem(tense, i), ending)
    }

    /// The preterite 3pl-minus-on stem for the -ra/-se/-re families
    /// (pusieron → pusier-), with its accented flavor (pusiér-).
    fn lex_pret_parts(&self, e: &LexEntry) -> Option<(String, String)> {
        let p3 = if let Some(p6) = &e.pret6 {
            p6[5].clone()
        } else if let Some(p1) = &e.pret1 {
            let stem = p1.strip_suffix('e').unwrap_or(p1);
            let ending = if stem.ends_with('j') { "eron" } else { "ieron" };
            format!("{stem}{ending}")
        } else {
            return None;
        };
        let base = p3.strip_suffix("on").unwrap_or(&p3).to_string();
        let accented = match base.rfind('e') {
            Some(i) => format!("{}é{}", &base[..i], &base[i + 1..]),
            None => base.clone(),
        };
        Some((base, accented))
    }

    /// A stored-paradigm finite form (prefix attached). `se` selects the
    /// -se imperfect subjunctive.
    fn conjugate_lex(&self, e: &LexEntry, tense: SimpleTense, i: usize, se: bool) -> String {
        let out = match tense {
            SimpleTense::Present => Some(e.pres[i].clone()),
            SimpleTense::Imperfect => e.impf6.as_ref().map(|f| f[i].clone()),
            SimpleTense::Preterite => {
                if let Some(p6) = &e.pret6 {
                    Some(p6[i].clone())
                } else if let Some(p1) = &e.pret1 {
                    let stem = p1.strip_suffix('e').unwrap_or(p1);
                    let ieron = if stem.ends_with('j') { "eron" } else { "ieron" };
                    Some(match i {
                        0 => p1.clone(),
                        1 => format!("{stem}iste"),
                        2 => format!("{stem}o"),
                        3 => format!("{stem}imos"),
                        4 => format!("{stem}isteis"),
                        _ => format!("{stem}{ieron}"),
                    })
                } else {
                    None
                }
            }
            SimpleTense::Future | SimpleTense::Conditional => {
                let endings = if tense == SimpleTense::Future {
                    &FUT
                } else {
                    &COND
                };
                let stem = e
                    .fut
                    .clone()
                    .unwrap_or_else(|| format!("{}{}", self.prefix_free_infinitive(), ""));
                Some(format!("{stem}{}", endings[i]))
            }
            SimpleTense::SubjunctivePresent => Some(if e.subj.contains(',') {
                e.subj.split(',').nth(i).unwrap().to_string()
            } else {
                let endings: &[&str; 6] = if self.group == Group::Ar {
                    &SPRS_AR
                } else {
                    &["a", "as", "a", "amos", "áis", "an"]
                };
                format!("{}{}", e.subj, endings[i])
            }),
            SimpleTense::SubjunctiveImperfect | SimpleTense::SubjunctiveFuture => {
                self.lex_pret_parts(e).map(|(base, acc)| {
                    let fut = tense == SimpleTense::SubjunctiveFuture;
                    if fut {
                        match i {
                            3 => format!("{acc}emos"),
                            4 => format!("{base}eis"),
                            _ => format!("{base}{}", ["e", "es", "e", "", "", "en"][i]),
                        }
                    } else if se {
                        let b = base.strip_suffix('r').unwrap_or(&base);
                        let a = acc.strip_suffix('r').unwrap_or(&acc);
                        match i {
                            3 => format!("{a}semos"),
                            4 => format!("{b}seis"),
                            _ => format!("{b}{}", ["se", "ses", "se", "", "", "sen"][i]),
                        }
                    } else {
                        match i {
                            3 => format!("{acc}amos"),
                            4 => format!("{base}ais"),
                            _ => format!("{base}{}", ["a", "as", "a", "", "", "an"][i]),
                        }
                    }
                })
            }
        };
        match out {
            Some(f) => format!("{}{f}", self.prefix),
            None => {
                // Regular for this verb: fall through to the rule path.
                self.regular_form(tense, i, se)
            }
        }
    }

    /// The infinitive without the lexicon prefix consideration (used for
    /// future/conditional fallbacks); prefix is already inside it.
    fn prefix_free_infinitive(&self) -> String {
        // e.fut is a bare stem to be prefixed; the fallback uses the
        // whole infinitive, which already contains the prefix, so the
        // caller must not re-prefix. Handled by returning a marker-free
        // string here and special-casing in conjugate().
        self.infinitive.clone()
    }

    /// A finite form without the -se doublet (that is in variants()).
    pub fn conjugate(&self, tense: SimpleTense, person: Person, number: Number) -> String {
        let i = person.index(number);
        if let Some(e) = &self.lex {
            let e = e.clone();
            // Future/conditional with no stored stem use the whole
            // infinitive, which already contains the prefix.
            if matches!(tense, SimpleTense::Future | SimpleTense::Conditional) && e.fut.is_none() {
                let endings = if tense == SimpleTense::Future {
                    &FUT
                } else {
                    &COND
                };
                return format!("{}{}", self.infinitive.replace('í', "i"), endings[i]);
            }
            return self.conjugate_lex(&e, tense, i, false);
        }
        self.regular_form(tense, i, false)
    }

    /// The rule-path form (regular + spelling + class).
    fn regular_form(&self, tense: SimpleTense, i: usize, se: bool) -> String {
        let ar = self.group == Group::Ar;
        match tense {
            SimpleTense::Present => {
                let endings = match self.group {
                    Group::Ar => &PRS_AR,
                    Group::Er => &PRS_ER,
                    Group::Ir => &PRS_IR,
                };
                self.attach_slot(SimpleTense::Present, i, endings[i])
            }
            SimpleTense::Imperfect => {
                let endings = if ar { &IPF_AR } else { &IPF_ERIR };
                self.attach(endings[i])
            }
            SimpleTense::Preterite => {
                let endings = if ar { &PRET_AR } else { &PRET_ERIR };
                self.attach_slot(SimpleTense::Preterite, i, endings[i])
            }
            SimpleTense::Future => format!("{}{}", self.infinitive.replace('í', "i"), FUT[i]),
            SimpleTense::Conditional => {
                format!("{}{}", self.infinitive.replace('í', "i"), COND[i])
            }
            SimpleTense::SubjunctivePresent => {
                let endings = if ar { &SPRS_AR } else { &SPRS_ERIR };
                self.attach_slot(SimpleTense::SubjunctivePresent, i, endings[i])
            }
            SimpleTense::SubjunctiveImperfect => {
                let endings = match (ar, se) {
                    (true, false) => &SIMP_RA_AR,
                    (true, true) => &SIMP_SE_AR,
                    (false, false) => &SIMP_RA_ERIR,
                    (false, true) => &SIMP_SE_ERIR,
                };
                self.attach_slot(SimpleTense::SubjunctiveImperfect, i, endings[i])
            }
            SimpleTense::SubjunctiveFuture => {
                let endings = if ar { &SFUT_AR } else { &SFUT_ERIR };
                self.attach_slot(SimpleTense::SubjunctiveFuture, i, endings[i])
            }
        }
    }

    /// Every standard spelling, canonical first: the -se imperfect
    /// subjunctive rides along with the -ra form.
    pub fn variants(&self, tense: SimpleTense, person: Person, number: Number) -> Vec<String> {
        let mut out = vec![self.conjugate(tense, person, number)];
        if tense == SimpleTense::SubjunctiveImperfect {
            let i = person.index(number);
            let form = match &self.lex {
                Some(e) => self.conjugate_lex(&e.clone(), tense, i, true),
                None => self.regular_form(tense, i, true),
            };
            out.push(form);
        }
        out
    }

    /// The imperative: tú, usted (subjunctive), nosotros, vosotros,
    /// ustedes. No first-person singular.
    pub fn imperative(&self, person: Person, number: Number) -> Option<String> {
        match (person, number) {
            (Person::First, Number::Singular) => None,
            // tú: the 3sg present (habla, come, vive), or the stored
            // short imperative (pon, ten, haz, di).
            (Person::Second, Number::Singular) => Some(match &self.lex {
                Some(e) => match &e.imp2 {
                    Some(f) => {
                        // Compounds of the short imperatives take an
                        // accent: antepón, detén, prevén.
                        if !self.prefix.is_empty() && f == "di" {
                            // decir derivatives use the long form:
                            // contradice, desdice, predice.
                            return Some(self.conjugate(
                                SimpleTense::Present,
                                Person::Third,
                                Number::Singular,
                            ));
                        }
                        let f = if !self.prefix.is_empty() {
                            match f.as_str() {
                                "pon" => "pón",
                                "ten" => "tén",
                                "ven" => "vén",
                                other => other,
                            }
                        } else {
                            f
                        };
                        format!("{}{f}", self.prefix)
                    }
                    None => self.conjugate(SimpleTense::Present, Person::Third, Number::Singular),
                },
                None => self.conjugate(SimpleTense::Present, Person::Third, Number::Singular),
            }),
            // vosotros: stem + ad/ed/id (íd after a bare vowel: oíd).
            (Person::Second, Number::Plural) => {
                let ending = match self.group {
                    Group::Ar => "ad",
                    Group::Er => "ed",
                    Group::Ir => {
                        if self.stem.ends_with(['a', 'e', 'o']) {
                            "íd"
                        } else {
                            "id"
                        }
                    }
                };
                Some(format!("{}{ending}", self.stem))
            }
            // usted, nosotros, ustedes: the present subjunctive.
            (p, n) => Some(self.conjugate(SimpleTense::SubjunctivePresent, p, n)),
        }
    }

    /// Gerund: hablando, comiendo, construyendo, leyendo.
    pub fn gerund(&self) -> String {
        if let Some(e) = &self.lex {
            if let Some(g) = &e.ger {
                return format!("{}{g}", self.prefix);
            }
        }
        match self.group {
            Group::Ar => self.attach("ando"),
            Group::Er | Group::Ir => self.attach_stem(&self.raised_stem(), "iendo"),
        }
    }

    /// Past participle: hablado, comido, leído; masculine singular is
    /// the citation form, the other three derive regularly.
    pub fn past_participle(&self) -> String {
        if let Some(e) = &self.lex {
            return format!("{}{}", self.prefix, e.pp);
        }
        // Productive irregular-participle families, then exact odd ones.
        let inf = &self.infinitive;
        for (suffix, repl) in [
            ("brir", "bierto"),
            ("cribir", "crito"),
            ("olver", "uelto"),
            ("morir", "muerto"),
        ] {
            if let Some(head) = inf.strip_suffix(suffix) {
                return format!("{head}{repl}");
            }
        }
        if inf == "romper" {
            return "roto".to_string();
        }
        if inf == "pudrir" {
            return "podrido".to_string();
        }
        match self.group {
            Group::Ar => self.attach("ado"),
            Group::Er | Group::Ir => self.attach("ido"),
        }
    }

    /// A gender/number-inflected past participle (hablada, habladas).
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

#[cfg(test)]
mod tests {
    use super::*;
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};
    use SimpleTense::{
        Conditional, Future, Imperfect, Present, Preterite, SubjunctiveFuture,
        SubjunctiveImperfect, SubjunctivePresent,
    };

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn regular_ar() {
        let h = v("hablar");
        assert_eq!(h.conjugate(Present, P1, SG), "hablo");
        assert_eq!(h.conjugate(Present, P2, PL), "habláis");
        assert_eq!(h.conjugate(Imperfect, P1, PL), "hablábamos");
        assert_eq!(h.conjugate(Preterite, P1, SG), "hablé");
        assert_eq!(h.conjugate(Preterite, P3, SG), "habló");
        assert_eq!(h.conjugate(Future, P3, PL), "hablarán");
        assert_eq!(h.conjugate(Conditional, P1, SG), "hablaría");
        assert_eq!(h.conjugate(SubjunctivePresent, P1, SG), "hable");
        assert_eq!(h.conjugate(SubjunctiveImperfect, P1, PL), "habláramos");
        assert_eq!(h.conjugate(SubjunctiveFuture, P1, SG), "hablare");
        assert_eq!(
            h.variants(SubjunctiveImperfect, P1, SG),
            vec!["hablara", "hablase"]
        );
        assert_eq!(h.imperative(P2, SG).unwrap(), "habla");
        assert_eq!(h.imperative(P3, SG).unwrap(), "hable");
        assert_eq!(h.imperative(P2, PL).unwrap(), "hablad");
        assert_eq!(h.gerund(), "hablando");
        assert_eq!(h.past_participle(), "hablado");
        assert_eq!(h.past_participle_inflected(true, true), "habladas");
    }

    #[test]
    fn regular_er_ir() {
        let c = v("comer");
        assert_eq!(c.conjugate(Present, P1, PL), "comemos");
        assert_eq!(c.conjugate(Preterite, P3, PL), "comieron");
        assert_eq!(c.conjugate(SubjunctivePresent, P1, SG), "coma");
        assert_eq!(c.imperative(P2, SG).unwrap(), "come");
        assert_eq!(c.imperative(P2, PL).unwrap(), "comed");
        let vv = v("vivir");
        assert_eq!(vv.conjugate(Present, P1, PL), "vivimos");
        assert_eq!(vv.conjugate(Present, P2, PL), "vivís");
        assert_eq!(vv.conjugate(Preterite, P3, SG), "vivió");
        assert_eq!(vv.imperative(P2, PL).unwrap(), "vivid");
        assert_eq!(vv.gerund(), "viviendo");
    }

    #[test]
    fn spelling_rules() {
        assert_eq!(v("buscar").conjugate(Preterite, P1, SG), "busqué");
        assert_eq!(v("buscar").conjugate(SubjunctivePresent, P1, SG), "busque");
        assert_eq!(v("llegar").conjugate(Preterite, P1, SG), "llegué");
        assert_eq!(v("empezar").conjugate(Preterite, P1, SG), "empecé");
        assert_eq!(v("averiguar").conjugate(Preterite, P1, SG), "averigüé");
        assert_eq!(v("coger").conjugate(Present, P1, SG), "cojo");
        assert_eq!(v("dirigir").conjugate(SubjunctivePresent, P1, SG), "dirija");
        assert_eq!(v("vencer").conjugate(Present, P1, SG), "venzo");
        assert_eq!(v("conocer").conjugate(Present, P1, SG), "conozco");
        assert_eq!(
            v("distinguir").conjugate(SubjunctivePresent, P1, SG),
            "distinga"
        );
        assert_eq!(v("delinquir").conjugate(Present, P1, SG), "delinco");
    }

    #[test]
    fn stem_classes() {
        assert_eq!(v("pensar").conjugate(Present, P1, SG), "pienso");
        assert_eq!(v("pensar").conjugate(Present, P1, PL), "pensamos");
        assert_eq!(v("contar").conjugate(Present, P3, SG), "cuenta");
        assert_eq!(v("jugar").conjugate(SubjunctivePresent, P1, SG), "juegue");
        assert_eq!(v("oler").conjugate(Present, P3, SG), "huele");
        assert_eq!(v("errar").conjugate(Present, P1, SG), "yerro");
        assert_eq!(v("avergonzar").conjugate(Present, P1, SG), "avergüenzo");
        assert_eq!(v("pedir").conjugate(Present, P1, SG), "pido");
        assert_eq!(v("pedir").conjugate(Preterite, P3, SG), "pidió");
        assert_eq!(v("sentir").conjugate(Present, P1, SG), "siento");
        assert_eq!(v("sentir").conjugate(Preterite, P3, SG), "sintió");
        assert_eq!(v("sentir").gerund(), "sintiendo");
        assert_eq!(v("dormir").conjugate(Preterite, P3, PL), "durmieron");
        assert_eq!(
            v("dormir").conjugate(SubjunctivePresent, P1, PL),
            "durmamos"
        );
        assert_eq!(v("seguir").conjugate(Present, P1, SG), "sigo");
        assert_eq!(v("seguir").gerund(), "siguiendo");
        assert_eq!(v("discernir").conjugate(Present, P3, SG), "discierne");
        assert_eq!(v("discernir").conjugate(Preterite, P3, SG), "discernió");
        assert_eq!(v("enviar").conjugate(Present, P1, SG), "envío");
        assert_eq!(v("actuar").conjugate(Present, P3, SG), "actúa");
        assert_eq!(v("adquirir").conjugate(Present, P1, SG), "adquiero");
        assert_eq!(
            v("readquirir").conjugate(SubjunctiveImperfect, P1, SG),
            "readquiriera"
        );
    }

    #[test]
    fn irregular_lexicon() {
        assert_eq!(v("ser").conjugate(Present, P1, SG), "soy");
        assert_eq!(v("ser").conjugate(Imperfect, P1, SG), "era");
        assert_eq!(v("ser").conjugate(Preterite, P3, SG), "fue");
        assert_eq!(v("ser").conjugate(SubjunctiveImperfect, P1, SG), "fuera");
        assert_eq!(v("ir").gerund(), "yendo");
        assert_eq!(v("estar").conjugate(SubjunctivePresent, P1, SG), "esté");
        assert_eq!(v("tener").conjugate(Present, P1, SG), "tengo");
        assert_eq!(v("tener").conjugate(Preterite, P1, SG), "tuve");
        assert_eq!(v("tener").conjugate(Future, P1, SG), "tendré");
        assert_eq!(v("obtener").conjugate(Preterite, P3, SG), "obtuvo");
        assert_eq!(v("obtener").imperative(P2, SG).unwrap(), "obtén");
        assert_eq!(
            v("proponer").conjugate(SubjunctiveImperfect, P1, SG),
            "propusiera"
        );
        assert_eq!(v("hacer").conjugate(Preterite, P3, SG), "hizo");
        assert_eq!(v("hacer").imperative(P2, SG).unwrap(), "haz");
        assert_eq!(v("decir").conjugate(Preterite, P3, PL), "dijeron");
        assert_eq!(
            v("decir").conjugate(SubjunctiveImperfect, P3, PL),
            "dijeran"
        );
        assert_eq!(v("contradecir").imperative(P2, SG).unwrap(), "contradice");
        assert_eq!(v("conducir").conjugate(Preterite, P1, SG), "conduje");
        assert_eq!(v("andar").conjugate(Preterite, P1, SG), "anduve");
        assert_eq!(v("mandar").conjugate(Preterite, P1, SG), "mandé");
        assert_eq!(v("toser").conjugate(Present, P1, SG), "toso");
        assert_eq!(v("mover").conjugate(Preterite, P1, SG), "moví");
        assert_eq!(v("oír").conjugate(Present, P3, SG), "oye");
        assert_eq!(v("oír").conjugate(Future, P1, SG), "oiré");
        assert_eq!(v("reír").conjugate(Present, P1, SG), "río");
        assert_eq!(v("sonreír").conjugate(Preterite, P3, SG), "sonrió");
        assert_eq!(v("volver").past_participle(), "vuelto");
        assert_eq!(v("escribir").past_participle(), "escrito");
        assert_eq!(v("abrir").past_participle(), "abierto");
        assert_eq!(v("romper").past_participle(), "roto");
        assert_eq!(v("mecer").conjugate(Present, P1, SG), "mezo");
        assert_eq!(v("argüir").conjugate(Present, P1, SG), "arguyo");
        assert_eq!(v("argüir").conjugate(Present, P1, PL), "argüimos");
        assert_eq!(v("bullir").conjugate(Preterite, P3, SG), "bulló");
        assert_eq!(v("reñir").gerund(), "riñendo");
    }

    #[test]
    fn y_insertion_and_vowel_stems() {
        let c = v("construir");
        assert_eq!(c.conjugate(Present, P1, SG), "construyo");
        assert_eq!(c.conjugate(Present, P3, SG), "construye");
        assert_eq!(c.conjugate(Present, P1, PL), "construimos");
        assert_eq!(c.conjugate(Preterite, P3, SG), "construyó");
        assert_eq!(c.conjugate(SubjunctivePresent, P1, SG), "construya");
        assert_eq!(c.gerund(), "construyendo");
        let l = v("leer");
        assert_eq!(l.conjugate(Preterite, P3, SG), "leyó");
        assert_eq!(l.conjugate(Preterite, P2, SG), "leíste");
        assert_eq!(l.conjugate(SubjunctiveImperfect, P1, SG), "leyera");
        assert_eq!(l.gerund(), "leyendo");
        assert_eq!(l.past_participle(), "leído");
        let ca = v("caer");
        assert_eq!(ca.conjugate(Preterite, P3, PL), "cayeron");
        assert_eq!(ca.past_participle(), "caído");
    }
}
