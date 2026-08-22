//! Tamil (literary) verb conjugation: a small agglutinative engine that
//! stacks a person-number-gender ending onto a tense stem, plus a
//! compiled-in table of the stems that Tamil does not let you predict
//! from the root.
//!
//! A Tamil verb is cited by its root (செய் "do", படி "study", வா
//! "come"). Three synthetic tenses each take a stem and the same set of
//! PNG endings:
//!
//! - the **present** stem ends in the marker கிற் (செய்கிற்,
//!   படிக்கிற்); the endings give செய்கிறேன், படிக்கிறான்;
//! - the **past** stem is the unpredictable one — its marker is
//!   lexical (செய்த், வந்த், கேட்ட், ஓடின், படித்த்) — and takes the
//!   same endings (செய்தேன், வந்தான்);
//! - the **future** stem ends in வ் (weak, செய்வ்) or ப்ப்/ப் (strong,
//!   படிப்ப், கேட்ப்): செய்வேன், படிப்பான். Its neuter is the separate
//!   -உம் form (செய்யும், படிக்கும்), which doubles as the future
//!   relative participle.
//!
//! The endings are constant: 1sg -ஏன், 1pl -ஓம், 2sg -ஆய், 2pl
//! -ஈர்கள், 3sg -ஆன்/-ஆள்/-ஆர் (m/f/honorific), 3sg neuter -அது, 3pl
//! -ஆர்கள் (epicene) / -அ (neuter). They begin with an independent
//! vowel and attach to a stem ending in a pure consonant (a pulli
//! ஂ்), so the join is Tamil's ordinary orthographic sandhi: drop the
//! pulli and write the vowel as its dependent sign (செய்த் + ஏன் →
//! செய்தேன்).
//!
//! Two past sub-patterns differ off the finite grid. The -இன் class
//! (ஓடின், அகற்றின்) builds its neuter, past relative participle and
//! adverbial participle on the bare -இ stem with a ய glide (ஓடியது,
//! ஓடிய, ஓடி), where the consonantal pasts add -உ/-அ directly (செய்து,
//! செய்த). Everything a root does not derive — the whole stem set —
//! lives in `data/tam/verbs.tsv`, mined from the two-oracle agreement.
//!
//! Negation and the perfect/progressive/modal periphrases are analytic
//! (auxiliary verbs and the negator இல்லை) and out of this synthetic
//! core, as they are for the oracles.

use std::collections::HashMap;
use std::sync::OnceLock;

/// One of the three synthetic tenses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    Past,
    Present,
    Future,
}

/// A person-number-gender slot. Tamil's third person splits by gender
/// (masculine, feminine), an honorific, a neuter, and — in the plural —
/// an epicene (rational) versus a neuter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Png {
    P1Sg,
    P1Pl,
    P2Sg,
    P2Pl,
    P3SgM,
    P3SgF,
    P3SgH,
    P3SgN,
    P3PlE,
    P3PlN,
}

/// A relative (adjectival) participle, one per tense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relative {
    Past,
    Present,
    Future,
}

/// Number, for the imperative (singular familiar vs plural/polite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Singular,
    Plural,
}

/// Why a root cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input is empty or is not a single word.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Tamil verb root")
    }
}

const PULLI: char = '\u{0BCD}';

/// Independent vowel → its dependent sign (அ carries no sign; it is the
/// consonant's inherent vowel).
fn vowel_sign(v: char) -> Option<&'static str> {
    Some(match v {
        '\u{0B85}' => "",         // அ
        '\u{0B86}' => "\u{0BBE}", // ஆ → ா
        '\u{0B87}' => "\u{0BBF}", // இ → ி
        '\u{0B88}' => "\u{0BC0}", // ஈ → ீ
        '\u{0B89}' => "\u{0BC1}", // உ → ு
        '\u{0B8A}' => "\u{0BC2}", // ஊ → ூ
        '\u{0B8E}' => "\u{0BC6}", // எ → ெ
        '\u{0B8F}' => "\u{0BC7}", // ஏ → ே
        '\u{0B90}' => "\u{0BC8}", // ஐ → ை
        '\u{0B92}' => "\u{0BCA}", // ஒ → ொ
        '\u{0B93}' => "\u{0BCB}", // ஓ → ோ
        '\u{0B94}' => "\u{0BCC}", // ஔ → ௌ
        _ => return None,
    })
}

