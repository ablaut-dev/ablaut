# Marathi gold-data oracles

Marathi (मराठी, `mar`) is an Indo-Aryan language written in Devanagari,
the same script as its sister Hindi (`src/hin.rs`). Its verbs inflect for
person, gender and number — Marathi keeps the three-gender system
(masculine, feminine, **neuter**) live on the verb, exactly as Gujarati
does (`src/guj.rs`) — across a present habitual, a perfective (the simple
past), a subjunctive, a future and an imperative, plus a set of
non-finite forms. The engine follows the same design as those two
languages: stem = infinitive minus the `-णे` suffix, agreement by
suffixation, and a small compiled-in table of the verbs whose stems are
not derivable.

## The oracle situation, and why this ships Beta

The plan (from the roadmap) was a Wiktionary extraction (kaikki.org) plus
a second independent source. UniMorph has **no** Marathi verb data, so
the second source is the Apertium morphological analyser. Both were
built and evaluated honestly; the outcome is a strong single-oracle
score with an independent non-finite cross-check, so Marathi ships
**Beta** rather than claiming a two-oracle *Verified* gate it does not
have across the finite paradigm.

1. **apertium-mar** (Apertium monolingual Marathi package, GPL; commit
   `959f483`). A hand-written lttoolbox dictionary with **no Wiktionary
   lineage** — the independent, primary oracle. `scripts/mar/fetch_apertium.sh`
   clones it and runs `lt-expand` to walk the dictionary into every
   surface/analysis pair, then `scripts/mar/apertium_to_tsv.py` maps the
   analyses onto the shared feature bundle: **56,551 forms over 1,304
   verb lemmas**, fully tagged for person × gender × number across the
   present habitual, perfective, subjunctive, future, imperative and the
   three non-finite forms. This is the paradigm the engine is scored
   against, per cell. Only `lt-expand` (from `lttoolbox`) is needed, not
   the full Apertium pipeline.

2. **kaikki.org Marathi** (Wiktextract of English Wiktionary, CC BY-SA).
   `scripts/mar/fetch_kaikki.sh` → `scripts/mar/kaikki_to_tsv.py` →
   `data/mar/kaikki.tsv`. The independent second leg, but with a
   limitation: **Wiktextract could not map the person/number row headers
   of the `mr-conj` table**, so every finite cell keeps only a
   tense+gender tag and loses its person and number. kaikki therefore
   cannot form a per-cell agreement loop over the finite paradigm the way
   apertium-pes does for Persian.

What kaikki *can* contribute is used, and honestly:

- **Non-finite two-oracle agreement (per cell).** The infinitive, the
  completive converb (करून), the prospective (करणार) and the purposive
  (करायला) are unambiguously tagged in kaikki and map onto apertium's
  `inf` / `trans+perf` / `pros` / `sup`. The two oracles agree on
  **493 of 493** shared non-finite cells over **124 shared lemmas**
  (100%). The single disagreement — the infinitive of होणे, which
  apertium truncates to होण — is resolved in
  [`disagreements.tsv`](disagreements.tsv) in kaikki's favour (होणे).
- **Finite set-level corroboration.** With person/number gone, the
  finite cells can still be checked as a *set*: **2,968 of 3,000**
  (98.9%) of kaikki's finite forms over the 124 shared lemmas appear in
  apertium's form set for the same lemma. The only residue is the two
  most suppletive verbs, असणे "be" and होणे "become", whose irregular
  cells the two sources spell differently — and those are exactly the
  verbs carried in `data/mar/verbs.tsv`.

So the finite paradigm is scored per cell against one independent oracle
(apertium-mar) and corroborated at the set level by a second (kaikki);
the non-finite forms clear a genuine per-cell two-oracle gate. That is a
Beta, not a Verified — the same honest posture Gujarati and Telugu take.

## Feature schema

`lemma ⇥ form ⇥ features`, UniMorph-style bundles shared by both
adapters. Person `{1,2,3}`, gender `{MASC,FEM,NEUT}`, number `{SG,PL}`.
The bounded cell set the engine models and is scored on:

- **present habitual** `V;IND;PRS;HAB;{p};{g};{n}` (करतो, करते, करतं…),
  the imperfective, agreeing with the subject in person/gender/number;
