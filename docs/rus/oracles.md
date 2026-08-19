# Russian gold-data oracles

The oracle pair for the Russian verification loop, chosen by the same
criterion as every other language: two machine-readable sources of
independent provenance, so their agreement is strong evidence and their
disagreements form the adjudication corpus.

## Why this pair

UniMorph rus is out: it is an English-Wiktionary scrape, so it shares
kaikki's lineage and its agreement would be circular. OpenCorpora is a
hand-checked Russian lexicon built by a native-language project with no
Wiktionary derivation — a genuinely independent second oracle.

1. **kaikki.org Russian** (en.wiktionary `ru-conj` templates via
   Wiktextract; CC BY-SA / GFDL). `scripts/rus/fetch_kaikki.sh` →
   `data/rus/kaikki.tsv`: 164,013 rows, 12,289 lemmas. The combining
   stress accents kaikki prints on every form (`возьмёт`, `взяла́`) are
   stripped — they are a pronunciation aid absent from OpenCorpora and
   from ordinary orthography — and the pre-reform ("dated") orthography
   layer (ъ, і) is dropped.
2. **OpenCorpora** (opencorpora.org, CC BY-SA; hand-checked).
   `scripts/rus/fetch_opencorpora.sh` (checksum-pinned to a 2024-04-23
   Wayback snapshot, since the origin is intermittently down) →
   `data/rus/opencorpora.tsv`: 411,088 rows, 30,951 verb lexemes.
   OpenCorpora splits one verb into separate consecutive lexeme blocks
   (finite `VERB`, `INFN`, participles, gerund); the converter pairs each
   `VERB` block with the next `INFN` block in its contiguous run to
   recover the infinitive lemma.

The two agree on **151,912 of 151,942** shared (lemma, feature) slots
where the engine is scored — **99.98%**. Across all shared slots the raw
agreement is 99.82% (273 slots are excluded from gold as oracle
disagreements). The residual disagreements are OpenCorpora ё-typos
(`поешься` for `поёшься`), a handful of stress-variant pasts
(`роздался`/`раздался`) and archaic/defective forms.

## Feature schema

Same TSV as the other languages: `lemma ⇥ form ⇥ features`. The finite
paradigm a learner reaches for, aspect-neutral in shape:

- `V;NFIN`
- `V;IND;PRS;{SG,PL};{1,2,3}` — imperfective synthetic present
- `V;IND;FUT;{SG,PL};{1,2,3}` — perfective synthetic future (identical
  morphology; the engine generates one non-past form for both labels)
- `V;IND;PST;{MASC,FEM,NEUT};SG` and `V;IND;PST;PL` — past in `-л`,
  inflected for gender/number, not person
- `V;IMP;{SG,PL};2` — the 2nd-person imperative

Perfective and imperfective are distinct lexemes and stay separate
lemmas. The periphrastic imperfective future (`буду` + infinitive),
participles and gerunds are out of scope.

## The engine

`src/rus.rs` is a rule engine for the two productive conjugations — 1st
(`-е-`/`-ё-`: читать → читаю; the `-овать`/`-евать` and `-нуть` subtypes)
and 2nd (`-и-`: говорить → говорю, with the productive 1sg mutation
любить → люблю, ходить → хожу) — plus the regular `-л` past and the
reflexive `-ся`/`-сь` postfix layered on top. The stressed perfective
prefix `вы-` de-stresses the stem (слать → шлёт but выслать → вышлет).

Stress is lexical and mobile and cannot be read off the infinitive; its
only orthographic trace in these unstressed forms is `ё`. Verbs whose
stem, mutation or stress the rules cannot predict live in
`data/rus/verbs.tsv` (2,881 non-reflexive base lemmas), stored so that
prefixed derivatives (написать ← писать) and reflexives (умываться ←
умывать) come free by suffix match; a base that is a suffix of an
otherwise-regular verb is flagged exact-match-only.

The lexicon is mined from the oracle agreement:
`scripts/rus/capture_irregulars.sh` records the verbs the rule engine
misses (scored against OpenCorpora alone, so OpenCorpora-only irregulars
are captured too), and `scripts/rus/mine_verbs.py` fills their paradigms,
storing per slot only the *irregular* form the rule cannot produce
(OpenCorpora canonical, kaikki filling gaps).

## Score

99.98% of the 151,942 agreed slots; 99.96% against OpenCorpora alone (the
CI gate) across all 30,951 lexemes, 100.00% lemma coverage.
