//! Panic-safety: no input string, however malformed, may crash the library.
//! Every form of every accepted verb must generate without panicking.

use ablaut::{AnalyticTense, Mood, Number, Person, Tense, Verb};
use proptest::prelude::*;

fn exercise(verb: &Verb) {
    let persons = [Person::First, Person::Second, Person::Third];
    let numbers = [Number::Singular, Number::Plural];
    let moods = [Mood::Indicative, Mood::KonjunktivI, Mood::KonjunktivII];
    for p in persons {
        for n in numbers {
            for m in moods {
                for t in [Tense::Present, Tense::Preterite] {
                    let _ = verb.conjugate(t, m, p, n);
                    let _ = verb.passive(t, m, p, n);
                    let _ = verb.statal_passive(t, m, p, n);
                }
                for a in [
                    AnalyticTense::Perfect,
                    AnalyticTense::Pluperfect,
                    AnalyticTense::FutureI,
                    AnalyticTense::FutureII,
                ] {
                    let _ = verb.analytic(a, m, p, n);
                }
            }
            let _ = verb.imperative(n);
        }
    }
    let _ = verb.past_participle();
    let _ = verb.present_participle();
    let _ = verb.zu_infinitive();
    let _ = verb.perfect_infinitive();
    let _ = verb.imperative_first_plural();
    let _ = verb.imperative_polite();
    let _ = verb.auxiliary();
    let _ = verb.is_lexical();
}

proptest! {
    /// Fully arbitrary unicode: constructor may reject, must never panic.
    #[test]
    fn arbitrary_strings_never_panic(s in "\\PC{0,24}") {
        if let Ok(v) = Verb::from_infinitive(&s) {
            exercise(&v);
        }
    }

    /// Infinitive-shaped strings: these mostly construct, so the whole
    /// paradigm generator gets exercised on plausible-but-unseen verbs.
    #[test]
    fn infinitive_shaped_strings_never_panic(
        stem in "[a-zäöüß]{0,12}",
        suffix in prop::sample::select(vec!["en", "eln", "ern", "n", "ieren", " fahren"]),
    ) {
        let s = format!("{stem}{suffix}");
        if let Ok(v) = Verb::from_infinitive(&s) {
            exercise(&v);
        }
    }
}

#[test]
fn hostile_fixed_inputs() {
    for s in [
        "",
        "n",
        "en",
        " ",
        "  en",
        "ßen",
        "ßß",
        "🦀en",
        "auf",
        "aufen",
        "zuzuzun",
        "einen",
        "geen",
        "e\u{301}n",
        "verver",
        " Rad fahren ",
        "a b c den",
        "\u{0}en",
        "ln",
        "rn",
        "een",
    ] {
        if let Ok(v) = Verb::from_infinitive(s) {
            exercise(&v);
        }
    }
}
