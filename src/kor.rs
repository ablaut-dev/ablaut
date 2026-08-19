//! Korean verb conjugation: a hangul jamo-level combination engine
//! with a compiled-in irregular lexicon.
//!
//! Korean predicates are a stem (어간) plus an ending (어미). The
//! dictionary form is the stem plus `-다`. Conjugation is highly regular
//! once two things are known: the final jamo of the stem (its coda 받침
//! and vowel, which drive euphonic 으-insertion and vowel harmony) and
//! the verb's irregular class. Seven irregular classes deviate from the
//! regular pattern before a vowel ending — ㅂ, ㄷ, ㅅ, 르, 러, 우, ㅎ —
//! and they live in `data/kor/verbs.tsv`, keyed by lemma.
//!
//! The engine composes forms at the jamo level (decompose a syllable
//! into lead/vowel/coda, mutate, recompose) so the combination rules —
//! ㅡ-deletion (쓰 + 어 → 써), 아/어 contraction (오 + 아 → 와, 되 + 어 →
//! 돼), ㄹ-drop before ㄴ/ㅂ (살 → 사니, 삽니다), the 하 → 해 (여) rule,
//! and the irregular alternations — are expressed directly.
//!
//! Four surface forms are generated, the ones the two gold oracles
//! (kaikki.org and the NIKL Basic Dictionary) both attest as whole
//! words: the `-아/어` intimate form (해체, 먹어), the `-(으)니` connective
//! (먹으니), the present adnominal `-는` (먹는), and the formal-polite
//! `-(스)ㅂ니다` (먹습니다). See `docs/kor/oracles.md`.

// ---------------------------------------------------------------------
// Hangul jamo composition.
// ---------------------------------------------------------------------

const SBASE: u32 = 0xAC00;
const VCOUNT: u32 = 21;
const TCOUNT: u32 = 28;

// Vowel (medial) jamo indices used by the rules.
const A: usize = 0; // ㅏ
const AE: usize = 1; // ㅐ
const EO: usize = 4; // ㅓ
const E: usize = 5; // ㅔ
const YEO: usize = 6; // ㅕ
const O: usize = 8; // ㅗ
const WA: usize = 9; // ㅘ
const WAE: usize = 10; // ㅙ
const OE: usize = 11; // ㅚ
const U: usize = 13; // ㅜ
const WEO: usize = 14; // ㅝ
const EU: usize = 18; // ㅡ
const I: usize = 20; // ㅣ

// Coda (final) jamo indices used by the rules.
const T_NONE: usize = 0;
const T_L: usize = 8; // ㄹ
const T_B: usize = 17; // ㅂ

const IEUNG: usize = 11; // ㅇ lead
const RIEUL_LEAD: usize = 5; // ㄹ lead
const H_LEAD: usize = 18; // ㅎ lead

/// Decompose a precomposed hangul syllable into (lead, vowel, coda)
/// jamo indices; None for anything outside the syllable block.
fn decompose(c: char) -> Option<(usize, usize, usize)> {
    let s = c as u32;
    if !(SBASE..SBASE + 11172).contains(&s) {
        return None;
    }
    let s = s - SBASE;
    #[allow(clippy::cast_possible_truncation)]
    Some((
        (s / (VCOUNT * TCOUNT)) as usize,
        ((s % (VCOUNT * TCOUNT)) / TCOUNT) as usize,
        (s % TCOUNT) as usize,
    ))
}

/// Recompose (lead, vowel, coda) jamo indices into a syllable.
fn compose(l: usize, v: usize, t: usize) -> char {
    #[allow(clippy::cast_possible_truncation)]
    let code = SBASE + (l as u32 * VCOUNT + v as u32) * TCOUNT + t as u32;
    char::from_u32(code).unwrap_or('\u{fffd}')
}

/// The syllable 아 or 어 (bare 아/어 ending) with an optional coda.
fn eo_syllable(harmony_a: bool, coda: usize) -> char {
    compose(IEUNG, if harmony_a { A } else { EO }, coda)
}

/// True when the stem's final vowel selects 아-harmony (ㅏ, ㅗ), false for
/// 어-harmony. `preceding` is used for ㅡ-stems, whose harmony looks one
/// syllable back.
fn harmony_a(vowel: usize) -> bool {
    vowel == A || vowel == O
}

/// Split a stem string into (prefix-before-last-syllable, last syllable
/// jamo triple). None when the stem does not end in a composed syllable.
fn last_syllable(stem: &str) -> Option<(String, usize, usize, usize)> {
    let last = stem.chars().next_back()?;
    let (l, v, t) = decompose(last)?;
    let head = &stem[..stem.len() - last.len_utf8()];
    Some((head.to_string(), l, v, t))
}

