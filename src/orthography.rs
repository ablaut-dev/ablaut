//! Morphophonological and orthographic surface rules (Layer D of
//! `docs/ontology.md`), applied when an ending attaches to a stem.

/// True if the stem requires *e*-epenthesis before endings in `-st`/`-t`:
/// stems in *d/t* (`arbeit-` → *du arbeitest*) and stems in *m/n* preceded by
/// an obstruent (`atm-` → *du atmest*, `rechn-` → *du rechnest*, but
/// `lern-` → *du lernst*, `wohn-` → *du wohnst*).
pub(crate) fn needs_epenthesis(stem: &str) -> bool {
    let mut chars = stem.chars().rev();
    let (last, before, before2) = (chars.next(), chars.next(), chars.next());
    match (before, last) {
        (_, Some('d' | 't')) => true,
        // Silent lengthening h (wohn-) blocks epenthesis, but the digraph
        // "ch" (rechn-) is an obstruent and licenses it.
        (Some('h'), Some('m' | 'n')) => before2 == Some('c'),
        (Some(b), Some('m' | 'n')) => !matches!(b, 'l' | 'r' | 'm' | 'n' | 'a' | 'e' | 'i' | 'o' | 'u' | 'ä' | 'ö' | 'ü'),
        _ => false,
    }
}

/// True if the stem triggers *s*-coalescence of the 2sg `-st` ending:
/// stems in *s/ss/ß/x/z* (*du heißt*, *du sitzt*, *du tanzt*).
fn coalesces_s(stem: &str) -> bool {
    matches!(stem.chars().last(), Some('s' | 'ß' | 'x' | 'z'))
}

/// Attach an ending to a stem, applying the surface rules.
///
/// `schwa_stem` marks verbs whose infinitive ends in *-eln/-ern*: their stem
/// ends in an unstressed schwa syllable, which licenses elision
/// (*ich sammle*, *du sammelst* rather than \**sammelest* in Konjunktiv I).
/// It must be a lexical/infinitive-derived flag, not a surface check on the
/// stem — `spiel-` also ends in "el" but never elides.
pub(crate) fn attach(stem: &str, ending: &str, schwa_stem: bool) -> String {
    // 1sg-style bare `-e` on an -eln stem: the stem's schwa elides (sammle).
    // Fuller e-initial endings keep their e (Konjunktiv I du sammelest);
    // dropping it would collapse Konjunktiv I into the indicative.
    if schwa_stem && ending == "e" && stem.ends_with("el") {
        return format!("{}le", &stem[..stem.len() - 2]);
    }
    // Stem-final e merges with an e-initial ending (knie+e → knie,
    // knie+est → kniest).
    if stem.ends_with('e') {
        if let Some(rest) = ending.strip_prefix('e') {
            return format!("{stem}{rest}");
        }
    }
    // s-coalescence: 2sg present -st loses its s (du tanzt).
    if ending == "st" && coalesces_s(stem) {
        return format!("{stem}t");
    }
    // e-epenthesis before -st and t-initial endings (arbeitest, arbeitete).
    if needs_epenthesis(stem) && (ending == "st" || ending.starts_with('t')) {
        return format!("{stem}e{ending}");
    }
    format!("{stem}{ending}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epenthesis() {
        assert!(needs_epenthesis("arbeit"));
        assert!(needs_epenthesis("red"));
        assert!(needs_epenthesis("atm"));
        assert!(needs_epenthesis("rechn"));
        assert!(!needs_epenthesis("lern"));
        assert!(!needs_epenthesis("wohn"));
        assert!(!needs_epenthesis("kauf"));
    }

    #[test]
    fn coalescence() {
        assert_eq!(attach("tanz", "st", false), "tanzt");
        assert_eq!(attach("heiß", "st", false), "heißt");
        assert_eq!(attach("kauf", "st", false), "kaufst");
    }
}
