//! Tagalog conjugation: aspect × voice, built with infixation and
//! CV-reduplication — two stem-internal operations the rest of ablaut's
//! suffix-oriented engines do not need, added here as reusable helpers.
//!
//! A Tagalog verb inflects for **aspect** (perfective / imperfective /
//! contemplated) crossed with **voice** (actor-focus / patient-focus).
//! Wiktionary and UniMorph tabulate the six inflected cells plus the
//! bare root, and that is the grid this module fills.
//!
//! Which affix family a root takes is lexical, like German strong-verb
//! class, and is stored one row per root in `data/tgl/verbs.tsv`:
//!
//! - **actor voice** is one of `-um-` (an infix: sulat → s·um·ulat),
//!   `mag-` (nagsulat), `mang-` (with nasal assimilation: kuha →
//!   nanguha), or the potentive `ma-` (kita → nakita);
//! - **patient voice** is the object suffix `-in` (with the perfective
//!   realized as the `-in-` infix: tanggap → t·in·anggap), the locative
//!   `-an`, or the benefactive/instrument prefix `i-`.
//!
//! The engine supplies the productive morphophonology on top of the
//! class: the `-um-`/`-in-` **infix** after the first consonant, **CV
//! reduplication** for the imperfective and contemplated aspects
//! (sulat → susulat), `mang-` **nasal assimilation** (the prefix nasal
//! agrees with the root onset, which itself fuses when it is p/t/k/s),
//! **o→u raising** before a suffix (abot → abutin), an optional linking
//! **-h-** before a vowel-final suffix (basa → basahin), and the two
//! productive allomorph variants attested in the oracles — intervocalic
//! **d→r** in reduplicated stems (dating → dumarating) and the **ni-**
//! shape of the patient perfective (lagnat → nilagnat). Because several
//! cells have more than one standard spelling, `forms` returns every
//! candidate variant.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The three aspects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Aspect {
    /// Completed action (perfective): `sumulat`, `sinulat`.
    Perfective,
    /// Ongoing/habitual (imperfective): `sumusulat`, `sinusulat`.
    Imperfective,
    /// Not-yet-begun (contemplated): `susulat`, `susulatin`.
    Contemplated,
}

/// The two voices (grammatical focus / trigger) this module covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Focus {
    /// Actor focus (AGFOC): the subject is the doer.
    Actor,
    /// Patient/object focus (PFOC): the subject is the undergoer.
    Patient,
}

/// Why an input cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input is empty or contains whitespace — not a single root.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Tagalog verb root")
    }
}

/// The actor-voice affix family of a root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Actor {
    Um,
    Mag,
    Mang,
    Ma,
}

/// The patient-voice affix family of a root (`None` = no patient voice).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Patient {
    In,
    An,
    I,
}

static LEXICON_TSV: &str = include_str!("../data/tgl/verbs.tsv");

#[derive(Debug, Clone)]
struct LexEntry {
    actor: Actor,
    patient: Option<Patient>,
    h_insert: bool,
    /// Overrides, indexed by [`slot_index`]: 0..3 actor pfv/ipfv/cont,
    /// 3..6 patient pfv/ipfv/cont. `None` falls through to the rule.
    overrides: [Option<String>; 6],
}

fn lexicon() -> &'static HashMap<String, LexEntry> {
    static MAP: OnceLock<HashMap<String, LexEntry>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in LEXICON_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            if c.len() < 4 {
                continue;
            }
            let actor = match c[1] {
                "mag" => Actor::Mag,
                "mang" => Actor::Mang,
                "ma" => Actor::Ma,
                _ => Actor::Um,
            };
            let patient = match c[2] {
                "in" => Some(Patient::In),
                "an" => Some(Patient::An),
                "i" => Some(Patient::I),
                _ => None,
            };
            let mut overrides: [Option<String>; 6] = Default::default();
            for (i, slot) in overrides.iter_mut().enumerate() {
                *slot = c
                    .get(4 + i)
                    .and_then(|s| (*s != "-" && !s.is_empty()).then(|| (*s).to_string()));
            }
            m.insert(
                c[0].to_string(),
                LexEntry {
                    actor,
                    patient,
                    h_insert: c[3] == "1",
                    overrides,
                },
            );
        }
        m
    })
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'A' | 'E' | 'I' | 'O' | 'U')
}

