//! Irish conjugation. Two conjugation classes with broad/slender
//! endings (glanann/brisheann-type vs ceannaíonn/bailíonn-type), all
//! parsed off the mined present base in `data/gle/verbs.tsv`; the
//! remaining tenses derive from four stems — present (glan-), past
//! base, future base (glanfaidh → glanfadh conditional) and the
//! t-stem of the autonomous (glantar → glantá/glantaí). Forms are
//! unmutated citation forms, the shared oracle convention; `lenite`
//! applies the display mutation (glan → ghlan, ól → d'ól).

use std::collections::HashMap;
use std::sync::OnceLock;

/// The synthetic tenses/moods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    Present,
    Past,
    PastHabitual,
    Future,
    Conditional,
    Imperative,
    Subjunctive,
}

/// The person slots Irish inflects synthetically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    /// The base form used with pronouns (glanann sé).
    Base,
    FirstSingular,
    SecondSingular,
    FirstPlural,
    SecondPlural,
    ThirdPlural,
    /// The autonomous form (glantar).
    Autonomous,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like an Irish verb lemma.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an Irish verb")
    }
}

/// Mined row: lemma, present base, past base, future base, verbal
/// noun, verbal adjective, imperative 2sg, imperative 2pl, present
/// autonomous ("-" = derive).
static VERBS_TSV: &str = include_str!("../data/gle/verbs.tsv");

#[derive(Debug, Clone, Default)]
struct Row {
    prs_base: Option<String>,
    pst_base: Option<String>,
    fut_base: Option<String>,
    vn: Option<String>,
    ptcp: Option<String>,
    imp2: Option<String>,
    imp2pl: Option<String>,
    prs_auto: Option<String>,
}

fn rows() -> &'static HashMap<&'static str, Row> {
    static MAP: OnceLock<HashMap<&'static str, Row>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for line in VERBS_TSV.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            let opt = |i: usize| {
                cols.get(i)
                    .filter(|c| **c != "-" && !c.is_empty())
                    .map(|c| (*c).to_string())
            };
            m.insert(
                cols[0],
                Row {
                    prs_base: opt(1),
                    pst_base: opt(2),
                    fut_base: opt(3),
                    vn: opt(4),
                    ptcp: opt(5),
                    imp2: opt(6),
                    imp2pl: opt(7),
                    prs_auto: opt(8),
                },
            );
        }
        m
    })
}

/// The parsed shape of the present base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// glanann — first conjugation, broad.
    OneBroad,
    /// briseann — first conjugation, slender.
    OneSlender,
    /// ceannaíonn — second conjugation, broad.
    TwoBroad,
    /// bailíonn — second conjugation, slender.
    TwoSlender,
    /// brúnn — monosyllabic vowel stem (brúigh, báigh).
    Vowel,
}

fn broad_final(s: &str) -> bool {
    for c in s.chars().rev() {
        match c {
            'a' | 'o' | 'u' | 'á' | 'ó' | 'ú' => return true,
            'e' | 'i' | 'é' | 'í' => return false,
            _ => {}
        }
    }
    true
}

/// A conjugatable Irish verb.
#[derive(Debug, Clone)]
pub struct Verb {
    lemma: String,
    class: Class,
    stem: String,
    row: Row,
}

