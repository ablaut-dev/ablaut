//! Modern Hebrew verb conjugation (unvoweled ktiv male).
//!
//! Hebrew verbs are built on triconsonantal (and quadriliteral) roots in one
//! of seven binyanim. The lemma is the unvoweled ktiv-male 3sg-masc past
//! (שמר). The engine resolves it to a few mined principal parts
//! (`data/heb/parts.tsv`: past-3sgm, future-3sgm, present-masc-sg, infinitive)
//! and derives the paradigm by the regular person/number/gender affixation of
//! the past (suffixes), future (prefix + stem + suffix, with mater reduction),
//! and present (participle gender/number). Weak roots — final-ה (ל״ה), hollow
//! (ע״ו/ע״י), hifil (medial hireq-yod), final-ת/נ mergers, פ״א — are handled by
//! dedicated rules; a small mined-override layer (`data/heb/overrides.tsv`)
//! patches the residue the rules do not reach.
//!
//! Feature strings are the shared canonical form: `V;` + tokens sorted, e.g.
//!   past 3sg f      → V;3;FEM;PST;SG
//!   future 2pl m    → V;2;FUT;MASC;PL
//!   present fem sg  → V;FEM;PRS;SG        (no person)
//!   infinitive      → V;NFIN

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
        write!(f, "not a Hebrew verb we know")
    }
}

static PARTS_TSV: &str = include_str!("../data/heb/parts.tsv");
static OVERRIDES_TSV: &str = include_str!("../data/heb/overrides.tsv");

const HE: char = 'ה';
const WAW: char = 'ו';
const YOD: char = 'י';
const TAV: char = 'ת';
const NUN: char = 'נ';
const ALEF: char = 'א';

/// Map a word-final letter to its medial form (used when a suffix follows).
fn medial(c: char) -> char {
    match c {
        'ך' => 'כ',
        'ם' => 'מ',
        'ן' => NUN,
        'ף' => 'פ',
        'ץ' => 'צ',
        other => other,
    }
}

/// Medialize the last letter of a stem (it is no longer word-final because a
/// suffix will follow).
fn medialize_last(stem: &[char]) -> Vec<char> {
    let mut v = stem.to_vec();
    if let Some(last) = v.last_mut() {
        *last = medial(*last);
    }
    v
}

/// Append a suffix to a stem, medializing the stem's final letter and
/// collapsing a geminate ת/נ at the seam (כרת+ת → כרת, הבן+נו → הבנו).
fn join(stem: &[char], suffix: &[char]) -> Vec<char> {
    let mut v = medialize_last(stem);
    if let (Some(&last), Some(&first)) = (v.last(), suffix.first()) {
        if last == first && (last == TAV || last == NUN) {
            v.extend_from_slice(&suffix[1..]);
            return v;
        }
    }
    v.extend_from_slice(suffix);
    v
}

/// The mined principal parts of one lemma; "-" columns become None.
#[derive(Debug, Clone)]
pub struct Parts {
    pub past: String,
    pub future: String,
    pub present: Option<String>,
    pub infinitive: Option<String>,
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
            let opt = |s: &str| (s != "-" && !s.is_empty()).then(|| s.to_string());
            m.insert(
                c[0].to_string(),
                Parts {
                    past: c[1].to_string(),
                    future: c[2].to_string(),
                    present: c.get(3).and_then(|s| opt(s)),
                    infinitive: c.get(4).and_then(|s| opt(s)),
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
enum Tense {
    Past,
    Present,
    Future,
    Imperative,
    Infinitive,
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
    tense: Tense,
    person: u8, // 0 for present (no person)
    number: Num,
    gender: Gen,
}

fn parse_feat(features: &str) -> Option<Feat> {
    let has = |t: &str| features.split(';').any(|x| x == t);
    let tense = if has("NFIN") {
        Tense::Infinitive
    } else if has("IMP") {
        Tense::Imperative
    } else if has("FUT") {
        Tense::Future
    } else if has("PRS") {
        Tense::Present
    } else if has("PST") {
        Tense::Past
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
        tense,
        person,
        number,
        gender,
    })
}

/// Which family of suffix a future/imperative slot takes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Suf {
    None,   // ∅
    VowelI, // -י   (2sg fem)
    VowelU, // -ו   (m-pl)
    Nun,    // -נה  (f-pl)
}

/// One Hebrew verb, resolved to its principal parts.
pub struct Verb {
    parts: Parts,
    lemma: String,
}

impl Verb {
    /// Look a lemma (unvoweled ktiv-male 3sg-masc past) up in the mined table.
    pub fn from_lemma(lemma: &str) -> Result<Self, Error> {
        table()
            .get(lemma)
            .map(|p| Verb {
                parts: p.clone(),
                lemma: lemma.to_string(),
            })
            .ok_or(Error::NotAVerb)
    }

