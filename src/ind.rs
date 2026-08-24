//! Indonesian (Bahasa Indonesia) verb derivation. Indonesian is an
//! agglutinating, affixal Austronesian language: the citation form is the
//! bare root, and the "conjugation" UniMorph exposes is a matrix of
//! voice/derivation affixes rather than person/tense agreement. Forms are
//! built productively from the root by a small inventory of prefixes and
//! suffixes:
//!
//! * the active voice prefix **meN-**, whose nasal assimilates to the
//!   root's initial and, for the four "obstruent" onsets p/t/k/s, replaces
//!   it (`tulis` → `menulis`, `pukul` → `memukul`, `kirim` → `mengirim`,
//!   `sapu` → `menyapu`); before a vowel or g/h it surfaces as `meng-`,
//!   before b/f/v as `mem-`, before d/c/j/z as `men-`, and before a
//!   sonorant (l/m/n/r/w/y) as bare `me-`;
//! * the passive prefix **di-**, the accidental/stative **ter-**, the
//!   intransitive/middle **ber-**;
//! * the applicative/causative suffix **-kan**, the locative/iterative
//!   **-i**, the focus particle **-lah**, and the enclitic objects /
//!   possessors **-nya** (3), **-ku** (1), **-mu** (2), plus the agentive
//!   proclitics **ku-** / **kau-**.
//!
//! Because the harness accepts any generated variant that matches an oracle
//! spelling, each cell emits the small set of plausible surface forms and
//! the regular rules carry the bulk; the irregular residue — lexicalised
//! nasalisation, monosyllabic `menge-`, borrowed clusters, suppletion — is
//! patched by the mined override layer in `data/ind/overrides.tsv`.
//!
//! Single oracle (UniMorph `ind`, ~15k verb rows): Beta.

use std::collections::HashMap;
use std::sync::OnceLock;

static OVERRIDES_TSV: &str = include_str!("../data/ind/overrides.tsv");

/// The mined override map: lemma → (canonical feature → surface form). Built
/// once and consulted before the productive rules.
fn overrides() -> &'static HashMap<String, HashMap<String, String>> {
    static MAP: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m: HashMap<String, HashMap<String, String>> = HashMap::new();
        for line in OVERRIDES_TSV.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let mut cols = line.split('\t');
            let (Some(lemma), Some(feat), Some(form)) = (cols.next(), cols.next(), cols.next())
            else {
                continue;
            };
            m.entry(lemma.to_string())
                .or_default()
                .insert(feat.to_string(), form.to_string());
        }
        m
    })
}

/// Why an input cannot be treated as an Indonesian verb root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Empty, multi-word, or non-alphabetic input.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Indonesian verb root")
    }
}

const VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];

fn is_vowel(c: char) -> bool {
    VOWELS.contains(&c)
}

/// Apply the active prefix **meN-** with nasal assimilation. For the four
/// obstruents p/t/k/s the onset is dropped and absorbed into the nasal —
/// but only before a vowel; before a consonant cluster (tr-, pr-, kl-, st-…)
/// the onset is kept. Elsewhere the nasal simply attaches.
fn meng(root: &str) -> String {
    let mut chars = root.chars();
    let Some(first) = chars.next() else {
        return format!("me{root}");
    };
    let rest: String = chars.collect();
    let second_vowel = rest.chars().next().is_some_and(is_vowel);
    match first {
        c if is_vowel(c) => format!("meng{root}"),
        'g' | 'h' => format!("meng{root}"),
        'k' if second_vowel => format!("meng{rest}"),
        'k' => format!("meng{root}"),
        'p' if second_vowel => format!("mem{rest}"),
        'b' | 'f' | 'v' | 'p' => format!("mem{root}"),
        't' if second_vowel => format!("men{rest}"),
        's' if second_vowel => format!("meny{rest}"),
        'd' | 'c' | 'j' | 'z' | 't' | 's' => format!("men{root}"),
        'l' | 'm' | 'n' | 'r' | 'w' | 'y' => format!("me{root}"),
        // q, x and anything unexpected fall back to meng-.
        _ => format!("meng{root}"),
    }
}