impl Verb {
    /// Build a verb from its citation form (the 2sg imperative:
    /// glan, ceannaigh).
    pub fn from_infinitive(lemma: &str) -> Result<Self, Error> {
        let lemma = lemma.trim();
        if lemma.is_empty()
            || lemma.contains(char::is_whitespace)
            || !lemma
                .chars()
                .all(|c| c.is_alphabetic() || c == '-' || c == '\'')
        {
            return Err(Error::NotAVerb);
        }
        let row = rows().get(lemma).cloned().unwrap_or_default();
        // Parse the class off the present base; fall back to the
        // lemma shape (-aigh/-igh are second conjugation).
        let default_prs = if let Some(b) = lemma.strip_suffix("aigh") {
            format!("{b}aíonn")
        } else if let Some(b) = lemma.strip_suffix("igh") {
            format!("{b}íonn")
        } else if broad_final(lemma) {
            format!("{lemma}ann")
        } else {
            format!("{lemma}eann")
        };
        let prs = row.prs_base.clone().unwrap_or(default_prs);
        let (class, stem) = if let Some(b) = lemma.strip_suffix("éigh") {
            // téigh, léigh, pléigh: é-stems with slender endings.
            (Class::Vowel, format!("{b}é"))
        } else if let Some(s) = prs.strip_suffix("aíonn") {
            (Class::TwoBroad, s.to_string())
        } else if let Some(s) = prs.strip_suffix("íonn") {
            (Class::TwoSlender, s.to_string())
        } else if let Some(s) = prs.strip_suffix("eann") {
            (Class::OneSlender, s.to_string())
        } else if let Some(s) = prs.strip_suffix("ann") {
            (Class::OneBroad, s.to_string())
        } else if let Some(s) = prs
            .strip_suffix("nn")
            .filter(|s| s.ends_with(['á', 'é', 'í', 'ó', 'ú', 'o', 'e']))
        {
            (Class::Vowel, s.to_string())
        } else if broad_final(&prs) {
            // Unparseable mined base (deir): it is itself the stem.
            (Class::OneBroad, prs.clone())
        } else {
            (Class::OneSlender, prs.clone())
        };
        Ok(Self {
            lemma: lemma.to_string(),
            class,
            stem,
            row,
        })
    }

    /// The citation form as normalized.
    pub fn infinitive(&self) -> &str {
        &self.lemma
    }

    /// Class-indexed ending: [OneBroad, OneSlender, TwoBroad,
    /// TwoSlender, Vowel].
    fn e(&self, five: [&str; 5]) -> String {
        let i = match self.class {
            Class::OneBroad => 0,
            Class::OneSlender => 1,
            Class::TwoBroad => 2,
            Class::TwoSlender => 3,
            Class::Vowel => 4,
        };
        format!("{}{}", self.stem, five[i])
    }

    /// The past base (glan, cheannaigh unmutated: ceannaigh).
    fn pst_base(&self) -> String {
        self.row.pst_base.clone().unwrap_or_else(|| {
            if self.class == Class::Vowel {
                self.lemma.clone()
            } else {
                self.e(["", "", "aigh", "igh", ""])
            }
        })
    }

    /// The future base (glanfaidh, ceannóidh).
    fn fut_base(&self) -> String {
        self.row
            .fut_base
            .clone()
            .unwrap_or_else(|| self.e(["faidh", "fidh", "óidh", "eoidh", "faidh"]))
    }

    /// The conditional base off the future (glanfaidh → glanfadh,
    /// brisfidh → brisfeadh, ceannóidh → ceannódh).
    fn cond_base(&self) -> String {
        let f = self.fut_base();
        if f.ends_with("faidh") || f.ends_with("óidh") || f.ends_with("eoidh") {
            format!("{}dh", f.strip_suffix("idh").unwrap_or(&f))
        } else if let Some(b) = f.strip_suffix("fidh") {
            format!("{b}feadh")
        } else {
            format!("{}dh", f.strip_suffix("idh").unwrap_or(&f))
        }
    }

    /// The present autonomous (glantar) and its t-stem, which drives
    /// the habitual 2sg and autonomous (glantá, glantaí).
    fn prs_auto(&self) -> String {
        self.row
            .prs_auto
            .clone()
            .unwrap_or_else(|| self.e(["tar", "tear", "aítear", "ítear", "itear"]))
    }

