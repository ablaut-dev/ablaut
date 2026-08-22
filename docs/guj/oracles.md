# Gujarati gold-data oracles

Gujarati (ગુજરાતી, `guj`) is an Indo-Aryan language written in its own
abugida (a clean Brahmic script, no Perso-Arabic ambiguities). Its verbs
inflect for person and number (present, future, imperative) and for
gender and number on the participles — the perfective participle is the
past tense — across Gujarati's three genders, with an analytic layer
(participle + copula) on top. The engine follows the same design as the
sister Indo-Aryan language Hindi (`src/hin.rs`): stem = infinitive minus
the infinitive suffix, gender × number agreement on participles, and a
bounded analytic layer.

## The oracle situation, and why this ships single-oracle (Beta)

The plan was the usual pair: a Wiktionary extraction (kaikki.org) and a
second independent source (UniMorph guj). For Gujarati the pair does not
hold up as an independent two-oracle *agreement*, and the reason is
documented here rather than papered over.

1. **UniMorph guj** (Batsuren & Cotterell; English-Wiktionary lineage,
   CC BY-SA 3.0), commit-pinned and sha256-checked.
   `scripts/guj/fetch_unimorph.sh` → `scripts/guj/unimorph_to_tsv.py` →
   `data/guj/unimorph.tsv`: a fully-generated paradigm of **90 verb
   lemmas**, each cited by its `-વું` infinitive, with person/number
   agreement on the present, future, present-progressive and imperative,
   and single forms for the past, past-progressive, the two converbs,
   the verbal noun and the conditional/counterfactual. This is the
   primary oracle and the one the engine is scored against.

2. **kaikki.org Gujarati** (Wiktextract of English Wiktionary, CC BY-SA).
   `scripts/guj/fetch_kaikki.sh` → `scripts/guj/kaikki_to_tsv.py` →
   `data/guj/kaikki.tsv`: ~100 verbs carry a filled `gu-conj` table.

The problem is **independence**, not thinness. kaikki guj and UniMorph
guj are the *same* English-Wiktionary lineage — both descend from the
`gu-conj` inflection template, extracted by two pipelines. Their raw
form sets, compared per shared lemma, overlap ~85% (the residue is the
multiword negatives and one 1sg spelling variant), so their "agreement"
would measure Wiktionary against itself, not two independent lexicons
concurring. Worse, Wiktextract could not map the Gujarati column
headers, so most kaikki cells are tagged `error-unrecognized-form` and
lose their person/number.

So Gujarati ships **single-oracle (Beta)**, scored against UniMorph guj,
with kaikki kept as an independent spot check — the same honest posture
Telugu takes (see `docs/tel/oracles.md`). A genuinely independent second
oracle (a morphological analyser such as apertium-guj, which exists but
covers only ~57 verbs and needs the lttoolbox FST toolchain) would
restore two-oracle footing as a strict addition against the same schema.

## Feature schema

`lemma ⇥ form ⇥ features`, UniMorph bundles. The bounded cell set the
engine models and is scored on:

- **present / subjunctive** `V;IND;PRS;POS;{1,2,3};{SG,PL}` and the
  homophonous `V;COND;POS;…` (કરું, કરે, કરીએ, કરો) — the third person
  does not split singular from plural (`3;SG+PL`);
- **future** `V;IND;FUT;POS;{1,2,3};{SG,PL}` (કરીશ, કરશે, કરીશું, કરશો);
- **present progressive** `V;IND;PRS;PROG;POS;…` (કરું છું);
- **past** `V;IND;PST;POS` — the perfective participle (neuter, કર્યું);
- **past progressive** `V;IND;PST;PROG;POS` (કરતું હતું);
- **imperative** `V;IMP;PRS;POS;{2;SG,2;PL,1;PL}` and the deferred polite
  `…;POL;2;{SG,PL}` (કર, કરો, કરીએ, કરજે, કરજો);
