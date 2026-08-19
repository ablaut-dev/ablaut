# Modern Greek: skipped — the second oracle is too thin and too noisy

The two-oracle method needs two *independent* machine-readable sources
whose agreement is strong enough to be gold. Modern Greek has a fine
first oracle but no viable second one, so the loop cannot be validated
and the language is not shipped.

Empirical checks (2026-08-19):

1. **kaikki Greek** (en.wiktionary `el-conjug-*` templates via
   Wiktextract; CC BY-SA / GFDL) is healthy: 13,565 verb entries,
   1,977 with a full conjugation table, 1,797 expanding to the active
   synthetic schema below. On its own this would be a usable oracle.

2. **Apertium-ell** (`apertium-ell.ell.dix`, GPL, commit `014936a`) is
   the only independent, non-Wiktionary candidate — and it is barely a
   verb lexicon. The monodix defines just **31** `vblex` paradigms,
   referenced by ~86 main-section entries. The pure-Python `.dix`
   expander (adapted from `scripts/slv/apertium_to_tsv.py`, no
   lttoolbox) yields only **45** lemmas with in-schema forms, of which
   **28** are shared with kaikki. The "~1,075 verb entries" figure the
   task assumed does not exist in the data.

3. Worse, much of what Apertium-ell does carry is wrong. Of the 461
   shared (lemma, feature) slots the two oracles both fill, they agree
   on only **423 — 91.76%**. The 38 disagreements are not spelling
   edge cases but genuine data bugs: misspelt forms (*δεέιξα* for
   *έδειξα*, *θήελες* for *θέλησες*, *δρέασες* for *έδρασες*), passive
   forms leaking into active slots (*αναλώνεστε*, *δείχθηκε*), and
   missing past augments (*διαβλέπαμε* for *διαβλέψαμε*). Much of the
   paradigm inventory lives in a `<!-- to be checked -->` / `check`
   section that is explicitly unfinished upstream.

Both ship gates fail decisively:

| gate            | required | observed |
| --------------- | -------- | -------- |
| shared lemmas   | ≥ 600    | **28**   |
| slot agreement  | ≥ 95%    | **91.76%** |

28 shared lemmas cannot adjudicate a Greek engine, and a second oracle
that disagrees with the first ~8% of the time — through outright errors
— would poison the gold rather than verify it. An honest skip is the
right result; kaikki alone is not two independent sources.

## Feature schema (for the record)

Modern Greek has **no infinitive**: the citation/lemma is the 1sg
present (γράφω). The active synthetic slots both scripts targeted:
present indicative `V;IND;PRS;{SG,PL};{1,2,3}`, past imperfect /
παρατατικός `V;IND;PST;IPFV;…`, aorist / simple past
`V;IND;PST;PFV;…`, the dependent perfective non-past (the -σω stem)
`V;PFV;DEP;…`, and the gerund / present active participle in -οντας
`V.PTCP;PRS`. The passive/mediopassive voice, the θα-/να- analytic
periphrases, the imperative and the passive perfect participle
(-μένος) were left out so both oracles would describe the same
paradigm.

Revisit if Apertium-ell's verb section is finished and expanded, or if
another independent Greek morphology (e.g. a downloadable generator
rather than a hosted API) becomes available.
