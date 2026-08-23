//! Modern Standard Arabic verb conjugation.
//!
//! Arabic is a root-and-pattern (templatic) language cited by the unvoweled
//! consonantal skeleton of the 3rd-person-masculine-singular perfect
//! (كتب). The skeleton does not fix the vocalisation, so the paradigm cannot
//! be voweled from the lemma alone: the engine looks the lemma up in a mined
//! principal-parts table (`data/ara/parts.tsv`) that stores four fully-voweled
//! 3sg-masc cells —
//!     PA  perfect   active   (كَتَبَ)
//!     IA  imperfect active   indicative (يَكْتُبُ)
//!     PP  perfect   passive  (كُتِبَ)
//!     IP  imperfect passive  indicative (يُكْتَبُ)
//! — and derives every other cell by the regular person/number/gender/mood/
//! voice affixation plus weak-root morphophonemics. A small mined-override
//! layer (`data/ara/overrides.tsv`) patches the weak-root residue the rules do
//! not reach.
//!
//! Feature strings are the shared canonical form: `V;` followed by the
//! remaining tokens sorted alphabetically, e.g.
//!   perfect 3sg f active   → V;3;ACT;FEM;IND;PRF;PST;SG
//!   imperfect 2pl m jussive → V;2;ACT;JUS;MASC;PL

use std::collections::HashMap;
use std::sync::OnceLock;

/// Why a form cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The lemma is not in the mined principal-parts table.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Arabic verb we know")
    }
}

static PARTS_TSV: &str = include_str!("../data/ara/parts.tsv");
static OVERRIDES_TSV: &str = include_str!("../data/ara/overrides.tsv");

// Combining diacritics and letters we manipulate.
const FATHA: char = '\u{064E}';
const DAMMA: char = '\u{064F}';
const KASRA: char = '\u{0650}';
const SUKUN: char = '\u{0652}';
const SHADDA: char = '\u{0651}';
const ALIF: char = 'ا'; // U+0627
const ALIF_MAQ: char = 'ى'; // U+0649
const WAW: char = 'و'; // U+0648
const YA: char = 'ي'; // U+064A
const TA: char = 'ت';
const NUN: char = 'ن';
const MIM: char = 'م';
const HAMZA_ALIF: char = 'أ'; // U+0623

fn is_mark(c: char) -> bool {
    matches!(c, '\u{064B}'..='\u{0652}' | '\u{0670}')
}

fn is_short_vowel(c: char) -> bool {
    matches!(c, FATHA | DAMMA | KASRA)
}

fn is_mater(c: char) -> bool {
    matches!(c, ALIF | ALIF_MAQ | WAW | YA)
}

/// A base letter with its following combining marks.
type Unit = (char, Vec<char>);

fn units(s: &str) -> Vec<Unit> {
    let mut out: Vec<Unit> = Vec::new();
    for c in s.chars() {
        if is_mark(c) {
            if let Some(last) = out.last_mut() {
                last.1.push(c);
            }
        } else {
            out.push((c, Vec::new()));
        }
    }
    out
}

fn flatten(u: &[Unit]) -> Vec<char> {
    let mut out = Vec::new();
    for (b, marks) in u {
        out.push(*b);
        out.extend(marks.iter().copied());
    }
    out
}

/// The short vowel carried by a unit, if any.
fn unit_vowel(u: &Unit) -> Option<char> {
    u.1.iter().copied().find(|c| is_short_vowel(*c))
}

fn has_shadda(u: &Unit) -> bool {
    u.1.contains(&SHADDA)
}

/// The four principal parts of one lemma; "-" columns become None.
#[derive(Debug, Clone)]
pub struct Parts {
    pub pa: String,
    pub ia: String,
    pub pp: Option<String>,
    pub ip: Option<String>,
}

fn table() -> &'static HashMap<String, Parts> {
    static MAP: OnceLock<HashMap<String, Parts>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in PARTS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            if c.len() < 3 {
                continue;
            }
            let opt = |s: &str| {
                if s == "-" || s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            };
            m.insert(
                c[0].to_string(),
                Parts {
                    pa: c[1].to_string(),
                    ia: c[2].to_string(),
                    pp: c.get(3).and_then(|s| opt(s)),
                    ip: c.get(4).and_then(|s| opt(s)),
                },
            );
        }
        m
    })
}

