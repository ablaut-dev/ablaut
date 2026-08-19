//! Japanese verb conjugation: the katsuyou-kei (活用形) engine plus a
//! compiled-in inflection-class lexicon.
//!
//! Japanese verb morphology is almost exceptionless *once the class is
//! known*, but the class is the one thing a verb's dictionary form cannot
//! reveal: 着る (kiru, "wear") is ichidan → 着 while 切る (kiru, "cut") is
//! godan → 切り. So the single stored fact per verb is its class
//! (`data/jpn/verbs.tsv`, mined from IPADIC); everything else is rule.
//!
//! Classes:
//! - **ichidan** (一段): drop the final る (食べる → 食べ).
//! - **godan** (五段): the final kana shifts along its consonant row
//!   (書く: 未然 書か, 連用 書き, 終止 書く, 已然/命令 書け), and the past
//!   た-form takes the euphonic *onbin* stem (書く → 書いた, 泳ぐ → 泳いだ,
//!   死ぬ → 死んだ, 待つ/切る/買う → 待った/切った/買った).
//! - **godan_iku** (行く-family): promoted-onbin past 行った, not *行いた.
//! - **godan_u** (問う-family): literary past 問うた.
//! - **rspecial** (なさる/くださる/…): 連用 なさい, not *なさり.
//! - **suru / zuru / kuru**: the three irregular verbs (and their
//!   compounds 愛する, 感ずる).
//!
//! The scored gold slots are 終止形 (`V;NFIN`, the citation form), 連用形
//! (`V;CONT`) and the plain past た-form (`V;PST`) — the surface strings
//! both oracles attest. The engine additionally exposes 未然形, 仮定形 and
//! 命令形 for the full table.

/// The six classical katsuyou-kei plus the plain past — the slots the
/// engine can fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Form {
    /// 終止形 — terminal / dictionary form (書く).
    Terminal,
    /// 連用形 — continuative / masu-stem (書き).
    Continuative,
    /// 未然形 — irrealis (書か).
    Irrealis,
    /// 仮定形 — hypothetical / realis (書け).
    Hypothetical,
    /// 命令形 — imperative (書け).
    Imperative,
    /// Plain past — た/だ-form (書いた).
    Past,
}

/// Inflection class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Ichidan,
    Godan,
    /// 行く-family: past takes 促音便 った (行った).
    GodanIku,
    /// 問う-family: literary past うた (問うた).
    GodanU,
    /// なさる-family: continuative in い (なさい).
    RSpecial,
    Suru,
    Zuru,
    Kuru,
}

impl Class {
    fn parse(code: &str) -> Option<Self> {
        Some(match code {
            "ichidan" => Self::Ichidan,
            "godan" => Self::Godan,
            "godan_iku" => Self::GodanIku,
            "godan_u" => Self::GodanU,
            "rspecial" => Self::RSpecial,
            "suru" => Self::Suru,
            "zuru" => Self::Zuru,
            "kuru" => Self::Kuru,
            _ => return None,
        })
    }
}

/// Why a word cannot be conjugated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Not in the inflection-class lexicon (the class is unknowable from
    /// the surface, so an unlisted verb cannot be conjugated).
    Unsupported,
    /// Empty or multi-word input.
    NotAVerb,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "not a known Japanese verb"),
            Self::NotAVerb => write!(f, "not a Japanese verb"),
        }
    }
}

/// The compiled-in inflection-class lexicon (lemma ⇥ class code).
static LEXICON_TSV: &str = include_str!("../data/jpn/verbs.tsv");

fn lookup_class(lemma: &str) -> Option<Class> {
    for line in LEXICON_TSV.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let mut cols = line.split('\t');
        if cols.next() == Some(lemma) {
            return cols.next().and_then(Class::parse);
        }
    }
    None
}

/// Godan row shift: the final u-row kana → its [a-row, i-row, e-row]
/// counterparts (未然 / 連用 / 已然·命令). う belongs to the wa-row so its
/// irrealis is わ.
fn godan_shift(last: char) -> Option<(char, char, char)> {
    Some(match last {
        'う' => ('わ', 'い', 'え'),
        'く' => ('か', 'き', 'け'),
        'ぐ' => ('が', 'ぎ', 'げ'),
        'す' => ('さ', 'し', 'せ'),
        'つ' => ('た', 'ち', 'て'),
        'ぬ' => ('な', 'に', 'ね'),
        'ぶ' => ('ば', 'び', 'べ'),
        'む' => ('ま', 'み', 'め'),
        'る' => ('ら', 'り', 'れ'),
        _ => return None,
    })
}

