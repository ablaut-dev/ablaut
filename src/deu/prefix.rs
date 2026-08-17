//! Verb prefixes (Layer A of `docs/design.md`): prefixed verbs are derived,
//! not stored. *aufstehen* conjugates exactly like *stehen*; only the
//! prefix's behavior is lexical.
//!
//! Separable prefixes are stressed and split off in finite forms
//! (*ich stehe auf*), infix *ge-* and *zu* (*aufgestanden*, *aufzustehen*).
//! Inseparable prefixes are unstressed, never split, and suppress the
//! participle's *ge-* (*verstanden*).
//!
//! Dual prefixes (*durch-, über-, um-, unter-, wieder-, wider-, voll-*) can
//! behave either way depending on the lexeme (*übersetzen* "translate" vs
//! "ferry across"); we default each to its statistically dominant behavior
//! until per-lexeme flags land in the lexicon.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separability {
    Separable,
    Inseparable,
    /// Multiword lemma (Rad fahren): behaves like a separable particle in
    /// finite forms (fahre Rad) but stays a free word everywhere
    /// (Rad gefahren, Rad zu fahren).
    Phrasal,
    /// Pseudo-separable compounds that never actually split (lobsingen,
    /// obsiegen): fused finite forms (lobsang) but separable-style ge- and
    /// zu-infixes (lobgesungen, lobzusingen).
    Fused,
    /// Reflexive lemma (sich freuen): the pronoun agrees with the subject
    /// in finite forms (freue mich, freust dich) and stays *sich* in
    /// citation forms (sich gefreut, sich zu freuen).
    Reflexive,
}

/// Longest-first so compounds win (herausfinden ≠ her+ausfinden).
const SEPARABLE: &[&str] = &[
    "auseinander",
    "entgegen",
    "gegenüber",
    "herunter",
    "herüber",
    "hinunter",
    "hinüber",
    "zusammen",
    "überein",
    "herauf",
    "heraus",
    "herein",
    "hervor",
    "hinauf",
    "hinaus",
    "hinein",
    "nieder",
    "weiter",
    "wieder",
    "zurück",
    "durcheinander",
    "hintereinander",
    "nebeneinander",
    "untereinander",
    "übereinander",
    "aufeinander",
    "aneinander",
    "beieinander",
    "gegeneinander",
    "ineinander",
    "umeinander",
    "voneinander",
    "zueinander",
    "abhanden",
    "aufrecht",
    "beiseite",
    "hinterher",
    "vornüber",
    "vorüber",
    "zufrieden",
    "zugrunde",
    "zunichte",
    "zustande",
    "zutage",
    "zuwider",
    "zupass",
    "zugute",
    "dafür",
    "darüber",
    "daneben",
    "entzwei",
    "stehen",
    "stand",
    "überhand",
    "falsch",
    "übrig",
    "brach",
    "näher",
    "lahm",
    "vonstatten",
    "stecken",
    "liegen",
    "stramm",
    "blank",
    "krank",
    "satt",
    "inne",
    "gern",
    "feil",
    "leer",
    "kahl",
    "hier",
    "rund",
    "bey",
    "auß",
    "mal",
    "wichtig",
    "recht",
    "selig",
    "übel",
    "quer",
    "tief",
    "weis",
    "schlecht",
    "schlapp",
    "stark",
    "scheckig",
    "gering",
    "gesund",
    "bereit",
    "sauber",
    "publik",
    "locker",
    "besser",
    "dicht",
    "glatt",
    "höher",
    "kehrt",
    "mobil",
    "flach",
    "klug",
    "halt",
    "neu",
    "alt",
    "arm",
    "bekannt",
    "schwarz",
    "gerade",
    "gefangen",
    "fremd",
    "krumm",
    "platt",
    "blau",
    "dank",
    "fehl",
    "frei",
    "kalt",
    "kund",
    "rot",
    "warm",
    "wund",
    "bloß",
    "drauf",
    "dran",
    "drin",
    "spazieren",
    "verloren",
    "zurecht",
    "zwischen",
    "entlang",
    "hintan",
    "richtig",
    "allein",
    "kennen",
    "sitzen",
    "empor",
    "gleich",
    "sicher",
    "statt",
    "fertig",
    "kaputt",
    "runter",
    "rüber",
    "offen",
    "schön",
    "still",
    "davon",
    "preis",
    "schief",
    "weich",
    "acht",
    "alle",
    "durch",
    "fern",
    "fest",
    "fort",
    "groß",
    "heim",
    "heiß",
    "hoch",
    "irre",
    "klar",
    "klein",
    "kurz",
    "lang",
    "lieb",
    "matt",
    "nahe",
    "nass",
    "raus",
    "rein",
    "wach",
    "wahr",
    "wett",
    "wohl",
    "gut",
    "rum",
    "los",
    "mit",
    "nach",
    "ran",
    "teil",
    "tot",
    "weg",
    "weh",
    "da",
    "ab",
    "an",
    "auf",
    "aus",
    "bei",
    "dar",
    "ein",
    "her",
    "hin",
    "um",
    "vor",
    "zu",
];

const INSEPARABLE: &[&str] = &[
    "hinter", "unter", "wider", "miss", "miß", "voll", "über", "emp", "ent", "ver", "zer", "be",
    "er", "ge",
];

/// Split an infinitive into (prefix, behavior, base infinitive) if the base
/// looks like a real verb. `valid` is the caller's judgment of the remainder
/// (lexicon hit, or a weak stem with a vowel and length ≥ 3 — this rejects
/// false splits like be+ten, zu+cken, ge+igen, fest+igen).
pub fn split(infinitive: &str, valid: impl Fn(&str) -> bool) -> Option<(&str, Separability, &str)> {
    let mut candidates: Vec<(&str, Separability)> = Vec::new();
    for p in SEPARABLE {
        candidates.push((p, Separability::Separable));
    }
    for p in INSEPARABLE {
        candidates.push((p, Separability::Inseparable));
    }
    // Longest prefix wins across both lists (verbessern: ver-, not v+erbessern).
    candidates.sort_by_key(|(p, _)| std::cmp::Reverse(p.len()));
    for (prefix, sep) in candidates {
        if let Some(base) = infinitive.strip_prefix(prefix) {
            if valid(base) {
                return Some((prefix, sep, base));
            }
        }
    }
    None
}