/// Mined per-cell overrides: (lemma, features) -> form.
fn overrides() -> &'static HashMap<(String, String), String> {
    static MAP: OnceLock<HashMap<(String, String), String>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in OVERRIDES_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            if c.len() < 3 {
                continue;
            }
            m.insert((c[0].to_string(), c[1].to_string()), c[2].to_string());
        }
        m
    })
}

// --- feature parsing ---

#[derive(Clone, Copy, PartialEq, Eq)]
enum Num {
    Sg,
    Du,
    Pl,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Gen {
    Masc,
    Fem,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mood {
    Perf,
    Ind,
    Subj,
    Jus,
    Imp,
}

#[derive(Clone, Copy)]
struct Feat {
    person: u8,
    number: Num,
    gender: Gen,
    passive: bool,
    mood: Mood,
}

fn parse_feat(features: &str) -> Option<Feat> {
    let has = |t: &str| features.split(';').any(|x| x == t);
    if has("V.PTCP") || has("V.MSDR") {
        return None;
    }
    let person = if has("1") {
        1
    } else if has("2") {
        2
    } else {
        3
    };
    let number = if has("DU") {
        Num::Du
    } else if has("PL") {
        Num::Pl
    } else {
        Num::Sg
    };
    let gender = if has("FEM") { Gen::Fem } else { Gen::Masc };
    let passive = has("PASS");
    let mood = if has("IMP") {
        Mood::Imp
    } else if has("JUS") {
        Mood::Jus
    } else if has("SBJV") {
        Mood::Subj
    } else if has("PRF") {
        Mood::Perf
    } else {
        Mood::Ind
    };
    Some(Feat {
        person,
        number,
        gender,
        passive,
        mood,
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ECls {
    Sg,
    Fs,
    Du,
    Plm,
    Plf,
}

fn ending_class(f: &Feat) -> ECls {
    match f.number {
        Num::Du => ECls::Du,
        Num::Pl => {
            if f.person == 1 {
                ECls::Sg
            } else if f.gender == Gen::Fem {
                ECls::Plf
            } else {
                ECls::Plm
            }
        }
        Num::Sg => {
            if f.person == 2 && f.gender == Gen::Fem {
                ECls::Fs
            } else {
                ECls::Sg
            }
        }
    }
}

fn prefix_cons(f: &Feat) -> char {
    match f.person {
        1 => {
            if f.number == Num::Sg {
                HAMZA_ALIF
            } else {
                NUN
            }
        }
        2 => TA,
        _ => {
            if f.gender == Gen::Fem && f.number != Num::Pl {
                TA
            } else {
                YA
            }
        }
    }
}

// --- weak-root classification ---

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Sound,
    Hollow,
    Defective,
    Doubled,
}

/// The imperfect stem: the imperfect principal part minus its person prefix
/// (first unit) and its final short vowel.
fn imperf_stem(impf: &str) -> Vec<Unit> {
    let u = units(impf);
    let mut s: Vec<Unit> = u[1..].to_vec();
    if let Some(last) = s.last_mut() {
        if last.1.last().is_some_and(|c| is_short_vowel(*c)) {
            last.1.pop();
        }
    }
    s
}

fn classify(is_units: &[Unit]) -> Class {
    let n = is_units.len();
    if n == 0 {
        return Class::Sound;
    }
    let last = &is_units[n - 1];
    if has_shadda(last) {
        return Class::Doubled;
    }
    if is_mater(last.0) && n >= 2 {
        let pen = &is_units[n - 2];
        if !is_mater(pen.0) {
            return Class::Defective;
        }
    }
    // Hollow: a bare medial mater (a genuine long vowel) followed by a
    // consonant. A vowel-bearing و/ي (e.g. form III قَاوَمَ) stays consonantal.
    if n >= 3 {
        let med = &is_units[n - 2];
        if is_mater(med.0) && med.1.is_empty() && !is_mater(last.0) {
            return Class::Hollow;
        }
    }
    Class::Sound
}

// --- perfect suffixes (chars appended after the bare stem-final consonant) ---

fn perfect_suffix(f: &Feat) -> Vec<char> {
    match (f.person, f.number, f.gender) {
        (1, Num::Sg, _) => vec![SUKUN, TA, DAMMA],
        (1, _, _) => vec![SUKUN, NUN, FATHA, ALIF], // plural
        (2, Num::Sg, Gen::Masc) => vec![SUKUN, TA, FATHA],
        (2, Num::Sg, Gen::Fem) => vec![SUKUN, TA, KASRA],
        (2, Num::Du, _) => vec![SUKUN, TA, DAMMA, MIM, FATHA, ALIF],
        (2, Num::Pl, Gen::Masc) => vec![SUKUN, TA, DAMMA, MIM, SUKUN],
        (2, Num::Pl, Gen::Fem) => vec![SUKUN, TA, DAMMA, NUN, FATHA, SHADDA],
        (_, Num::Sg, Gen::Masc) => vec![FATHA],
        (_, Num::Sg, Gen::Fem) => vec![FATHA, TA, SUKUN],
        (_, Num::Du, Gen::Masc) => vec![FATHA, ALIF],
        (_, Num::Du, Gen::Fem) => vec![FATHA, TA, FATHA, ALIF],
        (_, Num::Pl, Gen::Masc) => vec![DAMMA, WAW, ALIF],
        (_, Num::Pl, Gen::Fem) => vec![SUKUN, NUN, FATHA],
    }
}

// --- imperfect endings (chars appended after the bare stem-final consonant) ---

fn sound_ending(f: &Feat, mood: Mood) -> Vec<char> {
    match ending_class(f) {
        ECls::Sg => match mood {
            Mood::Ind => vec![DAMMA],
            Mood::Subj => vec![FATHA],
            _ => vec![SUKUN],
        },
        ECls::Fs => match mood {
            Mood::Ind => vec![KASRA, YA, NUN, FATHA],
            _ => vec![KASRA, YA],
        },
        ECls::Du => match mood {
            Mood::Ind => vec![FATHA, ALIF, NUN, KASRA],
            _ => vec![FATHA, ALIF],
        },
        ECls::Plm => match mood {
            Mood::Ind => vec![DAMMA, WAW, NUN, FATHA],
            _ => vec![DAMMA, WAW, ALIF],
        },
        ECls::Plf => vec![SUKUN, NUN, FATHA],
    }
}

/// Defective imperfect endings, appended after C2 (the last stem consonant).
/// `sv` is the stem vowel type: KASRA (i-type), DAMMA (u-type), FATHA (a-type).
fn defective_ending(sv: char, f: &Feat, mood: Mood) -> Vec<char> {
    let cls = ending_class(f);
    let ind = mood == Mood::Ind;
    match sv {
        KASRA => match cls {
            ECls::Sg => match mood {
                Mood::Ind => vec![KASRA, YA],
                Mood::Subj => vec![KASRA, YA, FATHA],
                _ => vec![KASRA],
            },
            ECls::Fs => {
                if ind {
                    vec![KASRA, YA, NUN, FATHA]
                } else {
                    vec![KASRA, YA]
                }
            }
            ECls::Du => {
                if ind {
                    vec![KASRA, YA, FATHA, ALIF, NUN, KASRA]
                } else {
                    vec![KASRA, YA, FATHA, ALIF]
                }
            }
            ECls::Plm => {
                if ind {
                    vec![DAMMA, WAW, NUN, FATHA]
                } else {
                    vec![DAMMA, WAW, ALIF]
                }
            }
            ECls::Plf => vec![KASRA, YA, NUN, FATHA],
        },
        DAMMA => match cls {
            ECls::Sg => match mood {
                Mood::Ind => vec![DAMMA, WAW],
                Mood::Subj => vec![DAMMA, WAW, FATHA],
                _ => vec![DAMMA],
            },
            ECls::Fs => {
                if ind {
                    vec![KASRA, YA, NUN, FATHA]
                } else {
                    vec![KASRA, YA]
                }
            }
            ECls::Du => {
                if ind {
                    vec![DAMMA, WAW, FATHA, ALIF, NUN, KASRA]
                } else {
                    vec![DAMMA, WAW, FATHA, ALIF]
                }
            }
            ECls::Plm => {
                if ind {
                    vec![DAMMA, WAW, NUN, FATHA]
                } else {
                    vec![DAMMA, WAW, ALIF]
                }
            }
            ECls::Plf => vec![DAMMA, WAW, NUN, FATHA],
        },
        _ => match cls {
            // a-type (FATHA)
            ECls::Sg => match mood {
                Mood::Jus => vec![FATHA],
                _ => vec![FATHA, ALIF_MAQ],
            },
            ECls::Fs => {
                if ind {
                    vec![FATHA, YA, SUKUN, NUN, FATHA]
                } else {
                    vec![FATHA, YA, SUKUN]
                }
            }
            ECls::Du => {
                if ind {
                    vec![FATHA, YA, FATHA, ALIF, NUN, KASRA]
                } else {
                    vec![FATHA, YA, FATHA, ALIF]
                }
            }
            ECls::Plm => {
                if ind {
                    vec![FATHA, WAW, SUKUN, NUN, FATHA]
                } else {
                    vec![FATHA, WAW, SUKUN, ALIF]
                }
            }
            ECls::Plf => vec![FATHA, YA, SUKUN, NUN, FATHA],
        },
    }
}

/// Build a broken (de-geminated) stem: the units before the geminate, then
/// `gem`, a linking vowel `connv`, and `gem` again. The consonant just before
/// the geminate closes the preceding syllable (loses its vowel) when a vowel
/// precedes it — either an earlier vowelled consonant, or, for the first stem
/// consonant, the person prefix (`prefix_onset`).
fn broken_stem(pre: &[Unit], gem: char, connv: char, prefix_onset: bool) -> Vec<char> {
    let mut pre = pre.to_vec();
    let m = pre.len();
    if m >= 1 && unit_vowel(&pre[m - 1]).is_some() {
        let preceded = if m >= 2 {
            unit_vowel(&pre[m - 2]).is_some()
        } else {
            prefix_onset
        };
        if preceded {
            pre[m - 1].1 = vec![SUKUN];
        }
    }
    let mut out = flatten(&pre);
    out.extend([gem, connv, gem]);
    out
}

/// The medial mater of a hollow imperfect stem (قُول → و).
fn hollow_short_vowel(medial: char) -> char {
    match medial {
        WAW => DAMMA,
        YA => KASRA,
        _ => FATHA,
    }
}

/// The root weak consonant recovered from the imperfect final mater.
fn weak_consonant(is_units: &[Unit]) -> char {
    match is_units.last().map(|u| u.0) {
        Some(WAW) => WAW,
        _ => YA,
    }
}

/// When a final root consonant meets an identical suffix consonant across a
/// sukun (feminine-plural/1pl -na on a ن-root, the -t- perfect suffixes on a
/// ت-root), the pair merges into one shadda'd consonant: أَتْقَنْنَ → أَتْقَنَّ,
/// ثَبَتْتَ → ثَبَتَّ. The vowel is written before the shadda per the gold.
fn assimilate_geminate(chars: Vec<char>) -> Vec<char> {
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        if i + 3 < chars.len()
            && !is_mark(chars[i])
            && chars[i] == chars[i + 2]
            && chars[i + 1] == SUKUN
            && is_short_vowel(chars[i + 3])
        {
            out.push(chars[i]);
            out.push(chars[i + 3]);
            out.push(SHADDA);
            i += 4;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Adjust the seat of a hamza-on-alif that inflection has put next to a long
/// aa (→ madda آ) or a kasra (→ ya-seat ئ).
fn fix_hamza_seat(chars: Vec<char>) -> Vec<char> {
    const HAMZA_MADDA: char = '\u{0622}'; // آ
    const HAMZA_YA: char = '\u{0626}'; // ئ
    const HAMZA_WAW: char = '\u{0624}'; // ؤ
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == HAMZA_ALIF
            && i + 3 < chars.len()
            && chars[i + 1] == FATHA
            && chars[i + 2] == HAMZA_ALIF
            && chars[i + 3] == SUKUN
        {
            // hamza-fatha + hamza-sukun (أَأْ) coalesces to madda (آ): أَأْتِي → آتِي
            out.push(HAMZA_MADDA);
            i += 4;
        } else if chars[i] == HAMZA_ALIF
            && i + 2 < chars.len()
            && chars[i + 1] == FATHA
            && chars[i + 2] == ALIF
        {
            out.push(HAMZA_MADDA);
            i += 3;
        } else if matches!(chars[i], HAMZA_ALIF | HAMZA_WAW)
            && i > 0
            && chars.get(i + 1) == Some(&KASRA)
        {
            // a hamza next to a kasra takes the ya-seat (ئ)
            out.push(HAMZA_YA);
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// One Arabic verb, resolved to its principal parts.
pub struct Verb {
    parts: Parts,
    lemma: String,
}

impl Verb {
    /// Look a lemma (unvoweled 3sg-masc perfect skeleton) up in the mined
    /// principal-parts table.
    pub fn from_lemma(lemma: &str) -> Result<Self, Error> {
        table()
            .get(lemma)
            .map(|p| Verb {
                parts: p.clone(),
                lemma: lemma.to_string(),
            })
            .ok_or(Error::NotAVerb)
    }

    /// The voweled form for a canonical feature string, or None for a cell
    /// the engine does not generate.
    pub fn form(&self, features: &str) -> Option<String> {
        if let Some(o) = overrides().get(&(self.lemma.clone(), features.to_string())) {
            return Some(o.clone());
        }
        let f = parse_feat(features)?;
        let chars = fix_hamza_seat(assimilate_geminate(self.build(&f)?));
        Some(chars.into_iter().collect())
    }

    fn build(&self, f: &Feat) -> Option<Vec<char>> {
        // pick the voice's principal parts
        let (perf, impf) = if f.passive {
            (self.parts.pp.as_deref()?, self.parts.ip.as_deref()?)
        } else {
            (self.parts.pa.as_str(), self.parts.ia.as_str())
        };
        let is_units = imperf_stem(impf);
        let class = classify(&is_units);
        let pv = unit_vowel(&units(impf)[0]).unwrap_or(FATHA);

        match f.mood {
            Mood::Perf => Some(self.perfect(f, class, perf, &is_units)),
            Mood::Imp => {
                if f.passive {
                    return None;
                }
                self.imperative(f, class, impf, &is_units, pv)
            }
            _ => Some(self.imperfect(f, class, &is_units, pv, f.mood)),
        }
    }

    fn perfect(&self, f: &Feat, class: Class, perf: &str, is_units: &[Unit]) -> Vec<char> {
        let suf = perfect_suffix(f);
        let cons_initial = suf.first() == Some(&SUKUN);
        let pu = units(perf);
        match class {
            Class::Sound => {
                let mut stem = flatten(&pu);
                if stem.last().is_some_and(|c| is_short_vowel(*c)) {
                    stem.pop();
                }
                stem.extend(suf);
                stem
            }
            Class::Hollow => {
                let pn = pu.len();
                let c3 = pu[pn - 1].0;
                if cons_initial {
                    // shorten the long medial vowel before a consonant suffix
                    let sv = if f.passive {
                        KASRA
                    } else if pn > 3 {
                        // derived forms shorten to fatha (احتاج → اِحْتَجْتُ)
                        FATHA
                    } else {
                        // form I: the historic root vowel, read from the imperfect
                        match is_units.get(is_units.len().wrapping_sub(2)).map(|u| u.0) {
                            Some(WAW) => DAMMA,
                            _ => KASRA,
                        }
                    };
                    let mut stem = pu[..pn - 2].to_vec();
                    if let Some(last) = stem.last_mut() {
                        last.1 = vec![sv];
                    }
                    let mut out = flatten(&stem);
                    out.push(c3);
                    out.extend(suf);
                    out
                } else {
                    let mut stem = flatten(&pu);
                    if stem.last().is_some_and(|c| is_short_vowel(*c)) {
                        stem.pop();
                    }
                    stem.extend(suf);
                    stem
                }
            }
            Class::Doubled => {
                let n = pu.len();
                let gem = pu[n - 1].0;
                let gemv = unit_vowel(&pu[n - 1]).unwrap_or(FATHA);
                if cons_initial {
                    // broken: the geminate splits, joined by a linking vowel
                    // (the theme vowel in the active, kasra in the passive).
                    let connv = if f.passive { KASRA } else { gemv };
                    let mut out = broken_stem(&pu[..n - 1], gem, connv, false);
                    out.extend(suf);
                    out
                } else {
                    // geminate: consonant carries the ending vowel then shadda
                    let mut out = flatten(&pu[..n - 1]);
                    out.extend([gem, suf[0], SHADDA]);
                    out.extend_from_slice(&suf[1..]);
                    out
                }
            }
            Class::Defective => self.perfect_defective(f, &pu, perf, is_units, &suf),
        }
    }

    fn perfect_defective(
        &self,
        f: &Feat,
        pu: &[Unit],
        perf: &str,
        is_units: &[Unit],
        suf: &[char],
    ) -> Vec<char> {
        let n = pu.len();
        let c2u = &pu[n - 2];
        let c2 = c2u.0;
        let shad = has_shadda(c2u);
        let v2 = unit_vowel(c2u).unwrap_or(FATHA);
        let w = weak_consonant(is_units);
        let is_3 = f.person == 3;
        // 3sg-masc is the identity (the perfect principal part itself).
        if is_3 && f.number == Num::Sg && f.gender == Gen::Masc {
            return perf.chars().collect();
        }
        let suf_ns: Vec<char> = if suf.first() == Some(&SUKUN) {
            suf[1..].to_vec()
        } else {
            suf.to_vec()
        };
        // Each cell is (vowel on C2, chars following C2's vowel).
        let (cvowel, rest): (char, Vec<char>) = if v2 == KASRA {
            // a-i type (نسي / passive رُمِي)
            let mut r;
            match (is_3, f.number, f.gender) {
                (true, Num::Sg, Gen::Fem) | (true, Num::Du, Gen::Fem) => {
                    r = vec![YA];
                    r.extend_from_slice(suf);
                }
                (true, Num::Du, Gen::Masc) => r = vec![YA, FATHA, ALIF],
                (true, Num::Pl, Gen::Masc) => {
                    return {
                        let mut o = flatten(&pu[..n - 2]);
                        o.push(c2);
                        o.push(DAMMA);
                        if shad {
                            o.push(SHADDA);
                        }
                        o.extend([WAW, ALIF]);
                        o
                    }
                }
                (true, Num::Pl, Gen::Fem) => r = vec![YA, NUN, FATHA],
                _ => {
                    r = vec![YA];
                    r.extend_from_slice(&suf_ns);
                }
            }
            (KASRA, r)
        } else {
            // a-a type (رمى / دعا)
            match (is_3, f.number, f.gender) {
                (true, Num::Sg, Gen::Fem) | (true, Num::Du, Gen::Fem) => {
                    (suf[0], suf[1..].to_vec())
                }
                (true, Num::Du, Gen::Masc) => (FATHA, vec![w, FATHA, ALIF]),
                (true, Num::Pl, Gen::Masc) => (FATHA, vec![WAW, SUKUN, ALIF]),
                (true, Num::Pl, Gen::Fem) => (FATHA, vec![w, SUKUN, NUN, FATHA]),
                _ => {
                    let mut r = vec![w];
                    r.extend_from_slice(suf);
                    (FATHA, r)
                }
            }
        };
        let mut out = flatten(&pu[..n - 2]);
        out.push(c2);
        out.push(cvowel);
        if shad {
            out.push(SHADDA);
        }
        out.extend(rest);
        out
    }

    fn imperfect(
        &self,
        f: &Feat,
        class: Class,
        is_units: &[Unit],
        pv: char,
        mood: Mood,
    ) -> Vec<char> {
        let mut out = vec![prefix_cons(f), pv];
        match class {
            Class::Sound => {
                out.extend(flatten(is_units));
                out.extend(sound_ending(f, mood));
            }
            Class::Hollow => {
                let n = is_units.len();
                let end = sound_ending(f, mood);
                let medial = is_units[n - 2].0;
                let c3 = is_units[n - 1].0;
                if end.first() == Some(&SUKUN) {
                    // jussive SG and plural-feminine shorten the long vowel
                    let mut stem = is_units[..n - 2].to_vec();
                    if let Some(last) = stem.last_mut() {
                        last.1 = vec![hollow_short_vowel(medial)];
                    }
                    out.extend(flatten(&stem));
                    out.push(c3);
                    out.extend(end);
                } else {
                    out.extend(flatten(is_units));
                    out.extend(end);
                }
            }
            Class::Defective => {
                let n = is_units.len();
                out.extend(flatten(&is_units[..n - 2]));
                let c2u = &is_units[n - 2];
                let sv = unit_vowel(c2u).unwrap_or(FATHA);
                let end = defective_ending(sv, f, mood);
                out.push(c2u.0);
                // C2 carries its vowel (from the ending) then a shadda if geminate
                out.push(end[0]);
                if has_shadda(c2u) {
                    out.push(SHADDA);
                }
                out.extend_from_slice(&end[1..]);
            }
            Class::Doubled => {
                let n = is_units.len();
                let gem = is_units[n - 1].0;
                let cls = ending_class(f);
                if cls == ECls::Plf {
                    // broken geminate before the feminine-plural -na ending.
                    // The linking vowel is fatha in the passive; in the active
                    // it is the theme vowel for form I, kasra for derived forms.
                    let connv = if f.passive {
                        FATHA
                    } else if n == 2 {
                        unit_vowel(&is_units[0]).unwrap_or(DAMMA)
                    } else {
                        KASRA
                    };
                    out.extend(broken_stem(&is_units[..n - 1], gem, connv, true));
                    out.extend([SUKUN, NUN, FATHA]);
                } else {
                    out.extend(flatten(&is_units[..n - 1]));
                    // assimilated jussive takes -a (fatha) rather than sukun
                    let end = if cls == ECls::Sg && mood == Mood::Jus {
                        vec![FATHA]
                    } else {
                        sound_ending(f, mood)
                    };
                    out.extend([gem, end[0], SHADDA]);
                    out.extend_from_slice(&end[1..]);
                }
            }
        }
        out
    }

    fn imperative(
        &self,
        f: &Feat,
        class: Class,
        impf: &str,
        is_units: &[Unit],
        pv: char,
    ) -> Option<Vec<char>> {
        let jus = self.imperfect(f, class, is_units, pv, Mood::Jus);
        // strip the person prefix (first unit: ت + its marks)
        let ju = units(&jus.into_iter().collect::<String>());
        if ju.is_empty() {
            return None;
        }
        let mut rest = flatten(&ju[1..]);
        let rest_units = &ju[1..];
        let starts_cluster = rest_units
            .first()
            .is_some_and(|u| u.1.contains(&SUKUN) || u.1.contains(&SHADDA));
        // Form IV takes a fixed hamzat-qaṭ' onset أَ. A geminating shadda in the
        // stem marks form II (أَيَّدَ), which is not form IV.
        let form_iv = self.parts.pa.starts_with(HAMZA_ALIF)
            && pv == DAMMA
            && !is_units.iter().any(has_shadda);
        if form_iv {
            let mut out = vec![HAMZA_ALIF, FATHA];
            out.extend(rest);
            return Some(out);
        }
        if !starts_cluster {
            // a hamza that carried a waw/ya seat medially (ؤَيِّد after damma)
            // moves to an initial alif seat (أَيِّد) when the prefix is dropped.
            if matches!(rest.first(), Some('\u{0624}') | Some('\u{0626}')) {
                rest[0] = HAMZA_ALIF;
            }
            return Some(rest);
        }
        // needs a prosthetic connective alif: damma if the stem's theme vowel
        // (the first short vowel of the imperfect stem) is damma, else kasra.
        let stem_vowel = is_units.iter().filter_map(unit_vowel).next();
        let onset = if stem_vowel == Some(DAMMA) {
            DAMMA
        } else {
            KASRA
        };
        let mut out = vec![ALIF, onset];
        let _ = impf;
        out.extend(rest);
        Some(out)
    }
}

/// A compact conjugation table — the finite paradigm grouped by mood and
/// voice, shared by the WebAssembly and Python bindings. Each vector is in
/// the fixed order 1sg · 2sg m · 2sg f · 3sg m · 3sg f · 2du · 3du m · 3du f
/// · 1pl · 2pl m · 2pl f · 3pl m · 3pl f (the imperative keeps only its five
/// 2nd-person cells: 2sg m · 2sg f · 2du · 2pl m · 2pl f). The active and
/// passive halves share that layout. Participles and the verbal noun are
/// nominal and left out.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// The unvoweled citation (3sg-masc perfect skeleton).
    pub lemma: String,
    pub perfect: Vec<Option<String>>,
    pub imperfect: Vec<Option<String>>,
    pub subjunctive: Vec<Option<String>>,
    pub jussive: Vec<Option<String>>,
    pub imperative: Vec<Option<String>>,
    pub perfect_passive: Vec<Option<String>>,
    pub imperfect_passive: Vec<Option<String>>,
    pub subjunctive_passive: Vec<Option<String>>,
    pub jussive_passive: Vec<Option<String>>,
}

/// (person, number, gender) in the traditional paradigm order; gender is ""
/// for the 1st person and the 2nd-person dual, which carry none.
const PNG: [(u8, &str, &str); 13] = [
    (1, "SG", ""),
    (2, "SG", "MASC"),
    (2, "SG", "FEM"),
    (3, "SG", "MASC"),
    (3, "SG", "FEM"),
    (2, "DU", ""),
    (3, "DU", "MASC"),
    (3, "DU", "FEM"),
    (1, "PL", ""),
    (2, "PL", "MASC"),
    (2, "PL", "FEM"),
    (3, "PL", "MASC"),
    (3, "PL", "FEM"),
];

const IMP_PNG: [(u8, &str, &str); 5] = [
    (2, "SG", "MASC"),
    (2, "SG", "FEM"),
    (2, "DU", ""),
    (2, "PL", "MASC"),
    (2, "PL", "FEM"),
];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        // Build the canonical (sorted-token) feature string for a cell and
        // ask the engine for it.
        let cell = |p: u8, num: &str, gen: &str, mood: &[&str], passive: bool| -> Option<String> {
            let mut toks: Vec<String> = vec![p.to_string(), num.to_string()];
            if !gen.is_empty() {
                toks.push(gen.to_string());
            }
            for m in mood {
                toks.push((*m).to_string());
            }
            toks.push(if passive { "PASS" } else { "ACT" }.to_string());
            toks.sort();
            v.form(&format!("V;{}", toks.join(";")))
        };
        let group = |mood: &[&str], passive: bool| -> Vec<Option<String>> {
            PNG.iter()
                .map(|(p, n, g)| cell(*p, n, g, mood, passive))
                .collect()
        };
        let imp = |passive: bool| -> Vec<Option<String>> {
            IMP_PNG
                .iter()
                .map(|(p, n, g)| cell(*p, n, g, &["IMP"], passive))
                .collect()
        };
        Self {
            lemma: v.lemma.clone(),
            perfect: group(&["PRF", "PST", "IND"], false),
            imperfect: group(&["IPFV", "IND"], false),
            subjunctive: group(&["SBJV"], false),
            jussive: group(&["JUS"], false),
            imperative: imp(false),
            perfect_passive: group(&["PRF", "PST", "IND"], true),
            imperfect_passive: group(&["IPFV", "IND"], true),
            subjunctive_passive: group(&["SBJV"], true),
            jussive_passive: group(&["JUS"], true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(lemma: &str, feat: &str) -> String {
        Verb::from_lemma(lemma).unwrap().form(feat).unwrap()
    }

    #[test]
    fn sound_jalasa() {
        // جلس is the form-I sound verb جَلَسَ / يَجْلِسُ in the table.
        assert_eq!(f("جلس", "V;3;ACT;IND;MASC;PRF;PST;SG"), "جَلَسَ");
        assert_eq!(f("جلس", "V;1;ACT;IND;PRF;PST;SG"), "جَلَسْتُ");
        assert_eq!(f("جلس", "V;3;ACT;IND;IPFV;MASC;SG"), "يَجْلِسُ");
        assert_eq!(f("جلس", "V;3;ACT;JUS;MASC;SG"), "يَجْلِسْ");
        assert_eq!(f("جلس", "V;2;ACT;IMP;MASC;SG"), "اِجْلِسْ");
    }

    #[test]
    fn hollow_qala() {
        assert_eq!(f("قال", "V;3;ACT;IND;MASC;PRF;PST;SG"), "قَالَ");
        assert_eq!(f("قال", "V;1;ACT;IND;PRF;PST;SG"), "قُلْتُ");
        assert_eq!(f("قال", "V;3;ACT;IND;IPFV;MASC;SG"), "يَقُولُ");
        assert_eq!(f("قال", "V;3;ACT;JUS;MASC;SG"), "يَقُلْ");
        assert_eq!(f("قال", "V;2;ACT;IMP;MASC;SG"), "قُلْ");
    }

    #[test]
    fn defective_rama() {
        assert_eq!(f("رمى", "V;3;ACT;IND;MASC;PRF;PST;SG"), "رَمَى");
        assert_eq!(f("رمى", "V;3;ACT;FEM;IND;PRF;PST;SG"), "رَمَتْ");
        assert_eq!(f("رمى", "V;3;ACT;IND;MASC;PL;PRF;PST"), "رَمَوْا");
        assert_eq!(f("رمى", "V;3;ACT;IND;IPFV;MASC;SG"), "يَرْمِي");
        assert_eq!(f("رمى", "V;3;ACT;JUS;MASC;SG"), "يَرْمِ");
        assert_eq!(f("رمى", "V;2;ACT;IMP;MASC;SG"), "اِرْمِ");
    }

    #[test]
    fn form_ii_kallama() {
        assert_eq!(f("كلم", "V;3;ACT;IND;MASC;PRF;PST;SG"), "كَلَّمَ");
        assert_eq!(f("كلم", "V;1;ACT;IND;PRF;PST;SG"), "كَلَّمْتُ");
        assert_eq!(f("كلم", "V;3;ACT;IND;IPFV;MASC;SG"), "يُكَلِّمُ");
        assert_eq!(f("كلم", "V;2;ACT;IMP;MASC;SG"), "كَلِّمْ");
    }
}