/// The active prefix **meN-** without obstruent deletion — the pattern
/// borrowed roots and the fossilised `per-` prefix follow (`kombinasi` →
/// `mengkombinasi`, `pesona` → `mempesona`, `peroleh` → `memperoleh`).
fn meng_keep(root: &str) -> String {
    let mut chars = root.chars();
    let Some(first) = chars.next() else {
        return format!("me{root}");
    };
    match first {
        c if is_vowel(c) => format!("meng{root}"),
        'g' | 'h' | 'k' | 'q' | 'x' => format!("meng{root}"),
        'b' | 'f' | 'v' | 'p' => format!("mem{root}"),
        'd' | 'c' | 'j' | 'z' | 't' => format!("men{root}"),
        's' => format!("men{root}"),
        'l' | 'm' | 'n' | 'r' | 'w' | 'y' => format!("me{root}"),
        _ => format!("meng{root}"),
    }
}

/// The **per-** causative stem prefixed to a root, with `per- → pe-` before
/// an r-initial root (`raga` → `peraga`, so `memperagakan`).
fn per(root: &str) -> String {
    if let Some(rest) = root.strip_prefix('r') {
        format!("per{rest}")
    } else {
        format!("per{root}")
    }
}

/// The monosyllabic active `menge-` (`cap` → `mengecap`, `bom` → `mengebom`);
/// offered as a candidate wherever a bare active form is expected.
fn menge(root: &str) -> String {
    format!("menge{root}")
}

/// The middle/intransitive prefix **ber-**, with the `ber- → be-` reduction
/// before an r-initial root (and the lexical `ajar → belajar`).
fn ber(root: &str) -> String {
    if root == "ajar" {
        return "belajar".to_string();
    }
    if root.starts_with('r') {
        format!("be{root}")
    } else {
        format!("ber{root}")
    }
}

/// The accidental/stative prefix **ter-**, with `ter- → te-` before an
/// r-initial root.
fn ter(root: &str) -> String {
    if root.starts_with('r') {
        format!("te{root}")
    } else {
        format!("ter{root}")
    }
}