    /// The form for a canonical feature string, or None for a cell the engine
    /// does not (yet) generate.
    pub fn form(&self, features: &str) -> Option<String> {
        if let Some(o) = overrides().get(&(self.lemma.clone(), features.to_string())) {
            return Some(o.clone());
        }
        let f = parse_feat(features)?;
        let chars = match f.tense {
            Tense::Past => self.past(&f),
            Tense::Present => self.present(&f),
            Tense::Future => self.future(&f),
            Tense::Imperative => self.imperative(&f),
            Tense::Infinitive => return self.parts.infinitive.clone(),
        }?;
        Some(chars.into_iter().collect())
    }

    /// True for the "short" qal present (statives and hollow verbs), whose
    /// participle equals the past and takes a -ה feminine (קם→קמה, גדל→גדלה).
    /// A נפעל participle also equals its past (נשמר) but is longer and takes -ת,
    /// so the length guard separates the two.
    fn short_qal(&self) -> bool {
        matches!(&self.parts.present, Some(p) if *p == self.parts.past)
            && self.parts.past.chars().count() <= 3
    }

    /// True for a hifil: its present participle is מ + root with a hireq-yod
    /// before the last radical (מדליק, מכיר, מבין), its past starts with ה, and
    /// it is neither a hofal (מו-) nor a metathesis hitpael (מ + sibilant + ת).
    /// Hifil takes a -ה feminine present and drops that medial yod in the past.
    fn is_hifil(&self) -> bool {
        let Some(pres) = &self.parts.present else {
            return false;
        };
        let p: Vec<char> = pres.chars().collect();
        let n = p.len();
        if p.first() != Some(&'מ') || !self.parts.past.starts_with(HE) {
            return false;
        }
        // The hireq-yod before the last radical is hifil's signature; a hofal
        // (מוקם) or hitpael (מסתתר) lacks it, so this alone separates them from
        // the surface-similar מו-/מסת- hifils (מוסיף, מסתיר).
        n >= 2 && p[n - 2] == YOD
    }

    /// True for a hollow verb (ע״ו/ע״י): its medial ו/י is a root letter and
    /// must survive mater reduction under a vowel suffix.
    fn is_hollow(&self) -> bool {
        // A two-letter past (קם, מת, רב) is always hollow/geminate; a short qal
        // whose future stem carries a medial mater is hollow too (אור, בוש).
        if self.parts.past.chars().count() == 2 {
            return true;
        }
        if self.short_qal() {
            let stem: Vec<char> = self.parts.future.chars().skip(1).collect();
            let n = stem.len();
            return n >= 2 && matches!(stem[n - 2], WAW | YOD);
        }
        false
    }

    // --- past ---