/// Attach a suffix to a stem across a Tamil consonant–vowel boundary.
///
/// A tense stem ends in a pure consonant (a pulli); a suffix begins with
/// an independent vowel. The join drops the pulli and writes that vowel
/// as its dependent sign: செய்த் + ஏன் → செய்தேன், கிற் + அது → கிறது.
/// If the stem is not pulli-final, or the suffix does not begin with an
/// independent vowel, the two are simply concatenated.
fn join(stem: &str, suffix: &str) -> String {
    let mut first = suffix.chars();
    let v = first.next();
    let sign = v.and_then(vowel_sign);
    if let (true, Some(sign)) = (stem.ends_with(PULLI), sign) {
        let base = &stem[..stem.len() - PULLI.len_utf8()];
        format!("{base}{sign}{}", first.as_str())
    } else {
        format!("{stem}{suffix}")
    }
}

/// The PNG endings, shared by all three tenses. Third-person neuter and
/// plural-neuter are handled off this table (present neuter plural takes
/// the கின்ற allomorph; the future neuter is the separate -உம் form).
fn ending(png: Png) -> &'static str {
    match png {
        Png::P1Sg => "\u{0B8F}\u{0BA9}\u{0BCD}", // ஏன்
        Png::P1Pl => "\u{0B93}\u{0BAE}\u{0BCD}", // ஓம்
        Png::P2Sg => "\u{0B86}\u{0BAF}\u{0BCD}", // ஆய்
        Png::P2Pl => "\u{0B88}\u{0BB0}\u{0BCD}\u{0B95}\u{0BB3}\u{0BCD}", // ஈர்கள்
        Png::P3SgM => "\u{0B86}\u{0BA9}\u{0BCD}", // ஆன்
        Png::P3SgF => "\u{0B86}\u{0BB3}\u{0BCD}", // ஆள்
        Png::P3SgH => "\u{0B86}\u{0BB0}\u{0BCD}", // ஆர்
        Png::P3SgN => "\u{0B85}\u{0BA4}\u{0BC1}", // அது
        Png::P3PlE => "\u{0B86}\u{0BB0}\u{0BCD}\u{0B95}\u{0BB3}\u{0BCD}", // ஆர்கள்
        Png::P3PlN => "\u{0B85}\u{0BA9}",        // அன
    }
}

const KIRRU: &str = "\u{0B95}\u{0BBF}\u{0BB1}\u{0BCD}"; // கிற்
const KINRRU: &str = "\u{0B95}\u{0BBF}\u{0BA9}\u{0BCD}\u{0BB1}\u{0BCD}"; // கின்ற்
const INN: &str = "\u{0BBF}\u{0BA9}\u{0BCD}"; // ‑ின் (the -இன் past marker's tail, ி + ன்)

static LEXICON_TSV: &str = include_str!("../data/tam/verbs.tsv");

/// A stored stem set. Every column is optional; a `-` falls through to
/// the productive (weak) default derived from the root.
#[derive(Debug, Clone, Default)]
struct LexEntry {
    present: Option<String>,
    past: Option<String>,
    future: Option<String>,
    um: Option<String>,
    infinitive: Option<String>,
    imperative_pl: Option<String>,
}

fn opt(s: &str) -> Option<String> {
    (s != "-" && !s.is_empty()).then(|| s.to_string())
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
            let g = |i: usize| c.get(i).copied().unwrap_or("-");
            m.insert(
                c[0].to_string(),
                LexEntry {
                    present: opt(g(1)),
                    past: opt(g(2)),
                    future: opt(g(3)),
                    um: opt(g(4)),
                    infinitive: opt(g(5)),
                    imperative_pl: opt(g(6)),
                },
            );
        }
        m
    })
}

/// A conjugatable Tamil verb: its root and the resolved stem set.
#[derive(Debug, Clone)]
pub struct Verb {
    root: String,
    /// Present stem, ending in கிற் (செய்கிற், படிக்கிற்).
    present: String,
    /// Past stem (செய்த், வந்த், ஓடின்).
    past: String,
    /// Future person stem, ending in வ்/ப் (செய்வ், படிப்ப்).
    future: String,
    /// The -உம் form (செய்யும்): future neuter and future participle.
    um: String,
    infinitive: String,
    imperative_pl: String,
}