    /// A finite form; None where Irish has no synthetic slot.
    pub fn form(&self, tense: Tense, slot: Slot) -> Option<String> {
        use Slot::*;
        use Tense::*;
        let auto = self.prs_auto();
        let t_base = auto.strip_suffix("ar").unwrap_or(&auto).to_string();
        // t_base ends -t (broad: glant) or -te (slender: briste).
        let t_slender = t_base.ends_with('e') || t_base.ends_with("eá");
        // Suppletive past-plural stems of the famous irregulars.
        let pst_pl: Option<&str> = match self.lemma.as_str() {
            "tar" => Some("tánga"),
            "clois" | "cluin" => Some("cuala"),
            "téigh" => Some("cua"),
            "abair" => Some("dúra"),
            _ => None,
        };
        let pst = self.pst_base();
        let fut = self.fut_base();
        let cond = self.cond_base();
        Some(match (tense, slot) {
            (Present, Base) => self
                .row
                .prs_base
                .clone()
                .unwrap_or_else(|| self.e(["ann", "eann", "aíonn", "íonn", "nn"])),
            (Present, FirstSingular) => self.e(["aim", "im", "aím", "ím", "im"]),
            (Present, FirstPlural) => self.e(["aimid", "imid", "aímid", "ímid", "imid"]),
            (Present, Autonomous) => auto,
            (Past, Base) => pst,
            // With a mined past base the plurals ride it (rug →
            // rugamar/rugadar/rugadh); C2 bases in -aigh/-igh keep
            // their class endings (cheannaigh-type: ceannaíomar).
            (Past, FirstPlural) => match (pst_pl, &self.row.pst_base) {
                (Some(p), _) => format!("{p}mar"),
                (None, Some(b))
                    if !b.ends_with("igh")
                        && *b != self.lemma
                        && !b.ends_with(|c| "aeiouáéíóú".contains(c)) =>
                {
                    format!("{b}{}", if broad_final(b) { "amar" } else { "eamar" })
                }
                _ if self.class == Class::Vowel && self.stem.ends_with('é') => {
                    format!("{}amar", self.stem)
                }
                _ => self.e(["amar", "eamar", "aíomar", "íomar", "mar"]),
            },
            (Past, ThirdPlural) => match (pst_pl, &self.row.pst_base) {
                (Some(p), _) => format!("{p}dar"),
                (None, Some(b))
                    if !b.ends_with("igh")
                        && *b != self.lemma
                        && !b.ends_with(|c| "aeiouáéíóú".contains(c)) =>
                {
                    format!("{b}{}", if broad_final(b) { "adar" } else { "eadar" })
                }
                _ if self.class == Class::Vowel && self.stem.ends_with('é') => {
                    format!("{}adar", self.stem)
                }
                _ => self.e(["adar", "eadar", "aíodar", "íodar", "dar"]),
            },
            (Past, Autonomous) => match (pst_pl, &self.row.pst_base) {
                (Some(p), _) => match self.lemma.as_str() {
                    "tar" => "tángthas".to_string(),
                    "téigh" => "cuathas".to_string(),
                    "abair" => "dúradh".to_string(),
                    _ => format!("{p}thas"),
                },
                (None, Some(b))
                    if !b.ends_with("igh")
                        && *b != self.lemma
                        && !b.ends_with(|c| "aeiouáéíóú".contains(c)) =>
                {
                    format!("{b}{}", if broad_final(b) { "adh" } else { "eadh" })
                }
                _ if self.class == Class::Vowel && self.stem.ends_with('é') => {
                    format!("{}adh", self.stem)
                }
                _ => self.e(["adh", "eadh", "aíodh", "íodh", "dh"]),
            },
            (PastHabitual, Base) => self.e(["adh", "eadh", "aíodh", "íodh", "dh"]),
            (PastHabitual, FirstSingular) => self.e(["ainn", "inn", "aínn", "ínn", "inn"]),
            (PastHabitual, SecondSingular) => format!("{t_base}á"),
            (PastHabitual, FirstPlural) => self.e(["aimis", "imis", "aímis", "ímis", "imis"]),
            (PastHabitual, ThirdPlural) => self.e(["aidís", "idís", "aídís", "ídís", "idís"]),
            (PastHabitual, Autonomous) => {
                if t_slender {
                    format!("{}í", &t_base[..t_base.len() - 1])
                } else {
                    format!("{t_base}aí")
                }
            }
            (Future, Base) => fut,
            (Future, FirstPlural) => {
                format!("{}imid", fut.strip_suffix("idh").unwrap_or(&fut))
            }
            (Future, Autonomous) => {
                // glanfaidh → glanfar, brisfidh → brisfear,
                // ceannóidh → ceannófar, baileoidh → baileofar.
                if fut.ends_with("óidh") || fut.ends_with("eoidh") {
                    format!("{}far", &fut[..fut.len() - "idh".len()])
                } else if let Some(b) = fut.strip_suffix("aidh") {
                    format!("{b}ar")
                } else {
                    format!("{}ear", fut.strip_suffix("idh")?)
                }
            }
            (Conditional, Base) => cond,
            (Conditional, FirstSingular | FirstPlural | ThirdPlural | SecondSingular)
            | (Conditional, Autonomous) => {
                // All conditional persons ride the future base:
                // glanfaidh → glanfainn/glanfá/glanfaimis/glanfaidís/
                // glanfaí; brisfidh → brisfinn/brisfeá/brisfí;
                // ceannóidh → ceannóinn/ceannófá/ceannófaí.
                let (b, endings): (&str, [&str; 5]) = if let Some(b) = fut.strip_suffix("faidh")
                {
                    // keep the f with the base
                    let _ = b;
                    (
                        &fut[..fut.len() - "aidh".len()],
                        ["ainn", "á", "aimis", "aidís", "aí"],
                    )
                } else if fut.ends_with("óidh") || fut.ends_with("eoidh") {
                    (
                        &fut[..fut.len() - "idh".len()],
                        ["inn", "fá", "imis", "idís", "faí"],
                    )
                } else if fut.ends_with("fidh") {
                    (
                        &fut[..fut.len() - "idh".len()],
                        ["inn", "eá", "imis", "idís", "í"],
                    )
                } else {
                    return None;
                };
                let i = match slot {
                    FirstSingular => 0,
                    SecondSingular => 1,
                    FirstPlural => 2,
                    ThirdPlural => 3,
                    Autonomous => 4,
                    _ => return None,
                };
                format!("{b}{}", endings[i])
            }
            (Imperative, SecondSingular) => {
                self.row.imp2.clone().unwrap_or_else(|| self.lemma.clone())
            }
            (Imperative, Base) => self.e(["adh", "eadh", "aíodh", "íodh", "dh"]),
            (Imperative, FirstSingular) => self.e(["aim", "im", "aím", "ím", "im"]),
            (Imperative, FirstPlural) => self.e(["aimis", "imis", "aímis", "ímis", "imis"]),
            (Imperative, SecondPlural) => self
                .row
                .imp2pl
                .clone()
                .unwrap_or_else(|| self.e(["aigí", "igí", "aígí", "ígí", "igí"])),
            (Imperative, ThirdPlural) => self.e(["aidís", "idís", "aídís", "ídís", "idís"]),
            (Imperative, Autonomous) => auto,
            (Subjunctive, Base) => self.e(["a", "e", "aí", "í", ""]),
            (Subjunctive, Autonomous) => auto,
            _ => return None,
        })
    }

