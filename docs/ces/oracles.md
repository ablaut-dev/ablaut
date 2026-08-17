# Czech gold-data oracles

1. **kaikki.org Czech** (en.wiktionary via Wiktextract; CC BY-SA):
   `scripts/ces/fetch_kaikki.sh` → `data/ces/kaikki.tsv` — 130,373
   rows, 4,322 verbs with full tables. The table-count gate passes
   decisively (the Polish extraction, by contrast, expands 3 tables).
2. **MorfFlex CZ 2.1** (LINDAT/CLARIAH-CZ, Hajič et al.; CC BY-NC-SA,
   used strictly test-time): `scripts/ces/fetch_morfflex.sh` →
   `data/ces/morfflex.tsv` — 2.2 M verb rows, 56,301 lemmas after
   filtering. Prague positional tags; the converter drops negated
   rows (pos11), clitic-fused forms (pos14 's': abdikovalas) and
   colloquial variants (pos15 6/7), and expands the combined gender
   codes (Q/W → feminine singular + neuter plural).

## Schema

V;NFIN, V;IND;PRS;{n};{p}, V;IMP;{SG;2, PL;1, PL;2},
V.PTCP;{PST,PASS};{MA,MI,F,N};{SG,PL}, V;CVB;PRS;{M,FN,PL}.

## Baseline and result (2026-08-17)

- 114,843 shared slots, 98.78% baseline (the disjoint corpus is
  dominated by kaikki homograph-entry noise: bílit picking up bít's
  forms)
- Engine: six present classes (dělá/mluví/kupuje/tiskne/trpí/bělejí),
  8,803 mined class assignments + 10,393 explicit rows; participle
  genders derive from the masculine (mluvil → mluvila/mluvili), the
  jít family's fleeting -e- (šel → šla) rides a mined feminine base,
  imperative plurals soften -i → -ě (tiskni → tiskněme)
- Agreement gold: 107,812 forms, **100.00%**
- MorfFlex alone (CI gate): 1,609,768 forms, 99.99% (gates 99.9/99.5)