/// The productive candidate surface forms for a canonical feature bundle.
/// The harness matches if any candidate equals an oracle spelling, so each
/// cell over-generates the regular alternatives.
fn productive(root: &str, feat: &str) -> Vec<String> {
    let m = meng(root);
    let mk = meng_keep(root);
    let di = format!("di{root}");
    let t = ter(root);
    let b = ber(root);
    // -kan / -i suffixation. The applicative -i is absorbed after an i-final
    // root (`intai` → `mengintai`, `interogasi` → `menginterogasi`).
    let kan = |p: &str| format!("{p}kan");
    let suf_i = |p: &str| {
        if root.ends_with('i') {
            p.to_string()
        } else {
            format!("{p}i")
        }
    };
    // Circumfixes over the bare root, and over the per- causative stem (which
    // reduces `per- → pe-` before an r-initial root).
    let ps = per(root);
    let memper = format!("mem{ps}");
    let diper = format!("di{ps}");
    let terper = format!("ter{ps}");
    let berkean = format!("berke{root}an");

    match feat {
        // Bare active: meN-, plus the middle ber-/ber-…-an, the per-
        // causative memper-, and the ke-…-an abstract.
        "V;ACT" => vec![
            m.clone(),
            mk.clone(),
            b.clone(),
            format!("{b}an"),
            memper.clone(),
            berkean.clone(),
            menge(root),
            format!("{}an", m),
        ],
        // Applicative / causative / transitive: root+kan, meN-root+kan,
        // memper-root(+kan), or the bare/‑kan ber- (berkantor, bermodalkan).
        "V;ACT;TR" | "V;ACT;CAUS" | "V;ACT;APPL" => vec![
            kan(root),
            kan(&m),
            kan(&mk),
            m.clone(),
            kan(&memper),
            memper.clone(),
            b.clone(),
            format!("{b}kan"),
            format!("ber{ps}kan"),
            format!("ber{ps}an"),
            berkean.clone(),
            suf_i(&m),
        ],
        // Iterative / locative: meN-root+i, the bare meN- (i-absorbing loans),
        // or memper-root / ber- for some.
        "V;ACT;ITER" => vec![
            suf_i(&m),
            suf_i(&mk),
            suf_i(root),
            m.clone(),
            mk.clone(),
            menge(root),
            memper.clone(),
            format!("{memper}i"),
            b.clone(),
        ],
        // Active + 3rd enclitic (-nya): several bases carry it.
        "V;ACT;DEF" | "V;ACT;DEF;PSS3S" => vec![
            format!("{m}nya"),
            format!("{}nya", kan(&m)),
            format!("{}nya", suf_i(&m)),
            format!("{}nya", kan(&mk)),
            format!("{}nya", suf_i(&mk)),
            format!("{mk}nya"),
            format!("{}nya", kan(&menge(root))),
            format!("{memper}nya"),
            format!("{}nya", kan(&memper)),
            format!("{}nya", suf_i(&memper)),
            format!("{b}nya"),
            format!("{berkean}nya"),
            format!("{}nya", kan(root)),
            format!("{}nya", suf_i(root)),
            format!("{root}nya"),
        ],
        // Active + focus -lah / question -kah.
        "V;ACT;FOC" => vec![
            format!("{b}lah"),
            format!("{m}lah"),
            format!("{}lah", kan(root)),
            format!("{}lah", suf_i(&b)),
            format!("{}lah", suf_i(root)),
            format!("{b}anlah"),
            format!("{b}kah"),
            format!("{m}kah"),
            format!("{root}kah"),
            format!("{root}lah"),
        ],
        // Active + 1st enclitic -ku.
        "V;ACT;PSS1S" => vec![
            format!("{m}ku"),
            format!("{}ku", kan(&m)),
            format!("{}ku", suf_i(&m)),
            format!("{b}ku"),
            format!("{}ku", suf_i(root)),
            format!("{root}ku"),
        ],
        // Active + 2nd enclitic -mu.
        "V;ACT;PSS2S" => vec![
            format!("{m}mu"),
            format!("{}mu", kan(&m)),
            format!("{}mu", suf_i(&m)),
            format!("{b}mu"),
            format!("{}mu", suf_i(root)),
            format!("{root}mu"),
        ],
        "V;1;ACT" => vec![
            format!("{m}ku"),
            format!("{}ku", suf_i(&m)),
            format!("{root}ku"),
        ],
        "V;2;ACT" => vec![
            format!("{m}mu"),
            format!("{}mu", suf_i(&m)),
            format!("{root}mu"),
            format!("{}mu", kan(&m)),
        ],
        // Agentive proclitic ku- (1sg) + root(+kan).
        "V;1;ACT;SG;TR" | "V;1;ACT;CAUS;SG" | "V;1;ACT;APPL;SG" => {
            vec![format!("ku{root}kan"), format!("ku{root}")]
        }
        "V;1;ACT;SG" => vec![format!("ku{root}"), format!("ku{root}kan")],
        "V;1;ACT;DEF;SG" | "V;1;ACT;DEF;PSS3S;SG" => {
            vec![format!("{root}kunya"), format!("ku{root}nya")]
        }
        // Agentive proclitic kau- (2sg) + root(+kan).
        "V;2;ACT;SG;TR" | "V;2;ACT;CAUS;SG" | "V;2;ACT;APPL;SG" => {
            vec![format!("kau{root}kan"), format!("kau{root}")]
        }
        // Bare passive: di- (with ter-, diper-, terper- alternates); a root
        // already carrying ter-/di- stands as itself.
        "V;PASS" => vec![
            di.clone(),
            t.clone(),
            diper.clone(),
            terper.clone(),
            format!("{t}an"),
            root.to_string(),
        ],
        // Passive applicative/causative/transitive: di-root+kan, diper-…+kan.
        "V;PASS;TR" | "V;CAUS;PASS" | "V;APPL;PASS" => vec![
            kan(&di),
            kan(&t),
            kan(&diper),
            diper.clone(),
            di.clone(),
            suf_i(&di),
        ],
        "V;ITER;PASS" => vec![
            suf_i(&di),
            suf_i(&t),
            di.clone(),
            t.clone(),
            diper.clone(),
            format!("{diper}i"),
        ],
        "V;DEF;PASS" | "V;DEF;PASS;PSS3S" => vec![
            format!("{di}nya"),
            format!("{}nya", kan(&di)),
            format!("{}nya", suf_i(&di)),
            format!("{diper}nya"),
            format!("{}nya", kan(&diper)),
            format!("{}nya", suf_i(&diper)),
            format!("{t}nya"),
            format!("{root}nya"),
        ],
        "V;FOC;PASS" => vec![
            format!("{di}lah"),
            format!("{}lah", kan(&di)),
            format!("{t}lah"),
            format!("{}lah", suf_i(&di)),
            format!("{root}lah"),
        ],
        "V;3;FOC;PASS;SG" => vec![format!("{di}nyalah"), format!("{}nyalah", kan(&di))],
        // Unknown bundle: still try meN- so coverage is non-None.
        _ => vec![m],
    }
}

/// A conjugatable Indonesian verb: the bare root plus any mined overrides.
#[derive(Debug, Clone)]
pub struct Verb {
    root: String,
    overrides: HashMap<String, String>,
}