    /// The verbal noun (glanadh, ceannach — heavily lexical, mined).
    pub fn verbal_noun(&self) -> String {
        self.row
            .vn
            .clone()
            .unwrap_or_else(|| self.e(["adh", "eadh", "ú", "iú", "dh"]))
    }

    /// The verbal adjective (glanta, ceannaithe).
    pub fn verbal_adjective(&self) -> String {
        self.row
            .ptcp
            .clone()
            .unwrap_or_else(|| self.e(["ta", "te", "aithe", "ithe", "ite"]))
    }

    /// The display mutation of the past: lenite an initial consonant
    /// (glan → ghlan) or prefix d' before a vowel or fh (ól → d'ól).
    #[must_use]
    pub fn lenite(form: &str) -> String {
        let mut chars = form.chars();
        let Some(first) = chars.next() else {
            return form.to_string();
        };
        let rest: String = chars.collect();
        match first {
            'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú' => {
                format!("d'{form}")
            }
            'f' => format!("d'fh{rest}"),
            'b' | 'c' | 'd' | 'g' | 'm' | 'p' | 't' => {
                format!("{first}h{rest}")
            }
            's' if rest
                .chars()
                .next()
                .is_some_and(|c| "aeiouáéíóúlnr".contains(c)) =>
            {
                format!("sh{rest}")
            }
            _ => form.to_string(),
        }
    }
}

