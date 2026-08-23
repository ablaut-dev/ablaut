# Classical Syriac (syc): parked — sparse seeds, participle-heavy

UniMorph `syc` has ~755 verb lemmas / 17k rows, but after stripping the
Syriac vowel points only **269** lemmas (36%) carry both a 3sg perfect and
imperfect, so principal-part seeding covers nowhere near the 99.5% lemma
bar. The paradigm is unusually participle-heavy (the active/passive
participles inflect for person and dominate the rows) and the roots are
written defectively (2–3 consonants), which the templatic engine can't
disambiguate from the citation alone. kaikki has no usable Syriac verb
extraction. A dedicated Aramaic FST oracle would be needed; parked until then.
