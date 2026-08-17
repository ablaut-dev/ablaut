# Finnish gold-data oracles

1. **kaikki.org Finnish** (en.wiktionary via Wiktextract; CC BY-SA):
   `scripts/fin/fetch_kaikki.sh` → `data/fin/kaikki.tsv` — 12,139
   verbs with full tables, the strongest kaikki showing in the
   project.
2. **Omorfi** (Flammie/Univ. Helsinki lineage, Apache/GPL — the open
   Finnish morphology, driven as a generator through the hfst Python
   bindings over `omorfi.generate.hfst`):
   `scripts/fin/fetch_omorfi.sh` → `data/fin/omorfi.tsv` — 466,088
   rows.

## Schema

V;NFIN, V;{IND;PRS, IND;PST, COND, POT};{n};{p} each with a PASS
impersonal, V;IMP;{SG;2, SG;3, PL;1, PL;2, PL;3} + PASS,
V.PTCP;{PRS,PST};{ACT,PASS}. Negatives and periphrastic perfects are
out of scope.

## Baseline and result (2026-08-17)

- 414,410 shared slots, 98.02% baseline — the largest agreement set
  in the project; the disjoint corpus is mostly gradation
  disagreements (omorfi conjugates *aitautua* without the t→d kaikki
  and standard usage apply)
- Engine: type-I defaults + 11,877 mined rows (weak/strong stems,
  past stems, nut, the two passive bases); the conditional contracts
  uo/yö/ie and halves long vowels (juo → joisi, nyhjää → nyhjäisi),
  the potential and koon-imperatives ride the nut base (tullut →
  tullee, tulkoon), vowel harmony follows the last non-neutral vowel
  with mined-nut override for neutral-final compounds (aivopestä →
  aivopessyt → -vät)
- Agreement gold: 406,209 forms, **100.00%**
- Omorfi alone (CI gate): 463,660 forms, 98.70% (gates 98.5/99.5) —
  the residual is omorfi's own broken classes on ~500 rare
  derivative verbs (höyläyttää → \*höyläyttin), where kaikki and the
  engine agree against it
