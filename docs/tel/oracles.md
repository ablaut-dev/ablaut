# Telugu gold-data oracles

Telugu (తెలుగు, `tel`) is a Dravidian language written in a clean
abugida. Its verbs are agglutinative and richly suffixing: a finite
form marks person, number and — in the third person — gender (the
*mahat*/*amahat* split of human-male versus everything else), plus
tense, on a stem that undergoes regular vowel alternation and syncope
at the suffix boundary.

## The oracle pair, and why it is really one

The plan was the usual pair: a Wiktionary extraction (kaikki.org) and a
second independent source (UniMorph tel). For Telugu the pair does not
hold up as a two-oracle *agreement* the way it does for the other
languages, and the reason is a hard data limitation, documented here
rather than papered over.

1. **UniMorph tel** (English Wiktionary lineage, CC BY-SA 3.0).
   `scripts/tel/fetch_unimorph.sh` (commit-pinned, sha256-checked) →
   `data/tel/unimorph.tsv`: **1,163 forms over 116 verb lemmas**, tagged
   with person, number, gender and one of three tenses — past (`PST`),
   present-durative (`PRS;DUR`) and future (`FUT`). This is the primary
   oracle and the one the engine is scored against.

2. **kaikki.org Telugu** (Wiktextract of English Wiktionary, CC BY-SA).
   `scripts/tel/fetch_kaikki.sh` → `data/tel/kaikki.tsv`. The Telugu
   verb section of Wiktionary has ~2,500 entries, but **only about 11 of
   them carry a filled conjugation table**; the rest give a romanization
   and nothing else. Worse, Wiktextract could not map the Telugu column
   headers, so almost every inflected cell is tagged
   `error-unrecognized-form` and loses its person/number/gender — only
   the tense survives. `scripts/tel/kaikki_to_tsv.py` recovers the
   agreement features from the (unambiguous) Telugu personal endings and
   emits the past and future forms, **142 forms over 11 lemmas**.

The intersection that would form an agreement gold is therefore tiny:
of kaikki's 11 tabulated lemmas, **exactly one** (అరుచు "shout") is also
in UniMorph's 116. There is no agreement surface to score on. kaikki is
kept as an *independent spot check* — run separately, like the treebank
check other languages use — not as an agreement partner. This is a
work-in-progress oracle situation: a broader Telugu Wiktextract, or a
morphological analyser such as an apertium-tel, would restore the
two-oracle footing as a strict addition against the same schema.

## Feature schema

`lemma ⇥ form ⇥ features`, UniMorph bundles. The paradigm the two
sources describe and the engine is scored on:

- the **past** `V;{1,2};{SG,PL};PST` and `V;3;{MASC,FEM};{SG,PL};PST`
  (అమ్మాను "I sold", అమ్మింది "she/it sold");
- the **present-durative** `…;PRS;DUR` (అమ్ముతున్నాను "I am selling");
- the **future** `…;FUT` (ఆడతాను "I will play").

The third person distinguishes `MASC` (human male, `-డు`/`-ారు`) from
`FEM` (feminine, neuter and honorific-neutral, `-ంది`/`-ారు`); the
first and second person do not mark gender. The second- and
third-person plurals collapse onto the human-plural ending `-రు`.

## The engine

`src/tel.rs` builds three tense stems off the citation root (which
always ends in `-ు`) and attaches one set of personal endings. Three
productive sub-classes shift the stems:

- the **nasal `-ను` class** geminates in the past (కను → కన్నాను, and a
  non-masculine 3rd singular in `-ది`: కన్నది; also తిను, విను, కొను,
  అందుకొను);
- the **`-యు` class** takes `-శ-` in the past (చేయు → చేశాను, non-masc
  3sg చేసింది) and `-స్తు-/-స్తా-` in the present and future;
- the **causative `-ించు` class** takes `-స్తు-/-స్తా-` in the present
  and future (అపహరించు → అపహరిస్తున్నాను, అపహరిస్తాను).

What is left lives in `data/tel/verbs.tsv` (18 rows): the vowel-syncope
pasts where the penultimate `-ు-` raises to `-ి-` (అరుచు → అరిచాను,
కడుగు → కడిగాను, అడుగు → అడిగాను), పోవు "go" (→ పోయ-/పోతా-), ఉండు "be"
(→ ఉన్న-), ఇచ్చు "give" (→ ఇస్తా-) and the geminating తిను/వెళ్లు.

## Score

**100.00%** of the 1,159 scored UniMorph forms, 116 of 116 lemmas
covered, every paradigm slot type covered.

**69 of those forms are adjudicated** in
[`adjudications.tsv`](adjudications.tsv): they are UniMorph entries
whose tense labels are demonstrably wrong (whole verbs with the past
and present-durative columns swapped — చంపు, పట్టు, తేలు, నూరు — or a
corrupted entry with noun senses in a verb slot — కట్టు), where the
engine's form is the correct Telugu form filed under the correct
(mislabelled) tense. Each is annotated with the specific defect. Net of
those, the engine matches **1,090 forms UniMorph gets right, and zero it
gets wrong**.

As an independent check, the engine is run against the kaikki spot-check
gold (`cargo run --bin golden_tel -- data/tel/kaikki.tsv`): it matches
**102 of 130** forms over the 11 tabulated verbs. The residue is kaikki
data noise (future columns holding past forms for తవ్వు; a shifted
vowel in పోలు) and a handful of verbs outside UniMorph whose stems the
engine does not yet store; none is a contradiction of a form UniMorph
also lists.

## Reproducing

```sh
./scripts/tel/fetch_unimorph.sh          # → data/tel/unimorph.tsv
./scripts/tel/fetch_kaikki.sh            # → data/tel/kaikki.tsv
cargo run --release --bin golden_tel -- data/tel/unimorph.tsv   # scored
cargo run --release --bin golden_tel -- data/tel/kaikki.tsv     # spot check
```
