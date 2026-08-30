//! Scottish Gaelic conjugation. Goidelic sibling of Irish, but with
//! no synthetic present (the present/progressive is periphrastic with
//! *bi* and out of scope). The synthetic core is derived off the
//! broad/slender shape of the imperative stem (= the lemma): future
//! `-aidh/-idh`, relative future `-as/-eas`, past (lenition only),
//! conditional/past-habitual `-adh/-eadh` with synthetic 1sg/1pl, the
//! imperative persons, and the impersonal/passive. The four lexical
//! principal parts — past, future, verbal noun, verbal adjective —
//! are mined in `data/gla/verbs.tsv`; a handful of suppletive verbs
//! are overridden from `data/gla/parts.tsv`. Forms are unmutated
//! citation forms (the shared oracle convention); `lenite` applies the
//! display mutation (glan → ghlan, òl → dh'òl).

use std::collections::HashMap;
use std::sync::OnceLock;

/// The synthetic tenses/moods Scottish Gaelic inflects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    Past,
    Future,
    /// The relative future (`a ghlanas`), a distinct synthetic form.
    RelativeFuture,
    /// The conditional / past-habitual (`ghlanadh`).
    Conditional,
    Imperative,
}

/// The person/voice slots. Scottish Gaelic inflects synthetically only
/// a few persons; the rest are analytic (verb + pronoun).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    /// The independent base form used with pronouns (glanaidh mi).
    Independent,
    /// The dependent form after a particle (cha ghlan, an glan).
    Dependent,
    FirstSingular,
    SecondSingular,
    FirstPlural,
    SecondPlural,
    /// The third-person / analytic base of the conditional and
    /// imperative (glanadh).
    Third,
    /// The impersonal / passive form (glanar, glanadh).
    Impersonal,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Scottish Gaelic verb lemma.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Scottish Gaelic verb")
    }
}

/// Mined principal parts: lemma, past, future, verbal noun, verbal
/// adjective ("-" = derive).
static VERBS_TSV: &str = include_str!("../data/gla/verbs.tsv");
/// Suppletive / irregular overrides: lemma, past, future, vn, ptcp,
/// fut-dep, pst-dep, cond-stem ("-" = fall back to the mined/derived
/// value). `cond-stem`, when present, replaces the lemma as the base
/// for the derived relative future, conditional and imperative forms.
static PARTS_TSV: &str = include_str!("../data/gla/parts.tsv");

#[derive(Debug, Clone, Default)]
struct Row {
    past: Option<String>,
    future: Option<String>,
    vn: Option<String>,
    ptcp: Option<String>,
    fut_dep: Option<String>,
    pst_dep: Option<String>,
    cond_stem: Option<String>,
}

fn parse_tsv(tsv: &str, cols_wanted: usize) -> HashMap<&str, Vec<Option<String>>> {
    let mut m = HashMap::new();
    for line in tsv.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let vals: Vec<Option<String>> = (1..cols_wanted)
            .map(|i| {
                cols.get(i)
                    .filter(|c| **c != "-" && !c.is_empty())
                    .map(|c| (*c).to_string())
            })
            .collect();
        m.insert(cols[0], vals);
    }
    m
}

fn rows() -> &'static HashMap<String, Row> {
    static MAP: OnceLock<HashMap<String, Row>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m: HashMap<String, Row> = HashMap::new();
        // Mined principal parts: past, future, vn, ptcp.
        for (lemma, v) in parse_tsv(VERBS_TSV, 5) {
            m.insert(
                lemma.to_string(),
                Row {
                    past: v[0].clone(),
                    future: v[1].clone(),
                    vn: v[2].clone(),
                    ptcp: v[3].clone(),
                    ..Row::default()
                },
            );
        }
        // Overrides win, field by field.
        for (lemma, v) in parse_tsv(PARTS_TSV, 8) {
            let row = m.entry(lemma.to_string()).or_default();
            if v[0].is_some() {
                row.past = v[0].clone();
            }
            if v[1].is_some() {
                row.future = v[1].clone();
            }
            if v[2].is_some() {
                row.vn = v[2].clone();
            }
            if v[3].is_some() {
                row.ptcp = v[3].clone();
            }
            row.fut_dep = v[4].clone();
            row.pst_dep = v[5].clone();
            row.cond_stem = v[6].clone();
        }
        m
    })
}

/// Broad if the final vowel of `s` is a/o/u (á/ó/ú/à/ò/ù), slender if
/// e/i (é/í/è/ì). Scottish Gaelic spelling agrees the ending with the
/// last vowel of the stem (*caol ri caol, leathann ri leathann*).
fn broad_final(s: &str) -> bool {
    for c in s.chars().rev() {
        match c {
            'a' | 'o' | 'u' | 'á' | 'ó' | 'ú' | 'à' | 'ò' | 'ù' => return true,
            'e' | 'i' | 'é' | 'í' | 'è' | 'ì' => return false,
            _ => {}
        }
    }
    true
}

