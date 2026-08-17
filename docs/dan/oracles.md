# Danish gold-data oracles

1. **kaikki.org Danish** (en.wiktionary via Wiktextract; CC BY-SA):
   `scripts/dan/fetch_kaikki.sh` → `data/dan/kaikki.tsv` — 22,473
   rows from the active/passive inflection boxes.
2. **COR — Det Centrale Ordregister** (Dansk Sprognævn, open data;
   the register behind Danish orthography): `scripts/dan/fetch_cor.sh`
   → `data/dan/cor-verbs.tsv` — 65,056 verb rows, ~7,200 lemmas.
   The older Retskrivningsordbog download is license-gated; COR is
   the open successor and an independent national lineage.

## Schema

Nine slots: V;NFIN;ACT/PASS, V;PRS;ACT/PASS, V;PST;ACT/PASS, V;IMP,
V.PTCP;PRS, V.PTCP;PST.

## Baseline and result (2026-08-17)

- 20,486 shared slots, 99.19% baseline (kaikki carries a handful of
  typo paradigms: *beslutes*, *opnåes*)
- Engine: Swedish-shaped — class-1 default (virke/virker/virkede/
  virket), 2,359 mined principal-parts rows matched exactly (the
  Swedish compound-boundary lesson transfers), s-passives by rule
  with vowel-final bare -s (så → sås) and a past-passive override
  column (hænge: hang / hængtes)
- Agreement gold: 20,205 forms, **100.00%**
- COR alone (CI gate): 61,629 forms, 99.98% (gates 99.8/99.5)
