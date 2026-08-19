# Hungarian: skipped — the second oracle is morphologically wrong

Re-attempted 2026-08-19 with a **new oracle pair**. The earlier skip
(below) blamed kaikki: Wiktextract could not expand the `hu-conj` module,
so only ~360 verbs yielded a table. **That blocker is gone** — kaikki now
expands cleanly. The wall is now the *second* oracle.

## kaikki now expands (the old blocker is resolved)

Of 14,053 verb entries, **4,712** carry a full conjugation table. There
is one Wiktextract quirk: every real cell is stamped
`error-unrecognized-form` and its person/number tag is shifted by a row,
the conditional and subjunctive blocks are not tagged with their mood at
all, and archaic tenses (the *elbeszélő múlt* láték, the *-and* future
látand) are interleaved with the modern paradigm. The **forms
themselves are correct** and appear in a fixed reading order, so
`scripts/hun/kaikki_to_tsv.py` decodes them positionally: it anchors on
the bare infinitive and the `past` tag, and segments the finite moods on
their `-lak`/`-lek` object forms (a reliable per-mood terminator). This
reconstructs the full paradigm exactly — *lát* comes out perfect across
all 48 finite slots plus the infinitive, with the definite/indefinite
split intact. **2,687** verbs decode to a full positional table.

Definiteness — the feature this language was expected to founder on —
reconciles cleanly: both oracles carry an explicit DEF/INDF tag and they
align.

## Apertium-hun is not a trustworthy generator

`apertium-hun` has ~12,600 `vblex` entries over ~98 shared paradigms, and
`scripts/hun/apertium_to_tsv.py` expands them (pure-Python, no
lttoolbox). But the paradigms **crudely concatenate suffixes and do not
apply Hungarian's obligatory phonology.** Confirmed in the raw monodix,
not an expansion artifact — the `tart__vblex` pardef literally stores the
subjunctive as `-sak/-son/-sák`:

| slot | kaikki (correct) | apertium-hun (wrong) | rule apertium skips |
|------|------------------|----------------------|---------------------|
| *abajgat* SBJV 3sg | abajgasson | abajgatson | t + j → ss (assibilation) |
| *szennyez* SBJV 1pl | szennyezzünk | szennyezsünk | z + j → zz |
| *fej* SBJV 2pl | fejjetek | fejsetek | j-assimilation |
| *elér* COND def 3sg | elérné | elérené | no linking vowel |
| *ver* PST 3sg | vert | verött | no linking vowel / rounding |
| *ésszerűsít* PST 3pl | ésszerűsítettek | ésszerűsíttek | -ít past epenthesis |
| *jellemez* PST 1sg | jellemeztem | jellemeztdm | (corrupt) |

Assibilation with the `-j-` suffix (`olvas+j → olvass`, `mos+j → moss`)
is precisely the phenomenon this engine would exist to get right, and it
is the phenomenon apertium gets wrong.

## Why this is a stop, not a loop

The two oracles agree on **81,959 of 91,275** shared (lemma, slot) pairs
across **2,177 shared lemmas** — **89.79%**, below the 95% gate. Crucially
the ~10% of disagreements are **not adjudicable edge cases**: on
inspection every one is an apertium error against a correct kaikki form,
concentrated in the assibilating (`-t`, `-sz`, `-z`, `-s`) and epenthetic
(`-ít`) classes and the past/conditional linking vowels.

Building gold from the agreement would therefore be actively misleading:
it would silently **exclude exactly the hard, interesting slots** (the
ones an engine must get right and could plausibly get wrong) and keep
only the phonologically trivial ones. An engine that produced the correct
assibilated forms would *disagree* with apertium there, and — because
those slots are dropped as "not agreed" — would never be scored on them.
A green check would certify nothing about the parts that matter.

This is the Latvian situation (`docs/lav/oracles.md`) in a different key:
one good source (kaikki) and one that cannot carry the other half of the
loop. UniMorph hu is banned (an English-Wiktionary scrape, circular with
kaikki); emMorph (`hu.hfstol`) is an *analyzer*, which can validate a gold
set but cannot independently generate one.

Revisit if a correct, downloadable Hungarian **generator** of independent
provenance appears (e.g. a released HFST/emMorph generation model, or an
apertium-hun that applies its twol phonology at monodix-expansion time).

---

## Original note (2026-08-17): kaikki's hu-conj tables do not expand

The EU-24 survey graded Hungarian "go" (kaikki ∩ emMorph). The
table-count gate failed the kaikki leg: of 14,053 verb entries, only
**360** yielded a full conjugation table, and they were almost all
-hat/-het potential derivatives (írhat, láthat) plus van — the ordinary
hu-conj templates for core verbs (ír, lát, beszél) came back with no
table rows at all. (Resolved: see above — Wiktextract now expands them.)
