# Swedish gold-data oracles

1. **kaikki.org Swedish** (en.wiktionary `sv-conj` via Wiktextract;
   CC BY-SA / GFDL): 67,536 rows. Participles are skipped — SALDO's
   verb entries do not carry them, so they cannot be cross-checked.
2. **SALDO morphology** (Språkbanken, CC BY — a genuinely independent
   national lexicon): 73,908 verb rows from the LMF XML.

UniMorph swe is an English-Wiktionary scrape and is not used.

## Schema

Swedish conjugates without person/number: slots are infinitive,
present, past, supine, imperative — each with an s-passive
(`…;ACT`/`…;PASS`) — plus the relic subjunctives (vore). Deponents
(hoppas) live entirely in the passive slots.

## Baseline and result (2026-08-17)

Scaffold baseline: 40,767 shared slots, 99.27% agreement. The engine
is a weak-1 default plus a mined principal-parts table
(`data/swe/parts.tsv`, ~1,900 rows incl. SALDO-only compounds, mined
exact — Swedish compound boundaries defeat suffix heuristics: fördärva
is not för+ärva). Final: 100.00% on agreement gold, 99.92% against
SALDO alone.
