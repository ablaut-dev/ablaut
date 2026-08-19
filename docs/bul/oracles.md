# Bulgarian: skipped — two independent oracles agree, but the overlap is too thin

Bulgarian has no infinitive: the citation/lemma form is the 1sg present
indicative (пиша), so the join key throughout is that form, and there is
no `V;NFIN` slot. The synthetic core both oracles emit — present,
imperfect (минало несвършено), aorist (минало свършено), the 2nd-person
imperative, and the bare (indefinite) aorist/imperfect/present active and
past passive participles — maps cleanly onto one shared UniMorph-style
schema (`scripts/bul/kaikki_to_tsv.py` and
`scripts/bul/apertium_to_tsv.py`, a pure-Python `.dix` expander adapted
from `scripts/slv/apertium_to_tsv.py`).

Empirical checks (2026-08-19):

1. **kaikki Bulgarian** (en.wiktionary `bg-conj` via Wiktextract;
   CC BY-SA). Passes the table-count gate: **2,742** of 31,039 verb
   entries carry a full conjugation table (the earlier gate saw 2,836;
   Wiktionary edits drift). → **2,636** distinct 1sg-present lemmas in the
   shared schema.
2. **Apertium-bul** monodix (GPL-2.0; independent of Wiktionary). The
   ~5,600 `vblex` tags advertised are overwhelmingly *inside* the 319
   paradigm definitions; the dictionary itself has only **1,434** verb
   section entries → **1,284** distinct 1sg-present lemmas.

The two are genuinely independent and, where they overlap, they agree:
**17,338 of 17,456** shared (lemma, feature) slots — **99.32%**. The 118
disagreements are the expected Bulgarian orthography splits — the ят
(е/я) alternation (*спели*/*спяли*, *посетела*/*посетяла*; 39 of 118) and
imperative suppletion (*дойди*/*ела*, *дайте*/*дадете*).

The blocker is **coverage, not quality**: the two verb lists share only
**518 lemmas** — below the 800-lemma bar for a trustworthy two-oracle
gold corpus. 763 of Apertium's 766 non-overlapping lemmas are absent from
kaikki *as words entirely* (prefixed perfectives and `-вам`
imperfectives — *препокрия*, *заключавам*, *оздравявам* — plus the large
`-ирам` loanword class that en.wiktionary has not tabulated), not merely
missing a table. There is no lemma-key normalization that recovers them;
the overlap is a hard property of the two sources.

Both converters are written and working, so the loop is ready the day a
second independent lineage widens the overlap — either kaikki's Bulgarian
conjugation coverage grows past ~4,000 verbs, or an open, non-Wiktionary
Bulgarian morphology lexicon (a released BulTreeBank lexicon, or an
expanded Apertium-bul) ships enough verbs to clear 800 shared lemmas.
Parked, not forced.
