//! The feature space of German conjugation (Layer B of `docs/ontology.md`).
//!
//! Feature bundles are plain Rust enums so that impossible requests are
//! unrepresentable: the imperative, for instance, is a separate method on
//! [`crate::Verb`] taking only a number, because it has no person or tense.

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

/// The two synthetic tenses of German. All other tenses (Perfekt, Futur, …)
/// are analytic: they are composed from an auxiliary plus a participle or
/// infinitive, and belong to the compositional layer, not the morphological
/// core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tense {
    Present,
    Preterite,
}

/// Synthetic moods. The imperative is deliberately absent: it exists only in
/// the 2nd person present and is exposed as its own method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mood {
    Indicative,
    KonjunktivI,
    KonjunktivII,
}

impl Person {
    /// Index into a six-slot paradigm row (1sg..3pl).
    pub(crate) fn index(self, number: Number) -> usize {
        let p = match self {
            Person::First => 0,
            Person::Second => 1,
            Person::Third => 2,
        };
        match number {
            Number::Singular => p,
            Number::Plural => p + 3,
        }
    }
}
