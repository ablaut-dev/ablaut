//! Bengali conjugation: a productive rule engine over the open verb
//! class plus a small compiled-in table of the roots whose stem grades
//! are not derivable.
//!
//! Every Bengali verb is cited by its **verbal noun**, which ends in
//! `-আ` (করা "do", বলা "say", নাচা "dance"). Dropping the `-আ` leaves
//! the **stem** (কর, বল, নাচ), and the finite system is built on it.
//! Unlike its Indo-Aryan sister Hindi (`src/hin.rs`), Bengali marks **no
//! grammatical gender**; instead the finite verb fuses tense with a rich
//! **person × honorific** agreement — five classes:
//!
//! - **আমি** (first person, করি);
//! - **তুই** (second person intimate, করিস);
//! - **তুমি** (second person familiar, করো);
//! - **সে** (third person ordinary, করে);
//! - **আপনি / তিনি** (honorific, second or third, করেন).
//!
//! Each of eight tense-aspects — simple present, simple past, future,
//! past habitual, present/past progressive and present/past perfect —
//! crosses those five classes, giving a dense 5×8 finite grid plus the
//! non-finite forms (the `-তে` infinitive নাচতে, the perfective/
//! conjunctive participle নেচে, and the conditional নাচলে).
//!
//! Two morphophonological alternations shape the stem:
//!
//! - **vowel raising** — a mid root vowel এ/ও raises to ই/উ everywhere
//!   except the তুমি/সে/আপনি present and the verbal noun (ওঠা: উঠি but
//!   ওঠে; কেনা: কিনি but কেনে). Whether a root raises is lexical
//!   (কেনা→কিন does, দেখা→দেখ does not), so raising roots store both
//!   grades in `data/ben/verbs.tsv`;
//! - **আ-fronting** — the perfective participle fronts a root আ to এ
//!   (নাচ→নেচে, কাটা→কেটে, জানা→জেনে), applied by rule.
//!
//! The `-আনো` causatives (ঘুমানো, চালানো, দেখানো) are fully productive
//! and handled by rule; the eight vowel-final roots (খাওয়া, দেওয়া,
//! যাওয়া …) are irregular across several stems and stored explicitly.
//! Compound and light-verb lemmas (`অনুবাদ করা`, `মনে রাখা`) conjugate
//! only their last word; the invariable material is carried along.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The five person × honorific agreement classes of the finite verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Person {
    /// আমি — first person (no honorific distinction).
    First,
    /// তুই — second person intimate.
    SecondIntimate,
    /// তুমি — second person familiar.
    SecondFamiliar,
    /// সে — third person ordinary.
    Third,
    /// আপনি / তিনি — honorific (second or third person).
    Honorific,
}

/// The eight tense-aspects the finite grid crosses with [`Person`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    /// করি — simple present.
    Present,
    /// করলাম — simple past.
    Past,
    /// করবো — future.
    Future,
    /// করতাম — past habitual.
    Habitual,
    /// করছি — present progressive.
    PresentProgressive,
    /// করছিলাম — past progressive.
    PastProgressive,
    /// করেছি — present perfect.
    PresentPerfect,
    /// করেছিলাম — past perfect.
    PastPerfect,
}

/// Why an input cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input's last word does not end in `-আ` (a Bengali verbal noun).
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Bengali verb")
    }
}

/// The compiled-in table of roots whose stem grades are not derivable.
static LEXICON_TSV: &str = include_str!("../data/ben/verbs.tsv");

/// A stored paradigm. Each `-` cell falls through to the rule.
#[derive(Debug, Clone, Default)]
struct LexEntry {
    base: Option<String>,
    high: Option<String>,
    past_stem: Option<String>,
    hab_stem: Option<String>,
    nfin_stem: Option<String>,
    perfect: Option<String>,
    prog: Option<String>,
    present: Option<[String; 5]>,
}

fn opt(s: &str) -> Option<String> {
    (s != "-" && !s.is_empty()).then(|| s.to_string())
}

