# Italian gold-data oracles

Same criterion as the other languages: two independently-derived
machine-readable sources.

## Why this pair

UniMorph ita is an English-Wiktionary scrape — not independent of
kaikki.

1. **kaikki.org Italian** (en.wiktionary `it-conj` via Wiktextract;
   CC BY-SA / GFDL). `scripts/ita/fetch_kaikki.sh` →
   `data/ita/kaikki.tsv`: 549,374 rows, 13,084 lemmas. en.wiktionary
   marks pedagogical stress on every form (pàrlo); the converter strips
   non-final accents, keeping Italian's real word-final ones (parlò).
   The table's auxiliary header becomes a V;AUX slot (avere/essere).
2. **Morph-It! 0.48** (Zanchetta & Baroni, UniBo; CC BY-SA 2.0 / LGPL
   dual license). Corpus-derived (la Repubblica) with manual curation —
   not a Wiktionary derivative. `scripts/ita/fetch_morphit.sh`
   (sha256-pinned) → `data/ita/morphit.tsv`: 380,135 rows,
   6,077 lemmas.

Tie-breaker: Treccani's coniugazione pages for manual spot checks.

## Feature schema

Standard TSV. Slots: `V;NFIN`, `V;GER`, `V.PTCP;{PRS,PST}` (Morph-It
carries all four gender/number cells, kaikki the masc sg), `V;AUX`
(kaikki only), IND PRS / PST;IPFV / PST;PFV (passato remoto) / FUT,
`V;COND`, SBJV PRS/PST, `V;IMP` (tu, Lei as 3sg, noi, voi, Loro as
3pl). Skipped: negative imperatives and clitic-suffixed forms
(parlami).

## Baseline (2026-08-17)

- 281,330 shared (lemma, feature) slots
- 99.58% agreement; 1,185 disjoint (0.42%)
- Disagreement classes for engine-time adjudication: Morph-It missing
  the -isc- infix on some -ire verbs (abbruto for abbrutisco), the
  passato remoto accent (Morph-It abbattè, standard abbatté), and the
  -fare compound family
