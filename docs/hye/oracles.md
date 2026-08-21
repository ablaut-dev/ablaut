# Eastern Armenian gold-data oracles

The oracle pair for the Armenian verification loop, chosen by the same
criterion as every other language: two machine-readable sources of
independent provenance, so their agreement is strong evidence and their
disagreements form the adjudication corpus.

## Why this pair

The Wiktionary leg here is **UniMorph hye**, not kaikki. Two reasons,
one practical and one substantive. The practical one: kaikki.org was
unreachable from the environment this language was built in (the egress
policy allows GitHub and the package registries and nothing else), so a
kaikki Armenian extraction could not be fetched, converted or checked,
and the loop was not going to rest on a source that had never been run.
The substantive one: for Armenian, UniMorph is the better artifact
anyway. It is an English-Wiktionary extraction that was *validated by an
Armenian morphologist* (Hossep Dolatian) as part of the UniMorph
release, and it spells the analytic tenses out as multi-word forms,
which is what the Armenian paradigm mostly consists of. The usual
objection to UniMorph — that pairing it with kaikki is circular, both
being Wiktionary — does not apply when it *is* the Wiktionary leg.
Adding a kaikki leg later would be a strict addition: a third source
against the same schema.

1. **UniMorph hye** (English Wiktionary, CC BY-SA 3.0).
   `scripts/hye/fetch_unimorph.sh` (commit-pinned, sha256-checked) →
   `data/hye/unimorph.tsv`: 133,534 rows, 899 verb lemmas × ~150 slots.
   The imperative's emphasis mark (գրի՛ր) is stripped — it is prosodic,
   not part of the spelling — and the handful of entries still written
   with the pre-1922 digraph եւ are rewritten to the ligature և.
2. **uniparser-grammar-eastern-armenian** (Timofey Arkhangelskiy, MIT):
   a formalized description of literary Eastern Armenian morphology,
   with no Wiktionary lineage. `scripts/hye/fetch_uniparser.sh`
   (version-pinned to 2.1.2) → `data/hye/uniparser.tsv`: 1,028,594 rows,
   14,229 verb lemmas. The package ships an *analyzer*, so
   `scripts/hye/uniparser_gen.py` drives it as a generator: it walks
   each verb lexeme's paradigm and follows the `<.>` continuation links
   the library leaves unexpanded (`գր<.>` + `aor-ec` → գրեցի…), stopping
   at the nominal paradigms, since a declined infinitive or participle
   (գրելը, գրողին) is noun inflection. Pre-reform spellings (the `-աւ`
   aorist) are dropped.

The two agree on **50,521 of 50,700** shared (lemma, feature) slots —
**99.65%** — across **823** shared lemmas. All 179 disagreements are
ruled on in [`disagreements.tsv`](disagreements.tsv); they fall into
five groups, and 149 of them are the grammar file failing to apply a
productive rule (the `-անալ/-ենալ` inchoatives, the `-նել` class, the
suppletive perfective of colloquial երթալ/էթալ/տենալ).

## Feature schema

`lemma ⇥ form ⇥ features`, UniMorph bundles. The synthetic core — where
the two oracles overlap and where the engine is scored — is:

- `V;NFIN` and its negation `V;NFIN;NEG` (գրել, չգրել);
- the converbs `V.CVB;IPFV` (գրում), `V.CVB;PFV` (գրել),
  `V.CVB;FUT;LGSPEC02` (գրելու), `V.CVB;FUT;LGSPEC03` (գրելիք),
  `V.CVB;SIM` (գրելիս) and the connegative `V.CVB;LGSPEC04` (գրի);
- the participles `V;V.PTCP;SUB` (գրող) and `V;V.PTCP;LGSPEC01` (գրած),
  each with its `չ-` negative;
- the aorist `V;IND;{SG,PL};{1,2,3};PST` (գրեցի) ± `NEG`;
- the subjunctive future `V;SBJV;…;FUT` (գրեմ), the subjunctive past
  `V;PRF;SBJV;…;FUT` (գրեի), and the conditionals under `կ-`,
  `V;COND;…;FUT` (կգրեմ) and `V;PRF;COND;…;FUT` (կգրեի);