impl Verb {
    /// Build a verb from its root.
    ///
    /// ```
    /// use ablaut::tam::{Png, Tense, Verb};
    /// let v = Verb::from_root("செய்").unwrap();
    /// assert_eq!(v.finite(Tense::Past, Png::P3SgM), "செய்தான்");
    /// assert_eq!(v.finite(Tense::Present, Png::P1Sg), "செய்கிறேன்");
    /// ```
    pub fn from_root(root: &str) -> Result<Self, Error> {
        let root = root.trim().to_string();
        if root.is_empty() || root.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        let lex = lexicon().get(&root);
        let present = lex
            .and_then(|l| l.present.clone())
            .unwrap_or_else(|| format!("{root}{KIRRU}"));
        let past = lex
            .and_then(|l| l.past.clone())
            .unwrap_or_else(|| join(&format!("{root}\u{0BCD}"), "\u{0BA4}\u{0BCD}"));
        let future = lex
            .and_then(|l| l.future.clone())
            .unwrap_or_else(|| format!("{root}\u{0BB5}\u{0BCD}")); // root + வ்
        let um = lex
            .and_then(|l| l.um.clone())
            .unwrap_or_else(|| format!("{root}\u{0BC1}\u{0BAE}\u{0BCD}")); // root + உம் (rough)
        let infinitive = lex
            .and_then(|l| l.infinitive.clone())
            .unwrap_or_else(|| join(root.trim_end_matches(PULLI), "\u{0B85}")); // + அ
        let imperative_pl = lex
            .and_then(|l| l.imperative_pl.clone())
            .unwrap_or_else(|| format!("{root}\u{0BC1}\u{0B99}\u{0BCD}\u{0B95}\u{0BB3}\u{0BCD}")); // + உங்கள்
        Ok(Self {
            root,
            present,
            past,
            future,
            um,
            infinitive,
            imperative_pl,
        })
    }

    /// The citation root.
    #[must_use]
    pub fn root(&self) -> &str {
        &self.root
    }

    /// Whether the past stem is the -இன் weak class, whose neuter,
    /// participle and adverbial forms build on the -இ stem.
    fn inn_class(&self) -> bool {
        self.past.ends_with(INN)
    }

    /// The -இ stem of an -இன் past (ஓடின் → ஓடி).
    fn i_stem(&self) -> String {
        self.past[..self.past.len() - "\u{0BA9}\u{0BCD}".len()].to_string()
    }

    /// A finite form: tense stem plus PNG ending.
    ///
    /// ```
    /// use ablaut::tam::{Png, Tense, Verb};
    /// let v = Verb::from_root("ஓடு").unwrap();
    /// assert_eq!(v.finite(Tense::Past, Png::P3SgM), "ஓடினான்");
    /// assert_eq!(v.finite(Tense::Past, Png::P3SgN), "ஓடியது");
    /// ```
    #[must_use]
    pub fn finite(&self, tense: Tense, png: Png) -> String {
        match tense {
            Tense::Present => match png {
                // The present neuter plural takes the கின்ற allomorph.
                Png::P3PlN => join(&self.present.replace(KIRRU, KINRRU), ending(Png::P3PlN)),
                _ => join(&self.present, ending(png)),
            },
            Tense::Future => match png {
                Png::P3SgN => self.um.clone(),
                // The future neuter plural is periphrastic across the two
                // oracles' conventions; fall back to the person stem.
                _ => join(&self.future, ending(png)),
            },
            Tense::Past => match png {
                Png::P3SgN if self.inn_class() => {
                    format!("{}\u{0BAF}\u{0BA4}\u{0BC1}", self.i_stem())
                } // + யது
                Png::P3PlN if self.inn_class() => format!("{}\u{0BA9}", self.i_stem()), // + ன
                _ => join(&self.past, ending(png)),
            },
        }
    }

    /// The infinitive (செய்ய, படிக்க).
    #[must_use]
    pub fn infinitive(&self) -> &str {
        &self.infinitive
    }

    /// The adverbial (verbal) participle (செய்து, ஓடி).
    #[must_use]
    pub fn adverbial(&self) -> String {
        if self.inn_class() {
            self.i_stem()
        } else {
            join(&self.past, "\u{0B89}") // + உ
        }
    }

    /// A relative (adjectival) participle.
    #[must_use]
    pub fn relative(&self, relative: Relative) -> String {
        match relative {
            Relative::Past if self.inn_class() => format!("{}\u{0BAF}", self.i_stem()), // + ய
            Relative::Past => join(&self.past, "\u{0B85}"),                             // + அ
            Relative::Present => join(&self.present, "\u{0B85}"),                       // + அ
            Relative::Future => self.um.clone(),
        }
    }

    /// The conditional (செய்தால், ஓடினால்).
    #[must_use]
    pub fn conditional(&self) -> String {
        join(&self.past, "\u{0B86}\u{0BB2}\u{0BCD}") // + ஆல்
    }

    /// The imperative (singular familiar = the bare root; plural/polite).
    #[must_use]
    pub fn imperative(&self, number: Number) -> String {
        match number {
            Number::Singular => self.root.clone(),
            Number::Plural => self.imperative_pl.clone(),
        }
    }
}

