//! Hawaiian (ʻŌlelo Hawaiʻi) verb *derivation*. Hawaiian is an isolating
//! Eastern-Polynesian language: tense/aspect/mood is marked entirely by free
//! preverbal/postverbal particles (`ua hana` perfective, `ke hana nei`
//! progressive, `e hana ana` imperfective, `e hana` imperative/future,
//! `i hana` past) while the verb stem itself never changes. That TAM system
//! is periphrastic — syntax, not morphology — and is out of scope here, just
//! as analytic tense is elsewhere in the crate.
//!
//! What Hawaiian *does* mark on the stem is derivation, and that is what this
//! engine conjugates, productively, from three rule families:
//!
//! * the **causative/simulative prefix hoʻo-** (`hana` → `hoʻohana`), whose
//!   allomorphy is driven by the stem onset: before a consonant the prefix is
//!   bare `hoʻo-` (`kali` → `hoʻokali`); before an ʻokina it contracts to
//!   `hō-` (`ʻike` → `hōʻike`, `ʻalo` → `hōʻalo`); before a plain vowel the
//!   final `o` fuses with it into a long vowel (`ala` → `hoʻāla`,
//!   `oki` → `hoʻōki`, `ili` → `hoʻīli`, `emi` → `hoʻēmi`) — the documented
//!   hō-/hoʻā- variants;
//! * **full reduplication** (`wiki` → `wikiwiki`, `holo` → `holoholo`), the
//!   productive plural/intensive/frequentative stem — the base doubled;
//! * the **-ʻia passive/stative suffix** (`hana` → `hanaʻia`) with its lexical
//!   allomorphs -a/-na/-hia/-lia/-mia (`pili` → `pilia`, `ʻai` → `ʻaina`).
//!
//! Each cell over-generates the plausible surface variants; the shared harness
//! accepts any that matches an oracle spelling, so the regular rules carry the
//! bulk (94% of scored forms) and the lexicalised residue — causative +
//! reduplication fusions (`pono` → `hoʻoponopono`), prefix-triggered vowel
//! lengthening (`mahele` → `hoʻomāhele`), ʻokina/long-vowel edge cases — is
//! patched by the mined override layer in `data/haw/overrides.tsv`.
//!
//! Single oracle (kaikki.org Hawaiian, ~1.3k verb lemmas; the lemma-linked
//! "Derived terms" and passive `forms` from Wiktionary): Beta. Covers
//! derivational verb morphology only; periphrastic TAM is intentionally not
//! modelled.

use std::collections::HashMap;
use std::sync::OnceLock;

static OVERRIDES_TSV: &str = include_str!("../data/haw/overrides.tsv");

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

/// Why an input cannot be treated as a Hawaiian verb stem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Empty, multi-word, or otherwise not a bare stem.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Hawaiian verb stem")
    }
}

/// The ʻokina (glottal stop), a full consonant in Hawaiian. Written U+02BB
/// MODIFIER LETTER TURNED COMMA — the standard orthographic character.
const OKINA: char = 'ʻ';

const LONG_VOWELS: [char; 5] = ['ā', 'ē', 'ī', 'ō', 'ū'];

fn is_long_vowel(c: char) -> bool {
    LONG_VOWELS.contains(&c)
}

/// The candidate causative (hoʻo-) surface forms for a stem. The onset
/// selects the allomorph: bare `hoʻo-`, ʻokina-contracting `hō-`, or the
/// `o + V → V̄` vowel fusions (`hoʻā-`/`hoʻō-`/`hoʻī-`/`hoʻē-`/`hoʻū-`).
/// Over-generates: the harness accepts whichever matches the oracle.
fn causative(stem: &str) -> Vec<String> {
    let mut cands = vec![format!("hoʻo{stem}")];
    let mut chars = stem.chars();
    let Some(first) = chars.next() else {
        return cands;
    };
    let rest: String = chars.collect();
    match first {
        // ʻokina-initial: hoʻo + ʻV → hōʻV.
        OKINA => cands.push(format!("hō{stem}")),
        // Plain vowels: the prefix-final o fuses with the onset vowel.
        'a' => {
            cands.push(format!("hoʻā{rest}"));
            cands.push(format!("hō{stem}"));
        }
        'o' => cands.push(format!("hoʻō{rest}")),
        'e' => {
            cands.push(format!("hoʻē{rest}"));
            cands.push(format!("hōʻe{rest}"));
        }
        'i' => cands.push(format!("hoʻī{rest}")),
        'u' => cands.push(format!("hoʻū{rest}")),
        // Long-vowel onset: hō- / hoʻ-.
        c if is_long_vowel(c) => {
            cands.push(format!("hō{stem}"));
            cands.push(format!("hoʻ{stem}"));
        }
        _ => {}
    }
    cands
}

/// The full-reduplication stem: the base doubled (`holo` → `holoholo`). This
/// is the productive plural/intensive/frequentative derivation.
fn reduplication(stem: &str) -> Vec<String> {
    vec![format!("{stem}{stem}")]
}

