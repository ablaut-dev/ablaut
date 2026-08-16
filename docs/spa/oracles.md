# Spanish gold-data oracles

The oracle pair for the Spanish verification loop, chosen by the same
criterion as French (see `docs/fra/oracles.md`): two machine-readable
sources with independent provenance, so their agreement is strong
evidence and their disagreements form the adjudication corpus.

## Why this pair

UniMorph spa is out: it is an English-Wiktionary scrape (so not
independent of kaikki) and its verb table has exactly 2^20 rows — the
Excel row limit, a strong smell of silent truncation in its pipeline.

1. **kaikki.org Spanish** (en.wiktionary `es-conj` templates via
   Wiktextract; CC BY-SA / GFDL). The tables expand cleanly.
   `scripts/spa/fetch_kaikki.sh` → `data/spa/kaikki.tsv`:
   591,377 simple-form rows, 8,805 lemmas.
2. **FreeLing Spanish dictionary** (TALP-UPC, LGPL-LR data; descends
   from the Spanish Resource Grammar — not a Wiktionary derivative).
   `scripts/spa/fetch_freeling.sh` (commit-pinned, sha256-checked)
   → `data/spa/freeling.tsv`: 497,556 rows, 7,658 lemmas.

Tie-breakers for adjudication: the RAE conjugator (dle.rae.es, manual
spot checks), and the LanguageTool Spanish tagger dictionary
(LGPL-2.1, a third non-Wiktionary lineage).

## Feature schema

Same TSV as German and French: `lemma ⇥ form ⇥ features`. Slots:
`V;NFIN`, `V;GER`, `V.PTCP;PST;{MASC,FEM};{SG,PL}`,
`V;IND;PRS`, `V;IND;PST;IPFV` (imperfecto), `V;IND;PST;PFV`
(pretérito), `V;IND;FUT`, `V;COND`, `V;SBJV;PRS`, `V;SBJV;PST`
(-ra and -se as variants of one slot), `V;SBJV;FUT` (archaic but in
both oracles), `V;IMP;{n};{p}` (tú, usted as 3sg, nosotros, vosotros,
ustedes as 3pl).

Excluded by policy, as compositional or variant layers: vos/voseo
forms, clitic combined forms (háblame), negative imperatives
(no + subjunctive), and all compound tenses (haber + participle).

## Baseline (2026-08-17)

- 339,923 shared (lemma, feature) slots
- 99.79% agreement; 723 disjoint slots (0.21%)
- Dominant disagreement classes, ruled during engine work:
  diphthongization disputes (FreeLing conjugates *aberrar* as
  *abierro*; kaikki and the RAE keep it regular — adjudicated),
  dual-conjugation verbs where kaikki carries both variants (*acostar*
  acuesta/acosta — classed with the diphthong canonical), and
  pronominal-only or defective lemmas.
