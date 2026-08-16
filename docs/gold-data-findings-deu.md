# Gold-data findings

An exhaustive register of the problems found in the two gold datasets while
validating ablaut against them, with a disposition for each. Nothing here
has been reported upstream yet; the **Disposition** column records the
intended channel. Individual per-form rulings live in
[adjudications-deu.tsv](adjudications-deu.tsv); this document groups them into
findings and adds the systematic patterns visible in the
oracle-disagreement corpus (2,137 slots where the two datasets contradict
each other, out of 195,204 shared).

Datasets: **UniMorph deu** (github.com/unimorph/deu, extracted from
Wiktionary) and **kaikki.org** (Wiktextract, also from Wiktionary).
Channels: *unimorph issue* (github.com/unimorph/deu),
*wiktionary edit* (fix the source page, both datasets inherit),
*wiktextract issue* (github.com/tatuylonen/wiktextract, extraction bug),
*variant contribution* (the schema already allows several rows per slot;
the fix is adding the missing variant, not a new representation),
*our policy* (handled on our side, documented here).

## A. UniMorph deu: data errors

| # | Finding | Evidence | Scale | Disposition |
|---|---|---|---|---|
| A1 | Paradigm of a different verb: the dataset's own V;NFIN row contradicts the lemma, and every form belongs to the base verb without its particle | einknicken → "knicken", erhärten, auswiegen | 39 lemmas: Aa,Sinn anessen,ausbessern auslöffeln,auswiegen beiseitigen,bespeien demolieren,einknicken erhärten,erlöschen ermuntern,etappieren gegenzeichnen,herunterstoßen hervorbrechen,hinknieen in,in in,in ins,ins ins,kajolen knibbeln,lieb matschen,mitessen mitleiden,pushen reversieren,rumalbern sakralisieren,schauspielern schäkern,suspendieren zurückfordern,zustürmen | unimorph issue |
| A2 | Konjunktiv II of *nennen* given as the indicative preterite (*nannte*); correct is *nennte* (Duden; siblings kennen/brennen/rennen are correct in the data) | nennen V;SBJV;\*;PST | 6 slots | unimorph issue |
| A3 | Imperative of *wissen* marked nonexistent; *wisse!*, *wisst!* exist | wissen V;IMP | 2 slots | unimorph issue |
| A4 | Sibilant-stem preterite 2sg *-est* variants missing (*aßest, lasest, saßest, hießest*); only the *-t* forms are listed | essen, lesen, sitzen, heißen, fließen, stoßen, vergessen, schließen, wachsen | ~10 lemmas | variant contribution |
| A5 | Trailing whitespace inside form cells | every form of *schließen*, *stehen*, *senden* | dozens of paradigms | unimorph issue |
| A6 | Modal-compound paradigms drop the particle in some rows | wegwollen preterite "wollten", abkönnen "konnten" | handful of lemmas | unimorph issue |
| A7 | Systematic generation bugs in derived verbs: doubled particles (*eineingebüßt*), *ge-* after inseparable prefixes (*vergeglüht*), doubled ending characters (*reversiertte*), dropped stem syllables (*einzustehen* for eingestehen, *schieflegen* for schiefgelegen), unsplit compounds split wrongly (*erstanden wiederauf*), stray spellings (*sacralisieren*) | see oracle_disagreements.tsv | ≥190 slots by pattern count (132 doubled endings, 47 misplaced *ge-*, 9 doubled prefixes, plus singletons) | unimorph issue |

## B. Wiktionary content errors (inherited by kaikki, sometimes both)

| # | Finding | Evidence | Disposition |
|---|---|---|---|
| B1 | *wachen* given auxiliary *sein*; it takes *haben* | kaikki V;AUX | wiktionary edit |
| B2 | *entgelten* conjugated weak (*entgeltete*); it is strong (*entgalt, entgolten*) | both datasets | wiktionary edit |
| B3 | *reinwaschen* conjugated weak (*waschte rein*); base *waschen* is strong (*wusch rein*) | both | wiktionary edit |
| B4 | *wegbegeben* conjugated weak (*wegbegebt*); it conjugates like *geben* (*begibst weg*) | both | wiktionary edit |
| B5 | *kahlscheren* derived from a verb "schieren" (*kahlschiert*); it builds on *scheren* (*schert kahl, kahlgeschoren*) | both | wiktionary edit |
| B6 | *gegenchecken* imperative split (*check gegen*) while its own finite forms are fused | kaikki | wiktionary edit |
| B7 | The *saugen* family is internally inconsistent: *absaugen* weak forms only, *einsaugen* strong forms only; both verbs allow both | kaikki | wiktionary edit |
| B8 | Junk lemma *herrklären* with a full generated paradigm | both | wiktionary edit (deletion request) |
| B9 | Corrupt generated form *schlägeet* for anschlagen Konjunktiv II | kaikki | wiktextract issue (template expansion) |
| B10 | *relaxen* past participle listed as English "relaxed" | kaikki | wiktionary edit |

## C. Legitimate variation, incompletely recorded

The single largest slice of the disagreement corpus is not error but
**asymmetric variant coverage**: one dataset lists one standard form, the
other lists the other (or both). Examples: *stünde/stände*,
*sandte/sendete*, fused vs split finite forms of dual-prefix verbs
(497 slots where UniMorph has only the fused form and kaikki only the
split one, 72 the reverse), reflexive marking (*sputend* vs
*sich sputend*), and the *-e* imperative doublets (*fang!/fange!*).

Disposition: *variant contribution* upstream where one side is simply
missing a standard form, and on our side the roadmap item of returning
variant sets from the API instead of a single canonical form. No schema
change is required anywhere: both datasets already permit multiple rows
per (lemma, feature) slot.

## D. Out-of-scope entries, handled by policy

| # | Finding | Disposition |
|---|---|---|
| D1 | Pre-1902 orthography lemmas (*beyssen, rathen, heyßen, thun-family, -iren verbs*) | our policy: lexicon rows for the common ones, no upstream action; they are valid historical entries |
| D2 | Swiss spellings (*beissen, heissen, schliessen* with ss) | our policy: supported via lexicon rows |
| D3 | Multiword proverb lemmas (*Mücken seihen und Kamele verschlucken*) | our policy: excluded from scoring |

## Status

Drafted, not yet filed. The prepared upstream texts live with the project;
filing A1 to A7 as one or several unimorph/deu issues and B1 to B10 as
Wiktionary edits is tracked as a launch-adjacent task.