    fn past(&self, f: &Feat) -> Option<Vec<char>> {
        let past: Vec<char> = self.parts.past.chars().collect();
        // 3sg masc is the citation itself.
        if f.person == 3 && f.number == Num::Sg && f.gender == Gen::Masc {
            return Some(past);
        }
        let suffix: &[char] = &self.past_suffix(f);
        let vowel_suffix = matches!(suffix, [HE] | [WAW]);

        // ל״ה: drop the final ה; consonant suffixes insert a linking י.
        if past.last() == Some(&HE) {
            let base = &past[..past.len() - 1];
            let mut out = base.to_vec();
            if f.person == 3 && f.gender == Gen::Fem && f.number == Num::Sg {
                out.extend([TAV, HE]); // בנתה
            } else if vowel_suffix {
                out.extend_from_slice(suffix); // בנו
            } else {
                out.push(YOD); // בניתי / בנית / בנינו
                out.extend_from_slice(suffix);
            }
            return Some(out);
        }

        // Non-ל״ה. Hifil drops its medial hireq-yod before a consonant suffix
        // (הדליק → הדלקת, הבין → הבנתי).
        let mut stem = past;
        if !vowel_suffix && self.is_hifil() {
            let n = stem.len();
            if n >= 2 && stem[n - 2] == YOD {
                stem.remove(n - 2);
            }
        }
        Some(join(&stem, suffix))
    }

    fn past_suffix(&self, f: &Feat) -> Vec<char> {
        match (f.person, f.number, f.gender) {
            (3, Num::Sg, Gen::Fem) => vec![HE],
            (3, Num::Pl, _) => vec![WAW],
            (2, Num::Sg, _) => vec![TAV],
            (2, Num::Pl, Gen::Masc) => vec![TAV, 'ם'],
            (2, Num::Pl, Gen::Fem) => vec![TAV, 'ן'],
            (1, Num::Sg, _) => vec![TAV, YOD],
            (1, Num::Pl, _) => vec![NUN, WAW],
            _ => vec![],
        }
    }

    // --- present (participle affixation) ---

    fn present(&self, f: &Feat) -> Option<Vec<char>> {
        let part: Vec<char> = self.parts.present.as_ref()?.chars().collect();
        let lamed_he = part.last() == Some(&HE);
        // masc sg is the participle itself.
        if f.gender == Gen::Masc && f.number == Num::Sg {
            return Some(part);
        }
        if f.number == Num::Sg {
            // feminine singular
            if lamed_he {
                // נפעל ל״ה takes a -ית feminine (נראה → נראית); qal ל״ה is
                // identical to the masculine (בונה). The נפעל participle is
                // נ + two radicals + ה with no holam-waw after the נ.
                if part.first() == Some(&NUN) && part.get(1) != Some(&WAW) {
                    let mut v = part[..part.len() - 1].to_vec();
                    v.extend([YOD, TAV]);
                    return Some(v);
                }
                return Some(part); // בונה
            }
            let mut v = medialize_last(&part);
            // A -ה feminine for the hifil / short qal (מדליקה, קמה); a -ת
            // feminine otherwise (שומרת), appended without gemination collapse
            // so a ת-final participle keeps both tavs (שובת → שובתת).
            v.push(if self.short_qal() || self.is_hifil() {
                HE
            } else {
                TAV
            });
            return Some(v);
        }
        // plural: drop a final ה, then +ים / +ות
        let base: Vec<char> = if lamed_he {
            part[..part.len() - 1].to_vec()
        } else {
            part
        };
        let mut v = medialize_last(&base);
        if f.gender == Gen::Fem {
            v.extend([WAW, TAV]);
        } else {
            v.extend([YOD, 'ם']);
        }
        Some(v)
    }

    // --- future ---

    fn future(&self, f: &Feat) -> Option<Vec<char>> {
        // 3sg masc is the mined future part.
        if f.person == 3 && f.number == Num::Sg && f.gender == Gen::Masc {
            return Some(self.parts.future.chars().collect());
        }
        let stem: Vec<char> = self.parts.future.chars().skip(1).collect();
        let prefix = self.future_prefix(f);
        let suf = future_suffix(f);
        let mut out = vec![prefix];
        out.extend(self.future_body(&stem, suf));
        Some(out)
    }

    /// The prefix consonant for a future/imperfect slot.
    fn future_prefix(&self, f: &Feat) -> char {
        match f.person {
            1 => {
                if f.number == Num::Sg {
                    ALEF
                } else {
                    NUN
                }
            }
            2 => TAV,
            _ => {
                // 3rd person: feminine (sg or pl) takes ת, masculine takes י.
                if f.gender == Gen::Fem {
                    TAV
                } else {
                    YOD
                }
            }
        }
    }

