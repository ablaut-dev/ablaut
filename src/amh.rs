//! Amharic (አማርኛ) verb conjugation, Ge'ez / Ethiopic script.
//!
//! Amharic is Ethiosemitic: verbs are built on consonantal roots realised
//! through a fixed skeleton of vowel "orders" (the Ethiopic abugida encodes a
//! consonant + one of seven vowels per codepoint). The lemma is the 3sg-masc
//! perfective (ሰበረ). The engine resolves it to a handful of mined principal
//! parts (`data/amh/parts.tsv`: perfective, imperfective, perfect,
//! imperfective-nonfinite and 2sg-masc imperative 3sg-masc stems) and derives
//! the paradigm by the regular subject-agreement affixation of each TAM:
//!
//!   * perfective — suffixes on the perfective stem (final radical revocalised)
//!   * imperfective (main + bare NFIN) — a person prefix + the imperfective
//!     stem + agreement, with the compound `-al` auxiliary for the main form
//!   * perfect — the reduced perfective base + the possessive-style auxiliary
//!   * imperative / jussive — the imperative stem, prefixed for 1st/3rd person
//!
//! A mined-override layer (`data/amh/overrides.tsv`) patches the irregular
//! residue the rules do not reach, and seeds the handful of lemmas that lack
//! principal parts (derived stems, the copula ነው).
//!
//! Feature strings are `V;` + tokens sorted: person 1/2/3, number SG/PL,
//! gender MASC/FEM, TAM PFV/PRF/PRS/IPFV/IMP (+ NFIN, LGSPEC1).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Why a form cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The lemma is not in the mined principal-parts table.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Amharic verb we know")
    }
}

static PARTS_TSV: &str = include_str!("../data/amh/parts.tsv");
static OVERRIDES_TSV: &str = include_str!("../data/amh/overrides.tsv");

// --- Ge'ez / Ethiopic syllable arithmetic ---
//
// The main Ethiopic block (U+1200..U+137F) lays out each consonant as a row of
// eight syllables: order 0 = ä (1st), 1 = u (2nd), 2 = i (3rd), 3 = a (4th),
// 4 = e (5th), 5 = ə/∅ (6th), 6 = o (7th), 7 = ʷa (8th). Changing a stem vowel
// is base-row + new order; appending agreement is pushing new syllables.

const A1: u8 = 0; // ä  (1st order)
const U: u8 = 1; // u  (2nd)
const I: u8 = 2; // i  (3rd)
const A: u8 = 3; // a  (4th)
const E: u8 = 4; // e  (5th)
const EPS: u8 = 5; // ə/∅ (6th)

const GEEZ_LO: u32 = 0x1200;
const GEEZ_HI: u32 = 0x1380;

// Order-0 (bare ä) base consonants used to build affixes.
const B_ALEF: char = '\u{12A0}'; // አ  glottal (ʔ) row; order 0 already reads "a"
const B_NUN: char = '\u{1290}'; // ነ
const B_TAW: char = '\u{1270}'; // ተ
const B_YOD: char = '\u{12E8}'; // የ
const B_LAM: char = '\u{1208}'; // ለ

/// The (row-start, order) of a Ge'ez syllable, or None if outside the block.
fn syllable(c: char) -> Option<(u32, u8)> {
    let u = c as u32;
    if (GEEZ_LO..GEEZ_HI).contains(&u) {
        let idx = u - GEEZ_LO;
        #[allow(clippy::cast_possible_truncation)]
        Some((GEEZ_LO + (idx / 8) * 8, (idx % 8) as u8))
    } else {
        None
    }
}

/// Change a syllable's vowel order (leaving non-Ethiopic chars untouched).
fn set_order(c: char, order: u8) -> char {
    match syllable(c) {
        Some((base, _)) => char::from_u32(base + u32::from(order)).unwrap_or(c),
        None => c,
    }
}

/// Build a syllable from an order-0 base consonant at a given order.
fn cons(base: char, order: u8) -> char {
    set_order(base, order)
}