/// Number of leading consonants before the first vowel.
fn onset_len(w: &str) -> usize {
    w.chars().take_while(|c| !is_vowel(*c)).count()
}

fn char_at(w: &str, i: usize) -> Option<char> {
    w.chars().nth(i)
}

/// Copy the first CV of the root (the first vowel alone if vowel-initial):
/// the reduplicant that marks imperfective and contemplated aspect.
fn reduplicate(root: &str) -> String {
    let chars: Vec<char> = root.chars().collect();
    let i = onset_len(root);
    if i >= chars.len() {
        return root.to_string();
    }
    if i == 0 {
        return format!("{}{root}", chars[0]);
    }
    format!("{}{}{root}", chars[0], chars[i])
}

/// Insert `af` after the first consonant; prefix it if `w` is vowel-initial.
fn infix(w: &str, af: &str) -> String {
    let mut it = w.chars();
    let Some(first) = it.next() else {
        return af.to_string();
    };
    if is_vowel(first) {
        return format!("{af}{w}");
    }
    format!("{first}{af}{}", it.as_str())
}

/// Raise the last `o` of the stem to `u` before a suffix (abot → abut-).
fn raise_last_o(root: &str) -> String {
    match root.rfind('o') {
        Some(idx) => format!("{}u{}", &root[..idx], &root[idx + 1..]),
        None => root.to_string(),
    }
}

/// Attach a vowel-initial suffix, inserting a linking `-h-` after a
/// vowel-final stem when the lexeme calls for it (basa → basahin).
fn suffix(root: &str, s: &str, h: bool) -> String {
    let r = raise_last_o(root);
    if h && r.chars().last().is_some_and(is_vowel) {
        return format!("{r}h{s}");
    }
    format!("{r}{s}")
}

/// The `mang-` prefix nasal, assimilated to the root onset.
fn nasal_for(onset: char) -> &'static str {
    match onset {
        '\0' | 'p' | 'b' => "m",
        't' | 'd' | 's' | 'n' => "n",
        _ => "ng", // k, g, l, r, w, y, h, ng, vowel
    }
}

/// The `mang-` base: assimilated nasal and the stem, whose p/t/k/s onset
/// fuses into the nasal (kuha → ng + uha).
fn mang_base(root: &str) -> (&'static str, String) {
    let first = char_at(root, 0).unwrap_or('\0');
    if is_vowel(first) {
        return (nasal_for('\0'), root.to_string());
    }
    let fuse = matches!(first, 'p' | 't' | 'k' | 's');
    let stem: String = if fuse {
        root.chars().skip(1).collect()
    } else {
        root.to_string()
    };
    (nasal_for(first), stem)
}

fn hyphen(word: &str) -> &'static str {
    if char_at(word, 0).is_some_and(is_vowel) {
        "-"
    } else {
        ""
    }
}

/// Add the intervocalic d→r variant of `form` (dadating → darating).
fn with_dr(form: &str) -> Option<String> {
    let chars: Vec<char> = form.chars().collect();
    for i in 1..chars.len().saturating_sub(1) {
        if chars[i] == 'd' && is_vowel(chars[i - 1]) && is_vowel(chars[i + 1]) {
            let mut out = chars.clone();
            out[i] = 'r';
            return Some(out.into_iter().collect());
        }
    }
    None
}

/// A conjugatable Tagalog verb: its root and its stored affix class.
#[derive(Debug, Clone)]
pub struct Verb {
    root: String,
    actor: Actor,
    patient: Option<Patient>,
    h_insert: bool,
    overrides: [Option<String>; 6],
}

const fn slot_index(focus: Focus, aspect: Aspect) -> usize {
    let base = match focus {
        Focus::Actor => 0,
        Focus::Patient => 3,
    };
    base + match aspect {
        Aspect::Perfective => 0,
        Aspect::Imperfective => 1,
        Aspect::Contemplated => 2,
    }
}

