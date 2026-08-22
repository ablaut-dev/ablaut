# Bengali gold-data oracles

Bengali (বাংলা, `ben`) is an Indo-Aryan language written in the
Bengali-Assamese abugida (a clean Brahmic script, no Perso-Arabic
ambiguities). Like its shipped sister Hindi (`src/hin.rs`) it builds the
finite system on the stem left when the citation form loses its ending,
with a bounded analytic/aspectual layer on top. Two things make Bengali
its own engine rather than a Hindi clone:

- **no grammatical gender** — participles and finite verbs do not agree
  in gender at all (Hindi's whole masculine/feminine axis is gone);
- **rich person × honorific agreement fused with tense** — the finite
  verb agrees in five classes: **আমি** (first), **তুই** (second
  intimate), **তুমি** (second familiar), **সে** (third ordinary) and
  **আপনি/তিনি** (honorific, second or third). Each of eight
  tense-aspects (simple present/past, future, past habitual,
  present/past progressive, present/past perfect) crosses those five,
  for a dense 5×8 finite grid.

Two stem alternations shape the paradigm: **vowel raising** (a mid root
vowel এ/ও raises to ই/উ outside the তুমি/সে/আপনি present — ওঠা: উঠি but
ওঠে) and **আ-fronting** in the perfective participle (নাচ→নেচে). See
`src/ben.rs` for the full description.

## The oracle situation, and why this ships single-oracle (Beta)

The plan was the usual pair: a Wiktionary extraction (kaikki.org) and a
second independent source (UniMorph ben). For Bengali the pair does not
hold up as an independent two-oracle *agreement*, and the reason is
documented here rather than papered over.

1. **UniMorph ben** (Batsuren & Cotterell; **source: Wikipedia**,
   CC BY-SA 3.0), commit-pinned and sha256-checked.
   `scripts/ben/fetch_unimorph.sh` → `scripts/ben/unimorph_to_tsv.py` →
   `data/ben/unimorph.tsv`: a fully-generated paradigm of **84 verb
   lemmas**, each cited by its `-আ` verbal noun, with the complete 5×8
   finite grid plus six non-finite forms (the `-তে` infinitive V.NFIN,
   the verbal noun V.MSDR, and the perfective / habitual / progressive /
   conditional participles). This is the primary oracle and the one the
   engine is scored against — **3,864 forms**.

2. **kaikki.org Bengali** (Wiktextract of English Wiktionary, CC BY-SA).
   `scripts/ben/fetch_kaikki.sh` → `scripts/ben/kaikki_to_tsv.py` →
   `data/ben/kaikki.tsv`: ~530 verbs carry a filled `bn-conj` table.

The problem is **independence**. UniMorph ben's own README gives its
source as *Wikipedia*, i.e. the same Wiktionary lineage kaikki extracts
from; the two are not two independent lexicons concurring. Two further
kaikki limits make it unsuitable as the *scored* oracle even setting
independence aside:

- kaikki tags **তুই and তুমি identically** (`familiar;second-person`),
  so it cannot resolve the second-person-familiar cells at all;
- kaikki mixes **registers and orthographic conventions** that UniMorph
  (and the engine) do not: the সাধু-ভাষা literary forms (করিব, করিয়া,
  দেখিতে) alongside the modern চলিত forms, and the বই-ও-less spelling of
  the future/past/habitual endings (করব / করল / করত beside
  করবো / করলো / করতো). Both spellings are standard; the two oracles
  simply pick different ones.

So Bengali ships **single-oracle (Beta)**, scored against UniMorph ben,
with kaikki kept as an independent spot check — the same honest posture
Telugu and Gujarati take (see `docs/tel/oracles.md`,
`docs/guj/oracles.md`). A genuinely independent second oracle (a
morphological analyser such as apertium-ben, which would need the
lttoolbox FST toolchain) would restore two-oracle footing as a strict
addition against the same schema.

## Feature schema

`lemma ⇥ form ⇥ features`, UniMorph bundles. The person × honorific
classes are, in UniMorph's tagging (which is cross-wired relative to the
surface forms — see below):

