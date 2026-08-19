//! Russian conjugation: a rule engine for the two productive
//! conjugations plus a compiled-in irregular lexicon.
//!
//! The finite paradigm scored here is deliberately small — the core a
//! learner reaches for — and aspect-neutral in shape:
//!
//! - **non-past** (present of an imperfective, simple future of a
//!   perfective; the two are morphologically identical, so one generator
//!   serves both feature labels),
//! - **past** in `-л`, inflected for gender in the singular
//!   (masc/fem/neut) and number-only in the plural,
//! - the 2nd-person **imperative**, and the **infinitive**.
//!
//! Two productive present classes are handled by rule:
//!
//! - 1st conjugation (`-е-`/`-ё-`): the productive `-ать`/`-ять`/`-еть`
//!   type (читать → читаю, читаешь) and the `-овать`/`-евать`
//!   (рисовать → рисую) and `-нуть` (крикнуть → крикну) subtypes.
//! - 2nd conjugation (`-и-`): говорить → говорю, говоришь, with the
//!   productive first-person-singular consonant mutation (любить →
//!   люблю, ходить → хожу, просить → прошу).
//!
//! Stress is lexical and mobile in Russian and cannot be read off the
//! infinitive; its only orthographic trace in these unstressed forms is
//! `ё`. Verbs whose stem, mutation, or stress the rules cannot predict
//! live in `data/rus/verbs.tsv`, stored as non-reflexive base lemmas so
//! that prefixed derivatives (написать ← писать) and reflexives
//! (умываться ← умывать) come free.

/// Grammatical person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Person {
    First,
    Second,
    Third,
}

/// Grammatical number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Singular,
    Plural,
}

/// Grammatical gender (past tense, singular).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not look like a Russian infinitive at all.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAVerb => write!(f, "not a Russian infinitive"),
        }
    }
}

const VOWELS: &str = "аеёиоуыэюя";
const HUSHERS: &str = "жшчщ";

fn is_vowel(c: char) -> bool {
    VOWELS.contains(c)
}

/// Drop the last `n` Unicode scalar values (Cyrillic is multi-byte).
fn drop_last(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    chars[..chars.len().saturating_sub(n)].iter().collect()
}

fn last_char(s: &str) -> Option<char> {
    s.chars().last()
}

fn ends_with_husher(s: &str) -> bool {
    last_char(s).is_some_and(|c| HUSHERS.contains(c))
}

// The reflexive postfix: -ся after a consonant or ь, -сь after a vowel.
fn reflexive_postfix(form: &str) -> &'static str {
    match last_char(form) {
        Some(c) if is_vowel(c) => "сь",
        _ => "ся",
    }
}

/// The productive first-person-singular consonant mutation of the 2nd
/// conjugation. Returns the mutated stem and whether the mutation was a
/// labial one (which takes the `-ю` ending: люблю, not *любу).
fn mutate_1sg(stem: &str) -> (String, bool) {
    for (from, to) in [("ст", "щ"), ("зд", "зж")] {
        if let Some(body) = stem.strip_suffix(from) {
            return (format!("{body}{to}"), false);
        }
    }
    let Some(last) = last_char(stem) else {
        return (stem.to_string(), false);
    };
    let body = drop_last(stem, 1);
    match last {
        'т' => (format!("{body}ч"), false),
        'д' => (format!("{body}ж"), false),
        'с' => (format!("{body}ш"), false),
        'з' => (format!("{body}ж"), false),
        'к' => (format!("{body}ч"), false),
        'г' => (format!("{body}ж"), false),
        'х' => (format!("{body}ш"), false),
        'б' => (format!("{body}бл"), true),
        'п' => (format!("{body}пл"), true),
        'в' => (format!("{body}вл"), true),
        'ф' => (format!("{body}фл"), true),
        'м' => (format!("{body}мл"), true),
        _ => (stem.to_string(), false),
    }
}

