# Latvian: skipped — two half-oracles do not make a loop

Empirical checks (2026-08-17):

1. **kaikki Latvian**: 911 of 60,264 verb entries expand a full
   table — thin, borderline-usable (the Slovenian precedent was 474).
2. **Tēzaurs.lv** dumps (CLARIN-LV serves TEI/LMF XML cleanly, 341 MB)
   carry lemma + paradigm class (`verb-2`, Konjugācija) but **no
   inflected forms** — the forms live in the LU MII morphology
   service, a hosted API rather than a downloadable artifact.
   Driving a rate-limited web service for the oracle would make CI
   depend on someone else's uptime.

Revisit if the LU MII generator ships as a downloadable model.