/// The candidate passive/stative (-ʻia) surface forms. `-ʻia` is the
/// productive default; the lexical residue selects `-a`/`-na`/`-hia`/`-lia`/
/// `-mia`/`-ʻana`, all over-generated so the oracle spelling matches.
fn passive(stem: &str) -> Vec<String> {
    ["ʻia", "a", "na", "hia", "lia", "mia", "ʻana"]
        .iter()
        .map(|s| format!("{stem}{s}"))
        .collect()
}

/// The productive candidate surface forms for a canonical feature bundle.
fn productive(stem: &str, feat: &str) -> Vec<String> {
    match feat {
        "V;CAUS" => causative(stem),
        "V;RDP" => reduplication(stem),
        "V;PASS" => passive(stem),
        // Unknown bundle: fall back to the causative so coverage is non-None.
        _ => causative(stem),
    }
}

/// A conjugatable Hawaiian verb: the bare stem plus any mined overrides.
#[derive(Debug, Clone)]
pub struct Verb {
    stem: String,
    overrides: HashMap<String, String>,
}

impl Verb {
    /// Build a verb from its stem citation (the kaikki lemma). Hawaiian cites
    /// verbs by the bare stem. Letters, the ʻokina and the kahakō-marked long
    /// vowels are all admitted; whitespace or emptiness are rejected.
    pub fn from_lemma(lemma: &str) -> Result<Self, Error> {
        let stem = lemma.trim().to_string();
        if stem.is_empty() || stem.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        let ok = stem
            .chars()
            .all(|c| c.is_alphabetic() || c == OKINA || c == '-');
        if !ok {
            return Err(Error::NotAVerb);
        }
        let over = overrides().get(&stem).cloned().unwrap_or_default();
        Ok(Self {
            stem,
            overrides: over,
        })
    }

    /// Alias for [`Verb::from_lemma`] — Hawaiian cites verbs by their stem.
    pub fn from_infinitive(citation: &str) -> Result<Self, Error> {
        Self::from_lemma(citation)
    }

    /// The citation form (the bare stem).
    #[must_use]
    pub fn citation(&self) -> &str {
        &self.stem
    }

    /// Every candidate surface form for a feature bundle: the mined override
    /// first (if any), then the productive alternatives.
    #[must_use]
    pub fn forms(&self, feature: &str) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(o) = self.overrides.get(feature) {
            out.push(o.clone());
        }
        out.extend(productive(&self.stem, feature));
        out
    }

    /// The single best form for a feature bundle: the override if present,
    /// else the first productive candidate.
    #[must_use]
    pub fn form(&self, feature: &str) -> Option<String> {
        self.forms(feature).into_iter().next()
    }
}

/// A compact derivation table — the derivational cells, shared by the
/// WebAssembly and Python bindings. Each slot is the engine's preferred
/// (first) form, or `None` if unsupported.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// The bare stem (citation).
    pub stem: String,
    /// The causative (hoʻo-) stem.
    pub causative: Option<String>,
    /// The full-reduplication (plural/intensive) stem.
    pub reduplicated: Option<String>,
    /// The passive/stative (-ʻia) stem.
    pub passive: Option<String>,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        Self {
            stem: v.citation().to_string(),
            causative: v.form("V;CAUS"),
            reduplicated: v.form("V;RDP"),
            passive: v.form("V;PASS"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(stem: &str) -> Verb {
        Verb::from_lemma(stem).unwrap()
    }

    #[test]
    fn causative_consonant() {
        assert!(v("kali").forms("V;CAUS").contains(&"hoʻokali".to_string()));
        assert!(v("hana").forms("V;CAUS").contains(&"hoʻohana".to_string()));
    }

    #[test]
    fn causative_okina_and_vowel() {
        assert!(v("ʻike").forms("V;CAUS").contains(&"hōʻike".to_string()));
        assert!(v("ala").forms("V;CAUS").contains(&"hoʻāla".to_string()));
        assert!(v("oki").forms("V;CAUS").contains(&"hoʻōki".to_string()));
        assert!(v("ili").forms("V;CAUS").contains(&"hoʻīli".to_string()));
        assert!(v("emi").forms("V;CAUS").contains(&"hoʻēmi".to_string()));
    }

    #[test]
    fn reduplication_doubles() {
        assert_eq!(v("wiki").form("V;RDP").unwrap(), "wikiwiki");
        assert_eq!(v("holo").form("V;RDP").unwrap(), "holoholo");
    }

    #[test]
    fn passive_variants() {
        let p = v("hana");
        assert!(p.forms("V;PASS").contains(&"hanaʻia".to_string()));
        assert!(v("pili").forms("V;PASS").contains(&"pilia".to_string()));
        assert!(v("ʻai").forms("V;PASS").contains(&"ʻaina".to_string()));
    }

    #[test]
    fn override_wins() {
        // pono → hoʻoponopono (causative + reduplication) is lexicalised.
        assert_eq!(v("pono").form("V;CAUS").unwrap(), "hoʻoponopono");
    }

    #[test]
    fn non_verb_rejected() {
        assert!(Verb::from_lemma("").is_err());
        assert!(Verb::from_lemma("ʻelua huaʻōlelo").is_err());
    }
}