/// The compiled-in irregular lexicon (non-reflexive base lemmas).
static LEXICON_TSV: &str = include_str!("../data/rus/verbs.tsv");

/// A stored irregular paradigm. Each tense is an optional override that
/// falls through to the rule engine when absent (`-`), so a verb that is
/// irregular in only one tense costs only that tense to store.
///
/// Columns: lemma, nonpast (6, comma-separated 1sg..3pl), past (4:
/// masc,fem,neut,pl), imperative (2: sg,pl), exact-only flag.
#[derive(Debug, Clone, Default)]
struct LexEntry {
    nonpast: Option<[String; 6]>,
    past: Option<[String; 4]>,
    imperative: Option<[String; 2]>,
}

fn six(s: &str) -> Option<[String; 6]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
}

fn four(s: &str) -> Option<[String; 4]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
}

fn two(s: &str) -> Option<[String; 2]> {
    if s == "-" {
        return None;
    }
    let v: Vec<String> = s.split(',').map(str::to_string).collect();
    v.try_into().ok()
}

fn parse_lex(cols: &[&str]) -> LexEntry {
    let g = |i: usize| cols.get(i).copied().unwrap_or("-");
    LexEntry {
        nonpast: six(g(1)),
        past: four(g(2)),
        imperative: two(g(3)),
    }
}

/// Longest-base suffix lookup, returning the derivational prefix
/// (написать → ("на", писать)). A base flagged exact-only (column 4 =
/// "1") is a suffix of otherwise-regular verbs and must match the whole
/// infinitive.
fn lexical(base: &str) -> Option<(String, LexEntry)> {
    let mut best: Option<(&str, Vec<&str>)> = None;
    for line in LEXICON_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let lemma = cols[0];
        let exact = cols.get(4).copied() == Some("1");
        let hit = if exact {
            base == lemma
        } else {
            base.ends_with(lemma)
        };
        if hit
            && !best
                .as_ref()
                .is_some_and(|(b, _)| b.chars().count() >= lemma.chars().count())
        {
            best = Some((lemma, cols));
        }
    }
    let (lemma, cols) = best?;
    let prefix = base[..base.len() - lemma.len()].to_string();
    Some((prefix, parse_lex(&cols)))
}

/// The productive present class of a base infinitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// -овать/-евать (рисовать → рисую).
    Ova,
    /// -нуть (крикнуть → крикну).
    Nu,
    /// 2nd conjugation -ить (говорить → говорю).
    I,
    /// productive 1st conjugation -ать/-ять/-еть (читать → читаю).
    Aj,
    /// consonant-final infinitive (-ти, -чь): rules do not apply, the
    /// lexicon must carry it.
    Athematic,
}

/// A conjugatable Russian verb.
#[derive(Debug, Clone)]
pub struct Verb {
    /// The infinitive as given (may carry the reflexive postfix).
    infinitive: String,
    reflexive: bool,
    /// The infinitive without the reflexive postfix.
    base: String,
    class: Class,
    lex: Option<Box<LexEntry>>,
    prefix: String,
}

impl Person {
    const fn index(self, number: Number) -> usize {
        let p = match self {
            Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
        };
        match number {
            Number::Singular => p,
            Number::Plural => p + 3,
        }
    }
}