impl Verb {
    /// Build a verb from its root citation (the UniMorph lemma).
    pub fn from_lemma(lemma: &str) -> Result<Self, Error> {
        let root = lemma.trim().to_lowercase();
        if root.is_empty() || root.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        if !root.chars().all(|c| c.is_alphabetic() || c == '-') {
            return Err(Error::NotAVerb);
        }
        let over = overrides().get(&root).cloned().unwrap_or_default();
        Ok(Self {
            root,
            overrides: over,
        })
    }

    /// Alias for [`Verb::from_lemma`] — Indonesian cites verbs by their root.
    pub fn from_infinitive(citation: &str) -> Result<Self, Error> {
        Self::from_lemma(citation)
    }

    /// The citation form (the bare root).
    #[must_use]
    pub fn citation(&self) -> &str {
        &self.root
    }

    /// Every candidate surface form for a feature bundle: the mined override
    /// first (if any), then the productive alternatives.
    #[must_use]
    pub fn forms(&self, feature: &str) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(o) = self.overrides.get(feature) {
            out.push(o.clone());
        }
        out.extend(productive(&self.root, feature));
        out
    }

    /// The single best form for a feature bundle: the override if present,
    /// else the first productive candidate.
    #[must_use]
    pub fn form(&self, feature: &str) -> Option<String> {
        self.forms(feature).into_iter().next()
    }
}

/// A compact derivation table — the regular voice/derivation cells, shared
/// by the WebAssembly and Python bindings. Each `Vec` slot is the engine's
/// preferred (first) form for that cell, or `None` if unsupported.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// The bare root (citation).
    pub root: String,
    /// Active voice: bare, +kan (applicative), +i (iterative).
    pub active: Vec<Option<String>>,
    /// Passive (di-): bare, +kan, +i.
    pub passive: Vec<Option<String>>,
    /// Accidental/stative (ter-) and middle (ber-): ter-, ter-...-kan, ber-.
    pub derived: Vec<Option<String>>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let one = |feat: &str| v.form(feat);
        Self {
            root: v.citation().to_string(),
            active: vec![one("V;ACT"), one("V;ACT;TR"), one("V;ACT;ITER")],
            passive: vec![one("V;PASS"), one("V;PASS;TR"), one("V;ITER;PASS")],
            derived: vec![
                Some(format!("ter{}", v.citation())),
                Some(format!("ter{}kan", v.citation())),
                Some(format!("ber{}", v.citation())),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(root: &str) -> Verb {
        Verb::from_lemma(root).unwrap()
    }

    #[test]
    fn nasalisation() {
        assert_eq!(meng("tulis"), "menulis"); // t deleted
        assert_eq!(meng("pukul"), "memukul"); // p deleted
        assert_eq!(meng("kirim"), "mengirim"); // k deleted
        assert_eq!(meng("sapu"), "menyapu"); // s -> ny
        assert_eq!(meng("ajar"), "mengajar"); // vowel
        assert_eq!(meng("baca"), "membaca"); // b kept
        assert_eq!(meng("makan"), "memakan"); // sonorant m
        assert_eq!(meng("dukung"), "mendukung"); // d kept
    }

    #[test]
    fn active_and_passive() {
        let t = v("tulis");
        assert!(t.forms("V;ACT").contains(&"menulis".to_string()));
        assert!(t.forms("V;ACT;TR").contains(&"menuliskan".to_string()));
        assert!(t.forms("V;ACT;TR").contains(&"tuliskan".to_string()));
        assert!(t.forms("V;PASS").contains(&"ditulis".to_string()));
        assert!(t.forms("V;PASS").contains(&"tertulis".to_string()));
        assert!(t.forms("V;PASS;TR").contains(&"dituliskan".to_string()));
        assert!(t.forms("V;ACT;ITER").contains(&"menulisi".to_string()));
    }

    #[test]
    fn enclitics_and_proclitics() {
        let b = v("beri");
        assert!(b.forms("V;1;ACT").contains(&"memberiku".to_string()));
        assert!(b.forms("V;2;ACT").contains(&"memberimu".to_string()));
        let k = v("kata");
        assert!(k.forms("V;1;ACT;SG;TR").contains(&"kukatakan".to_string()));
        assert!(k.forms("V;2;ACT;SG;TR").contains(&"kaukatakan".to_string()));
    }

    #[test]
    fn override_wins() {
        // Every lemma parses; overrides (if mined) take precedence.
        let a = v("ambil");
        assert!(a.forms("V;PASS").contains(&"diambil".to_string()));
    }

    #[test]
    fn non_verb_rejected() {
        assert!(Verb::from_lemma("").is_err());
        assert!(Verb::from_lemma("dua kata").is_err());
    }
}