/// The euphonic (onbin) past ending for a godan final kana: the onbin
/// stem's tail plus the (possibly voiced) auxiliary. Returns the whole
/// ending that replaces the final kana (書く: く → いた).
fn godan_past_ending(last: char) -> Option<&'static str> {
    Some(match last {
        'く' => "いた",
        'ぐ' => "いだ",
        'す' => "した",
        'つ' | 'る' | 'う' => "った",
        'ぬ' | 'ぶ' | 'む' => "んだ",
        _ => return None,
    })
}

/// A conjugatable Japanese verb: its dictionary form and class.
#[derive(Debug, Clone)]
pub struct Verb {
    lemma: String,
    class: Class,
}

/// Lemma with the final `char` removed, and that char.
fn split_last(lemma: &str) -> (String, char) {
    let last = lemma.chars().next_back().unwrap_or('\0');
    let head = &lemma[..lemma.len() - last.len_utf8()];
    (head.to_string(), last)
}

impl Verb {
    /// Build a verb from its dictionary form. The class comes from the
    /// compiled-in lexicon; a verb not listed there is `Unsupported`.
    pub fn from_infinitive(lemma: &str) -> Result<Self, Error> {
        let lemma = lemma.trim();
        if lemma.is_empty() || lemma.contains(char::is_whitespace) {
            return Err(Error::NotAVerb);
        }
        match lookup_class(lemma) {
            Some(class) => Ok(Self {
                lemma: lemma.to_string(),
                class,
            }),
            None => Err(Error::Unsupported),
        }
    }

    /// The dictionary / terminal form.
    #[must_use]
    pub fn infinitive(&self) -> String {
        self.lemma.clone()
    }

    /// The stem shared by 連用形 and 未然形 of the irregular -する/-ずる
    /// verbs: the lemma minus the する/ずる tail.
    fn irregular_head(&self, tail: &str) -> String {
        self.lemma
            .strip_suffix(tail)
            .unwrap_or(&self.lemma)
            .to_string()
    }

    /// One surface form for a slot. `None` when the class does not define
    /// it (no slot is currently ever `None`, but the shape mirrors the
    /// other engines).
    #[must_use]
    pub fn form(&self, form: Form) -> String {
        match self.class {
            Class::Ichidan => self.ichidan(form),
            Class::Godan | Class::GodanIku | Class::GodanU | Class::RSpecial => self.godan(form),
            Class::Suru => self.suru(form),
            Class::Zuru => self.zuru(form),
            Class::Kuru => self.kuru(form),
        }
    }

    fn ichidan(&self, form: Form) -> String {
        let (stem, _) = split_last(&self.lemma);
        match form {
            Form::Terminal => self.lemma.clone(),
            Form::Continuative | Form::Irrealis => stem,
            Form::Hypothetical => format!("{stem}れ"),
            Form::Imperative => format!("{stem}ろ"),
            Form::Past => format!("{stem}た"),
        }
    }

    fn godan(&self, form: Form) -> String {
        let (head, last) = split_last(&self.lemma);
        // godan_shift is defined for every u-row kana; the lexicon only
        // ever assigns godan classes to such lemmas.
        let (a, i, e) = godan_shift(last).unwrap_or((last, last, last));
        match form {
            Form::Terminal => self.lemma.clone(),
            Form::Continuative => {
                if self.class == Class::RSpecial {
                    // なさる → なさい (i-onbin of the continuative).
                    format!("{head}い")
                } else {
                    format!("{head}{i}")
                }
            }
            Form::Irrealis => format!("{head}{a}"),
            Form::Hypothetical | Form::Imperative => format!("{head}{e}"),
            Form::Past => {
                let ending = match self.class {
                    Class::GodanIku => "った",
                    Class::GodanU => "うた",
                    _ => godan_past_ending(last).unwrap_or("った"),
                };
                format!("{head}{ending}")
            }
        }
    }

    fn suru(&self, form: Form) -> String {
        let head = self.irregular_head("する");
        match form {
            Form::Terminal => self.lemma.clone(),
            Form::Continuative | Form::Irrealis => format!("{head}し"),
            Form::Hypothetical => format!("{head}すれ"),
            Form::Imperative => format!("{head}しろ"),
            Form::Past => format!("{head}した"),
        }
    }

    fn zuru(&self, form: Form) -> String {
        let head = self.irregular_head("ずる");
        match form {
            Form::Terminal => self.lemma.clone(),
            Form::Continuative | Form::Irrealis => format!("{head}じ"),
            Form::Hypothetical => format!("{head}ずれ"),
            Form::Imperative => format!("{head}じろ"),
            Form::Past => format!("{head}じた"),
        }
    }

