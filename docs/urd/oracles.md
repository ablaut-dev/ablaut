# Urdu gold-data oracles

Urdu is Hindustani in the Perso-Arabic (Nastaliq) script — the same
spoken language as Hindi (`src/hin.rs`). The engine `src/urd.rs` is a
script twin of the Hindi one, and the oracle loop follows the same
criterion: two machine-readable sources of independent provenance, so
their agreement is strong evidence and their disagreements form the
adjudication corpus.

## Why this pair

1. **kaikki.org Urdu** (Wiktextract of English Wiktionary, CC BY-SA).
   `scripts/urd/fetch_kaikki.sh` → `data/urd/kaikki.tsv`: **1,020 forms
   over 178 verb lemmas**, converted by `scripts/urd/kaikki_to_tsv.py`.

2. **apertium-urd** (Apertium monolingual Urdu package, GPL; commit
   `8d83e85d`). A hand-written lttoolbox dictionary with **no Wiktionary
   lineage** — the independent leg. `scripts/urd/fetch_apertium.sh` clones
   it and `scripts/urd/apertium_to_tsv.py` expands the `.dix` directly
   (the monodix is flat, so no `lttoolbox` install is needed) and maps its
   analyses onto the shared feature bundle: **22,191 forms over 680 verb
   lemmas**.

UniMorph `urd` is deliberately **not** the second leg: it is a 2017
single-commit English-Wiktionary extraction and so shares kaikki's
lineage — using it as the agreement partner would be circular. It serves
as a documented cross-check only (see below).

### What the pair can and cannot gate

The `ur-conj` template kaikki draws on emits **no person tags at all** —
the subjunctive, the synthetic future and the whole analytic
(participle + copula) layer are printed without any person distinction
(the bare subjunctive is even tagged `error-unrecognized-form`). Person
cannot be recovered from kaikki, so the kaikki adapter emits only the
slots it tags unambiguously — the **person-independent core**: the
infinitive and oblique infinitive, and the imperfective and perfective
participles (by gender/number). That core is the surface kaikki shares
with apertium and is what the two-oracle gate scores.

The person-resolved finite paradigm the engine also produces
(subjunctive, three imperatives, synthetic future, analytic layer) is
corroborated by the independent apertium oracle and by UniMorph
separately — see **Corroboration** below.

## Agreement

kaikki ∩ apertium agree on **777 of 781** shared (lemma, feature) slots —
**99.49%** — across **141** shared lemmas. The 4 disagreements are ruled
in [`disagreements.tsv`](disagreements.tsv):

- **2** vowel-stem masculine-singular perfectives where apertium keeps
  the ی-glide the grammar requires (رویا, چھویا) and kaikki drops it
  (روا) → ruled o2 (apertium), which the engine follows;
- **1** kaikki tabulation defect (a doubled نا infinitive, نبھانانا) →
  o2;
- **1** stray apertium participle mis-form (پیتیں for masc-pl پیتے) → o1.

Two systematic orthographic splits are folded in the adapters so that
genuinely identical forms compare equal (both variants are standard
Urdu): the **feminine-plural participle nasal** (گرجیں ~ گرجی — the
engine, like Hindi, keeps a single number-less feminine cell) and the
**noon-ghunna nasalization mark** `U+0658` (کھان٘سنا ~ کھانسنا), the
latter stripped by `perso_arabic::normalize` alongside the harakat.

## Feature schema

`lemma ⇥ form ⇥ features`, the UniMorph-style bundle Urdu shares with
Hindi. The gate scores the person-independent subset:

- `V;NFIN;LGSPEC1` (اترنا), `V;NFIN;LGSPEC2` (oblique اترنے);
- `V;V.PTCP;{MASC,FEM};{SG,PL};IPFV` (اترتا…), `…;PFV` (اترا…).

The engine additionally produces, and the corroboration below covers:
`V;{1,2,3};{SG,PL};SBJV` (subjunctive), `…;SBJV;{MASC,FEM}` (synthetic
future, written apart: اتروں گا), `V;2;{SG,PL};IMP;{INFM,FORM}` (three
imperatives), and the analytic layer `V;p;n;{HAB,PRF,PROG};{PRS,PST,SBJV,
LGSPEC3,LGSPEC4};g`.

## Corroboration of the finite paradigm

Scored against the independent **apertium** oracle alone (680 lemmas),
the engine matches:

- imperative **100%**, subjunctive **98.5%**, participle **99.5%**,
  infinitive **100%**.
- The synthetic **future** disagrees only on a spelling convention:
  apertium writes the particle joined (اترونگا) whereas kaikki, UniMorph
  and standard modern Urdu write it apart (اتروں گا), which the engine
  follows.

Scored against **UniMorph urd** (56 lemmas, the full person-resolved
paradigm), the simple verbs match up to the feminine-plural analytic
nasal (the engine emits the unmarked اتری تھی for اتری تھیں, the same
modelling choice as Hindi). UniMorph's raw full-paradigm number is
depressed by two of its own defects — the "independent-vowel" perfective
mis-form (کیی for کی, the same class of defect documented for Hindi
UniMorph) and broken gender/person agreement in its compound-verb
(X کرنا) paradigms — which is exactly why it is a cross-check, not the
gate.

## The engine

`src/urd.rs` mirrors `src/hin.rs`: a productive rule over the open class
(stem = infinitive minus final نا) plus a compiled-in table of the
suppletive/contracted verbs (`data/urd/verbs.tsv` — ہونا, جانا, کرنا,
دینا, لینا, پینا, جینا, سینا). Consonant stems take bare endings, vowel
stems (ا/آ/و) take the hamza-glide (کھایا, کھاؤں). Compounds conjugate
their last word and reprint the lead (حاصل کرنا → حاصل کیا). All forms are
produced in the normalized Perso-Arabic orthography of
`src/perso_arabic.rs` (harakat and the noon-ghunna mark stripped, Arabic
letters folded), the same module Persian reuses.

## Score

**100.00%** of the 777 agreed slots, 141 of 141 lemmas covered, all 10
agreed slot types covered and every one of the 4 oracle disagreements
resolved → **Verified** (on the person-independent core; the finite
paradigm is corroborated as above).

## Reproducing

```sh
./scripts/urd/fetch_kaikki.sh      # → data/urd/kaikki.tsv (needs curl)
./scripts/urd/fetch_apertium.sh    # → data/urd/apertium.tsv (needs git)
cargo run --release --bin golden_urd -- \
    data/urd/kaikki.tsv data/urd/apertium.tsv --check   # two-oracle gate
./scripts/urd/fetch_unimorph.sh    # → data/urd/unimorph.tsv (cross-check)
cargo run --release --bin golden_urd -- data/urd/apertium.tsv  # finite core
```
