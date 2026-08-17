# Romanian gold-data oracles

Same criterion as the other languages: two independently-derived
machine-readable sources.

## Why this pair

UniMorph ron is an English-Wiktionary scrape — not independent of
kaikki. MULTEXT-East's Romanian lexicon would have been a third
lineage, but CLARIN.SI's bitstream for it is currently broken
server-side (Input/output error).

1. **kaikki.org Romanian** (en.wiktionary `ro-conj` via Wiktextract;
   CC BY-SA / GFDL). `scripts/ron/fetch_kaikki.sh` →
   `data/ron/kaikki.tsv`: 243,913 rows, 6,972 lemmas. The infinitive
   particle (a vorbi) and subjunctive să are stripped so slots align.
2. **dexonline** (GPL; the digitized DEX with its own inflection
   engine — an independent Romanian-Academy lineage). Used strictly as
   a test-time oracle; nothing GPL ships. The 378 MB SQL dump is
   streamed for the Lexeme/InflectedForm tables:
   `scripts/ron/fetch_dexonline.sh` → `data/ron/dexonline.tsv`:
   485,311 rows, 12,566 lemmas.

## Feature schema

Standard TSV. Romanian brings the synthetic pluperfect
(`V;IND;PST;PQP`: vorbisem) beside the perfect simple; the future is
analytic (voi vorbi) and out of scope. Slots: NFIN, GER,
PTCP;PST;MASC;SG, IND PRS / PST;IPFV / PST;PFV / PST;PQP, SBJV;PRS,
IMP 2sg/2pl.

## Baseline (2026-08-17)

- 227,471 shared (lemma, feature) slots
- 99.13% agreement; 1,977 disjoint (0.87%) — the widest scaffold gap
  of the covered languages, reflecting dexonline's variant-rich
  inflection models; the adjudication corpus for engine work

## Engine results (2026-08-17)

Theme-vowel classes with the productive augments as defaults (-ez,
-esc, -ăsc); bare presents and the -e/-ea classes are mined into
`data/ron/classes.tsv` (1,184 assignments), and verbs the class rules
cannot reach carry explicit rows in `data/ron/verbs.tsv` (3,587).
Rules carry the derived tenses: imperfect from the theme, perfect
simple and pluperfect from the participle with stress reduction
(trase → trăsei, coapse → copsei), t→ț/d→z/s→ș palatalization and
e→ea/o→oa diphthongization in the present.

- Agreement gold (kaikki ∩ dexonline): 225,494 forms, **100.00%**
- dexonline alone (CI gate): 424,475 forms, 99.69% (gate 99.5); the
  residual is dexonline's multi-model variant verbs (fărmăca-type
  double inflection models)
