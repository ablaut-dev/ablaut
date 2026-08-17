# Dutch: skipped — no second independent oracle

The verification loop requires two independently-derived
machine-readable gold sources. Dutch has exactly one that is freely
obtainable:

1. **kaikki.org Dutch** (en.wiktionary via Wiktextract) — usable,
   full conjugation tables.
2. Everything else fails independence or availability:
   - **UniMorph nld** is scraped from Wiktionary — same lineage as
     kaikki, not independent.
   - **GiGaNT-Molex / e-Lex** (Instituut voor de Nederlandse Taal)
     require a signed license and portal login; the public bitstream
     URLs 404.
   - **CELEX** is licensed (LDC).
   - **OpenTaal** is a plain spelling wordlist with no morphological
     features — it cannot say which form fills which slot.

A single-oracle loop cannot distinguish engine error from oracle
error, which is the whole point of the agreement gold. Skipped until
a second lineage becomes freely available.