/// Revocalise the last syllable of a stem to a given order.
fn revowel_last(stem: &[char], order: u8) -> Vec<char> {
    let mut v = stem.to_vec();
    if let Some(last) = v.last_mut() {
        *last = set_order(*last, order);
    }
    v
}

/// Revocalise the last syllable to the 2sg-fem -i, palatalising a coronal
/// (ሂድ → ሂጂ, ታወጣ → ታወጪ; a non-coronal like r is unchanged). With `spread`, a
/// final -l palatalises to the glide -yi and pushes the i onto a preceding ə
/// radical (ወክል → ወኪዪ, imperative only), leaving a full-vowel radical alone.
fn revowel_last_i(stem: &[char], spread: bool) -> Vec<char> {
    let mut v = stem.to_vec();
    let n = v.len();
    if let Some(last) = v.last().copied() {
        if let Some((row, _)) = syllable(last) {
            if let Some(pal) = palatal_row(char::from_u32(row).unwrap_or(last)) {
                if spread && row == 0x1208 && n >= 2 && matches!(syllable(v[n - 2]), Some((_, EPS)))
                {
                    v[n - 2] = set_order(v[n - 2], I);
                }
                v[n - 1] = set_order(pal, I);
                return v;
            }
        }
        v[n - 1] = set_order(last, I);
    }
    v
}

/// True if the stem's final radical is (or can be palatalised to) a palatal or
/// glide — such a stem takes the fused 2sg-fem -a ending, not the linking -ya.
fn final_palatalish(stem: &[char]) -> bool {
    matches!(
        stem.last().and_then(|c| syllable(*c)),
        Some((r, _)) if palatal_row(char::from_u32(r).unwrap_or(' ')).is_some()
            || matches!(r, 0x1278 | 0x1298 | 0x1300 | 0x1328 | 0x1238 | 0x12E0 | 0x12E8)
    )
}

/// Palatalise the stem's final radical if it can (else unchanged), keeping its
/// vowel order — used to build the fused 2sg-fem -a form (ረጭ → ረጫ after -a).
fn palatalize_last(stem: &[char]) -> Vec<char> {
    let mut v = stem.to_vec();
    if let Some(last) = v.last_mut() {
        if let Some((row, ord)) = syllable(*last) {
            if let Some(p) = palatal_row(char::from_u32(row).unwrap_or(*last)) {
                *last = set_order(p, ord);
            }
        }
    }
    v
}

/// True if the stem's final syllable carries a full vowel (order a) — a
/// vowel-final root that takes the -h/-hu subject allomorphs.
fn vowel_final(stem: &[char]) -> bool {
    matches!(stem.last().and_then(|c| syllable(*c)), Some((_, A)))
}

/// True if the stem takes the -h/-hu perfective allomorph rather than -k/-ku:
/// a vowel-final root, or a root whose final radical is a velar/uvular or
/// palatal, where k would dissimilate (ወደቀ → ወደቅህ, ማረከ → ማረክህ).
fn takes_h(stem: &[char]) -> bool {
    if vowel_final(stem) {
        return true;
    }
    matches!(
        stem.last().and_then(|c| syllable(*c)),
        Some((
            0x1240 | 0x1250 | 0x12A8 | 0x12B8 | 0x1308 | 0x1318 // q qʼ k x g gʼ
                | 0x1278 | 0x1298 | 0x1300 | 0x1328 | 0x1238 | 0x12E0 | 0x12E8, // č ñ j č̣ š ž y
            _
        ))
    )
}

/// The palatal counterpart of an order-0 coronal (for the 1sg perfect and the
/// 2sg-fem imperative of some classes). None = does not palatalise.
fn palatal_row(base: char) -> Option<char> {
    let (row, _) = syllable(base)?;
    let p = match row {
        0x1270 => 0x1278,          // t  → č
        0x12F0 => 0x1300,          // d  → ǧ
        0x1220 | 0x1230 => 0x1238, // s / ś → š
        0x1290 => 0x1298,          // n  → ñ
        0x12D8 => 0x12E0,          // z  → ž
        0x1320 => 0x1328,          // ṭ  → č̣
        0x1338 | 0x1340 => 0x1328, // ṣ / ṣ́ → č̣
        0x1208 => 0x12E8,          // l  → y
        _ => return None,
    };
    char::from_u32(p)
}