/// The full conjugation table of a Tamil verb — shared by the WebAssembly
/// and Python bindings. Each finite row is
/// `[1sg, 1pl, 2sg, 2pl, 3sgm, 3sgf, 3sgh, 3sgn, 3ple, 3pln]`.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub root: String,
    pub present: [String; 10],
    pub past: [String; 10],
    pub future: [String; 10],
    pub infinitive: String,
    pub adverbial: String,
    pub relative_past: String,
    pub relative_present: String,
    pub relative_future: String,
    pub conditional: String,
    /// `[singular, plural]`.
    pub imperative: [String; 2],
}

const SLOTS: [Png; 10] = [
    Png::P1Sg,
    Png::P1Pl,
    Png::P2Sg,
    Png::P2Pl,
    Png::P3SgM,
    Png::P3SgF,
    Png::P3SgH,
    Png::P3SgN,
    Png::P3PlE,
    Png::P3PlN,
];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row = |t: Tense| SLOTS.map(|png| v.finite(t, png));
        Self {
            root: v.root().to_string(),
            present: row(Tense::Present),
            past: row(Tense::Past),
            future: row(Tense::Future),
            infinitive: v.infinitive().to_string(),
            adverbial: v.adverbial(),
            relative_past: v.relative(Relative::Past),
            relative_present: v.relative(Relative::Present),
            relative_future: v.relative(Relative::Future),
            conditional: v.conditional(),
            imperative: [v.imperative(Number::Singular), v.imperative(Number::Plural)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(root: &str) -> Verb {
        Verb::from_root(root).unwrap()
    }

    #[test]
    fn weak_sey() {
        let s = v("செய்");
        assert_eq!(s.finite(Tense::Past, Png::P1Sg), "செய்தேன்");
        assert_eq!(s.finite(Tense::Past, Png::P3SgM), "செய்தான்");
        assert_eq!(s.finite(Tense::Past, Png::P3SgN), "செய்தது");
        assert_eq!(s.finite(Tense::Past, Png::P3PlN), "செய்தன");
        assert_eq!(s.finite(Tense::Present, Png::P1Sg), "செய்கிறேன்");
        assert_eq!(s.finite(Tense::Present, Png::P3SgN), "செய்கிறது");
        assert_eq!(s.finite(Tense::Present, Png::P3PlN), "செய்கின்றன");
        assert_eq!(s.finite(Tense::Future, Png::P1Sg), "செய்வேன்");
        assert_eq!(s.finite(Tense::Future, Png::P3SgN), "செய்யும்");
        assert_eq!(s.infinitive(), "செய்ய");
        assert_eq!(s.adverbial(), "செய்து");
        assert_eq!(s.relative(Relative::Past), "செய்த");
        assert_eq!(s.relative(Relative::Present), "செய்கிற");
        assert_eq!(s.relative(Relative::Future), "செய்யும்");
        assert_eq!(s.conditional(), "செய்தால்");
        assert_eq!(s.imperative(Number::Singular), "செய்");
        assert_eq!(s.imperative(Number::Plural), "செய்யுங்கள்");
    }

    #[test]
    fn strong_nada() {
        let n = v("நட");
        assert_eq!(n.finite(Tense::Past, Png::P3SgM), "நடந்தான்");
        assert_eq!(n.finite(Tense::Present, Png::P3SgM), "நடக்கிறான்");
        assert_eq!(n.finite(Tense::Future, Png::P3SgM), "நடப்பான்");
        assert_eq!(n.finite(Tense::Future, Png::P3SgN), "நடக்கும்");
        assert_eq!(n.relative(Relative::Past), "நடந்த");
        assert_eq!(n.infinitive(), "நடக்க");
    }

    #[test]
    fn inn_class_odu() {
        let o = v("ஓடு");
        assert_eq!(o.finite(Tense::Past, Png::P1Sg), "ஓடினேன்");
        assert_eq!(o.finite(Tense::Past, Png::P3SgN), "ஓடியது");
        assert_eq!(o.finite(Tense::Past, Png::P3PlN), "ஓடின");
        assert_eq!(o.adverbial(), "ஓடி");
        assert_eq!(o.relative(Relative::Past), "ஓடிய");
        assert_eq!(o.conditional(), "ஓடினால்");
    }

    #[test]
    fn suppletive_vaa() {
        let w = v("வா");
        assert_eq!(w.finite(Tense::Past, Png::P3SgM), "வந்தான்");
        assert_eq!(w.finite(Tense::Present, Png::P1Sg), "வருகிறேன்");
        assert_eq!(w.finite(Tense::Future, Png::P3SgM), "வருவான்");
        assert_eq!(w.infinitive(), "வர");
    }

    #[test]
    fn rejects_non_verbs() {
        for input in ["", "செய் படி"] {
            assert_eq!(Verb::from_root(input).err(), Some(Error::NotAVerb));
        }
    }
}
