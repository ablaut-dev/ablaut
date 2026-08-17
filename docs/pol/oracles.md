# Polish: skipped — the loop degenerates in practice

Polish looks well-resourced but the two-oracle criterion fails on
inspection (2026-08-17):

1. **SGJP / Morfeusz** (sgjp.pl, 2025-05-11 dump): excellent — full
   paradigms, composed past-tense forms with person clitics
   (mogłem/mogłaś), ~30k verb lemmas. This would be the primary.
2. No usable second lineage:
   - **kaikki.org Polish**: Wiktextract fails to expand the Polish
     conjugation templates — of 12,277 verb entries with a
     conjugation section, exactly **3** yield a full table
     (kupować, chować, dumać); robić, pisać, być, mieć all come back
     with only the infinitive row.
   - **UniMorph pol**: only **844 verb lemmas**, Wiktionary-derived,
     with known quality issues.
   - **PoliMorf, Morfologik, NKJP tagset resources**: all merged
     from or aligned with the SGJP family — not independent.

844 UniMorph lemmas against SGJP would be a token agreement gold for
a language whose conjugation (aspect pairs, four past-tense genders,
person clitics, ~100 inflection patterns) most needs the safety net.
Per the project rule — poor verification loop → skip.
