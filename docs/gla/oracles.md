# Scottish Gaelic gold-data oracle

**kaikki.org Scottish Gaelic** (en.wiktionary via Wiktextract; CC BY-SA):
`scripts/gla/fetch_kaikki.sh` downloads the verb dump and
`scripts/gla/kaikki_to_tsv.py` derives two files:

- `data/gla/kaikki.tsv` — the golden gold (lemma ⇥ form ⇥ features).
- `data/gla/verbs.tsv` — mined principal parts (lemma, past, future,
  verbal noun, verbal adjective) that the engine embeds.

Single oracle ⇒ **Beta** tier. 1,402 verbs; 794 contribute at least one
scored form (164 have full `{{gd-conj}}` tables, the rest carry
principal parts only).

## Normalization

kaikki prints forms with their initial mutations and particles baked in
(`ghlan`, `chuir`, `dh'òl`, `dh'fhosgail`). The past, conditional and
relative future are lenited in the independent column; the converter
de-lenites them to the unmutated citation form (the shared oracle
convention), and the engine's `lenite` re-applies the display mutation.
Future, imperative and non-finite forms are already citation forms and
are left untouched. De-lenition is lemma-aware so roots that genuinely
begin consonant+`h` (`bhòt`, `thig`) are not mis-stripped.

Rows tagged `error-unrecognized-form` (the relative future) are kept;
`table-tags`/`inflection-template` noise, multi-word analytic forms
(`chuireadh sinn`), the emphatic/negative/interrogative columns and the
periphrastic present are dropped.

## Schema

`V;VN` (verbal noun), `V.PTCP` (verbal adjective), and
`V;{PST,FUT,COND,IMP};{IND,DEP,IMPRS,1SG,1PL,2SG,2PL,3,REL}` — the
synthetic slots. Scottish Gaelic has **no synthetic present** (the
present/progressive is periphrastic with *bi*) so no present row is
generated.

## Out of scope

The copula / substantive verb *bi*/*is* and their impersonal/negative
paradigm lemmas (`thathar`, `nach`, …) are wholly periphrastic and are
excluded from the gold (`EXCLUDE` in the converter). *thoir* is the one
suppletive verb whose conditional stem (*bheir-*) is supplied through
`data/gla/parts.tsv`; the remaining suppletives (*rach, thig, abair,
faigh, dèan, beir, cluinn, ruig, can*) have only principal parts, which
are mined directly.

## Result

- 5,396 scored forms, **100.00%**; lemma coverage 794/794 (100.00%).
- Gate: `min_form_pct` 99.8, `min_lemma_coverage_pct` 99.0.
