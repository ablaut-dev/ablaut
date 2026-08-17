# Slovenian gold-data oracles

1. **kaikki.org Slovene** (en.wiktionary via Wiktextract; CC BY-SA):
   `scripts/slv/fetch_kaikki.sh` → `data/slv/kaikki.tsv` — 474 verbs
   with full tables, the thinnest agreement set in the project, but
   they are the frequent core. The tables use tonal orthography
   (dẹ́lati, rekəł); the converter strips every combining mark except
   the caron and normalizes ə → e, ł → l.
2. **Sloleks 3.0** (CJVT, CC BY-SA 4.0 — the Slovene national
   inflectional lexicon, 17,286 verb lemmas):
   `scripts/slv/fetch_sloleks.sh` → `data/slv/sloleks.tsv` — 432,719
   rows. Fetched via the Hugging Face SQLite mirror; CLARIN.SI's own
   bitstreams (2.0 and 3.0 alike) return server-side I/O errors, the
   same failure that blocked Romanian MULTEXT-East.

## Schema

V;NFIN, V;SUP (the supine: delat), V;IND;PRS;{SG,DU,PL};{1,2,3},
V;IMP;{SG;2, DU;1, DU;2, PL;1, PL;2}, V.PTCP;PST;{M,F,N};{SG,DU,PL}.
Slovenian is the first language in the project with a dual.

## Baseline and result (2026-08-17)

- 10,983 shared slots, 97.11% baseline (the disjoint corpus is
  kaikki extraction glitches: blagoslovimm-type doubled letters)
- Engine: four present classes (delam/govorim/kupujem/dvignem) over
  nine-slot rows, imperative duals/plurals off the 2sg, l-participle
  off one base with a mined feminine for fleeting -e- (rekel → rekla)
- Agreement gold: 10,757 forms, **100.00%**
- Sloleks alone (CI gate): 431,891 forms, 99.96% (gates 99.9/99.5)