- **perfective** `V;IND;PST;PFV;{p};{g};{n}` (केला, केली, केलं…), the
  simple past;
- **subjunctive** `V;SBJV;{g};{n}` (करावा, करावी, करावं…) — it does not
  distinguish person;
- **future** `V;IND;FUT;{p};{n}` (करेन, करशील, करेल…) — no gender;
- **imperative** `V;IMP;{p};{n}` (कर, करा, करू, करो, करोत);
- the non-finite **infinitive** `V;NFIN`, **completive converb**
  `V;CVB;PFV` (करून), **prospective** `V;PROSP` (करणार) and **purposive**
  `V;PURP` (करायला).

Deliberately **excluded** from the scored set: the **combined-gender
honorific plural** cells apertium tags `MF`/`MFN` in the perfective (the
तुम्ही-agreement केला(त) beside the plain plural केले), the **emphatic
clitics** (+च/+ही), the **case-marked gerunds**, the **perfect and
agentive participles** (केलेला, करणारा) and the analytic **compound
tenses** (present continuous करतो आहे, past continuous करत होतो), which
are built periphrastically with the auxiliary असणे. apertium's two
`PST;PFV;{1,2};MF;PL` slots are the only paradigm slot types the engine
does not cover.

### apertium data hygiene

`scripts/mar/apertium_to_tsv.py` drops a handful of apertium-mar
dictionary bugs that no engine could match: a stemless garbage entry
(णे), a lemma whose stem is doubled in the citation key (सांगसांगणे for
सांगणे), a दीर्घ/ह्रस्व misspelling (लिहीणे), three lemmas cited with an
un-metathesised हा stem that disagrees with their own forms
(रहाणे/पहाणे/वहाणे — the standard lemmas are राहणे/पाहणे/वाहणे), the
no-space light-verb compounds (प्राप्तहोणे), and the non-standard
हो-stem होणे subjunctive (होवा; standard Marathi is व्हावा, on which the
engine and kaikki agree). The one remaining oracle error the engine
beats — apertium's truncated infinitive होण — is logged in
[`adjudications.tsv`](adjudications.tsv).

## The engine

`src/mar.rs` drops the `-णे` to get the stem and builds the whole system
by suffixation — Marathi is the most agglutinative of the Indo-Aryan set,
so the productive rule is almost entirely a matter of stacking a suffix:
the present habitual on `stem + त`, the perfective on `stem + ल`, the
subjunctive on `stem + ाव`, the future and imperative directly on the
stem, and the three non-finite forms. An `आ`-final stem coalesces the
doubled long-*a* (गा + ाव → गावा, not गाावा). Compound lemmas
(`अभ्यास करणे`) conjugate only their last word.

What the rules cannot predict lives in `data/mar/verbs.tsv` (9 rows): the
irregular perfective of करणे (केल-), the contracted stems of
देणे/घेणे (दिल-/घेतल-, subjunctive द्याव-/घ्याव-, converb देऊन/घेऊन), the
suppletive perfectives of जाणे (गेल-), येणे (आल-) and होणे (झाल-,
subjunctive व्हाव-), and three verbs with an irregular perfective base
(म्हणणे → म्हटल-, मिळणे → मिळाल-, सांगणे → सांगितल-).

## Score

**100.00%** of the **56,551** scored apertium-mar forms, **1,304 of
1,304** lemmas covered, **44 of 46** paradigm slot types covered (the two
uncovered are the combined-gender honorific perfective plurals, excluded
by design), **1** oracle error adjudicated → **Beta**.

The independent kaikki cross-check: **493 of 493** shared non-finite
cells agree per cell over 124 lemmas (the one disagreement resolved), and
**2,968 of 3,000** finite forms are corroborated at the set level.

## Reproducing

```sh
./scripts/mar/fetch_apertium.sh    # → data/mar/apertium.tsv (needs lttoolbox)
./scripts/mar/fetch_kaikki.sh      # → data/mar/kaikki.tsv (needs curl)
cargo run --release --bin golden_mar -- data/mar/apertium.tsv --check       # scored (Beta)
cargo run --release --bin golden_mar -- data/mar/apertium.tsv data/mar/kaikki.tsv --check  # non-finite two-oracle gate
```