// ---------------------------------------------------------------------
// Irregular classes.
// ---------------------------------------------------------------------

/// The irregular class of a stem before a vowel ending. `Regular`
/// covers the productive majority (plain codas, ㄹ-stems, ㅡ-deletion,
/// and the predictable 하 → 여 rule), and is never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Regular,
    /// ㅂ → 우/오 (돕 → 도와, 굽 → 구워, 도우니).
    Bieup,
    /// ㄷ → ㄹ (듣 → 들어, 들으니).
    Digut,
    /// ㅅ drops (짓 → 지어, 지으니).
    Siot,
    /// 르 → ㄹㄹ (모르 → 몰라).
    Reu,
    /// 르 → 르러 (이르 → 이르러).
    Reo,
    /// 우 → contraction (푸 → 퍼).
    U,
    /// ㅎ drops, vowel fronts (빨갛 → 빨개, 빨가니).
    Hieut,
    /// The 그러다 class: a coda-less ㅓ contracts to ㅐ (그러 → 그래,
    /// 어쩌 → 어째); everything else regular.
    Eoae,
}

fn class_from_code(code: &str) -> Class {
    match code {
        "b" => Class::Bieup,
        "d" => Class::Digut,
        "s" => Class::Siot,
        "reu" => Class::Reu,
        "reo" => Class::Reo,
        "u" => Class::U,
        "h" => Class::Hieut,
        "eae" => Class::Eoae,
        _ => Class::Regular,
    }
}

/// The compiled-in irregular lexicon: `lemma⇥class` rows.
static LEXICON_TSV: &str = include_str!("../data/kor/verbs.tsv");

fn lexical_class(infinitive: &str) -> Class {
    for line in LEXICON_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let mut f = line.split('\t');
        if f.next() == Some(infinitive) {
            return f.next().map_or(Class::Regular, class_from_code);
        }
    }
    Class::Regular
}

// ---------------------------------------------------------------------
// The verb.
// ---------------------------------------------------------------------

/// Why an infinitive cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input does not end in `-다` / is not hangul.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a Korean verb (must be a 하다-style 다-final stem)")
    }
}

/// A conjugatable Korean verb: its dictionary form, the stem (그 form
/// minus 다), and its irregular class.
#[derive(Debug, Clone)]
pub struct Verb {
    infinitive: String,
    stem: String,
    class: Class,
}

impl Verb {
    /// Build a verb from its dictionary form (a hangul stem + `-다`).
    pub fn from_infinitive(infinitive: &str) -> Result<Self, Error> {
        let inf = infinitive.trim();
        if inf.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        let stem = inf.strip_suffix('다').ok_or(Error::NotAVerb)?;
        if stem.is_empty() || stem.chars().any(|c| decompose(c).is_none()) {
            return Err(Error::NotAVerb);
        }
        Ok(Self {
            infinitive: inf.to_string(),
            stem: stem.to_string(),
            class: lexical_class(inf),
        })
    }

    /// The dictionary form.
    #[must_use]
    pub fn infinitive(&self) -> String {
        self.infinitive.clone()
    }

    /// Whether the stem is exactly 돕 or 곱 (the two ㅂ-verbs that take
    /// 오/와 rather than 우/워).
    fn is_o_bieup(&self) -> bool {
        self.stem == "돕" || self.stem == "곱"
    }

    /// The intimate `-아/어` form (해체 종결, 먹어), applying vowel
    /// contraction and the irregular alternations.
    #[must_use]
    pub fn intimate(&self) -> String {
        let Some((head, l, v, t)) = last_syllable(&self.stem) else {
            return self.stem.clone();
        };
        // 하 (ㅎ-lead + ㅏ, no coda) → 여-irregular → 하여 → 해.
        if t == T_NONE && l == H_LEAD && v == A {
            return format!("{head}{}", compose(H_LEAD, AE, T_NONE));
        }
        match self.class {
            Class::Bieup => {
                // Drop ㅂ; 돕/곱 → +와, else → +워.
                let base = format!("{head}{}", compose(l, v, T_NONE));
                let tail = if self.is_o_bieup() {
                    compose(IEUNG, WA, T_NONE)
                } else {
                    compose(IEUNG, WEO, T_NONE)
                };
                format!("{base}{tail}")
            }
            Class::Digut => {
                // ㄷ → ㄹ, then a bare 아/어 syllable.
                let base = format!("{head}{}", compose(l, v, T_L));
                format!("{base}{}", eo_syllable(harmony_a(v), T_NONE))
            }
            Class::Siot => {
                // ㅅ drops, hiatus stays: 지 + 어 → 지어.
                let base = format!("{head}{}", compose(l, v, T_NONE));
                format!("{base}{}", eo_syllable(harmony_a(v), T_NONE))
            }
            Class::Reu => {
                // 르 → the previous syllable gains ㄹ, then 라/러.
                let Some((h2, l2, v2, _)) = last_syllable(&head) else {
                    return self.stem.clone();
                };
                let base = format!("{h2}{}", compose(l2, v2, T_L));
                let ra = compose(RIEUL_LEAD, if harmony_a(v2) { A } else { EO }, T_NONE);
                format!("{base}{ra}")
            }
            Class::Reo => {
                // 이르 → 이르러: append 러 (ㄹ + ㅓ).
                format!("{}{}", self.stem, compose(RIEUL_LEAD, EO, T_NONE))
            }
            Class::U => {
                // 푸 → 퍼 (ㅜ + ㅓ → ㅓ).
                format!("{head}{}", compose(l, EO, T_NONE))
            }
            Class::Hieut => {
                // ㅎ drops; ㅏ → ㅐ.
                let nv = if v == A { AE } else { v };
                format!("{head}{}", compose(l, nv, T_NONE))
            }
            Class::Eoae => format!("{head}{}", compose(l, AE, T_NONE)),
            Class::Regular => self.regular_intimate(head, l, v, t),
        }
    }

