# Swahili gold-data oracles

The oracle pair for the Swahili verification loop: two machine-readable
sources of independent provenance, so their agreement is strong evidence
and their disagreements form the adjudication corpus.

1. **UniMorph swc** (English Wiktionary, CC BY-SA 3.0).
   `scripts/swa/fetch_unimorph.sh` (commit-pinned, sha256-checked) →
   `data/swa/swc.tsv`. The raw file is a generated paradigm: 10,200 rows
   over 47 verb stems (the rest are adjectives and nouns, dropped). It
   encodes the subject noun class with LGSPEC tags — `LGSPEC1..6` are the
   six class pairs 1/2, 3/4, 5/6, 7/8, 9/10, 11/10 — and it omits the
   negative paradigm entirely.
2. **kaikki.org** (Wiktextract, CC BY-SA). `scripts/swa/fetch_kaikki.sh`
   → `data/swa/kaikki.tsv`, from the 2,986 verb tables in the Swahili
   verb extraction. kaikki carries the full concord matrix (classes
   1–18, including the locatives), the object-concord and relative
   layers, and the negatives. It spells fully-inflected words only for
   the person subjects and classes 1/2 in most tenses, and for *every*
   class in the a-tense (gnomic); the other class × tense cells are
   rendered as template strings (`positive subject concord + -lisoma`)
   and dropped by the adapter.

## Lemma normalization (the swc adapter)

swc keys every row on the **bare stem** (`soma`, `jibu`, `enda`), i.e.
the infinitive minus the `ku-` marker. kaikki keys its *conjugating*
entries on the same bare stem, so once swc's LGSPEC class tags are
decoded to the shared `CL<n>` / person subject tokens
(`scripts/swa/swc_to_tsv.py`), the two lemma columns line up directly —
no `ku-` has to be stripped or added. `scripts/swa/kaikki_to_tsv.py`
emits the identical canonical schema `V;TAM[;SUBJ][;NEG]`, so the shared
harness can intersect them.

After normalization the two oracles share **45 lemmas** and **1,134**
whole-word slots.

## Feature schema

`lemma ⇥ form ⇥ features`, a compact UniMorph-style bundle
`V;TAM[;SUBJ][;NEG]`. The subject token is a person (`1SG`, `2SG`,
`1PL`, `2PL`) or a noun class (`CL1`…`CL18`). The TAMs are the present
`PRS` (-na-), past `PST` (-li-), future `FUT` (-ta-), perfect `PRF`
(-me-), the a-tense/gnomic `GNOM` (fused -a-), the subjunctive `SBJV`
(final -e), the habitual `HAB` (hu-), the consecutive `SEQ` (-ka-), the
situative `SIT` (-ki-) and the two conditionals `CONDP`/`CONDPST`
(-nge-/-ngali-), plus the non-finite `NFIN` and the imperative `IMP;SG`/
`IMP;PL`.

### What is scored, and the negatives caveat

Because kaikki only spells real words for the person and class-1/2 cells
outside the gnomic, the scored two-oracle agreement is the productive
core both sources render as whole words: the infinitive, habitual,
present and subjunctive over the persons and classes 1/2, and the
**a-tense over every noun class** (1–11, the classes swc covers). The
past/future/perfect/conditional cells survive only where kaikki spells a
word, and mostly fall to swc's template placeholders — they are in the
engine but not double-covered.

**Negatives are kaikki-only.** swc omits the negative paradigm, so the
whole negative half of the verb (`sisomi`, `hatusomi`, `kutosoma`) is a
single-oracle layer: the engine produces it, but it never enters the
agreement and is never scored against a second source.

## The engine

`src/swa.rs` implements the productive slot template — subject concord,
TAM marker, root, final vowel — as the default rule, with the noun-class
agreement matrix (plain, a-tense and negative concords for classes
1–18) as the one lexical table. Two stem facts are read off the citation
stem: a **monosyllabic** stem (one vowel: `-fa`, `-la`, `-nywa`) makes
the infinitival `ku-` reappear under the primary TAM markers
(`a-na-ku-fa`), and a **non-`a` final vowel** marks an Arabic loan whose
subjunctive and negative keep the vowel (`jibu` → `nijibu`, not
\*`jibe`). `data/swa/verbs.tsv` holds only the genuine irregulars: the
suppletive `-enda` (ku-retaining `kwenda`, subjunctive `-ende`,
imperative `nenda`) and the suppletive imperative of `-ja` (`njoo`).

## Score

**100.00%** of the 1,134 agreed slots, 100.00% lemma coverage (45/45),
every paradigm slot type covered, and all **36** oracle disagreements
resolved in [`disagreements.tsv`](disagreements.tsv). Every one of the
36 is the swc generator erring where kaikki is right: it drops the
monosyllabic (and `-enda`) `ku-` retention (`tunakufa`, `tunakwenda`),
applies it wrongly to disyllabic vowel stems (`kuimba`, `kukwepa`,
`anaoa`), drops the 2pl `m-` → `mw-` glide before a vowel stem (`mwone`,
`muumbe`), and occasionally drops the subject concord outright
(`kwepe` for `tukwepe`). The engine follows kaikki throughout
(resolution `o2`).
