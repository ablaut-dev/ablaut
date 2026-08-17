# Croatian: skipped — both oracle candidates failed on inspection

The EU-24 survey graded Croatian "go" (hrLex ∩ kaikki). Both legs
failed the empirical checks (2026-08-17):

1. **kaikki Serbo-Croatian**: Wiktextract does not expand the sh
   conjugation templates — **2 of 14,195** verb entries yield a full
   table (gledati comes back with just the infinitive). The same
   failure mode as Polish, and exactly what the table-count gate
   exists to catch.
2. **hrLex 1.3** (CLARIN.SI): the bitstream returns a server-side
   `FileNotFoundException … (Input/output error)` — CLARIN.SI's
   asset store is broken across handles (also Sloleks 2.0/3.0 and
   Romanian MULTEXT-East). Slovenian was rescued by a Hugging Face
   mirror; no hrLex mirror exists.
3. **Apertium hbs** is real and hand-built, but it would be the
   *only* oracle — a single-oracle loop cannot separate engine error
   from oracle error.

Revisit when either the sh templates gain a table-expansion in
Wiktextract or CLARIN.SI's store recovers.
