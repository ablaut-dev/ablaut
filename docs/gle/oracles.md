# Irish gold-data oracles

1. **kaikki.org Irish** (en.wiktionary via Wiktextract; CC BY-SA):
   `scripts/gle/fetch_kaikki.sh` → `data/gle/kaikki.tsv` — 1,365
   verbs with full tables. Analytic rows (glanann tú) are multi-word
   and skipped; initial mutations are normalized away (ghlan → glan,
   d'ól → ól) since kaikki prints past lenition and BuNaMo does not.
2. **BuNaMo** (the Irish National Morphology Database, Foras na
   Gaeilge / Michal Boleslav Měchura; ODbL):
   `scripts/gle/fetch_bunamo.sh` → `data/gle/bunamo.tsv` — 3,359 verb
   XMLs, 117,449 rows. Independent forms only.

## Schema

V;VN (verbal noun), V.PTCP (verbal adjective), and
V;{PRS,PST,PSTHAB,FUT,COND,IMP,SBJV};{BASE,1SG,2SG,1PL,2PL,3PL,AUTO}
— the synthetic slots; Irish fills the rest analytically with
pronouns. Forms are unmutated citation forms; the engine's `lenite`
applies the display mutation.

## Baseline and result (2026-08-17)

- 27,892 shared slots, 98.25% baseline (the disjoint corpus is
  syncope disagreements: adhnfaidís vs adhanfaidís)
- Engine: five classes parsed off the mined present base
  (glanann/briseann/ceannaíonn/bailíonn/brúnn — the fifth is the
  monosyllabic vowel stems, with é-stems linking their past plurals:
  léigh → léamar), four driving stems (present, past, future →
  conditional, t-stem of the autonomous → glantá/glantaí), 981 mined
  rows, and hardcoded suppletive past plurals for tar and
  clois/cluin (tángamar, cualathas)
- Agreement gold: 26,053 forms, **100.00%**
- BuNaMo alone (CI gate): 109,821 forms, 99.47% (gates 99.0/99.5)