fn five(s: &str) -> Option<[String; 5]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
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
                    base: opt(g(1)),
                    high: opt(g(2)),
                    past_stem: opt(g(3)),
                    hab_stem: opt(g(4)),
                    nfin_stem: opt(g(5)),
                    perfect: opt(g(6)),
                    prog: opt(g(7)),
                    present: five(g(8)),
                },
            );
        }
        m
    })
}

/// Front a root's last আ to এ (নাচ→নেচ, আস→এস): the perfective
/// participle's vowel change. A root with no আ is returned unchanged.
fn front(s: &str) -> String {
    if let Some(pos) = s.rfind('া') {
        let mut out = s.to_string();
        out.replace_range(pos..pos + 'া'.len_utf8(), "ে");
        return out;
    }
    if let Some(pos) = s.rfind('আ') {
        let mut out = s.to_string();
        out.replace_range(pos..pos + 'আ'.len_utf8(), "এ");
        return out;
    }
    s.to_string()
}

/// A conjugatable Bengali verb, with every stem resolved at build time.
#[derive(Debug, Clone)]
pub struct Verb {
    /// Invariable material before the verb word (`মনে ` in মনে রাখা),
    /// empty for a simple verb. Re-attached to every form.
    prefix: String,
    /// The verb word's verbal-noun citation (রাখা), without the prefix.
    infinitive: String,
    /// Present তুমি/সে/আপনি stem.
    base: String,
    /// Present আমি/তুই stem + future stem.
    high: String,
    /// Simple past + conditional stem.
    past_stem: String,
    /// Past-habitual stem.
    hab_stem: String,
    /// `-তে` infinitive + progressive-participle stem.
    nfin_stem: String,
    /// Perfective participle (নেচে): the V.PTCP;PRF/HAB slot form.
    perfect_ptcp: String,
    /// The base the perfect *tenses* are built on (usually the
    /// participle; the `-আনো` causatives split it: নেচে-vs-ঘুমিয়ে).
    perfect_base: String,
    /// Progressive base including the ছ/চ্ছ marker (করছ, খাচ্ছ).
    prog_base: String,
    /// Explicit present row for the irregular roots.
    present: Option<[String; 5]>,
    /// Whether the simple-past ল marker geminates (bôl → bôllam).
    geminate: bool,
}

// Person-indexed suffix rows, in order [আমি, তুই, তুমি, সে, আপনি].
const PAST: [&str; 5] = ["লাম", "লি", "লে", "লো", "লেন"];
const FUTURE: [&str; 5] = ["বো", "বি", "বে", "বে", "বেন"];
const HABITUAL: [&str; 5] = ["তাম", "তিস", "তে", "তো", "তেন"];
/// Progressive personal endings, glued after the ছ/চ্ছ base (present).
const PROG_PRS: [&str; 5] = ["ি", "িস", "ো", "ে", "েন"];
/// Progressive/perfect past endings, glued after the ছ/চ্ছ base.
const CH_PST: [&str; 5] = ["িলাম", "িলি", "িলে", "িলো", "িলেন"];
/// Perfect present endings, glued after the perfective participle.
const PRF_PRS: [&str; 5] = ["ছি", "ছিস", "ছো", "ছে", "ছেন"];
/// Perfect past endings, glued after the perfective participle.
const PRF_PST: [&str; 5] = ["ছিলাম", "ছিলি", "ছিলে", "ছিলো", "ছিলেন"];

/// The five agreement classes in array order.
pub const PERSONS: [Person; 5] = [
    Person::First,
    Person::SecondIntimate,
    Person::SecondFamiliar,
    Person::Third,
    Person::Honorific,
];

impl Person {
    const fn idx(self) -> usize {
        match self {
            Self::First => 0,
            Self::SecondIntimate => 1,
            Self::SecondFamiliar => 2,
            Self::Third => 3,
            Self::Honorific => 4,
        }
    }
}