- the imperatives `V;IMP;{SG,PL};2` (գրիր, գրեք);
- the derivation slots `V;PASS` (գրվել) and `V;CAUS` (գրեցնել), whose
  fillers are separate lemmas on the Wiktionary side.

The **analytic** tenses — present գրում եմ, imperfect գրում էի, perfect
գրել եմ, pluperfect գրել էի, future գրելու եմ, future-in-the-past
գրելու էի, and their negatives with the copula fronted (չեմ գրում) —
are in the engine and in UniMorph, but the grammar-derived oracle emits
the copula as a separate token, so they cannot enter the agreement.
They are scored against UniMorph alone (see **Score** below).

## The engine

`src/hye.rs` builds four stems off the infinitive and its class (`-ել`
vs `-ալ`): the present stem (subjunctive, imperfective converb,
conditional in `կ-`), the perfect stem (perfective converb, resultative
participle, plural imperative), the aorist stem with one of three
ending sets, and the subject-participle stem. Two productive subclasses
shift them — the `-անալ/-ենալ` inchoatives (մեծանալ → մեծացա, մեծացել)
and the `-ցնել` causatives (ուրախացնել → ուրախացրի, ուրախացրու) — and
the analytic layer is the converbs plus the copula.

What is left over lives in `data/hye/verbs.tsv` (42 rows): the `-նել`
class that drops its `-ն-` (հագնել → հագա), the suppletives (գալ → եկա,
ուտել → կերա, տալ → տվեցի), the `-առնալ` verbs (դառնալ → դարձա) and the
adjudicated archaisms. It is mined by `scripts/hye/mine_verbs.py` from
the oracle agreement, restricted to the lemmas the rule engine misses
with an empty table (`scripts/hye/capture_irregulars.sh`), and extended
by every adjudicated lemma, so a ruling in `disagreements.tsv` reaches
the engine instead of only the report.

## The core-verb gap, and the treebank spot check

UniMorph hye's 899 lemmas are a thin slice of the language, and they
happen to include **none** of the everyday irregulars: գալ, տալ, ուտել,
անել, լինել, տեսնել, դնել, գտնել and their kin are absent, so no
agreement exists to mine them from. They are hand-supplied in
`scripts/hye/manual.tsv` (24 rows) from the uniparser grammar, with that
grammar's mechanical slips corrected — it derives թողրել/թողրած for
թողնել and տվր for the imperative of տալ.

Because that is the one place where a single source would otherwise go
unchecked, a third, corpus-derived reference is used as a spot check:
the **UD Armenian-ArmTDP** treebank (hand-annotated, CC BY-SA 4.0),
converted by `scripts/hye/fetch_armtdp.sh` → `data/hye/armtdp.tsv`
(3,866 attested slots over 1,428 lemmas). A treebank attests forms
rather than filling paradigms, so it is not an oracle; it is run as a
report:

```sh
./scripts/hye/fetch_armtdp.sh
python3 scripts/hye/armtdp_check.py > target/armtdp_core.tsv
cargo run --release --bin golden_hye -- target/armtdp_core.tsv
```

All 24 core lemmas are attested, and the engine matches **313 of 330**
attested slots. Every one of the 17 misses was inspected and none is an
engine error:

- **8 tagging slips** — subjunctive and conditional *past* forms tagged
  `Tense=Pres` (կդներ, կթռչեր, կասեի, չլինեի, չլինեինք, առներ, ասեր),
  and an aorist 1sg slot filled by a plural token (արեցինք);
- **3 lemma slips** — tokens of the causatives հագցնել and կերցնել
  filed under the lemmas հագնել and ուտել;
- **6 colloquial doublets** both sources acknowledge — մտի/մտիր,
  թռի/թռիր, տվիք/տվեցիք, բերիր/բերեցիր, արիր/արեցիր, եղող/լինող.

## Score

**100.00%** of the 50,521 agreed slots, 100.00% lemma coverage, every
paradigm slot type covered and all 179 oracle disagreements resolved.
Against UniMorph alone — which adds the analytic tenses — 99.61% of
133,534 forms, with the analytic layer itself at 99.67%; the residue
there is Western-Armenian and pre-reform lemmas (խօսել, սիրուել) that
the second oracle does not corroborate.
