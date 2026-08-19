# Icelandic gold-data oracles

The oracle pair for the Icelandic verification loop, chosen by the same
criterion as the other languages: two machine-readable sources of
independent provenance, so their agreement is evidence and their
disagreements are the adjudication corpus.

## Why this pair

UniMorph Icelandic is out for the usual reason: it is an
English-Wiktionary scrape, so it shares kaikki's lineage and its
agreement would be circular. The independent oracle is **BÍN**, the Árni
Magnússon Institute's national morphological database — a different
provenance entirely from Wiktionary.

1. **BÍN — Beygingarlýsing íslensks nútímamáls** (Sigrúnarsnið/SHsnid
   CSV; CC BY-SA 4.0). The authoritative lexicon of modern Icelandic
   inflection. `scripts/isl/fetch_bin.sh` (checksum-pinned against the
   Institute's own published sum) → `data/isl/bin.tsv`: 289,445 active-
   voice rows, 10,329 lemmas. This is the CI gold.
2. **kaikki.org Icelandic** (en.wiktionary via Wiktextract; CC BY-SA).
   `scripts/isl/fetch_kaikki.sh` → `data/isl/kaikki.tsv`: 7,529 rows,
   3,278 lemmas.

## Agreement, and the shape of kaikki's coverage

The two agree on **5,749 of 5,791** shared (lemma, feature) slots —
**99.27%** — across **1,900 shared lemmas**. The 42 disagreements are
spelling/ablaut edge cases (*ofmátu* vs *ofmetið*, *kúrað* vs *kúrt*) and
BÍN vs Wiktionary lemma splits; they are excluded from gold, not scored.

A caveat worth stating plainly: kaikki's Icelandic verb extraction is
**thin on the finite paradigm**. The English-Wiktionary tables reduce, per
verb, to the infinitive (the headword), the supine, and the third-person
past — plus the fully declined adjectival past participle, which is out of
this conjugator's single-form `V.PTCP;PST` schema. Present, subjunctive
and imperative forms are essentially absent from kaikki (a handful of
whole-paradigm entries aside). So the two-oracle agreement independently
verifies the infinitive, the supine and the 3rd-person past across 1,900
verbs; the rest of the paradigm is verified against BÍN alone. BÍN is the
authoritative national lexicon, and its 99.27% concord with the
independent kaikki sample on the slots kaikki does cover is the evidence
that it is trustworthy gold.

Because kaikki is not fetched in CI (and would gate almost nothing if it
were), the CI regression gate scores the engine against **BÍN directly**
— the full paradigm, all persons and moods — exactly as the Catalan gate
scores against FreeLing directly. Run `golden_isl` with no arguments to
score the two-oracle intersection instead.

## Feature schema

Same TSV as the other languages: `lemma ⇥ form ⇥ features`. Active-voice
(GM) synthetic slots only: `V;NFIN`, the present/past indicative
(`V;IND;PRS`/`V;IND;PST` by `{SG,PL};{1,2,3}`), the present/past
subjunctive (`V;SBJV;PRS`/`V;SBJV;PST`), the imperative (`V;IMP;{SG,PL};2`
— the clipped singular and the -ið plural), the present participle
(`V;V.PTCP;PRS`) and the supine (`V.PTCP;PST`, the indeclinable past
participle used with *hafa*). Middle voice (MM) and the question/clitic
forms are excluded.

## The engine

Icelandic verbs all share the `-a` infinitive, so the class is not
recoverable from the citation form. One class is productive and rule-
generated — the ō-class *kalla* (kalla/kallar/kallaði/kallað), with the
one live spelling rule, u-umlaut (`a → ö` on the stem's final vowel before
a u-theme ending: köllum, kölluðu). Every deviating verb — the weak dental
classes (i-mutation, syncope: telja→taldi), the strong ablaut verbs
(gefa→gaf→gáfu→gefið), the contracted *þvo/fá*, the impersonals — carries a
mined principal-parts row in `data/isl/verbs.tsv`, stored as base lemmas so
prefixed derivatives (*endurtaka* ← *taka*) come free by suffix match.
Bases that are a suffix of an otherwise-regular verb are flagged
exact-only.

The lexicon is mined from the oracle agreement:
`scripts/isl/capture_irregulars.sh` records the verbs the ō-rule misses
(scored against BÍN, all lemmas), and `scripts/isl/mine_verbs.py` fills
their paradigms from the BÍN ∩ kaikki merge.

## Score

100.00% of the 289,442 BÍN-scored slots (5 BÍN lemma/infinitive quirks
adjudicated — see `docs/isl/adjudications.tsv`), 99.99% lemma coverage;
and 100.00% of the 5,749 two-oracle-agreed slots.