impl Verb {
    /// Build a verb from its verbal-noun citation.
    ///
    /// ```
    /// use ablaut::ben::{Person, Tense, Verb};
    /// let v = Verb::from_infinitive("করা").unwrap();
    /// assert_eq!(v.finite(Tense::Present, Person::First), "করি");
    /// assert_eq!(v.finite(Tense::Past, Person::Third), "করলো");
    /// assert_eq!(v.perfective(), "করে");
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::NotAVerb`] when the last word does not end in
    /// `-আ`.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim();
        if inf.is_empty() {
            return Err(Error::NotAVerb);
        }
        // A compound lemma conjugates only its last word.
        let (prefix, word) = match inf.rfind(' ') {
            Some(i) => (inf[..=i].to_string(), inf[i + 1..].to_string()),
            None => (String::new(), inf.to_string()),
        };

        let lex = lexicon().get(&word);

        // Classify by suffix. `-ওয়া` and `-আনো` are checked before the
        // bare `-আ` verbal-noun ending they both contain a shape of.
        if let Some(stem) = word.strip_suffix("ানো") {
            // Productive causative: a vowel(আ)-final stem, no raising.
            if stem.is_empty() {
                return Err(Error::NotAVerb);
            }
            let s = format!("{stem}া");
            // The perfect *tense* base fronts আ→ই + য়ে (ঘুমা→ঘুমিয়ে);
            // the participle *slot* is just স্টেম+য় (ঘুমায়).
            let raised = {
                let mut r = s.clone();
                if let Some(pos) = r.rfind('া') {
                    r.replace_range(pos..pos + 'া'.len_utf8(), "ি");
                }
                r
            };
            return Ok(Self {
                prefix,
                base: s.clone(),
                high: s.clone(),
                past_stem: s.clone(),
                hab_stem: s.clone(),
                nfin_stem: s.clone(),
                perfect_ptcp: format!("{s}য়"),
                perfect_base: format!("{raised}য়ে"),
                prog_base: format!("{s}চ্ছ"),
                present: Some([
                    format!("{s}ই"),
                    format!("{s}স"),
                    format!("{s}ও"),
                    format!("{s}য়"),
                    format!("{s}ন"),
                ]),
                geminate: false,
                infinitive: word,
            });
        }

        // The plain stem: strip the `-ওয়া` of a vowel root, else `-আ`.
        let stem = word
            .strip_suffix("ওয়া")
            .or_else(|| word.strip_suffix('আ'))
            .or_else(|| word.strip_suffix('া'))
            .ok_or(Error::NotAVerb)?
            .to_string();
        if stem.is_empty() {
            return Err(Error::NotAVerb);
        }

        let base = lex
            .and_then(|l| l.base.clone())
            .unwrap_or_else(|| stem.clone());
        let high = lex
            .and_then(|l| l.high.clone())
            .unwrap_or_else(|| stem.clone());
        let past_stem = lex
            .and_then(|l| l.past_stem.clone())
            .unwrap_or_else(|| high.clone());
        let hab_stem = lex
            .and_then(|l| l.hab_stem.clone())
            .unwrap_or_else(|| high.clone());
        let nfin_stem = lex
            .and_then(|l| l.nfin_stem.clone())
            .unwrap_or_else(|| high.clone());
        let perfect_ptcp = lex
            .and_then(|l| l.perfect.clone())
            .unwrap_or_else(|| format!("{}ে", front(&high)));
        let prog_base = lex
            .and_then(|l| l.prog.clone())
            .unwrap_or_else(|| format!("{high}ছ"));
        let present = lex.and_then(|l| l.present.clone());

        Ok(Self {
            prefix,
            geminate: past_stem.ends_with('ল'),
            perfect_base: perfect_ptcp.clone(),
            base,
            high,
            past_stem,
            hab_stem,
            nfin_stem,
            perfect_ptcp,
            prog_base,
            present,
            infinitive: word,
        })
    }

    /// Re-attach the invariable prefix.
    fn out(&self, form: &str) -> String {
        format!("{}{}", self.prefix, form)
    }

    /// The normalized citation (verbal noun), e.g. করা / মনে রাখা.
    #[must_use]
    pub fn infinitive(&self) -> String {
        format!("{}{}", self.prefix, self.infinitive)
    }

    /// The verbal noun (the `-আ` citation) — an alias of [`Self::infinitive`].
    #[must_use]
    pub fn verbal_noun(&self) -> String {
        self.infinitive()
    }

    /// The `-তে` infinitive (করতে "to do").
    #[must_use]
    pub fn verbal_infinitive(&self) -> String {
        self.out(&format!("{}তে", self.nfin_stem))
    }

    /// The perfective / conjunctive participle (করে, নেচে "having done").
    #[must_use]
    pub fn perfective(&self) -> String {
        self.out(&self.perfect_ptcp)
    }

    /// The habitual participle — syncretic with the perfective (করে).
    #[must_use]
    pub fn habitual_participle(&self) -> String {
        self.out(&self.perfect_ptcp)
    }

    /// The progressive participle — syncretic with the `-তে` infinitive
    /// (করতে).
    #[must_use]
    pub fn progressive_participle(&self) -> String {
        self.verbal_infinitive()
    }

    /// The conditional participle (করলে "if one does").
    #[must_use]
    pub fn conditional(&self) -> String {
        self.out(&self.join_past("লে"))
    }

    /// Glue a ল-initial suffix to the past stem, geminating when the
    /// stem ends in ল (বল + লাম → বল্লাম).
    fn join_past(&self, ending: &str) -> String {
        if self.geminate {
            format!("{}্{ending}", self.past_stem)
        } else {
            format!("{}{ending}", self.past_stem)
        }
    }

    /// One finite cell: a tense-aspect crossed with a person/honorific.
    #[must_use]
    pub fn finite(&self, tense: Tense, person: Person) -> String {
        let i = person.idx();
        let form = match tense {
            Tense::Present => match &self.present {
                Some(p) => return self.out(&p[i]),
                None => {
                    let (stem, end) = match person {
                        Person::First => (&self.high, "ি"),
                        Person::SecondIntimate => (&self.high, "িস"),
                        Person::SecondFamiliar => (&self.base, "ো"),
                        Person::Third => (&self.base, "ে"),
                        Person::Honorific => (&self.base, "েন"),
                    };
                    format!("{stem}{end}")
                }
            },
            Tense::Past => self.join_past(PAST[i]),
            Tense::Future => format!("{}{}", self.high, FUTURE[i]),
            Tense::Habitual => format!("{}{}", self.hab_stem, HABITUAL[i]),
            Tense::PresentProgressive => format!("{}{}", self.prog_base, PROG_PRS[i]),
            Tense::PastProgressive => format!("{}{}", self.prog_base, CH_PST[i]),
            Tense::PresentPerfect => format!("{}{}", self.perfect_base, PRF_PRS[i]),
            Tense::PastPerfect => format!("{}{}", self.perfect_base, PRF_PST[i]),
        };
        self.out(&form)
    }
}

/// The conjugation table of a Bengali verb — shared by the WebAssembly
/// and Python bindings. Every person row is
/// `[আমি, তুই, তুমি, সে, আপনি]`.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// The verbal-noun citation (করা).
    pub infinitive: String,
    /// The `-তে` infinitive (করতে).
    pub verbal_infinitive: String,
    /// The perfective / conjunctive participle (করে, নেচে).
    pub perfective: String,
    /// The habitual participle (syncretic with the perfective).
    pub habitual_participle: String,
    /// The progressive participle (syncretic with the `-তে` infinitive).
    pub progressive_participle: String,
    /// The conditional participle (করলে).
    pub conditional: String,
    /// করি, করিস, করো, করে, করেন — the simple present.
    pub present: [String; 5],
    /// করলাম … — the simple past.
    pub past: [String; 5],
    /// করবো … — the future.
    pub future: [String; 5],
    /// করতাম … — the past habitual.
    pub habitual: [String; 5],
    /// করছি … — the present progressive.
    pub present_progressive: [String; 5],
    /// করছিলাম … — the past progressive.
    pub past_progressive: [String; 5],
    /// করেছি … — the present perfect.
    pub present_perfect: [String; 5],
    /// করেছিলাম … — the past perfect.
    pub past_perfect: [String; 5],
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        let row = |t: Tense| PERSONS.map(|p| v.finite(t, p));
        Self {
            infinitive: v.infinitive(),
            verbal_infinitive: v.verbal_infinitive(),
            perfective: v.perfective(),
            habitual_participle: v.habitual_participle(),
            progressive_participle: v.progressive_participle(),
            conditional: v.conditional(),
            present: row(Tense::Present),
            past: row(Tense::Past),
            future: row(Tense::Future),
            habitual: row(Tense::Habitual),
            present_progressive: row(Tense::PresentProgressive),
            past_progressive: row(Tense::PastProgressive),
            present_perfect: row(Tense::PresentPerfect),
            past_perfect: row(Tense::PastPerfect),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Person::{First as P1, Honorific as HON, SecondFamiliar as TUMI};
    use Person::{SecondIntimate as TUI, Third as SE};
    use Tense::{
        Future as FUT, Habitual as HAB, Past as PST, PastPerfect as PSTPRF,
        PastProgressive as PSTPROG, Present as PRS, PresentPerfect as PRSPRF,
        PresentProgressive as PRSPROG,
    };

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn regular_consonant_stem() {
        let k = v("করা");
        assert_eq!(k.infinitive(), "করা");
        assert_eq!(k.finite(PRS, P1), "করি");
        assert_eq!(k.finite(PRS, TUI), "করিস");
        assert_eq!(k.finite(PRS, TUMI), "করো");
        assert_eq!(k.finite(PRS, SE), "করে");
        assert_eq!(k.finite(PRS, HON), "করেন");
        assert_eq!(k.finite(PST, P1), "করলাম");
        assert_eq!(k.finite(PST, TUMI), "করলে");
        assert_eq!(k.finite(PST, SE), "করলো");
        assert_eq!(k.finite(FUT, P1), "করবো");
        assert_eq!(k.finite(FUT, HON), "করবেন");
        assert_eq!(k.finite(HAB, P1), "করতাম");
        assert_eq!(k.finite(HAB, SE), "করতো");
        assert_eq!(k.finite(PRSPROG, P1), "করছি");
        assert_eq!(k.finite(PRSPROG, SE), "করছে");
        assert_eq!(k.finite(PSTPROG, P1), "করছিলাম");
        assert_eq!(k.finite(PRSPRF, P1), "করেছি");
        assert_eq!(k.finite(PRSPRF, SE), "করেছে");
        assert_eq!(k.finite(PSTPRF, HON), "করেছিলেন");
        assert_eq!(k.verbal_infinitive(), "করতে");
        assert_eq!(k.perfective(), "করে");
        assert_eq!(k.conditional(), "করলে");
    }

    #[test]
    fn front_perfective_and_gemination() {
        // আ-fronting in the perfective participle only.
        assert_eq!(v("নাচা").perfective(), "নেচে");
        assert_eq!(v("নাচা").finite(PRS, SE), "নাচে");
        assert_eq!(v("নাচা").finite(PRSPRF, P1), "নেচেছি");
        assert_eq!(v("কাটা").perfective(), "কেটে");
        assert_eq!(v("আসা").perfective(), "এসে");
        assert_eq!(v("জানা").perfective(), "জেনে");
        // এ roots that do not raise keep their vowel (দেখ, not দিখ).
        assert_eq!(v("দেখা").finite(PRS, P1), "দেখি");
        assert_eq!(v("দেখা").perfective(), "দেখে");
        // ল-final stems geminate the past marker.
        assert_eq!(v("বলা").finite(PST, P1), "বল্লাম");
        assert_eq!(v("বলা").finite(PST, SE), "বল্লো");
        assert_eq!(v("বলা").conditional(), "বল্লে");
        assert_eq!(v("বলা").finite(HAB, P1), "বলতাম"); // ত marker: no gemination
    }

    #[test]
    fn raising_roots() {
        // ও→উ, but the তুমি/সে/আপনি present keeps the base grade.
        let o = v("ওঠা");
        assert_eq!(o.finite(PRS, P1), "উঠি");
        assert_eq!(o.finite(PRS, TUMI), "ওঠো");
        assert_eq!(o.finite(PRS, SE), "ওঠে");
        assert_eq!(o.finite(PST, P1), "উঠলাম");
        assert_eq!(o.perfective(), "উঠে");
        assert_eq!(o.verbal_infinitive(), "উঠতে");
        // এ→ই raising.
        assert_eq!(v("কেনা").finite(PRS, P1), "কিনি");
        assert_eq!(v("কেনা").finite(PRS, SE), "কেনে");
        assert_eq!(v("কেনা").perfective(), "কিনে");
        assert_eq!(v("লেখা").finite(PRS, P1), "লিখি");
        assert_eq!(v("লেখা").finite(PRS, TUMI), "লেখো");
        // raising ও-root ending in ল geminates on its raised grade.
        assert_eq!(v("খোলা").finite(PST, P1), "খুল্লাম");
        assert_eq!(v("খোলা").finite(PRS, SE), "খোলে");
        assert_eq!(v("খোলা").perfective(), "খুলে");
    }

    #[test]
    fn vowel_roots() {
        let kh = v("খাওয়া");
        assert_eq!(kh.finite(PRS, P1), "খাই");
        assert_eq!(kh.finite(PRS, TUI), "খাস");
        assert_eq!(kh.finite(PRS, SE), "খায়");
        assert_eq!(kh.finite(PST, P1), "খেলাম");
        assert_eq!(kh.finite(FUT, P1), "খাবো");
        assert_eq!(kh.finite(HAB, P1), "খেতাম");
        assert_eq!(kh.finite(PRSPROG, P1), "খাচ্ছি");
        assert_eq!(kh.finite(PRSPRF, P1), "খেয়েছি");
        assert_eq!(kh.verbal_infinitive(), "খেতে");
        assert_eq!(kh.perfective(), "খেয়ে");
        // যাওয়া is suppletive in the past / perfect (গ-), plain elsewhere.
        let ja = v("যাওয়া");
        assert_eq!(ja.finite(PST, P1), "গেলাম");
        assert_eq!(ja.finite(HAB, P1), "যেতাম");
        assert_eq!(ja.finite(FUT, P1), "যাবো");
        assert_eq!(ja.perfective(), "গিয়ে");
        assert_eq!(ja.verbal_infinitive(), "যেতে");
        // দেওয়া: raised দি past, base দে infinitive.
        let de = v("দেওয়া");
        assert_eq!(de.finite(PRS, TUMI), "দাও");
        assert_eq!(de.finite(PST, P1), "দিলাম");
        assert_eq!(de.verbal_infinitive(), "দেতে");
        assert_eq!(de.perfective(), "দিয়ে");
        assert_eq!(v("হওয়া").finite(PST, P1), "হলাম");
        assert_eq!(v("হওয়া").finite(PRSPROG, P1), "হচ্ছি");
    }

    #[test]
    fn causative_ano() {
        let g = v("ঘুমানো");
        assert_eq!(g.finite(PRS, P1), "ঘুমাই");
        assert_eq!(g.finite(PRS, SE), "ঘুমায়");
        assert_eq!(g.finite(PST, P1), "ঘুমালাম");
        assert_eq!(g.finite(FUT, P1), "ঘুমাবো");
        assert_eq!(g.finite(HAB, P1), "ঘুমাতাম");
        assert_eq!(g.finite(PRSPROG, P1), "ঘুমাচ্ছি");
        assert_eq!(g.finite(PRSPRF, P1), "ঘুমিয়েছি");
        assert_eq!(g.verbal_infinitive(), "ঘুমাতে");
        assert_eq!(g.perfective(), "ঘুমায়");
    }

    #[test]
    fn compound_lemma() {
        let a = v("মনে রাখা");
        assert_eq!(a.infinitive(), "মনে রাখা");
        assert_eq!(a.finite(PRS, P1), "মনে রাখি");
        assert_eq!(a.perfective(), "মনে রেখে");
        let b = v("ভুলে যাওয়া");
        assert_eq!(b.finite(PST, P1), "ভুলে গেলাম");
        assert_eq!(b.perfective(), "ভুলে গিয়ে");
    }

    #[test]
    fn rejects_non_verbs() {
        for input in ["", "বই", "run", "কর"] {
            assert_eq!(Verb::from_infinitive(input).err(), Some(Error::NotAVerb));
        }
    }
}
