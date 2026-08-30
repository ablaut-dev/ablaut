# Navajo (nav): parked — templatic polysynthesis, no productive path

UniMorph `nav` is actually clean: 10,544 verb triples over **493 verbs** in a
uniform subject-agreement × mode/aspect grid (5 modes × person 1/2/3/4 ×
number, up to 50 cells). The blocker is not data — it is **architecture**.

A quantitative build attempt tested every productive route ablaut's
"principal parts + rules" contract relies on, over the full corpus:

| Productive rule | Best-case coverage |
|---|---|
| Subject agreement (3sg → 1sg within a mode) | 13.1% (137 distinct swap rules) |
| Distributive plural (`da-`) | 25.0% |
| 4th person from 3rd (`ji-`/`j-`) | 19.8% |
| Iterative from imperfective/perfective (`ná-`) | 4.0% / 0.0% |
| Optative from imperfective | 6.0% |
| **Union of all best-case rules** | **13.9% derivable → 86.1% stored** |

Navajo verbs fuse ~5–8 ordered prefix-complex morphemes (disjunct/thematic +
`da`-distributive + deictic 4th-person + iterative + mode conjugation-marker +
subject + classifier) plus a per-mode stem, with heavy morphophonemic fusion
(vowel contraction, tone, d-/l-effect). `da-` is an infix whose slot depends
on each verb's disjunct material (`iichįʼ` → `daʼiichįʼ`); the 3sg→1sg
transform needs 137 prefix-swaps because the classifier fuses (ł→sh for 1sg,
l→s …); and the perfective conjugation class (ø/yi/ni/si) is lexically
idiosyncratic. None of this is recoverable from the aligned surface forms
UniMorph provides.

A correct generator is a full Athabaskan position-class FST over a
morpheme-segmented lexicon (a digitized Young & Morgan), which no available
oracle supplies and which is a different architecture from every ablaut
engine. A storage-backed table would gate ~100% only against the data it
stores — a circular, meaningless check. Beta was the ceiling anyway (kaikki
`nav` is Wiktionary-lineage, not an independent second oracle). Revisit only
alongside a bespoke FST + segmented lexicon.
