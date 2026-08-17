# Hungarian: skipped — kaikki's hu-conj tables do not expand

The EU-24 survey graded Hungarian "go" (kaikki ∩ emMorph). The
table-count gate failed the kaikki leg (2026-08-17): of 14,053 verb
entries, only **360** yield a full conjugation table, and they are
almost all -hat/-het potential derivatives (írhat, láthat) plus van
— the ordinary hu-conj templates for core verbs (ír, lát, beszél)
come back with no table rows at all.

emMorph (analyzer, hu.hfstol via emmorphpy) is a fine independent
lineage, but with no kaikki side there is nothing to agree with —
an analyzer can validate a gold set, not conjure one.

Revisit when Wiktextract learns the hu-conj module.