    /// The stem+suffix body of a future form (everything after the prefix).
    fn future_body(&self, stem: &[char], suf: Suf) -> Vec<char> {
        let lamed_he = stem.last() == Some(&HE);
        if lamed_he {
            let base = &stem[..stem.len() - 1];
            return match suf {
                Suf::None => stem.to_vec(),
                Suf::VowelI => {
                    let mut v = base.to_vec();
                    v.push(YOD);
                    v
                }
                Suf::VowelU => {
                    let mut v = base.to_vec();
                    v.push(WAW);
                    v
                }
                Suf::Nun => {
                    let mut v = base.to_vec();
                    v.extend([YOD, NUN, HE]);
                    v
                }
            };
        }
        match suf {
            Suf::None => stem.to_vec(),
            Suf::VowelI => join(&self.reduce_holam(stem), &[YOD]),
            Suf::VowelU => join(&self.reduce_holam(stem), &[WAW]),
            Suf::Nun => join(&reduce_mater(stem), &[NUN, HE]),
        }
    }

    /// Drop the theme-vowel holam ו before a vowel suffix (ישמור → ישמרו). A
    /// hollow verb's medial ו/י is a root letter and is kept (יקום → יקומו).
    fn reduce_holam(&self, stem: &[char]) -> Vec<char> {
        let n = stem.len();
        if !self.is_hollow() && n >= 2 && stem[n - 2] == WAW {
            let mut v = stem.to_vec();
            v.remove(n - 2);
            return v;
        }
        stem.to_vec()
    }
}

/// Drop any medial mater (ו/י) before the -נה feminine-plural ending
/// (ישמור → תשמרנה, יקום → תקמנה, ידליק → תדלקנה).
fn reduce_mater(stem: &[char]) -> Vec<char> {
    let n = stem.len();
    if n >= 2 && matches!(stem[n - 2], WAW | YOD) {
        let mut v = stem.to_vec();
        v.remove(n - 2);
        return v;
    }
    stem.to_vec()
}

fn future_suffix(f: &Feat) -> Suf {
    match (f.number, f.gender) {
        (Num::Sg, Gen::Fem) if f.person == 2 => Suf::VowelI,
        (Num::Pl, Gen::Fem) => Suf::Nun,
        (Num::Pl, Gen::Masc) => Suf::VowelU,
        _ => Suf::None,
    }
}

impl Verb {
    // --- imperative (not scored; derived from the future) ---

    fn imperative(&self, f: &Feat) -> Option<Vec<char>> {
        // Build the matching 2nd-person future and drop its prefix.
        let fut = Feat {
            tense: Tense::Future,
            person: 2,
            number: f.number,
            gender: f.gender,
        };
        let full = self.future(&fut)?;
        Some(full[1..].to_vec())
    }
}

/// A compact conjugation table shared by the WebAssembly and Python bindings.
/// Each vector is in the fixed paradigm order (see `PNG`). The present has no
/// person; the imperative keeps only its four 2nd-person cells.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// The unvoweled citation (3sg-masc past).
    pub lemma: String,
    pub past: Vec<Option<String>>,
    pub present: Vec<Option<String>>,
    pub future: Vec<Option<String>>,
    pub imperative: Vec<Option<String>>,
    pub infinitive: Option<String>,
}

/// (person, number, gender) for the finite past/future paradigm.
const PNG: [(u8, &str, &str); 10] = [
    (1, "SG", ""),
    (2, "SG", "MASC"),
    (2, "SG", "FEM"),
    (3, "SG", "MASC"),
    (3, "SG", "FEM"),
    (1, "PL", ""),
    (2, "PL", "MASC"),
    (2, "PL", "FEM"),
    (3, "PL", "MASC"),
    (3, "PL", "FEM"),
];

/// (number, gender) for the present participle.
const PRS_NG: [(&str, &str); 4] = [("SG", "MASC"), ("SG", "FEM"), ("PL", "MASC"), ("PL", "FEM")];

