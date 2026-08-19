# Dutch gold-data oracles

The oracle pair for the Dutch verification loop, chosen by the same
criterion as the other languages: two machine-readable sources of
independent provenance, so their agreement is strong evidence and their
disagreements form the adjudication corpus.

## Why this pair

UniMorph nld is out for the usual reason: it is an English-Wiktionary
scrape, so it shares kaikki's lineage and its agreement would be
circular.

1. **kaikki.org Dutch** (en.wiktionary `nl-conj-*` templates via
   Wiktextract; CC BY-SA / GFDL). `scripts/nld/fetch_kaikki.sh` →
   `data/nld/kaikki.tsv`: 83,499 rows, 6,679 lemmas. Register and
   dialect variants (Flanders, formal, archaic), the present
   subjunctive and the nominal gerund are dropped so only the standard
   paradigm remains.
2. **Apertium-nld** monodix (GPL-2.0; read at test time, never
   redistributed) — a rule-based MT lexicon with no Wiktionary lineage.
   Its `vblex` paradigms are expanded to the shared TSV by a pure-Python
   `.dix` cartesian expander (no `lttoolbox` binary needed),
   `scripts/nld/apertium_to_tsv.py`. `scripts/nld/fetch_apertium.sh`
   (commit-pinned, sha256-checked) → `data/nld/apertium.tsv`: 28,012
   rows, 2,338 lemmas.

The two agree on **27,836 of 27,902** shared (lemma, feature) slots —
**99.76%**, across 2,329 shared lemmas. The 66 disagreements are
Apertium data slips (the `-eren` verbs it fails to lengthen, *masker*
for *maskeer*; typos such as *bomberderen*), strong-verb doublets
(*bedierven*/*bedorven*) and `ge-` prefix disputes; they are excluded
from gold, not scored.

## Feature schema

`lemma ⇥ form ⇥ features`. Dutch has few synthetic slots: `V;NFIN`; the
person-marked present indicative `V;IND;PRS;{SG,PL};{1,2,3}` (the plural
is person-invariant, emitted for all three persons so the two oracles'
tags line up); the number-marked past indicative `V;IND;PST;{SG,PL}`
(Dutch marks number but not person in the past); the singular
imperative `V;IMP`; and the two participles `V.PTCP;PRS`, `V.PTCP;PST`.
The compound perfects (*hebben*/*zijn* + participle) are the analytic
layer's business and out of schema.

## The engine

A regular (weak) verb is built from one stem derived from the `-en`
infinitive by the orthographic rules that keep vowel length and voicing
visible: consonant-doubling collapse (`pakken → pak`), vowel doubling in
the closed stem (`maken → maak`, `studeren → studeer`) with the native
schwa suffixes `-elen/-enen/-emen` held short (`wandelen → wandel`), and
the `v → f` / `z → s` devoicing spelling (`leven → leef`). The past and
participle follow *'t kofschip* (`werkte`/`gewerkt` vs `woonde`/
`gewoond`), and the participle drops `ge-` after an unstressed
inseparable prefix (`bedanken → bedankt`).

The strong verbs (ablaut past and `-en` participle), the schwa `-eren`
verbs, the doubling exceptions and the non-`-en` suppletives live in
`data/nld/verbs.tsv`, matched exactly. It is mined from the oracle
agreement: `scripts/nld/capture_irregulars.sh` records the verbs the
rule engine misses, and `scripts/nld/mine_verbs.py` fills their
paradigms. The everyday auxiliaries and modals (*hebben*, *zijn*,
*kunnen*, *zullen*, *mogen*, *willen*, *moeten*) that Apertium classes
outside `vblex` — so they never enter the agreement — are supplied by
hand from kaikki in `scripts/nld/manual.tsv`.

## Score

100.00% of the 27,836 agreed slots, 100.00% lemma coverage.
