# French gold-data oracles

Research and baseline measurements for replicating the German verification
loop (two independent machine-readable oracles + adjudication of
disagreements) for French. Research date: 2026-08-16.

## The structural difference from German

For German, UniMorph deu and kaikki.org are treated as two oracles. For
French (and Spanish, Italian, Portuguese), **UniMorph is itself a scrape of
English Wiktionary** ([unimorph/fra](https://github.com/unimorph/fra),
CC BY-SA 3.0, 367,732 verb forms / 7,535 lemmas, verbs only, stub README) —
the same ultimate source as kaikki. "UniMorph agrees with kaikki" carries
near-zero evidential weight in French. The second oracle slot must be filled
by an independent lineage.

## Chosen oracle pair

1. **kaikki.org / Wiktextract French** —
   [dictionary/French](https://kaikki.org/dictionary/French/), en.wiktionary
   extraction, CC BY-SA / GFDL. The `fr-conj-auto` conjugation templates
   expand cleanly (spot-checked; 80+ tagged forms per verb, no failures).
   After conversion: **364,820 simple-tense forms / 7,328 verb lemmas**
   (`scripts/fetch_kaikki_fra.sh` → `data/kaikki/fra.tsv`).
2. **Lefff 3.4** (Lexique des Formes Fléchies du Français, INRIA/Alexina,
   Sagot) — **LGPL-LR**, hand-curated academic lexicon, **not
   Wiktionary-derived**. Canonical source:
   [gitlab.inria.fr/almanach/alexina/lefff](https://gitlab.inria.fr/almanach/alexina/lefff)
   (intensional); we fetch the compiled extensional `.mlex`. After
   conversion: **397,371 verb form rows / 7,819 verb lemmas**
   (`scripts/fetch_lefff.sh` → `data/lefff/fra.tsv`).

Third-oracle candidates for adjudication tie-breaks (in preference order):

- **[Verbiste](https://perso.b2b2c.ca/~sarrazip/dev/verbiste.html)** —
  GPL-2+, ~7,000 verbs, hand-built deterministic XML templates, maintained
  since 2003. Independent of both lineages.
- **[Grammalecte](https://grammalecte.net)'s conjugueur + lexicon** —
  MPL 2.0, Dicollecte lineage (OpenOffice dictionary), independent.
- **DELAF fr** ([Unitex](https://unitexgramlab.org), LGPL-LR, 683k forms /
  102k lemmas, LADL hand-built) — independent, good for coverage disputes.
- kaikki's **frwiktionary edition** ([link](https://kaikki.org/frwiktionary/))
  — different editor community and templates than en.wiktionary, but still
  Wiktionary-family; use only as a weak signal.

Rejected as oracles: **UniMorph fra** (Wiktionary-derived, see above),
**mlconjug3** (ML-predicted forms), **verbecc**'s ML fallback (its French
template data is Verbiste's, so it adds nothing over Verbiste),
**pattern.fr** (unmaintained, Wiktionary/Lexique-mined),
**Flexique** (CC BY-NC-SA), **Lexique.org** (frequency lexicon, incomplete
paradigms), **GLAWI/GLÀFF** (fr.wiktionary-derived).

## Feature schema

Both converters emit the shared `lemma<TAB>form<TAB>features` TSV:
`V;NFIN`, `V.PTCP;PRS`, `V.PTCP;PST;{MASC,FEM};{SG,PL}`,
`V;IND;PRS`, `V;IND;PST;IPFV` (imparfait), `V;IND;PST;PFV` (passé simple),
`V;IND;FUT`, `V;COND`, `V;SBJV;PRS`, `V;SBJV;PST`, `V;IMP` — finite slots
suffixed `;{SG,PL};{1,2,3}`. Compound tenses are excluded (compositional
layer's business, as in German). Lefff compact tags (`PS13s`) are
cross-product expanded; kaikki pronominal-verb clitics (*s'absentant*,
*absente-toi*, *nous absenterions*) are stripped to bare forms to match
Lefff (same policy as German reflexives).

## Baseline cross-oracle agreement (2026-08-16)

`python3 scripts/cross_oracle.py data/lefff/fra.tsv data/kaikki/fra.tsv …`

- shared (lemma, feature) slots: **277,379**
- oracles agree: **277,037 (99.88%)**
- disjoint variants: **342 (0.12%)**

The residual disagreement corpus is dominated by a single legitimate-variant
class: the **1990 rectifications orthographiques** on *-eler/-eter* verbs —
Lefff records the traditional doubled consonant (*ruisselle*,
*ensorcellera*, *trompette*), kaikki the reform grave-accent variants
(*ruissèle*, *ensorcèlera*, *trompète*). Both are standard; the engine
should eventually emit variant sets here (cf. German *stünde/stände*).
Smaller classes remain to be triaged (e.g. *enameure/enamoure*).

This is dramatically cleaner than the German baseline (1.09% disjoint), as
expected from two curated sources with disjoint failure modes.

## License posture

Same as German: the oracles are used for **verification only**; the shipped
exception lexicon stays MIT OR Apache-2.0. Lefff is LGPL-LR (weak copyleft,
no ShareAlike), so even this is a friendlier verification source than
CC BY-SA kaikki. Verbecc/Verbiste GPL code is never linked, only compared
against.

## Wider survey (Spanish, Italian, Portuguese)

Kept for the record, since French was chosen from a four-language survey:

- **Spanish**: kaikki es-conj expands fine (625,916 verb senses). UniMorph
  spa is Wiktionary-derived **and contains exactly 2^20 verb rows — the
  Excel row limit — a truncation smell**. Independent lineages: FreeLing
  dictionary (LGPL-LR, 555k forms, from the Spanish Resource Grammar) and
  LanguageTool's [spanish-dict-tools](https://github.com/jaumeortola/spanish-dict-tools)
  (LGPL-2.1). Fred Jehle's 600-verb database is CC BY-**NC**-SA.
- **Italian**: weakest ecosystem. Only solid independent lexicon is
  [Morph-It!](https://www.docs.sslmit.unibo.it/doku.php?id=resources:morph-it)
  (504,906 forms / 34,968 lemmas, dual CC BY-SA / LGPL, corpus-based).
  pattern.it is Wiktionary-mined with an 86%-accurate fallback.
- **Portuguese**: best permissive lexicon in the whole survey —
  [MorphoBr](https://github.com/LR-POR/MorphoBr) (**Apache-2.0**, corrected
  merge of DELAF-PB + LABEL-LEX lineages). Also LanguageTool
  [portuguese-pos-dict](https://github.com/languagetool-org/portuguese-pos-dict)
  (LGPL-2.1, the only resource separating pt-BR / pt-PT-45 / pt-PT-90
  orthographies). EU/BR + AO90 variant bookkeeping is the main complication.
- **All four**: verbecc's non-French seed data was machine-bootstrapped from
  the French Verbiste model (via mlconjug) — flag-raiser at best, never gold.