    fn kuru(&self, form: Form) -> String {
        // Kana-spelled くる shifts く→き (やってくる → やってき); a kanji
        // spelling (来る) keeps the kanji stem (来), whose reading carries
        // the vowel change. `base` is the invariant part before the final
        // く/来 of the stem.
        let (head, _) = split_last(&self.lemma); // lemma minus る
        let kana = self.lemma.ends_with("くる");
        let base = if kana {
            head[..head.len() - 'く'.len_utf8()].to_string()
        } else {
            head // "...来"
        };
        // The stem vowels: continuative き, irrealis こ, imperative こい,
        // hypothetical くれ — spelled onto `base` in kana, or read off the
        // kanji stem which is `base` itself.
        let (cont, irr, hyp, imp) = if kana {
            (
                format!("{base}き"),
                format!("{base}こ"),
                format!("{base}くれ"),
                format!("{base}こい"),
            )
        } else {
            (
                base.clone(),
                base.clone(),
                format!("{base}れ"),
                format!("{base}い"),
            )
        };
        match form {
            Form::Terminal => self.lemma.clone(),
            Form::Continuative => cont,
            Form::Irrealis => irr,
            Form::Hypothetical => hyp,
            Form::Imperative => imp,
            Form::Past => format!("{cont}た"),
        }
    }
}

/// The conjugation table of a Japanese verb as one plain struct — shared
/// by the WebAssembly and Python bindings.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Table {
    /// 終止形 — dictionary form.
    pub terminal: String,
    /// 連用形 — continuative / masu-stem.
    pub continuative: String,
    /// 未然形 — irrealis.
    pub irrealis: String,
    /// 仮定形 — hypothetical.
    pub hypothetical: String,
    /// 命令形 — imperative.
    pub imperative: String,
    /// Plain past — た/だ-form.
    pub past: String,
}

impl Table {
    #[must_use]
    pub fn build(v: &Verb) -> Self {
        Self {
            terminal: v.form(Form::Terminal),
            continuative: v.form(Form::Continuative),
            irrealis: v.form(Form::Irrealis),
            hypothetical: v.form(Form::Hypothetical),
            imperative: v.form(Form::Imperative),
            past: v.form(Form::Past),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(lemma: &str) -> Verb {
        Verb::from_infinitive(lemma).unwrap()
    }

    fn all(lemma: &str) -> (String, String, String) {
        let verb = v(lemma);
        (
            verb.form(Form::Continuative),
            verb.form(Form::Past),
            verb.form(Form::Imperative),
        )
    }

    #[test]
    fn godan_rows() {
        assert_eq!(all("書く"), ("書き".into(), "書いた".into(), "書け".into()));
        assert_eq!(all("泳ぐ"), ("泳ぎ".into(), "泳いだ".into(), "泳げ".into()));
        assert_eq!(all("話す"), ("話し".into(), "話した".into(), "話せ".into()));
        assert_eq!(all("待つ"), ("待ち".into(), "待った".into(), "待て".into()));
        assert_eq!(all("死ぬ"), ("死に".into(), "死んだ".into(), "死ね".into()));
        assert_eq!(all("呼ぶ"), ("呼び".into(), "呼んだ".into(), "呼べ".into()));
        assert_eq!(all("読む"), ("読み".into(), "読んだ".into(), "読め".into()));
        assert_eq!(all("切る"), ("切り".into(), "切った".into(), "切れ".into()));
        assert_eq!(all("買う"), ("買い".into(), "買った".into(), "買え".into()));
    }

    #[test]
    fn godan_irrealis() {
        assert_eq!(v("書く").form(Form::Irrealis), "書か");
        assert_eq!(v("買う").form(Form::Irrealis), "買わ");
    }

    #[test]
    fn onbin_exceptions() {
        assert_eq!(v("行く").form(Form::Past), "行った");
        assert_eq!(v("問う").form(Form::Past), "問うた");
    }

    #[test]
    fn ichidan() {
        assert_eq!(
            all("食べる"),
            ("食べ".into(), "食べた".into(), "食べろ".into())
        );
        assert_eq!(v("見る").form(Form::Continuative), "見");
    }

    #[test]
    fn irregular() {
        assert_eq!(all("する"), ("し".into(), "した".into(), "しろ".into()));
        assert_eq!(
            all("愛する"),
            ("愛し".into(), "愛した".into(), "愛しろ".into())
        );
        assert_eq!(v("感ずる").form(Form::Continuative), "感じ");
        assert_eq!(v("来る").form(Form::Continuative), "来");
        assert_eq!(v("来る").form(Form::Past), "来た");
    }

    #[test]
    fn class_is_lexical() {
        // 着る ichidan, 切る godan — same surface, different class.
        assert_eq!(v("着る").form(Form::Continuative), "着");
        assert_eq!(v("切る").form(Form::Continuative), "切り");
    }

    #[test]
    fn unknown_verb() {
        assert!(matches!(
            Verb::from_infinitive("ぬぬぬる"),
            Err(Error::Unsupported)
        ));
    }
}