/// A conjugatable Scottish Gaelic verb.
#[derive(Debug, Clone)]
pub struct Verb {
    lemma: String,
    /// The base for derived forms (usually the lemma; a suppletive
    /// override for a few irregular verbs).
    stem: String,
    broad: bool,
    row: Row,
}

impl Verb {
    /// Build a verb from its citation form (the 2sg imperative: glan,
    /// cuir, ceannaich).
    pub fn from_infinitive(lemma: &str) -> Result<Self, Error> {
        let lowered = lemma.to_lowercase();
        let lemma = lowered.trim();
        if lemma.is_empty()
            || lemma.contains(char::is_whitespace)
            || !lemma
                .chars()
                .all(|c| c.is_alphabetic() || c == '-' || c == '\'')
        {
            return Err(Error::NotAVerb);
        }
        let row = rows().get(lemma).cloned().unwrap_or_default();
        let stem = row.cond_stem.clone().unwrap_or_else(|| lemma.to_string());
        let broad = broad_final(&stem);
        Ok(Self {
            lemma: lemma.to_string(),
            stem,
            broad,
            row,
        })
    }

    /// The citation form as normalized.
    pub fn infinitive(&self) -> &str {
        &self.lemma
    }

    /// Append the broad or slender variant of an ending to the stem.
    fn e(&self, broad: &str, slender: &str) -> String {
        format!("{}{}", self.stem, if self.broad { broad } else { slender })
    }

    /// The past base (glan, chuir→cuir; suppletive chaidh→caidh mined).
    fn past_base(&self) -> String {
        self.row.past.clone().unwrap_or_else(|| self.lemma.clone())
    }

    /// The future base (glanaidh, cuiridh; suppletive gheibh mined).
    fn future_base(&self) -> String {
        self.row
            .future
            .clone()
            .unwrap_or_else(|| self.e("aidh", "idh"))
    }

    /// A finite form; None where Scottish Gaelic has no synthetic slot.
    pub fn form(&self, tense: Tense, slot: Slot) -> Option<String> {
        use Slot::*;
        use Tense::*;
        Some(match (tense, slot) {
            (Past, Independent) => self.past_base(),
            (Past, Dependent) => self.row.pst_dep.clone().unwrap_or_else(|| self.past_base()),
            (Past, Impersonal) => self.e("adh", "eadh"),
            (Future, Independent) => self.future_base(),
            (Future, Dependent) => self
                .row
                .fut_dep
                .clone()
                .unwrap_or_else(|| self.lemma.clone()),
            (Future, Impersonal) => self.e("ar", "ear"),
            (RelativeFuture, Independent) => self.e("as", "eas"),
            (Conditional, Third | Dependent) => self.e("adh", "eadh"),
            (Conditional, FirstSingular) => self.e("ainn", "inn"),
            (Conditional, FirstPlural) => self.e("amaid", "eamaid"),
            (Conditional, Impersonal) => self.e("tadh", "teadh"),
            (Imperative, SecondSingular) => self.lemma.clone(),
            (Imperative, FirstSingular) => self.e("am", "eam"),
            (Imperative, FirstPlural) => self.e("amaid", "eamaid"),
            (Imperative, SecondPlural) => self.e("aibh", "ibh"),
            (Imperative, Third) => self.e("adh", "eadh"),
            (Imperative, Impersonal) => self.e("ar", "ear"),
            _ => return None,
        })
    }

    /// The verbal noun (glanadh, cur — heavily lexical, mined).
    pub fn verbal_noun(&self) -> String {
        self.row.vn.clone().unwrap_or_else(|| self.e("adh", "eadh"))
    }

    /// The verbal adjective / past participle (glante, cuirte — mined).
    pub fn verbal_adjective(&self) -> String {
        self.row.ptcp.clone().unwrap_or_else(|| self.e("ta", "te"))
    }

    /// The display mutation of the past and conditional: lenite an
    /// initial consonant (glan → ghlan) or prefix dh' before a vowel or
    /// fh (òl → dh'òl, fosgail → dh'fhosgail).
    #[must_use]
    pub fn lenite(form: &str) -> String {
        let mut chars = form.chars();
        let Some(first) = chars.next() else {
            return form.to_string();
        };
        let rest: String = chars.collect();
        match first {
            'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'à' | 'è' | 'ì' | 'ò'
            | 'ù' => format!("dh'{form}"),
            'f' => format!("dh'fh{rest}"),
            'b' | 'c' | 'd' | 'g' | 'm' | 'p' | 't' => format!("{first}h{rest}"),
            's' if rest
                .chars()
                .next()
                .is_some_and(|c| "aeiouáéíóúàèìòùlnr".contains(c)) =>
            {
                format!("sh{rest}")
            }
            _ => form.to_string(),
        }
    }
}