/// The full conjugation table of an Irish verb — shared by the
/// WebAssembly and Python bindings. Rows run [base, 1sg, 2sg, 1pl,
/// 2pl, 3pl, autonomous], with None for analytic-only slots.
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct Table {
    pub lemma: String,
    pub verbal_noun: String,
    pub verbal_adjective: String,
    pub present: [Option<String>; 7],
    pub past: [Option<String>; 7],
    pub past_habitual: [Option<String>; 7],
    pub future: [Option<String>; 7],
    pub conditional: [Option<String>; 7],
    pub imperative: [Option<String>; 7],
    pub subjunctive: [Option<String>; 7],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let slots = [
            Slot::Base,
            Slot::FirstSingular,
            Slot::SecondSingular,
            Slot::FirstPlural,
            Slot::SecondPlural,
            Slot::ThirdPlural,
            Slot::Autonomous,
        ];
        let row = |t: Tense| slots.map(|s| v.form(t, s));
        Self {
            lemma: v.infinitive().to_string(),
            verbal_noun: v.verbal_noun(),
            verbal_adjective: v.verbal_adjective(),
            present: row(Tense::Present),
            past: row(Tense::Past),
            past_habitual: row(Tense::PastHabitual),
            future: row(Tense::Future),
            conditional: row(Tense::Conditional),
            imperative: row(Tense::Imperative),
            subjunctive: row(Tense::Subjunctive),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(l: &str) -> Verb {
        Verb::from_infinitive(l).unwrap()
    }

    #[test]
    fn classes() {
        let g = v("glan");
        assert_eq!(g.form(Tense::Present, Slot::Base).unwrap(), "glanann");
        assert_eq!(
            g.form(Tense::Present, Slot::FirstSingular).unwrap(),
            "glanaim"
        );
        assert_eq!(g.form(Tense::Future, Slot::Base).unwrap(), "glanfaidh");
        assert_eq!(g.form(Tense::Conditional, Slot::Base).unwrap(), "glanfadh");
        assert_eq!(g.form(Tense::Present, Slot::Autonomous).unwrap(), "glantar");
        assert_eq!(
            g.form(Tense::PastHabitual, Slot::Autonomous).unwrap(),
            "glantaí"
        );
        let c = v("ceannaigh");
        assert_eq!(c.form(Tense::Present, Slot::Base).unwrap(), "ceannaíonn");
        assert_eq!(c.form(Tense::Future, Slot::Base).unwrap(), "ceannóidh");
        assert_eq!(c.form(Tense::Past, Slot::Base).unwrap(), "ceannaigh");
        let b = v("bris");
        assert_eq!(b.form(Tense::Present, Slot::Base).unwrap(), "briseann");
        assert_eq!(
            b.form(Tense::Present, Slot::Autonomous).unwrap(),
            "bristear"
        );
    }

    #[test]
    fn lenition() {
        assert_eq!(Verb::lenite("glan"), "ghlan");
        assert_eq!(Verb::lenite("ól"), "d'ól");
        assert_eq!(Verb::lenite("fág"), "d'fhág");
        assert_eq!(Verb::lenite("scríobh"), "scríobh");
        assert_eq!(Verb::lenite("suigh"), "shuigh");
    }
}
