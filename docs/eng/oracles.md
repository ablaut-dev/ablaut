# English gold-data oracles

1. **kaikki.org English** (en.wiktionary via Wiktextract; CC BY-SA):
   `scripts/eng/fetch_kaikki.sh` → `data/eng/kaikki.tsv` — 216,666
   rows. Dialectal/obsolete/nonstandard rows dropped; UK/US doublets
   kept as variants.
2. **AGID** (SCOWL/wordlist project; hand-checked inflection database,
   independent of Wiktionary): `scripts/eng/fetch_agid.sh` →
   `data/eng/agid.tsv` — 82,803 rows, 16,280 verbs. The historical
   tarball mirrors 404; the source repo serves `infl.txt` directly.

## Schema

Five slots: V;NFIN, V;PST, V.PTCP;PST, V.PTCP;PRS, V;PRS;3;SG.

## Baseline and result (2026-08-17)

- 74,827 shared slots, 99.81% baseline agreement (142 disjoint)
- Engine: regular -ed/-ing/-s rules + 1,651 mined rows (irregulars
  and stress-dependent consonant doubling: stop → stopping but
  visit → visiting — orthography alone cannot decide, so doubling
  verbs are data, not rules)
- Agreement gold: 74,685 forms, **100.00%**
- AGID alone (CI gate): 80,295 forms, 99.94% (gates 99.5/99.5)
