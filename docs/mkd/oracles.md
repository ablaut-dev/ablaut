# Macedonian: two independent oracles (apertium-mkd ∩ UniMorph mkd)

Macedonian passes the two-oracle criterion. The pair is independent by
lineage:

1. **apertium-mkd** — a hand-built lttoolbox dictionary, no Wiktionary
   lineage. `lt-expand` walks it into `surface:lemma<tags>` pairs;
   `scripts/mkd/apertium_to_tsv.py` maps its `<vblex>` tags onto the
   shared UniMorph-style bundle. ~817 verb lemmas.
2. **UniMorph mkd** — the Wiktionary lineage, ~3.5k verb lemmas. Stress
   marks (combining acute) are stripped in
   `scripts/mkd/unimorph_to_tsv.py` — Macedonian orthography does not
   write them and apertium does not emit them, so this aligns the two.

## Scope and result

Macedonian has no infinitive; the lemma is the 3sg present, and its final
thematic vowel gives the conjugation class (-а игра, -и носи, -е пише).
`src/mkd.rs` generates the imperfective synthetic system by rule —
present, imperfect (минато определено несвршено), the imperfect
l-participle, imperative — plus the passive participle and the non-finite
converb (глаголски прилог) and verbal noun (глаголска именка). Irregular
verbs carry explicit rows in `data/mkd/verbs.tsv`.

On the agreement set — **14,189 forms over 758 lemmas** where both oracles
overlap — the engine scores **100.00%**.

### Out of scope: the aorist

The aorist (минато определено свршено, V;PST) and its l-participle are
**not** generated. Both oracles carry them, so they appear as 10 uncovered
slot types in the harness report. The aorist stem is lexically
idiosyncratic (multiple ablaut/suffix patterns) and is left for a
follow-up; the engine covers the imperfective system in full.

## Oracle-vs-oracle disagreements

795 slots are excluded from the gold because the oracles disagree; they
are ruled on in `disagreements.tsv`, in three systematic buckets:

- **Converb (437), ruled `o1`** — apertium's `-јќи` is the standard
  spelling; UniMorph writes the ending with plain к (`-јки`), dropping ќ.
  The engine follows apertium.
- **j-stems (27 covered-slot splits), ruled `o2`** — vowel-final stems
  (брои, крие, пее) take a ј before the back-vowel present/imperative
  endings (брои → бројам, бројат, број) and -еја in the imperfect 3pl.
  UniMorph is correct; apertium over-regularizes (броам). The engine
  takes these verbs' finite forms from UniMorph (`data/mkd/verbs.tsv`).
- **Passive participle (25), ruled `variant`** — -ан vs -ен are both
  attested (бришан / бришен).
- The remainder are aorist forms (out of scope), where UniMorph also
  carries some Latin-character encoding noise; ruled `o1`.