impl Verb {
    /// Build a verb from its root (`sulat`, `kain`). The affix class is
    /// looked up in the compiled lexicon; an unknown root falls back to
    /// the commonest productive pattern (`-um-` actor, `-in` patient).
    ///
    /// ```
    /// use ablaut::tgl::{Aspect, Focus, Verb};
    /// let v = Verb::from_infinitive("sulat").unwrap();
    /// assert_eq!(v.form(Focus::Actor, Aspect::Perfective), "sumulat");
    /// assert_eq!(v.form(Focus::Actor, Aspect::Imperfective), "sumusulat");
    /// ```
    pub fn from_infinitive(root: &str) -> Result<Self, Error> {
        let root = root.trim().to_lowercase();
        if root.is_empty() || root.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        let lex = lexicon().get(&root);
        Ok(Self {
            actor: lex.map_or(Actor::Um, |l| l.actor),
            patient: lex.map_or(Some(Patient::In), |l| l.patient),
            h_insert: lex.map_or(true, |l| l.h_insert),
            overrides: lex.map(|l| l.overrides.clone()).unwrap_or_default(),
            root,
        })
    }

    /// The bare root, the citation form (`V;NFIN`).
    #[must_use]
    pub fn root(&self) -> &str {
        &self.root
    }

    /// Every accepted spelling of one aspect × voice cell. The first is
    /// the primary form; later ones are attested allomorph variants.
    /// Empty when the root has no patient voice and `Patient` is asked.
    #[must_use]
    pub fn forms(&self, focus: Focus, aspect: Aspect) -> Vec<String> {
        if let Some(o) = &self.overrides[slot_index(focus, aspect)] {
            return vec![o.clone()];
        }
        match focus {
            Focus::Actor => self.actor_forms(aspect),
            Focus::Patient => self.patient_forms(aspect),
        }
    }

    /// The primary spelling of a cell (first of [`Verb::forms`]), or the
    /// root if the cell does not exist.
    #[must_use]
    pub fn form(&self, focus: Focus, aspect: Aspect) -> String {
        self.forms(focus, aspect)
            .into_iter()
            .next()
            .unwrap_or_else(|| self.root.clone())
    }

    fn actor_forms(&self, aspect: Aspect) -> Vec<String> {
        let root = &self.root;
        let redup = reduplicate(root);
        let form = match (self.actor, aspect) {
            (Actor::Um, Aspect::Perfective) => infix(root, "um"),
            (Actor::Um, Aspect::Imperfective) => infix(&redup, "um"),
            (Actor::Um, Aspect::Contemplated) => redup.clone(),
            (Actor::Mag, Aspect::Perfective) => format!("nag{}{root}", hyphen(root)),
            (Actor::Mag, Aspect::Imperfective) => format!("nag{}{redup}", hyphen(root)),
            (Actor::Mag, Aspect::Contemplated) => format!("mag{}{redup}", hyphen(root)),
            (Actor::Mang, _) => {
                let (n, stem) = mang_base(root);
                let rs = reduplicate(&stem);
                let (prefix, body) = match aspect {
                    Aspect::Perfective => ("na", &stem),
                    Aspect::Imperfective => ("na", &rs),
                    Aspect::Contemplated => ("ma", &rs),
                };
                format!("{prefix}{n}{}{body}", hyphen(&stem))
            }
            (Actor::Ma, Aspect::Perfective) => format!("na{root}"),
            (Actor::Ma, Aspect::Imperfective) => format!("na{redup}"),
            (Actor::Ma, Aspect::Contemplated) => format!("ma{redup}"),
        };
        push_variants(form, root)
    }