impl Verb {
    /// Build a verb from its infinitive. A reflexive `-ся`/`-сь`
    /// infinitive conjugates and re-attaches its postfix.
    ///
    /// # Errors
    /// Returns [`Error::NotAVerb`] for input that is not a single-word
    /// infinitive.
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let lowered = infinitive.trim().to_lowercase();
        if lowered.is_empty() || lowered.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        let (base, reflexive) = match lowered
            .strip_suffix("ся")
            .or_else(|| lowered.strip_suffix("сь"))
        {
            Some(b) => (b.to_string(), true),
            None => (lowered.clone(), false),
        };
        // An infinitive ends in -ть, -ти or -чь.
        if !(base.ends_with("ть") || base.ends_with("ти") || base.ends_with("чь")) {
            return Err(Error::NotAVerb);
        }
        let class = classify(&base);
        let (prefix, lex) = match lexical(&base) {
            Some((p, e)) => (p, Some(Box::new(e))),
            None => (String::new(), None),
        };
        Ok(Self {
            infinitive: lowered,
            reflexive,
            base,
            class,
            lex,
            prefix,
        })
    }

    /// The infinitive as normalized.
    #[must_use]
    pub fn infinitive(&self) -> String {
        self.infinitive.clone()
    }

    /// Prepend the derivational prefix to a stored base form. The
    /// perfective prefix вы- is always stressed, so it pulls the stress
    /// off the stem and the stem's ё (which only ever marks a stressed
    /// vowel) reverts to е: слать → шлёт but выслать → вышлет.
    fn combine(&self, form: &str) -> String {
        // …but not the imperfective -авать verbs (выдавать → выдаёт), whose
        // вы- is unstressed and whose stress (and ё) stays on the stem.
        if self.prefix.ends_with("вы") && !self.base.ends_with("авать") {
            format!("{}{}", self.prefix, form.replace('ё', "е"))
        } else {
            format!("{}{}", self.prefix, form)
        }
    }

    /// Re-attach the reflexive postfix if this verb is reflexive.
    fn reflexivize(&self, form: String) -> String {
        if self.reflexive {
            let p = reflexive_postfix(&form);
            format!("{form}{p}")
        } else {
            form
        }
    }

    /// The past-tense stem (infinitive minus `-ть`).
    fn past_stem(&self) -> String {
        drop_last(&self.base, 2)
    }

    /// The six non-past forms by rule, indexed [1sg..3pl].
    fn nonpast_rule(&self, i: usize) -> String {
        let je = |stem: &str| {
            // ю, ешь, ет, ем, ете, ют
            let ends = ["ю", "ешь", "ет", "ем", "ете", "ют"];
            format!("{stem}{}", ends[i])
        };
        match self.class {
            Class::Ova => je(&self.ova_stem()),
            Class::Nu => {
                let stem = drop_last(&self.base, 3); // крикн-
                let ends = ["у", "ешь", "ет", "ем", "ете", "ут"];
                format!("{stem}{}", ends[i])
            }
            Class::I => {
                let r = drop_last(&self.base, 3); // говор-
                match i {
                    0 => {
                        // 1sg mutates: hushers take -у (хожу), labials and
                        // the rest -ю (люблю, говорю).
                        let (m, labial) = mutate_1sg(&r);
                        let vowel = if !labial && ends_with_husher(&m) {
                            "у"
                        } else {
                            "ю"
                        };
                        format!("{m}{vowel}")
                    }
                    1 => format!("{r}ишь"),
                    2 => format!("{r}ит"),
                    3 => format!("{r}им"),
                    4 => format!("{r}ите"),
                    _ => {
                        let end = if ends_with_husher(&r) { "ат" } else { "ят" };
                        format!("{r}{end}")
                    }
                }
            }
            Class::Aj => je(&self.past_stem()),
            Class::Athematic => je(&self.past_stem()),
        }
    }

    /// The present stem of an -овать/-евать verb.
    fn ova_stem(&self) -> String {
        let body = drop_last(&self.base, 5); // рисов-ать → рис
        if self.base.ends_with("евать") {
            // -евать → -ую after a consonant (ночую), -юю after a vowel
            // (воюю).
            if is_vowel_end(&body) {
                format!("{body}ю")
            } else {
                format!("{body}у")
            }
        } else {
            format!("{body}у")
        }
    }

    fn imperative_rule(&self, plural: bool) -> String {
        let sg = match self.class {
            Class::Ova => format!("{}й", self.ova_stem()),
            Class::Nu => format!("{}и", drop_last(&self.base, 3)),
            Class::I => format!("{}и", drop_last(&self.base, 3)),
            Class::Aj | Class::Athematic => format!("{}й", self.past_stem()),
        };
        if plural {
            format!("{sg}те")
        } else {
            sg
        }
    }

    /// A non-past form (imperfective present / perfective simple future).
    #[must_use]
    pub fn non_past(&self, person: Person, number: Number) -> String {
        let i = person.index(number);
        let stored = self
            .lex
            .as_ref()
            .and_then(|l| l.nonpast.as_ref())
            .map(|forms| self.combine(&forms[i]));
        self.reflexivize(stored.unwrap_or_else(|| self.nonpast_rule(i)))
    }

    /// A past-tense form. Gender is honoured in the singular and ignored
    /// in the plural.
    #[must_use]
    pub fn past(&self, gender: Gender, number: Number) -> String {
        let i = match (number, gender) {
            (Number::Plural, _) => 3,
            (Number::Singular, Gender::Masculine) => 0,
            (Number::Singular, Gender::Feminine) => 1,
            (Number::Singular, Gender::Neuter) => 2,
        };
        let form = self
            .lex
            .as_ref()
            .and_then(|l| l.past.as_ref())
            .map(|forms| self.combine(&forms[i]));
        let ends = ["л", "ла", "ло", "ли"];
        self.reflexivize(form.unwrap_or_else(|| format!("{}{}", self.past_stem(), ends[i])))
    }

    /// The 2nd-person imperative.
    #[must_use]
    pub fn imperative(&self, number: Number) -> String {
        let plural = number == Number::Plural;
        let form = self
            .lex
            .as_ref()
            .and_then(|l| l.imperative.as_ref())
            .map(|f| self.combine(&f[usize::from(plural)]));
        self.reflexivize(form.unwrap_or_else(|| self.imperative_rule(plural)))
    }

    /// Every standard form of the imperative. Normally one form; a
    /// stem-stressed base gives the short imperative (багрянь) while the
    /// productive rule gives багряни, and Russian оracles split on which
    /// a given verb (and its reflexive) takes, so both are offered.
    #[must_use]
    pub fn imperative_variants(&self, number: Number) -> Vec<String> {
        let mut out = vec![self.imperative(number)];
        let rule = self.reflexivize(self.imperative_rule(number == Number::Plural));
        if !out.contains(&rule) {
            out.push(rule);
        }
        out
    }

    /// Every standard form of a non-past slot. Normally one form; for a
    /// lemma that spells two aspectual lexemes the same way (присыпать =
    /// imperfective присыпаю *and* perfective присыплю) both the stored
    /// form and the productive rule form are genuine, so both are
    /// offered — the caller keeps whichever its reading needs.
    #[must_use]
    pub fn non_past_variants(&self, person: Person, number: Number) -> Vec<String> {
        let mut out = vec![self.non_past(person, number)];
        let rule = self.reflexivize(self.nonpast_rule(person.index(number)));
        if !out.contains(&rule) {
            out.push(rule);
        }
        out
    }
}