    fn regular_intimate(&self, head: String, l: usize, v: usize, t: usize) -> String {
        if t != T_NONE {
            // Coda present: bare 아/어 syllable (먹어, 잡아, 살아, 만들어).
            return format!(
                "{head}{}{}",
                compose(l, v, t),
                eo_syllable(harmony_a(v), T_NONE)
            );
        }
        if v == EU {
            // ㅡ-deletion: harmony looks one syllable back.
            let a = last_syllable(&head).is_some_and(|(_, _, pv, _)| harmony_a(pv));
            return format!("{head}{}", compose(l, if a { A } else { EO }, T_NONE));
        }
        if let Some(cv) = contract(v) {
            return format!("{head}{}", compose(l, cv, T_NONE));
        }
        // No contraction rule: keep the hiatus (쉬어, 띄어).
        format!(
            "{head}{}{}",
            compose(l, v, T_NONE),
            eo_syllable(harmony_a(v), T_NONE)
        )
    }

    /// The `-(으)니` connective (먹으니, 사니, 하니).
    #[must_use]
    pub fn connective_ni(&self) -> String {
        let ni = '니';
        let Some((head, l, v, t)) = last_syllable(&self.stem) else {
            return format!("{}{ni}", self.stem);
        };
        match self.class {
            Class::Bieup => {
                let base = format!("{head}{}", compose(l, v, T_NONE));
                format!("{base}우{ni}")
            }
            Class::Digut => format!("{head}{}으{ni}", compose(l, v, T_L)),
            Class::Siot => format!("{head}{}으{ni}", compose(l, v, T_NONE)),
            Class::Hieut => format!("{head}{}{ni}", compose(l, v, T_NONE)),
            _ => self.regular_ni(head, l, v, t, ni),
        }
    }

    fn regular_ni(&self, head: String, l: usize, v: usize, t: usize, ni: char) -> String {
        match t {
            T_NONE => format!("{head}{}{ni}", compose(l, v, T_NONE)),
            T_L => format!("{head}{}{ni}", compose(l, v, T_NONE)), // 살 → 사니
            _ => format!("{head}{}으{ni}", compose(l, v, t)),
        }
    }

    /// The present adnominal `-는` (먹는, 사는, 하는).
    #[must_use]
    pub fn adnominal_present(&self) -> String {
        let neun = '는';
        let Some((head, l, v, t)) = last_syllable(&self.stem) else {
            return format!("{}{neun}", self.stem);
        };
        if t == T_L {
            // ㄹ drops before ㄴ.
            return format!("{head}{}{neun}", compose(l, v, T_NONE));
        }
        format!("{head}{}{neun}", compose(l, v, t))
    }

    /// The formal-polite `-(스)ㅂ니다` (먹습니다, 갑니다, 삽니다).
    #[must_use]
    pub fn formal_present(&self) -> String {
        let Some((head, l, v, t)) = last_syllable(&self.stem) else {
            return format!("{}습니다", self.stem);
        };
        match t {
            // Vowel-final: ㅂ joins the final syllable as a coda.
            T_NONE => format!("{head}{}니다", compose(l, v, T_B)),
            // ㄹ-final: drop ㄹ, then add the ㅂ coda (살 → 삽니다).
            T_L => format!("{head}{}니다", compose(l, v, T_B)),
            // Any other coda: 습니다 (먹습니다, 돕습니다, 듣습니다).
            _ => format!("{head}{}습니다", compose(l, v, t)),
        }
    }
}