/// The plain velar row for a labiovelar row (the labialised velar surfaces in
/// the 3sg-masc perfect; the recovered base needs the plain radical the other
/// cells use: ቈ/ቋ → ቀ, ጐ → ገ). Returns None for non-labiovelar rows.
fn delabialize_row(row: u32) -> Option<u32> {
    match row {
        0x1248 => Some(0x1240), // ቈ → ቀ
        0x1288 => Some(0x12B8), // ኈ → ኸ
        0x12B0 => Some(0x12A8), // ኰ → ከ
        0x1310 => Some(0x1308), // ጐ → ገ
        _ => None,
    }
}

/// Revocalise the exposed final radical of a recovered perfect base to ä,
/// mapping any labialised velar back to its plain row.
fn base_radical(c: char) -> char {
    match syllable(c) {
        Some((row, _)) => {
            let base = delabialize_row(row).unwrap_or(row);
            char::from_u32(base + u32::from(A1)).unwrap_or(c)
        }
        None => c,
    }
}

/// The mined principal parts of one lemma; "-" columns become None.
#[derive(Debug, Clone)]
pub struct Parts {
    pub pfv: String,
    pub ipfv: Option<String>,
    pub prf: Option<String>,
    pub prs: Option<String>,
    pub ipfvn: Option<String>,
    pub imp: Option<String>,
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
            if c.len() < 2 {
                continue;
            }
            let opt = |i: usize| {
                c.get(i)
                    .and_then(|s| (*s != "-" && !s.is_empty()).then(|| s.to_string()))
            };
            m.insert(
                c[0].to_string(),
                Parts {
                    pfv: c[1].to_string(),
                    ipfv: opt(2),
                    prf: opt(3),
                    prs: opt(4),
                    ipfvn: opt(5),
                    imp: opt(6),
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

/// Lemmas that have at least one override entry (so `from_lemma` can treat an
/// override-only lemma — one lacking principal parts — as supported).
fn override_lemmas() -> &'static HashSet<String> {
    static SET: OnceLock<HashSet<String>> = OnceLock::new();
    SET.get_or_init(|| overrides().keys().map(|(l, _)| l.clone()).collect())
}

// --- feature parsing ---

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tam {
    Pfv,
    Prf,
    Prs,
    Ipfv,
    Imp,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Num {
    Sg,
    Pl,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Gen {
    Masc,
    Fem,
    None,
}

#[derive(Clone, Copy)]
struct Feat {
    tam: Tam,
    person: u8, // 0 = unspecified (present participle)
    number: Num,
    gender: Gen,
    nfin: bool,
}

fn parse_feat(features: &str) -> Option<Feat> {
    let has = |t: &str| features.split(';').any(|x| x == t);
    let tam = if has("IMP") {
        Tam::Imp
    } else if has("PFV") {
        Tam::Pfv
    } else if has("PRF") {
        Tam::Prf
    } else if has("PRS") {
        Tam::Prs
    } else if has("IPFV") {
        Tam::Ipfv
    } else {
        return None;
    };
    let person = if has("1") {
        1
    } else if has("2") {
        2
    } else if has("3") {
        3
    } else {
        0
    };
    let number = if has("PL") { Num::Pl } else { Num::Sg };
    let gender = if has("FEM") {
        Gen::Fem
    } else if has("MASC") {
        Gen::Masc
    } else {
        Gen::None
    };
    Some(Feat {
        tam,
        person,
        number,
        gender,
        nfin: has("NFIN"),
    })
}

/// One Amharic verb, resolved to its principal parts.
pub struct Verb {
    parts: Parts,
    lemma: String,
}

impl Verb {
    /// Look a lemma (3sg-masc perfective) up in the mined table, or accept an
    /// override-only lemma (one seeded entirely from the override layer).
    pub fn from_lemma(lemma: &str) -> Result<Self, Error> {
        if let Some(p) = table().get(lemma) {
            return Ok(Verb {
                parts: p.clone(),
                lemma: lemma.to_string(),
            });
        }
        if override_lemmas().contains(lemma) {
            return Ok(Verb {
                parts: Parts {
                    pfv: lemma.to_string(),
                    ipfv: None,
                    prf: None,
                    prs: None,
                    ipfvn: None,
                    imp: None,
                },
                lemma: lemma.to_string(),
            });
        }
        Err(Error::NotAVerb)
    }

    /// The form for a canonical feature string, or None for a cell the engine
    /// does not generate.
    pub fn form(&self, features: &str) -> Option<String> {
        if let Some(o) = overrides().get(&(self.lemma.clone(), features.to_string())) {
            return Some(o.clone());
        }
        let f = parse_feat(features)?;
        let chars = match f.tam {
            Tam::Pfv => self.perfective(&f),
            Tam::Prf => self.perfect(&f),
            Tam::Ipfv => self.imperfective(&f),
            Tam::Imp => self.imperative(&f),
            Tam::Prs => None,
        }?;
        Some(chars.into_iter().collect())
    }

    // --- perfective (suffixes on the perfective stem) ---

    fn perfective(&self, f: &Feat) -> Option<Vec<char>> {
        let stem: Vec<char> = self.parts.pfv.chars().collect();
        // A vowel-final root keeps its vowel and takes the -h/-hu allomorphs;
        // a consonant-final root revocalises the final radical to ə and takes
        // the -k/-ku allomorphs.
        let vf = vowel_final(&stem);
        let h = takes_h(&stem);
        // A palatal or glide final radical keeps its vowel before a consonant
        // suffix (ቋጨ → ቋጨሽ); a plain radical reduces to ə (ሰበረ → ሰበርሽ).
        let keep = vf
            || matches!(
                stem.last().and_then(|c| syllable(*c)),
                Some((
                    0x1278 | 0x1298 | 0x1300 | 0x1328 | 0x1238 | 0x12E0 | 0x12E8, // č ñ j č̣ š ž y
                    _
                ))
            );
        let cbase = if keep {
            stem.clone()
        } else {
            revowel_last(&stem, EPS)
        };
        let png = (f.person, f.number, f.gender);
        let out = match png {
            (3, Num::Sg, Gen::Masc) => stem,
            (3, Num::Sg, Gen::Fem) => push(&stem, &['\u{127D}']), // -äčč
            (3, Num::Pl, _) => revowel_last(&stem, U),            // -u
            (2, Num::Sg, Gen::Masc) => push(&cbase, &[if h { '\u{1205}' } else { '\u{12AD}' }]), // -h / -k
            (2, Num::Sg, Gen::Fem) => push(&cbase, &['\u{123D}']), // -š
            (2, Num::Pl, _) => push(&revowel_last(&stem, A), &['\u{127D}', '\u{1201}']), // -aččhu
            (1, Num::Sg, _) => push(&cbase, &[if h { '\u{1201}' } else { '\u{12A9}' }]), // -hu / -ku
            (1, Num::Pl, _) => push(&cbase, &['\u{1295}']),                              // -n
            _ => return None,
        };
        Some(out)
    }

    // --- perfect (reduced base + possessive-style auxiliary) ---

    /// The perfect (converb) base, recovered from the mined 3sg-masc perfect by
    /// dropping the final -al auxiliary and revocalising the exposed radical to
    /// ä: ሰብሯል → ሰብረ, አውጥቷል → አውጥተ, ገምግሟል → ገምግመ.
    fn perfect_base(&self) -> Option<Vec<char>> {
        let prf: Vec<char> = self.parts.prf.as_ref()?.chars().collect();
        if prf.len() < 2 {
            return None;
        }
        let mut v = prf[..prf.len() - 1].to_vec();
        let n = v.len();
        v[n - 1] = base_radical(v[n - 1]);
        Some(v)
    }

    fn perfect(&self, f: &Feat) -> Option<Vec<char>> {
        let png = (f.person, f.number, f.gender);
        if png == (3, Num::Sg, Gen::Masc) {
            return Some(self.parts.prf.as_ref()?.chars().collect());
        }
        let base = self.perfect_base()?;
        let out = match png {
            (3, Num::Sg, Gen::Fem) => push(&revowel_last(&base, A), &['\u{1208}', '\u{127D}']), // -aläčč
            (3, Num::Pl, _) => push(&base, &['\u{12CB}', '\u{120D}']), // -wal
            (2, Num::Sg, Gen::Masc) => push(&base, &['\u{1200}', '\u{120D}']), // -hal
            (2, Num::Sg, Gen::Fem) => push(&base, &['\u{123B}', '\u{120D}']), // -šal
            (2, Num::Pl, _) => push(
                &revowel_last(&base, A),
                &['\u{127D}', '\u{128B}', '\u{120D}'],
            ), // -ačč(h)wal
            (1, Num::Sg, _) => self.perfect_1sg(&base),
            (1, Num::Pl, _) => push(&base, &['\u{1293}', '\u{120D}']), // -nal
            _ => return None,
        };
        Some(out)
    }

    /// 1sg perfect: the final radical either palatalises before the -alähu
    /// ending (säddäbä → säddäb + a → ... , ሄደ → ሄጃለሁ) or, if it cannot
    /// palatalise, takes an -e- linking vowel + -yalähu (ሰበረ → ሰብሬያለሁ).
    fn perfect_1sg(&self, base: &[char]) -> Vec<char> {
        let n = base.len();
        let last = base[n - 1];
        if let Some((row, _)) = syllable(last) {
            if let Some(pal) = palatal_row(char::from_u32(row).unwrap_or(last)) {
                let mut v = base[..n - 1].to_vec();
                v.push(set_order(pal, A)); // palatal + a
                v.extend(['\u{1208}', '\u{1201}']); // -lähu
                return v;
            }
        }
        let mut v = revowel_last(base, E);
        v.extend(['\u{12EB}', '\u{1208}', '\u{1201}']); // -yalähu
        v
    }

    // --- imperfective (prefix + imperfective stem + agreement) ---

    /// (imperfective stem, causative-prefix flag). The stem is the mined bare
    /// imperfective (ipfvn) minus its person prefix; the flag records whether
    /// that prefix carried the "a" vowel of an ʔ-causative (ያ- vs ይ-).
    fn imperfective_stem(&self) -> Option<(Vec<char>, bool)> {
        let ipfvn: Vec<char> = self.parts.ipfvn.as_ref()?.chars().collect();
        if ipfvn.len() < 2 {
            return None;
        }
        let causative = matches!(syllable(ipfvn[0]), Some((_, A)));
        Some((ipfvn[1..].to_vec(), causative))
    }

    /// The person prefix for the imperfective/jussive, its vowel-bearing
    /// consonant set to `vowel`. 1pl prepends a fixed ə (እ).
    fn subject_prefix(&self, f: &Feat, vowel: u8, alef_bare: bool) -> Vec<char> {
        match (f.person, f.number) {
            (1, Num::Sg) => {
                // ʔ order 0 already reads "a"; a bare ə prefix is order EPS.
                let o = if alef_bare && vowel == A { A1 } else { vowel };
                vec![cons(B_ALEF, o)]
            }
            (1, Num::Pl) => vec![cons(B_ALEF, EPS), cons(B_NUN, vowel)],
            (2, _) => vec![cons(B_TAW, vowel)],
            (3, _) => {
                if f.gender == Gen::Fem {
                    vec![cons(B_TAW, vowel)]
                } else {
                    vec![cons(B_YOD, vowel)]
                }
            }
            _ => vec![],
        }
    }

    fn imperfective(&self, f: &Feat) -> Option<Vec<char>> {
        // 3sg-masc is a mined principal part.
        if (f.person, f.number, f.gender) == (3, Num::Sg, Gen::Masc) {
            return if f.nfin {
                Some(self.parts.ipfvn.as_ref()?.chars().collect())
            } else {
                Some(self.parts.ipfv.as_ref()?.chars().collect())
            };
        }
        let (stem, causative) = self.imperfective_stem()?;
        let vowel = if causative { A } else { EPS };
        let mut out = self.subject_prefix(f, vowel, true);
        if f.nfin {
            let body = match (f.person, f.number, f.gender) {
                (2, Num::Sg, Gen::Fem) => revowel_last_i(&stem, false),
                (_, Num::Pl, _) if f.person != 1 => revowel_last(&stem, U),
                _ => stem,
            };
            out.extend(body);
            return Some(out);
        }
        // Main imperfective: NFIN stem's final radical → a, + the -al auxiliary.
        match (f.person, f.number, f.gender) {
            (2, Num::Sg, Gen::Fem) => {
                // A palatal or glide final radical fuses with -i and takes
                // -aläš directly (ትበልጫለሽ, ትለያለሽ); otherwise the linking -ya-
                // appears (ትሰብሪያለሽ).
                if final_palatalish(&stem) {
                    out.extend(revowel_last(&palatalize_last(&stem), A));
                    out.extend(['\u{1208}', '\u{123D}']); // -läš
                } else {
                    out.extend(revowel_last(&stem, I));
                    out.extend(['\u{12EB}', '\u{1208}', '\u{123D}']); // -yaläš
                }
            }
            _ => {
                out.extend(revowel_last(&stem, A));
                out.extend(self.ipfv_aux(f));
            }
        }
        Some(out)
    }

    /// The conjugated `-al` auxiliary of the main imperfective (after the
    /// a-final stem), keyed by subject.
    fn ipfv_aux(&self, f: &Feat) -> Vec<char> {
        match (f.person, f.number, f.gender) {
            (3, Num::Sg, Gen::Masc) => vec!['\u{120D}'], // -l
            (3, Num::Sg, Gen::Fem) => vec!['\u{1208}', '\u{127D}'], // -läčč
            (3, Num::Pl, _) => vec!['\u{1209}'],         // -lu
            (2, Num::Sg, Gen::Masc) => vec!['\u{1208}', '\u{1205}'], // -läh
            (2, Num::Pl, _) => vec!['\u{120B}', '\u{127D}', '\u{1201}'], // -laččhu
            (1, Num::Sg, _) => vec!['\u{1208}', '\u{1201}'], // -lähu
            (1, Num::Pl, _) => vec!['\u{1208}', '\u{1295}'], // -län
            _ => vec![],
        }
    }

    // --- imperative / jussive ---

    /// The imperative stem (2sg-masc). None only if unmineable.
    fn imperative_stem(&self) -> Option<Vec<char>> {
        if let Some(imp) = &self.parts.imp {
            return Some(imp.chars().collect());
        }
        // No mined imperative: derive a jussive base from the imperfective stem
        // by the type-A ablaut (CäCəC → CəCäC), or rebuild an ʔ-causative base.
        let (stem, causative) = self.imperfective_stem()?;
        if causative {
            let mut v = vec![B_ALEF];
            v.extend(stem);
            return Some(v);
        }
        // A ተ- passive keeps the imperfective vocalisation in the jussive
        // (ተሰበረ → ይሰበር), without the basic ablaut.
        if syllable(self.parts.pfv.chars().next().unwrap_or(' ')) == Some((0x1270, A1)) {
            return Some(stem);
        }
        let mut v = stem;
        if let Some(first) = v.first_mut() {
            *first = set_order(*first, EPS);
        }
        if v.len() >= 3 {
            v[1] = set_order(v[1], A1);
        }
        Some(v)
    }

    /// The base used for the prefixed jussive (1st/3rd person). A mined ተ-
    /// passive imperative drops its ተ in the jussive (ተመልከት → መልከት); a
    /// derived stem with no mined imperative (ተዳደረ) keeps the imperfective
    /// vocalisation, ተ and all.
    fn jussive_base(&self) -> Option<Vec<char>> {
        if let Some(imp) = &self.parts.imp {
            let s: Vec<char> = imp.chars().collect();
            if s.len() > 2 && syllable(s[0]) == Some((0x1270, A1)) {
                return Some(s[1..].to_vec());
            }
            return Some(s);
        }
        self.imperative_stem()
    }

    fn imperative(&self, f: &Feat) -> Option<Vec<char>> {
        // 2nd person: bare imperative stem + number/gender suffix.
        if f.person == 2 {
            let stem = self.imperative_stem()?;
            let out = match (f.number, f.gender) {
                (Num::Sg, Gen::Fem) => revowel_last_i(&stem, true),
                (Num::Pl, _) => revowel_last(&stem, U),
                _ => stem,
            };
            return Some(out);
        }
        // 1st/3rd person jussive: person prefix + jussive base. A base opening
        // with ʔ is a causative — the ʔ merges into the prefix vowel (ያድርግ) —
        // if the perfective is long enough; a short ʔ-initial root instead
        // elides the ʔ and exposes its first radical (አዘዘ → ይዘዝ).
        let base = self.jussive_base()?;
        let first = base.first().and_then(|c| syllable(*c));
        // A causative opens ʔ-a (አ, order 0); an ʔ-initial root opens ʔ-ə (እ),
        // whose glottal elides in the jussive.
        let causative = matches!(first, Some((0x12A0, A1)));
        let alef = matches!(first, Some((0x12A0, _)));
        let (vowel, body): (u8, Vec<char>) = if causative {
            (A, base[1..].to_vec())
        } else if alef {
            let mut b = base[1..].to_vec();
            if b.len() >= 2 && matches!(syllable(b[0]), Some((_, EPS))) {
                b[0] = set_order(b[0], A1);
            }
            (EPS, b)
        } else {
            (EPS, base)
        };
        // 1sg jussive takes an l- prefix (ልስበር), not the imperfective's ə- (እ).
        let mut out = if (f.person, f.number) == (1, Num::Sg) {
            vec![cons(B_LAM, vowel)]
        } else {
            self.subject_prefix(f, vowel, true)
        };
        let body = if f.number == Num::Pl && f.person == 3 {
            revowel_last(&body, U)
        } else {
            body
        };
        out.extend(body);
        Some(out)
    }
}

/// Append a suffix to a stem.
fn push(stem: &[char], suffix: &[char]) -> Vec<char> {
    let mut v = stem.to_vec();
    v.extend_from_slice(suffix);
    v
}

/// A compact conjugation table shared by the WebAssembly and Python bindings.
/// Each finite vector is in the fixed paradigm order (see `PNG`).
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// The citation (3sg-masc perfective).
    pub lemma: String,
    pub perfective: Vec<Option<String>>,
    pub perfect: Vec<Option<String>>,
    pub imperfective: Vec<Option<String>>,
    pub imperfective_nfin: Vec<Option<String>>,
    pub jussive: Vec<Option<String>>,
}

/// (person, number, gender-token) for the finite paradigm.
const PNG: [(u8, &str, &str); 8] = [
    (1, "SG", ""),
    (2, "SG", "MASC"),
    (2, "SG", "FEM"),
    (3, "SG", "MASC"),
    (3, "SG", "FEM"),
    (1, "PL", ""),
    (2, "PL", ""),
    (3, "PL", ""),
];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let cell = |extra: &[&str]| -> Vec<Option<String>> {
            PNG.iter()
                .map(|(p, n, g)| {
                    let ps = p.to_string();
                    let mut toks = vec![ps.as_str(), n];
                    if !g.is_empty() {
                        toks.push(g);
                    }
                    toks.extend_from_slice(extra);
                    toks.sort_unstable();
                    v.form(&format!("V;{}", toks.join(";")))
                })
                .collect()
        };
        Self {
            lemma: v.lemma.clone(),
            perfective: cell(&["PFV"]),
            perfect: cell(&["PRF"]),
            imperfective: cell(&["IPFV"]),
            imperfective_nfin: cell(&["IPFV", "NFIN"]),
            jussive: cell(&["IMP"]),
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
    fn perfective_sebere() {
        assert_eq!(f("ሰበረ", "V;3;MASC;PFV;SG"), "ሰበረ");
        assert_eq!(f("ሰበረ", "V;3;FEM;PFV;SG"), "ሰበረች");
        assert_eq!(f("ሰበረ", "V;3;PFV;PL"), "ሰበሩ");
        assert_eq!(f("ሰበረ", "V;2;MASC;PFV;SG"), "ሰበርክ");
        assert_eq!(f("ሰበረ", "V;2;PFV;PL"), "ሰበራችሁ");
        assert_eq!(f("ሰበረ", "V;1;PFV;SG"), "ሰበርኩ");
        assert_eq!(f("ሰበረ", "V;1;PFV;PL"), "ሰበርን");
    }

    #[test]
    fn imperfective_sebere() {
        assert_eq!(f("ሰበረ", "V;3;IPFV;MASC;SG"), "ይሰብራል");
        assert_eq!(f("ሰበረ", "V;3;IPFV;MASC;NFIN;SG"), "ይሰብር");
        assert_eq!(f("ሰበረ", "V;3;FEM;IPFV;SG"), "ትሰብራለች");
        assert_eq!(f("ሰበረ", "V;2;FEM;IPFV;SG"), "ትሰብሪያለሽ");
        assert_eq!(f("ሰበረ", "V;1;IPFV;PL"), "እንሰብራለን");
        assert_eq!(f("ሰበረ", "V;3;IPFV;NFIN;PL"), "ይሰብሩ");
    }

    #[test]
    fn perfect_sebere() {
        assert_eq!(f("ሰበረ", "V;3;MASC;PRF;SG"), "ሰብሯል");
        assert_eq!(f("ሰበረ", "V;3;FEM;PRF;SG"), "ሰብራለች");
        assert_eq!(f("ሰበረ", "V;3;PL;PRF"), "ሰብረዋል");
        assert_eq!(f("ሰበረ", "V;1;PL;PRF"), "ሰብረናል");
        assert_eq!(f("ሰበረ", "V;1;PRF;SG"), "ሰብሬያለሁ");
    }

    #[test]
    fn imperative_sebere() {
        assert_eq!(f("ሰበረ", "V;2;IMP;MASC;SG"), "ስበር");
        assert_eq!(f("ሰበረ", "V;2;FEM;IMP;SG"), "ስበሪ");
        assert_eq!(f("ሰበረ", "V;2;IMP;PL"), "ስበሩ");
        assert_eq!(f("ሰበረ", "V;1;IMP;SG"), "ልስበር");
        assert_eq!(f("ሰበረ", "V;3;IMP;MASC;SG"), "ይስበር");
        assert_eq!(f("ሰበረ", "V;3;IMP;PL"), "ይስበሩ");
    }

    #[test]
    fn causative_aderege() {
        // ʔ-causative: the prefix vowel fuses with the initial ʔ (ያ-, ላ-).
        assert_eq!(f("አደረገ", "V;3;IPFV;MASC;SG"), "ያደርጋል");
        assert_eq!(f("አደረገ", "V;1;IMP;SG"), "ላድርግ");
        assert_eq!(f("አደረገ", "V;3;IMP;MASC;SG"), "ያድርግ");
    }

    #[test]
    fn seedless_lemma_via_override() {
        // ነው (copula) has no principal parts; it is served entirely from the
        // override layer, yet still resolves as a supported verb.
        assert_eq!(f("ነው", "V;3;MASC;PRS;SG"), "ነው");
        assert!(Verb::from_lemma("ተያየ").is_ok());
    }

    #[test]
    fn unknown_lemma_rejected() {
        assert_eq!(Verb::from_lemma("xyz").err(), Some(Error::NotAVerb));
    }

    #[test]
    fn table_shape() {
        let v = Verb::from_lemma("ሰበረ").unwrap();
        let t = Table::build(&v);
        assert_eq!(t.perfective.len(), 8);
        assert_eq!(t.perfective[3].as_deref(), Some("ሰበረ"));
        assert_eq!(t.jussive.len(), 8);
    }
}
