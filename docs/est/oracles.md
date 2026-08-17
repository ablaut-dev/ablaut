# Estonian gold-data oracles

1. **kaikki.org Estonian** (en.wiktionary via Wiktextract; CC BY-SA):
   `scripts/est/fetch_kaikki.sh` → `data/est/kaikki.tsv` — 595 verbs
   with full tables (negatives and periphrastics excluded).
2. **Vabamorf** (Filosoft, LGPL — the Estonian national
   morphological analyzer, driven as a *generator* through estnltk):
   `scripts/est/fetch_vabamorf.sh` → `data/est/vabamorf.tsv` —
   synthesize() over the kaikki lemma list × 41 slot codes. An
   independent lineage without needing a downloadable lexicon: the
   first generator-as-oracle in the project.

## Schema

V;NFIN;MA (+INE/ELA/TRANSL/ABE/PASS case forms), V;NFIN;DA,
V;IND;{PRS,PST};{n};{p} + IMPRS, V;COND;{PRS,PRF};{n};{p} + IMPRS,
V;IMP;{SG;2, SG;3, PL;1, PL;2, PL;3} + IMPRS, V;QUOT,
V.PTCP;{PRS,PST};{ACT,PASS}.

## Baseline and result (2026-08-17)

- 24,261 shared slots, 99.68% baseline
- Engine: rules off two stems — the ma-stem (past rääkisin,
  imperatives rääkigu/rääkigem, nud, quotative) and the weak present
  stem (räägin, conditional räägiksin, tud and the t-impersonals
  räägiti/räägitagu). Consonant gradation is underivable, so 238
  mined rows carry it, plus the andev-class present participle and
  the andis-type past 3sg.
- Agreement gold: 24,184 forms, **100.00%**
- Vabamorf alone (CI gate): 27,675 forms, 99.79% (gates 99.5/99.5)
