# Portuguese gold-data oracles

The oracle pair for the Portuguese verification loop, on the same
criterion as French and Spanish: independent provenance, so agreement
is strong evidence and disagreement is the adjudication corpus.

## Why this pair

UniMorph por is an English-Wiktionary scrape — not independent of
kaikki.

1. **kaikki.org Portuguese** (en.wiktionary `pt-conj` via Wiktextract;
   CC BY-SA / GFDL). `scripts/por/fetch_kaikki.sh` →
   `data/por/kaikki.tsv`: 439,241 rows, 6,077 lemmas. The European and
   Brazilian preterite doublets (falámos/falamos) are variants of one
   slot — the AO90 analogue of the French 1990 doublets.
2. **MorphoBr** (LR-POR; **Apache-2.0**, the only fully permissive
   lexicon in the project). Descends from DELAF-PB/LABEL-LEX with
   systematic error correction; pt-BR-leaning.
   `scripts/por/fetch_morphobr.sh` (commit-pinned) →
   `data/por/morphobr.tsv`: 1,973,538 rows, 28,088 lemmas.

Tie-breakers: the LanguageTool Portuguese tagger dictionaries
(LGPL-2.1, Natura/Minho lineage — the one resource cleanly separating
pt-PT/pt-BR orthographies), and Priberam for manual spot checks.

## Feature schema

Same TSV as the other languages. Portuguese adds two slots no other
language has: the personal infinitive `V;NFIN;{n};{p}` (falares,
falarmos) beside the impersonal `V;NFIN`, and the synthetic pluperfect
`V;IND;PST;PQP` (falara). Otherwise: `V;GER`,
`V.PTCP;PST;{g};{n}`, IND PRS / PST;IPFV / PST;PFV / FUT, `V;COND`,
SBJV PRS/PST/FUT, `V;IMP;{n};{p}`.

Excluded by policy: negative imperatives (não + subjunctive,
multiword), clitic combined forms and mesoclisis (falá-lo-ei — the
compositional layer's business).

## Baseline (2026-08-17)

- 378,677 shared (lemma, feature) slots
- 99.95% agreement; 206 disjoint (0.05%) — the cleanest scaffold of
  the four languages so far
- Disagreement classes: hiatus-accent verbs (MorphoBr abaulo vs kaikki
  abaúlo), and a handful of single-lemma disputes (assoar,
  arrepender) for adjudication during engine work