fn is_vowel_end(s: &str) -> bool {
    last_char(s).is_some_and(is_vowel)
}

fn classify(base: &str) -> Class {
    let n = base.chars().count();
    if (base.ends_with("овать") || base.ends_with("евать")) && n > 6 {
        Class::Ova
    } else if base.ends_with("нуть") && n > 4 {
        Class::Nu
    } else if base.ends_with("ить") && n > 4 {
        Class::I
    } else if base.ends_with("ть") {
        Class::Aj
    } else {
        Class::Athematic
    }
}

/// The full finite conjugation table of a Russian verb — shared by the
/// WebAssembly and Python bindings. Person rows are
/// [я, ты, он/она/оно, мы, вы, они].
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// Present (imperfective) or simple future (perfective).
    pub non_past: [String; 6],
    /// [masc, fem, neut] singular past.
    pub past_singular: [String; 3],
    /// Plural past.
    pub past_plural: String,
    /// [2sg, 2pl].
    pub imperative: [String; 2],
}

const SLOTS: [(Person, Number); 6] = [
    (Person::First, Number::Singular),
    (Person::Second, Number::Singular),
    (Person::Third, Number::Singular),
    (Person::First, Number::Plural),
    (Person::Second, Number::Plural),
    (Person::Third, Number::Plural),
];

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        Self {
            infinitive: v.infinitive(),
            non_past: SLOTS.map(|(p, n)| v.non_past(p, n)),
            past_singular: [
                v.past(Gender::Masculine, Number::Singular),
                v.past(Gender::Feminine, Number::Singular),
                v.past(Gender::Neuter, Number::Singular),
            ],
            past_plural: v.past(Gender::Masculine, Number::Plural),
            imperative: [v.imperative(Number::Singular), v.imperative(Number::Plural)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Number::{Plural as PL, Singular as SG};
    use Person::{First as P1, Second as P2, Third as P3};

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn productive_aj() {
        let r = v("читать");
        assert_eq!(r.non_past(P1, SG), "читаю");
        assert_eq!(r.non_past(P2, SG), "читаешь");
        assert_eq!(r.non_past(P3, SG), "читает");
        assert_eq!(r.non_past(P1, PL), "читаем");
        assert_eq!(r.non_past(P2, PL), "читаете");
        assert_eq!(r.non_past(P3, PL), "читают");
        assert_eq!(r.past(Gender::Masculine, SG), "читал");
        assert_eq!(r.past(Gender::Feminine, SG), "читала");
        assert_eq!(r.past(Gender::Neuter, SG), "читало");
        assert_eq!(r.past(Gender::Masculine, PL), "читали");
        assert_eq!(r.imperative(SG), "читай");
        assert_eq!(r.imperative(PL), "читайте");
    }

    #[test]
    fn second_conjugation() {
        assert_eq!(v("говорить").non_past(P1, SG), "говорю");
        assert_eq!(v("говорить").non_past(P2, SG), "говоришь");
        assert_eq!(v("говорить").non_past(P3, PL), "говорят");
        assert_eq!(v("говорить").imperative(SG), "говори");
        // productive 1sg mutation
        assert_eq!(v("любить").non_past(P1, SG), "люблю");
        assert_eq!(v("любить").non_past(P2, SG), "любишь");
        assert_eq!(v("ходить").non_past(P1, SG), "хожу");
        assert_eq!(v("просить").non_past(P1, SG), "прошу");
        assert_eq!(v("возить").non_past(P1, SG), "вожу");
        // husher stems take -ат / -у
        assert_eq!(v("учить").non_past(P3, PL), "учат");
        assert_eq!(v("учить").non_past(P1, SG), "учу");
    }

    #[test]
    fn ova_and_nu() {
        assert_eq!(v("рисовать").non_past(P1, SG), "рисую");
        assert_eq!(v("рисовать").non_past(P2, SG), "рисуешь");
        assert_eq!(v("рисовать").imperative(SG), "рисуй");
        assert_eq!(v("воевать").non_past(P1, SG), "воюю");
        assert_eq!(v("крикнуть").non_past(P1, SG), "крикну");
        assert_eq!(v("крикнуть").non_past(P2, SG), "крикнешь");
        assert_eq!(v("крикнуть").non_past(P3, PL), "крикнут");
    }

    #[test]
    fn reflexive() {
        let r = v("умываться");
        assert_eq!(r.non_past(P1, SG), "умываюсь");
        assert_eq!(r.non_past(P2, SG), "умываешься");
        assert_eq!(r.past(Gender::Masculine, SG), "умывался");
        assert_eq!(r.past(Gender::Feminine, SG), "умывалась");
        assert_eq!(r.imperative(SG), "умывайся");
        assert_eq!(r.imperative(PL), "умывайтесь");
    }
}