- **converbs** `V;LGSPEC1` (કરી) and `V;LGSPEC2` (કરીને), the **verbal
  noun** `V;V.MSDR` (કરવાનું), and the **conditional/counterfactual**
  `V;LGSPEC3` (કરત) / `V;LGSPEC4` (કરતું હોત).

Deliberately **excluded** from the schema (so the scored set is a clean,
defensible cut): the **negatives** (`…;NEG`), which Gujarati forms
analytically with the particle નહીં / ન before the positive form; and
the **passive** (`V;PASS`), **potential** (`V;POT`), **optative**
(`V;OPT`), **present subjunctive** (`V;SBJV;PRS;POS`) and **future
progressive** (`V;IND;FUT;PROG;POS`) that UniMorph additionally lists.

The engine also generates the full **gender × number** paradigm of both
participles (કર્યો/કર્યા/કરી/કર્યું/કર્યાં), exercised in the unit tests;
UniMorph carries only the neuter cell of the past, so gender is not
scored here — it is covered by the schema but not the oracle.

### Orthographic normalization

Two folds are applied identically in both adapters: candrabindu →
anusvara (ઁ → ં), and the trailing exclamation mark UniMorph writes on
plain imperatives (ઇચ્છો! → ઇચ્છો) is stripped as punctuation.

## The engine

`src/guj.rs` drops the `-વું` to get the stem and builds the whole
system by rule: the present/subjunctive endings, the future in `-શ-`,
the imperative by politeness, the gender/number-agreeing perfective and
imperfective participles, the two converbs and the verbal noun, and the
analytic layer (present + છ- copula for the progressive; imperfective
participle + હતું/હોત for the past progressive and counterfactual). A
vowel-final stem takes independent vowel signs where a consonant stem
takes bare matras (નહાઉં vs કરું, નહાયું vs કર્યું), the same
Indo-Aryan glide/matra split Hindi shows. Compound lemmas
(`વચ્ચે આવવું`) conjugate only their last word.

What the rules cannot predict lives in `data/guj/verbs.tsv` (5 rows):
the suppletive જવું "go" (past ગયું, present stem જા-), the `-ધ-` pasts
of લેવું/દેવું (લીધું/દીધું) with their લે-/દે- presents, ખાવું "eat"
(past ખાધું, otherwise regular), and થવું "become" (present stem થા-,
past થયું).

## Score

**100.00%** of the **2,880** scored UniMorph forms, **90 of 90** lemmas
covered, every paradigm slot type covered:

| category    | matched   |
| ----------- | --------- |
| present     | 900/900   |
| future      | 450/450   |
| progressive | 450/450   |
| past        | 180/180   |
| imperative  | 450/450   |
| nonfinite   | 270/270   |
| conditional | 180/180   |

**15 of those forms are adjudicated** in [`adjudications.tsv`](adjudications.tsv):
every one is a cell of the suppletive verb જવું "go", which UniMorph
mechanically regularizes off the bare stem જ (જયું, જઉં, જય, જ) where
the correct Gujarati is suppletive or takes the present stem જા- (ગયું,
જાઉં, જાય, જા). The engine gives the correct forms, so each is ruled
`ours`. Net of those, the engine matches **2,865 forms UniMorph gets
right, and zero it gets wrong**.

As an independent check, the engine is run against the kaikki spot-check
gold (`cargo run --bin golden_guj -- data/guj/kaikki.tsv`): it matches
**1,194 of 1,197** forms over 100 verbs. The three residual mismatches
are: ખાવું's past, where the engine's correct ખાધું beats kaikki's
regularized ખાયું; and the 1sg present-progressive of લેવું/દેવું, where
the engine writes લઉં/દઉં and kaikki the equally-standard variant
લેઉં/દેઉં. None is a contradiction of a form UniMorph also lists.

## Reproducing

```sh
./scripts/guj/fetch_unimorph.sh          # → data/guj/unimorph.tsv
./scripts/guj/fetch_kaikki.sh            # → data/guj/kaikki.tsv
cargo run --release --bin golden_guj -- data/guj/unimorph.tsv   # scored
cargo run --release --bin golden_guj -- data/guj/kaikki.tsv     # spot check
```
