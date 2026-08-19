# Catalan gold-data oracles

The oracle pair for the Catalan verification loop, chosen by the same
criterion as Spanish (see `docs/spa/oracles.md`): two machine-readable
sources of independent provenance, so their agreement is strong evidence
and their disagreements form the adjudication corpus.

## Why this pair

UniMorph cat is out for the same reason as Spanish: it is an
English-Wiktionary scrape, so it shares kaikki's lineage and its
agreement would be circular.

1. **kaikki.org Catalan** (en.wiktionary `ca-conj` templates via
   Wiktextract; CC BY-SA / GFDL). The tables expand cleanly.
   `scripts/cat/fetch_kaikki.sh` → `data/cat/kaikki.tsv`: 201,746 rows,
   4,024 lemmas.
2. **FreeLing Catalan dictionary** (TALP-UPC, LGPL-LR data; descends
   from the Spanish Resource Grammar family — not a Wiktionary
   derivative). `scripts/cat/fetch_freeling.sh` (commit-pinned,
   sha256-checked) → `data/cat/freeling.tsv`: 498,024 rows, 8,722
   lemmas.

The two agree on **187,236 of 187,346** shared (lemma, feature) slots —
**99.94%**. The 110 disagreements are diaeresis/accent edge cases
(*subduïm* vs *subduim*, *satisfeu* vs *satisféu*) and are excluded from
gold, not scored.

## Feature schema

Same TSV as German, French and Spanish: `lemma ⇥ form ⇥ features`. Slots:
`V;NFIN`, `V;GER`, `V.PTCP;PST;{MASC,FEM};{SG,PL}`, the imperative
`V;IMP;{SG,PL};{1,2,3}`, and the seven synthetic tense/moods
(`V;IND;PRS`, `V;IND;PST;IPFV`, `V;IND;PST;PFV`, `V;IND;FUT`, `V;COND`,
`V;SBJV;PRS`, `V;SBJV;PST`). Catalan has no synthetic future subjunctive.

## The engine

Central (IEC) orthography. Three conjugations (`-ar`, `-er`/`-re`, `-ir`
with its inchoative `-eix-` subclass) with the sound-preserving spelling
rules (`c→qu`, `ç→c`, `g→gu`, `j→g`, `gu→gü`, `qu→qü`) and the syllabic-i
diaeresis (*crear → creï*, *agrair → agraïm*). The irregular verbs — the
velar-stem and strong-preterite `-re`/`-er` classes, the suppletives
(*ser*, *anar*, *fer*, *dir*, *veure*, *tenir*, *venir*, …) and the
athematic participles (*dit*, *fet*, *imprès*) — live in
`data/cat/verbs.tsv`, stored as base lemmas so that prefixed derivatives
(*comprendre* ← *prendre*, *recórrer* ← *córrer*) come free by suffix
match. Bases that are a suffix of an otherwise-regular verb (`dir` inside
*accedir*) are flagged exact-match-only.

The lexicon is mined from the oracle agreement:
`scripts/cat/capture_irregulars.sh` records the verbs the rule engine
misses, and `scripts/cat/mine_verbs.py` fills their paradigms from the
FreeLing ∩ kaikki merge.

## Score

100.00% of the 187,186 agreed slots, 99.97% lemma coverage.
