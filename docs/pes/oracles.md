# Persian (Farsi) gold-data oracles

The oracle pair for the Persian verification loop, chosen by the same
criterion as every other language: two machine-readable sources of
independent provenance, so their agreement is strong evidence and their
disagreements form the adjudication corpus.

## Why this pair

The two legs come from genuinely different lineages — one a Wiktionary
extraction, one a hand-built morphological FST — so their agreement is
not circular. UniMorph `fas` is deliberately **not** used: it is itself
an English-Wiktionary extraction and shares kaikki's lineage, so it
would not be an independent second oracle (it may serve as a later
cross-check, not as an agreement partner).

1. **kaikki.org Persian** (Wiktextract of English Wiktionary,
   CC BY-SA). `scripts/pes/fetch_kaikki.sh` → `data/pes/kaikki.tsv`:
   **101,599 forms over 936 verb lemmas**, converted by
   `scripts/pes/kaikki_to_tsv.py`. Wiktionary fills several registers per
   verb (literary Iranian, colloquial Tehrani, Dari); every variant is
   emitted and the harness unions them, so the agreement with apertium
   selects the literary form.

2. **apertium-pes** (Apertium monolingual Persian package, GPL; commit
   `16757db6`). A hand-written lttoolbox dictionary with **no Wiktionary
   lineage** — the independent leg. `scripts/pes/fetch_apertium.sh`
   clones it and runs `lt-expand` to walk the dictionary into every
   surface/analysis pair, then `scripts/pes/apertium_to_tsv.py` maps the
   analyses onto the shared feature bundle: **6,502 forms over 190 verb
   lemmas**. Only `lt-expand` (from `lttoolbox`) is needed, not the full
   Apertium pipeline.

The two agree on **2,683 of 2,801** shared (lemma, feature) slots —
**95.8%** — across **101** shared lemmas. All 118 disagreements are ruled
in [`disagreements.tsv`](disagreements.tsv); every one is variant-level
(the same lexeme written two attested ways), in three groups:

- **47** are the present-perfect 3rd singular, where kaikki writes the
  copula apart (کرده است) and apertium joins it (کرده‌است → کردهاست) —
  the same form. The engine emits both spellings, so it matches either.
- **~54** are the subjunctive/imperative `بـ` before a vowel-initial
  stem, written `بآ…`/`بی…` interchangeably (بآشامم ~ بیاشامم).
- **~17** are three literary verbs (افشردن, انباشتن, نهفتن) with two
  competing attested present stems (افشار/افشر, انبار/انباز, نهفت/نهنب);
  the engine follows kaikki's.

## Feature schema

`lemma ⇥ form ⇥ features`, a small UniMorph-style bundle shared by both
adapters so the harness can intersect them. The person/number suffix is
`{1,2,3};{SG,PL}`.

- the **past** `V;IND;PST` (کردم) — kaikki files the imperfect
  (می‌کردم) under the same tag, so both spellings are accepted;
- the **present** `V;IND;PRS` (می‌کنم);
- the **subjunctive** `V;SBJV` (بکنم) and the **aorist** `V;AOR` (کنم,
  kaikki only);
- the **present perfect** `V;IND;PRF` (کرده‌ام), **pluperfect**
  `V;IND;PLUP`, **future** `V;FUT`, **perfect subjunctive**
  `V;SBJV;PRF` and the two **progressives** `V;IND;{PRS,PST};PROG`
  (kaikki only — apertium-pes does not spell these out);
- the **imperative** `V;IMP;2;{SG,PL}` (بکن, بکنید);
- the non-finite `V;NFIN` (کردن), `V;PTCP;PST` (کرده), `V;PTCP;PRS`
  (کننده).

apertium-pes covers the past, present, subjunctive, perfect, imperative
and non-finite slots; the aorist, pluperfect, future, perfect
subjunctive and progressives are exercised against kaikki alone (run
`golden_pes` with a single gold file).

## The engine

`src/pes.rs` implements the two-stem system. The **past stem** is
regular (infinitive minus its final ن) and builds the preterite, the
perfect/pluperfect (through the past participle کرده), the future and
the past participle. The **present stem** is irregular and stored in
`data/pes/verbs.tsv` (**163 rows** — that file *is* the present-stem
list); the productive `ـیدن` class (فهمیدن → فهم) needs no entry. Regular
personal endings, the `می`/`بـ`/`نـ` prefixes and the periphrastic
perfect/future/progressive complete the paradigm. Compounds conjugate
their last word and reprint the lead (بحث می‌کنم); the light-verb `بـ`
drops on کردن/شدن (بحث کنم) and is kept elsewhere.

All forms are produced and compared in the normalized Perso-Arabic
orthography of `src/perso_arabic.rs` (ZWNJ stripped, Arabic letters
folded to their Persian shapes, harakat removed) — the same module the
upcoming Urdu engine will reuse.

## Score

**100.00%** of the 2,683 agreed slots, 101 of 101 lemmas covered, all 28
paradigm slot types covered and every one of the 118 oracle
disagreements resolved → **Verified**.

## Reproducing

```sh
./scripts/pes/fetch_kaikki.sh      # → data/pes/kaikki.tsv (needs curl)
./scripts/pes/fetch_apertium.sh    # → data/pes/apertium.tsv (needs lttoolbox)
cargo run --release --bin golden_pes -- \
    data/pes/kaikki.tsv data/pes/apertium.tsv --check   # two-oracle gate
cargo run --release --bin golden_pes -- data/pes/kaikki.tsv  # kaikki alone
```