| class            | pronoun     | UniMorph tag  | present (করা) |
| ---------------- | ----------- | ------------- | ------------- |
| first            | আমি         | `1`           | করি           |
| second intimate  | তুই         | `2;LGSPEC1`   | করিস          |
| second familiar  | তুমি        | `3;INFM`      | করো           |
| third ordinary   | সে          | `2;POL`       | করে           |
| honorific        | আপনি / তিনি | `3;POL`       | করেন          |

UniMorph labels the **সে** (third-ordinary) form `2;POL` and the
**তুমি** (second-familiar) form `3;INFM` — the person/honorific tags are
swapped relative to the pronoun each form actually goes with. The
`golden_ben` adapter maps each `(person, politeness)` tag pair to the
engine's agreement class by the form it denotes, so the scoring is
correct; `src/ben.rs` and the bindings name the classes by pronoun.

The eight tense-aspects: `PRS`, `PST`, `FUT`, `PST;HAB` (past habitual),
`PRS;PROG` / `PST;PROG` (progressive), `PRS;PRF` / `PST;PRF` (perfect).
Non-finite: `V.NFIN` (করতে), `V.MSDR` (করা), `V.PTCP;PRF` (করে),
`V.PTCP;HAB` (করে), `V.PTCP;PROG` (করতে), `V.PTCP;COND` (করলে).

The habitual and progressive participles **reduplicate** in the gold's
compound lemmas (মনে রাখা → মনে রেখেরেখে) but not in the simple verbs
(রাখা → রেখে); the adapter accepts either spelling.

## The engine

`src/ben.rs` drops the `-আ` (or `-ওয়া` / `-আনো`) to get the stem and
builds the whole system by rule: the person × honorific endings on each
of the eight tense-aspects, the আ-fronted perfective participle, the
ল-gemination of the simple past (বলা → বল্লাম), and the non-finite
forms. The fully productive `-আনো` causatives (ঘুমানো, চালানো) are
handled by rule. Two closed sets that the rule cannot predict live in
`data/ben/verbs.tsv` (20 rows): the **raising** monosyllabic roots
(কেনা→কিন, ওঠা→উঠ …), whose mid vowel raises lexically, and the eight
**vowel-final** roots (খাওয়া, দেওয়া, যাওয়া …), irregular across several
stems (যাওয়া is suppletive in the past গেলাম / perfect গিয়ে). Compound
lemmas (`মনে রাখা`, `অনুবাদ করা`) conjugate only their last word.

## Score

**100.00%** of the **3,864** scored UniMorph forms, **84 of 84** lemmas
covered, every paradigm slot type covered, **zero adjudications**:

| category    | matched   |
| ----------- | --------- |
| present     | 420/420   |
| past        | 420/420   |
| future      | 420/420   |
| habitual    | 420/420   |
| progressive | 840/840   |
| perfect     | 840/840   |
| nonfinite   | 504/504   |

As an independent check, the engine is run against the kaikki spot-check
gold (`cargo run --bin golden_ben -- data/ben/kaikki.tsv`): it matches
**5,883 of 9,209** forms over 307 verbs. The bulk of the residual is
**not** engine error but the register/orthography mismatch documented
above — kaikki lists the সাধু literary forms (করিব, করিয়া) and the
ও-less future/past/habitual spellings (করব, করল, করত) that UniMorph and
the engine do not, and hyphenated reduplication (করে-করে). Where kaikki
gives the modern চলিত form the engine agrees; no residual is a genuine
contradiction of a form UniMorph also lists.

## Reproducing

```sh
./scripts/ben/fetch_unimorph.sh          # → data/ben/unimorph.tsv
./scripts/ben/fetch_kaikki.sh            # → data/ben/kaikki.tsv
cargo run --release --bin golden_ben -- data/ben/unimorph.tsv   # scored
cargo run --release --bin golden_ben -- data/ben/kaikki.tsv     # spot check
```
