//! Perso-Arabic script normalization — a **general, language-neutral**
//! module shared by every engine written in the Arabic script.
//!
//! It exists so that Persian (`pes`, Farsi) and the upcoming Urdu
//! (`urd`, Nastaliq) engines — and any other Perso-Arabic language added
//! later — fold their input the same way instead of each re-deriving the
//! rules. Nothing here is Persian-specific: the folding table only
//! touches characters whose *Arabic* code point has a distinct
//! *Perso-Arabic* code point that both Persian and Urdu prefer (the kaf
//! ك↔ک and the yeh ي↔ی), never a letter one language keeps and another
//! drops.
//!
//! Four independent concerns, each toggleable through [`Options`] so a
//! caller can keep what a given corpus needs and leave the rest:
//!
//! * **RTL / directional controls** — the invisible bidi marks that
//!   creep into copy-pasted text (LRM `U+200E`, RLM `U+200F`, ALM
//!   `U+061C`) carry no phonology and are always dropped.
//! * **ZWNJ / ZWJ** — the zero-width non-joiner `U+200C` is orthographic
//!   glue (Persian می‌رود, Urdu کر‌نا): whether it is written is a
//!   spelling choice, and different lexicons make different ones (kaikki
//!   strips it, so میرود; a dictionary may keep it). Folding it to a
//!   consistent state — stripped, here — lets forms compare equal. The
//!   zero-width *joiner* `U+200D` is dropped with it.
//! * **Short-vowel diacritics** (harakat) — fatḥa/kasra/ḍamma, the
//!   tanwīn, the šadda and sukūn, the superscript alef `U+0670`, and the
//!   tatwīl/kashida `U+0640` elongation. These are optional pointing;
//!   corpora rarely agree on them, so they are stripped.
//! * **Arabic ↔ Perso-Arabic letter folding** — the Arabic kaf `ك`
//!   `U+0643` → keheh `ک` `U+06A9`, the Arabic yeh `ي` `U+064A` and alef
//!   maqṣūra `ى` `U+0649` → farsi yeh `ی` `U+06CC`, the teh marbūṭa `ة`
//!   `U+0629` → heh `ه`, and the hamza-seated alefs `أ`/`إ`/`ٱ` → bare
//!   alef `ا`. Alef madda `آ` is **kept** — it is a distinct vowel, not
//!   a spelling variant.
//!
//! [`normalize`] applies all four (the right default for a Persian or
//! Urdu lexicon); [`normalize_with`] takes an explicit [`Options`].
//!
//! ```
//! use ablaut::perso_arabic::normalize;
//! // Arabic yeh/kaf folded, ZWNJ and harakat stripped:
//! assert_eq!(normalize("مي‌كُنَد"), "میکند");
//! ```

/// Which normalizations to apply. All four are independent; [`all`]
/// (also [`Default`]) turns everything on, which is what a Persian or
/// Urdu corpus wants.
///
/// [`all`]: Options::all
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Drop the bidirectional control marks (LRM, RLM, ALM).
    pub strip_directional: bool,
    /// Drop the zero-width non-joiner and joiner.
    pub strip_joiners: bool,
    /// Drop the short-vowel diacritics, tanwīn, šadda, sukūn,
    /// superscript alef and tatwīl.
    pub strip_diacritics: bool,
    /// Fold Arabic letter shapes onto their Perso-Arabic equivalents.
    pub fold_letters: bool,
}

impl Options {
    /// Everything on — the Persian/Urdu default.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            strip_directional: true,
            strip_joiners: true,
            strip_diacritics: true,
            fold_letters: true,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::all()
    }
}

/// A bidirectional/directional control mark: invisible, phonology-free.
#[must_use]
pub fn is_directional(c: char) -> bool {
    matches!(c, '\u{200E}' | '\u{200F}' | '\u{061C}')
}

/// The zero-width non-joiner (`U+200C`) or joiner (`U+200D`).
#[must_use]
pub fn is_joiner(c: char) -> bool {
    matches!(c, '\u{200C}' | '\u{200D}')
}