/// (number, gender) for the four 2nd-person imperative cells.
const IMP_NG: [(&str, &str); 4] = [("SG", "MASC"), ("SG", "FEM"), ("PL", "MASC"), ("PL", "FEM")];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let cell = |toks: Vec<&str>| -> Option<String> {
            let mut t: Vec<String> = toks.into_iter().map(str::to_string).collect();
            t.sort();
            v.form(&format!("V;{}", t.join(";")))
        };
        let finite = |tense: &str| -> Vec<Option<String>> {
            PNG.iter()
                .map(|(p, n, g)| {
                    let ps = p.to_string();
                    let mut toks = vec![ps.as_str(), n, tense];
                    if !g.is_empty() {
                        toks.push(g);
                    }
                    cell(toks)
                })
                .collect()
        };
        let present = PRS_NG
            .iter()
            .map(|(n, g)| cell(vec![n, g, "PRS"]))
            .collect();
        let imperative = IMP_NG
            .iter()
            .map(|(n, g)| cell(vec!["2", n, g, "IMP"]))
            .collect();
        Self {
            lemma: v.lemma.clone(),
            past: finite("PST"),
            present,
            future: finite("FUT"),
            imperative,
            infinitive: v.form("V;NFIN"),
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
    fn strong_shamar() {
        assert_eq!(f("שמר", "V;3;MASC;PST;SG"), "שמר");
        assert_eq!(f("שמר", "V;3;FEM;PST;SG"), "שמרה");
        assert_eq!(f("שמר", "V;3;PL;PST"), "שמרו");
        assert_eq!(f("שמר", "V;1;PST;SG"), "שמרתי");
        assert_eq!(f("שמר", "V;MASC;PRS;SG"), "שומר");
        assert_eq!(f("שמר", "V;FEM;PRS;SG"), "שומרת");
        assert_eq!(f("שמר", "V;MASC;PL;PRS"), "שומרים");
        assert_eq!(f("שמר", "V;2;FEM;FUT;SG"), "תשמרי");
        assert_eq!(f("שמר", "V;3;FUT;MASC;PL"), "ישמרו");
    }

    #[test]
    fn lamed_he_bana() {
        assert_eq!(f("בנה", "V;3;FEM;PST;SG"), "בנתה");
        assert_eq!(f("בנה", "V;3;PL;PST"), "בנו");
        assert_eq!(f("בנה", "V;1;PST;SG"), "בניתי");
        assert_eq!(f("בנה", "V;FEM;PRS;SG"), "בונה");
        assert_eq!(f("בנה", "V;2;FEM;FUT;SG"), "תבני");
        assert_eq!(f("בנה", "V;2;FEM;FUT;PL"), "תבנינה");
    }

    #[test]
    fn hollow_kam() {
        assert_eq!(f("קם", "V;3;FEM;PST;SG"), "קמה");
        assert_eq!(f("קם", "V;1;PST;SG"), "קמתי");
        assert_eq!(f("קם", "V;FEM;PRS;SG"), "קמה");
        assert_eq!(f("קם", "V;2;FEM;FUT;SG"), "תקומי");
        assert_eq!(f("קם", "V;3;FUT;MASC;PL"), "יקומו");
    }

    #[test]
    fn piel_diber() {
        assert_eq!(f("דיבר", "V;FEM;PRS;SG"), "מדברת");
        assert_eq!(f("דיבר", "V;2;FEM;FUT;SG"), "תדברי");
        assert_eq!(f("דיבר", "V;2;FEM;FUT;PL"), "תדברנה");
    }

    #[test]
    fn hifil_hidlik() {
        assert_eq!(f("הדליק", "V;2;MASC;PST;SG"), "הדלקת");
        assert_eq!(f("הדליק", "V;1;PST;SG"), "הדלקתי");
        assert_eq!(f("הדליק", "V;3;FEM;PST;SG"), "הדליקה");
        assert_eq!(f("הדליק", "V;FEM;PRS;SG"), "מדליקה");
        assert_eq!(f("הדליק", "V;2;FEM;FUT;SG"), "תדליקי");
        assert_eq!(f("הדליק", "V;2;FEM;FUT;PL"), "תדלקנה");
    }
}