    fn patient_forms(&self, aspect: Aspect) -> Vec<String> {
        let Some(patient) = self.patient else {
            return Vec::new();
        };
        let root = &self.root;
        let redup = reduplicate(root);
        let mut out = Vec::new();
        let form = match (patient, aspect) {
            (Patient::In, Aspect::Perfective) => infix(root, "in"),
            (Patient::In, Aspect::Imperfective) => infix(&redup, "in"),
            (Patient::In, Aspect::Contemplated) => suffix(&redup, "in", self.h_insert),
            (Patient::An, Aspect::Perfective) => format!("{}an", infix(root, "in")),
            (Patient::An, Aspect::Imperfective) => format!("{}an", infix(&redup, "in")),
            (Patient::An, Aspect::Contemplated) => suffix(&redup, "an", self.h_insert),
            (Patient::I, Aspect::Perfective) => format!("i{}", infix(root, "in")),
            (Patient::I, Aspect::Imperfective) => format!("i{}", infix(&redup, "in")),
            (Patient::I, Aspect::Contemplated) => format!("i{redup}"),
        };
        // The ni- allomorph of the consonant-initial patient perfective
        // and imperfective (lagnat → nilagnat, nililagnat).
        if matches!(patient, Patient::In | Patient::An)
            && !char_at(root, 0).is_some_and(is_vowel)
            && aspect != Aspect::Contemplated
        {
            let suf = if patient == Patient::An { "an" } else { "" };
            let base = if aspect == Aspect::Perfective {
                root
            } else {
                &redup
            };
            out.push(format!("ni{base}{suf}"));
        }
        let mut variants = push_variants(form, root);
        variants.append(&mut out);
        variants
    }
}

/// The primary form plus its intervocalic d→r variant, if any.
fn push_variants(form: String, root: &str) -> Vec<String> {
    let mut out = vec![form.clone()];
    if char_at(root, 0) == Some('d') {
        if let Some(v) = with_dr(&form) {
            out.push(v);
        }
    }
    out
}

/// The full six-cell conjugation table, shared by the bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// Actor focus: `[perfective, imperfective, contemplated]`.
    pub actor: [String; 3],
    /// Patient focus: `[perfective, imperfective, contemplated]`; empty
    /// strings if the root takes no patient voice.
    pub patient: [String; 3],
}

const ASPECTS: [Aspect; 3] = [
    Aspect::Perfective,
    Aspect::Imperfective,
    Aspect::Contemplated,
];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row = |focus: Focus| {
            ASPECTS.map(|a| v.forms(focus, a).into_iter().next().unwrap_or_default())
        };
        Self {
            infinitive: v.root().to_string(),
            actor: row(Focus::Actor),
            patient: row(Focus::Patient),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Aspect::{Contemplated as C, Imperfective as I, Perfective as P};
    use Focus::{Actor as AG, Patient as PF};

    fn v(root: &str) -> Verb {
        Verb::from_infinitive(root).unwrap()
    }

    #[test]
    fn um_class_infix_and_reduplication() {
        let s = v("sulat"); // -um- actor, -an patient in the lexicon
        assert_eq!(s.form(AG, P), "sumulat");
        assert_eq!(s.form(AG, I), "sumusulat");
        assert_eq!(s.form(AG, C), "susulat");
    }

    #[test]
    fn um_vowel_initial_prefixes() {
        let a = v("abuso");
        assert_eq!(a.form(AG, P), "umabuso");
        assert_eq!(a.form(AG, I), "umaabuso");
        assert_eq!(a.form(AG, C), "aabuso");
    }

    #[test]
    fn patient_in_infix_and_suffix() {
        let t = v("tanggap");
        assert_eq!(t.form(PF, P), "tinanggap");
        assert_eq!(t.form(PF, I), "tinatanggap");
        assert_eq!(t.form(PF, C), "tatanggapin");
    }

    #[test]
    fn mag_class() {
        let d = v("dagdag");
        assert_eq!(d.form(AG, P), "nagdagdag");
        assert!(d.forms(AG, I).contains(&"nagdaragdag".to_string()));
    }

    #[test]
    fn mang_nasal_assimilation() {
        // k fuses into the ng nasal: na + ng + uha → nanguha.
        assert_eq!(mang_base("kuha"), ("ng", "uha".to_string()));
        // b keeps its onset, nasal is m: pa·na → mamana would be b→m; here
        // d stays and the nasal is n.
        assert_eq!(mang_base("daya"), ("n", "daya".to_string()));
    }

    #[test]
    fn dr_sandhi_variant() {
        let d = v("dating");
        assert!(d.forms(AG, I).contains(&"dumarating".to_string()));
    }

    #[test]
    fn unknown_root_falls_back() {
        let x = v("blogblog");
        assert_eq!(x.form(AG, P), "bumlogblog"); // -um- default
    }

    #[test]
    fn rejects_non_roots() {
        for bad in ["", "  ", "mag sulat"] {
            assert_eq!(Verb::from_infinitive(bad).err(), Some(Error::NotAVerb));
        }
    }
}