/// The full conjugation table of a Scottish Gaelic verb — shared by
/// the WebAssembly and Python bindings. Rows run [1sg, 2sg, 3sg, 1pl,
/// 2pl, 3pl, impersonal], with None for slots that do not exist.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    #[cfg_attr(feature = "serde", serde(rename = "infinitive"))]
    pub lemma: String,
    pub verbal_noun: String,
    pub verbal_adjective: String,
    pub past: [Option<String>; 7],
    pub future: [Option<String>; 7],
    pub relative_future: [Option<String>; 7],
    pub conditional: [Option<String>; 7],
    pub imperative: [Option<String>; 7],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        Self {
            lemma: v.infinitive().to_string(),
            verbal_noun: v.verbal_noun(),
            verbal_adjective: v.verbal_adjective(),
            past: Self::row(v, Tense::Past),
            future: Self::row(v, Tense::Future),
            relative_future: Self::row(v, Tense::RelativeFuture),
            conditional: Self::row(v, Tense::Conditional),
            imperative: Self::row(v, Tense::Imperative),
        }
    }

    /// One display row per tense: [mi, thu, e/i, sinn, sibh, iad,
    /// impersonal]. Synthetic forms are used where they exist;
    /// everywhere else the analytic base composes with its pronoun. The
    /// past and conditional carry the initial lenition (glan → ghlan,
    /// òl → dh'òl).
    fn row(v: &Verb, t: Tense) -> [Option<String>; 7] {
        use Slot::*;
        const PRONOUNS: [&str; 6] = ["mi", "thu", "e/i", "sinn", "sibh", "iad"];
        let mutate = matches!(t, Tense::Past | Tense::Conditional);
        let mutated = |f: String| if mutate { Verb::lenite(&f) } else { f };
        // The analytic base: the 3sg / non-synthetic verb form.
        let base_slot = match t {
            Tense::Conditional | Tense::Imperative => Third,
            _ => Independent,
        };
        let base = v.form(t, base_slot).map(&mutated);
        let cell = |slot: Slot, i: usize| {
            v.form(t, slot)
                .map(&mutated)
                .or_else(|| base.clone().map(|b| format!("{b} {}", PRONOUNS[i])))
        };
        [
            cell(FirstSingular, 0),
            cell(SecondSingular, 1),
            base.clone().map(|b| format!("{b} {}", PRONOUNS[2])),
            cell(FirstPlural, 3),
            cell(SecondPlural, 4),
            base.clone().map(|b| format!("{b} {}", PRONOUNS[5])),
            v.form(t, Impersonal).map(&mutated),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(l: &str) -> Verb {
        Verb::from_infinitive(l).unwrap()
    }

    #[test]
    fn broad_regular() {
        let g = v("glan");
        assert_eq!(
            g.form(Tense::Future, Slot::Independent).unwrap(),
            "glanaidh"
        );
        assert_eq!(g.form(Tense::Past, Slot::Independent).unwrap(), "glan");
        assert_eq!(g.form(Tense::Conditional, Slot::Third).unwrap(), "glanadh");
        assert_eq!(g.verbal_noun(), "glanadh");
        // Display lenition.
        assert_eq!(Verb::lenite("glan"), "ghlan");
        let table = Table::build(&g);
        assert_eq!(table.past[0].as_deref(), Some("ghlan mi"));
    }

    #[test]
    fn slender_regular() {
        let c = v("cuir");
        assert_eq!(c.form(Tense::Past, Slot::Independent).unwrap(), "cuir");
        assert_eq!(c.form(Tense::Future, Slot::Independent).unwrap(), "cuiridh");
        assert_eq!(
            c.form(Tense::RelativeFuture, Slot::Independent).unwrap(),
            "cuireas"
        );
        assert_eq!(
            c.form(Tense::Conditional, Slot::FirstSingular).unwrap(),
            "cuirinn"
        );
        assert_eq!(
            c.form(Tense::Imperative, Slot::SecondPlural).unwrap(),
            "cuiribh"
        );
        assert_eq!(c.verbal_noun(), "cur");
        assert_eq!(Verb::lenite("cuir"), "chuir");
    }

    #[test]
    fn suppletive_faigh() {
        let f = v("faigh");
        assert_eq!(f.form(Tense::Future, Slot::Independent).unwrap(), "gheibh");
        assert_eq!(f.form(Tense::Past, Slot::Independent).unwrap(), "fuair");
    }

    #[test]
    fn lenition() {
        assert_eq!(Verb::lenite("glan"), "ghlan");
        assert_eq!(Verb::lenite("òl"), "dh'òl");
        assert_eq!(Verb::lenite("fosgail"), "dh'fhosgail");
        // s + a mute consonant does not lenite (sg-, sp-, st-, sm-).
        assert_eq!(Verb::lenite("sguab"), "sguab");
        assert_eq!(Verb::lenite("seas"), "sheas");
    }
}