/// The 아/어 contraction of a coda-less stem vowel, or None when the
/// hiatus is kept.
fn contract(v: usize) -> Option<usize> {
    Some(match v {
        A => A,     // 가 + 아 → 가
        EO => EO,   // 서 + 어 → 서
        O => WA,    // 오 + 아 → 와
        U => WEO,   // 주 + 어 → 줘
        I => YEO,   // 마시 + 어 → 마셔
        AE => AE,   // 개 + 어 → 개
        E => E,     // 세 + 어 → 세
        OE => WAE,  // 되 + 어 → 돼
        YEO => YEO, // 켜 + 어 → 켜
        _ => return None,
    })
}

// ---------------------------------------------------------------------
// The table.
// ---------------------------------------------------------------------

/// The conjugation table of a Korean verb: the four whole-word surface
/// forms the engine is verified against.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    pub infinitive: String,
    /// Intimate `-아/어` (해체, 먹어).
    pub intimate: String,
    /// `-(으)니` connective (먹으니).
    pub connective_ni: String,
    /// Present adnominal `-는` (먹는).
    pub adnominal_present: String,
    /// Formal-polite `-(스)ㅂ니다` (먹습니다).
    pub formal_present: String,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        Self {
            infinitive: v.infinitive(),
            intimate: v.intimate(),
            connective_ni: v.connective_ni(),
            adnominal_present: v.adnominal_present(),
            formal_present: v.formal_present(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(inf: &str) -> Verb {
        Verb::from_infinitive(inf).unwrap()
    }

    #[test]
    fn regular_consonant_stem() {
        let m = v("먹다");
        assert_eq!(m.intimate(), "먹어");
        assert_eq!(m.connective_ni(), "먹으니");
        assert_eq!(m.adnominal_present(), "먹는");
        assert_eq!(m.formal_present(), "먹습니다");
        assert_eq!(v("잡다").intimate(), "잡아");
        assert_eq!(v("잡다").connective_ni(), "잡으니");
    }

    #[test]
    fn vowel_contraction() {
        assert_eq!(v("가다").intimate(), "가");
        assert_eq!(v("오다").intimate(), "와");
        assert_eq!(v("보다").intimate(), "봐");
        assert_eq!(v("주다").intimate(), "줘");
        assert_eq!(v("되다").intimate(), "돼");
        assert_eq!(v("마시다").intimate(), "마셔");
        assert_eq!(v("기다리다").intimate(), "기다려");
        assert_eq!(v("가다").formal_present(), "갑니다");
        assert_eq!(v("오다").connective_ni(), "오니");
    }

    #[test]
    fn hada_and_eu() {
        assert_eq!(v("하다").intimate(), "해");
        assert_eq!(v("공부하다").intimate(), "공부해");
        assert_eq!(v("하다").formal_present(), "합니다");
        assert_eq!(v("쓰다").intimate(), "써");
        assert_eq!(v("크다").intimate(), "커");
        assert_eq!(v("따르다").intimate(), "따라"); // regular 르 (ㅡ-deletion)
        assert_eq!(v("쓰다").connective_ni(), "쓰니");
    }

    #[test]
    fn rieul_stem() {
        assert_eq!(v("살다").connective_ni(), "사니");
        assert_eq!(v("살다").adnominal_present(), "사는");
        assert_eq!(v("살다").formal_present(), "삽니다");
        assert_eq!(v("살다").intimate(), "살아");
        assert_eq!(v("만들다").formal_present(), "만듭니다");
        assert_eq!(v("만들다").adnominal_present(), "만드는");
    }

    #[test]
    fn irregular_lexicon() {
        assert_eq!(v("듣다").intimate(), "들어");
        assert_eq!(v("듣다").connective_ni(), "들으니");
        assert_eq!(v("듣다").adnominal_present(), "듣는");
        assert_eq!(v("돕다").intimate(), "도와");
        assert_eq!(v("줍다").intimate(), "주워");
        assert_eq!(v("돕다").connective_ni(), "도우니");
        assert_eq!(v("돕다").formal_present(), "돕습니다");
        assert_eq!(v("짓다").intimate(), "지어");
        assert_eq!(v("낫다").intimate(), "나아");
        assert_eq!(v("짓다").connective_ni(), "지으니");
        assert_eq!(v("부르다").intimate(), "불러");
        assert_eq!(v("무르다").intimate(), "물러");
        assert_eq!(v("부르다").connective_ni(), "부르니");
    }

    #[test]
    fn rejects_non_verbs() {
        assert!(Verb::from_infinitive("hello").is_err());
        assert!(Verb::from_infinitive("먹").is_err());
        assert!(Verb::from_infinitive("먹다 보다").is_err());
    }
}