/// A short-vowel or other combining Arabic diacritic that carries no
/// distinction in an unvocalized corpus: the harakat `U+064B..=U+0652`,
/// the superscript alef `U+0670`, and the tatwīl/kashida `U+0640`.
#[must_use]
pub fn is_diacritic(c: char) -> bool {
    matches!(c, '\u{064B}'..='\u{0652}' | '\u{0670}' | '\u{0640}')
}

/// Fold a single Arabic-script character onto its canonical
/// Perso-Arabic form; characters with no folding are returned as-is.
/// Deliberately conservative — only the shapes Persian and Urdu both
/// normalize (see the module comment). Alef madda `آ` is not folded.
#[must_use]
pub fn fold_letter(c: char) -> char {
    match c {
        '\u{0643}' => '\u{06A9}',                           // ك kaf → ک keheh
        '\u{064A}' | '\u{0649}' => '\u{06CC}',              // ي yeh, ى maqṣūra → ی
        '\u{0629}' => '\u{0647}',                           // ة teh marbūṭa → ه heh
        '\u{0623}' | '\u{0625}' | '\u{0671}' => '\u{0627}', // أ إ ٱ → ا alef
        _ => c,
    }
}

/// Normalize with the full Persian/Urdu defaults ([`Options::all`]):
/// strip directional marks, joiners and diacritics, and fold Arabic
/// letters. The result is what should be stored, compared and indexed.
#[must_use]
pub fn normalize(s: &str) -> String {
    normalize_with(s, Options::all())
}

/// Normalize under an explicit [`Options`]. The passes compose in a
/// single left-to-right walk; input is otherwise left untouched (no
/// case folding — the script is caseless — and no whitespace changes).
#[must_use]
pub fn normalize_with(s: &str, opts: Options) -> String {
    let drop = |c: char| {
        (opts.strip_directional && is_directional(c))
            || (opts.strip_joiners && is_joiner(c))
            || (opts.strip_diacritics && is_diacritic(c))
    };
    s.chars()
        .filter(|&c| !drop(c))
        .map(|c| if opts.fold_letters { fold_letter(c) } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_arabic_kaf_and_yeh() {
        // Arabic kaf U+0643 and yeh U+064A → Perso-Arabic ک / ی.
        assert_eq!(normalize("كتاب"), "کتاب");
        assert_eq!(normalize("يک"), "یک");
        assert_eq!(fold_letter('\u{0643}'), '\u{06A9}');
        assert_eq!(fold_letter('\u{064A}'), '\u{06CC}');
    }

    #[test]
    fn strips_zwnj_and_diacritics() {
        // می‌کُنَد with ZWNJ and harakat → میکند.
        assert_eq!(normalize("می‌کُنَد"), "میکند");
        assert_eq!(normalize("مَدْرَسَه"), "مدرسه");
    }

    #[test]
    fn strips_directional_marks() {
        assert_eq!(normalize("\u{200F}رفتن\u{200E}"), "رفتن");
    }

    #[test]
    fn madda_alef_is_kept_but_hamza_seats_fold() {
        assert_eq!(normalize("آمد"), "آمد"); // آ kept
        assert_eq!(normalize("أحمد"), "احمد"); // أ → ا
        assert_eq!(normalize("مسئله"), "مسئله"); // ئ untouched
    }

    #[test]
    fn options_are_independent() {
        let keep_zwnj = Options {
            strip_joiners: false,
            ..Options::all()
        };
        assert_eq!(normalize_with("می‌کند", keep_zwnj), "می‌کند");
        let no_fold = Options {
            fold_letters: false,
            ..Options::all()
        };
        assert_eq!(normalize_with("كتاب", no_fold), "كتاب");
    }

    #[test]
    fn idempotent() {
        for w in ["میکند", "کتاب", "آمد", "رفته‌ام"] {
            assert_eq!(normalize(&normalize(w)), normalize(w));
        }
    }
}
