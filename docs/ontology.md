# The ontology of German verb conjugation

This document is the design spec of `ablaut`. It maps the entire domain of German
verb inflection into four layers. Every architectural decision in the library
follows from a boundary drawn here.

The terminology follows the [UniMorph schema](https://unimorph.github.io/)
(the lingua franca of computational morphology) for features, and traditional
Germanic philology (Grimm 1819; Duden *Die Grammatik*, Band 4; Eisenberg,
*Grundriss der deutschen Grammatik*) for verb classes.

## Layer A — The lexicon: what must be *known* about a verb

These properties are lexical facts. They cannot be derived from the
infinitive's surface form and must be stored:

| Property | Values | Example |
|---|---|---|
| Inflection class | weak · strong · mixed · suppletive | *kaufen* · *singen* · *denken* · *sein* |
| Ablaut series (strong only) | classes I–VII | *singen–sang–gesungen* → class III |
| Perfect auxiliary | *haben* · *sein* (some verbs allow both) | *laufen* → *sein* |
| Prefix behavior | none · separable · inseparable · dual | *aufstehen* · *verstehen* · *übersetzen* |

Key facts about the size of this lexicon:

- **Weak verbs** (the open, productive class — every new verb like *googeln* or
  *downloaden* is weak) need *no* lexical entry beyond defaults.
- **Strong verbs** number roughly 200 base lemmas. Their vowel gradation
  (*Ablaut*) falls into seven historical classes formalized by Jacob Grimm
  (1819) that still describe modern German. This is the entire "irregularity"
  of the language, and it fits in a few kilobytes.
- **Mixed verbs** (~9: *denken, brennen, kennen, nennen, rennen, senden,
  wenden, bringen, wissen*) take weak endings on a changed stem
  (*denken → dachte → gedacht*).
- **Suppletive/irregular**: *sein, haben, werden, tun* and the modals
  (*dürfen, können, mögen, müssen, sollen, wollen*).
- **Prefixed verbs are derived, not stored.** *aufstehen* conjugates exactly
  like *stehen*; only the prefix's behavior (separable/inseparable) is lexical.
  Dual-behavior prefixes (*durch-, über-, um-, unter-, wieder-*) need a
  per-lexeme flag because the same surface verb can be both
  (*übersetzen* "to translate" [inseparable] vs. "to ferry across" [separable]).

## Layer B — The feature space: what a "form" is

A conjugated form is a point in a feature grid:

- **Person**: 1 · 2 · 3
- **Number**: singular · plural
- **Tense** (synthetic): present (*Präsens*) · preterite (*Präteritum*)
- **Mood**: indicative · Konjunktiv I · Konjunktiv II · imperative
- **Voice**: active · processual passive (*werden*) · statal passive (*sein*)
- **Non-finite forms**: infinitive · *zu*-infinitive · present participle
  (Partizip I) · past participle (Partizip II)

Not every combination exists: the imperative has only 2sg and 2pl (plus the
formal *Sie* forms, which are borrowed Konjunktiv I). The library encodes the
feature grid as Rust types so that **impossible combinations are
unrepresentable** — there is no runtime "invalid feature bundle" error because
such a request cannot be constructed.

## Layer C — Synthetic vs. analytic forms

This is the central architectural insight. Only a small core of German is
**synthetic** (expressed in one word):

- Präsens, Präteritum (all persons/numbers, indicative + Konjunktiv I/II)
- Imperative
- Partizip I, Partizip II
- Infinitive

Everything else is **analytic** (periphrastic) — built compositionally from an
auxiliary, itself conjugated by the same synthetic core, plus a participle or
infinitive:

| Analytic form | Recipe |
|---|---|
| Perfekt | *haben/sein* (present) + Partizip II |
| Plusquamperfekt | *haben/sein* (preterite) + Partizip II |
| Futur I | *werden* (present) + infinitive |
| Futur II | *werden* (present) + Partizip II + *haben/sein* |
| *würde*-form | *werden* (Konjunktiv II) + infinitive |
| Processual passive | *werden* + Partizip II (any tense) |
| Statal passive | *sein* + Partizip II (any tense) |

Consequently the engine has two modules of very different sizes: a
**morphological core** producing ~30 synthetic forms per verb, and a thin
**compositional layer** deriving the other ~120 forms by combination. The
correctness burden lives almost entirely in the core.

## Layer D — Morphophonology and orthography

Surface adjustment rules applied after stem + ending concatenation. The full
inventory for standard German:

1. ***e*-epenthesis**: insert *e* before *-st/-t/-te…* when the stem ends in
   *d/t* (*arbeit-* → *du arbeitest*), or in *m/n* preceded by an obstruent
   (*atm-* → *du atmest*, *rechn-* → *du rechnest*; but *lern-* → *du lernst*,
   *wohn-* → *du wohnst*).
2. ***s*-coalescence**: the 2sg present *-st* loses its *s* after stems ending
   in *s/ss/ß/x/z* (*du heißt*, *du sitzt*, *du tanzt*).
3. ***-eln/-ern* verbs**: infinitive ends in *-n*, not *-en*; the stem's schwa
   may elide in 1sg present (*ich sammle*, *ich wand(e)re*).
4. **Umlaut in Konjunktiv II** of strong verbs (*kam → käme*).
5. ***e/i* stem alternation** in 2/3sg present and 2sg imperative of certain
   strong verbs (*sprechen → du sprichst, sprich!*). Verbs with this
   alternation take no imperative *-e* (*sprich!*, never \**spreche!*).

And two rules governing the past participle's *ge-* prefix:

6. No *ge-* for **inseparably prefixed** verbs (*verstanden*) and for
   ***-ieren* verbs** (*studiert*).
7. Separable prefixes **infix** *ge-* and *zu* (*aufgestanden*, *aufzustehen*).

## Scope boundary: morphology, not syntax

Separable prefixes split in verb-second clauses (*ich stehe **auf***). Where
the finite verb and its prefix land is **syntax**, not morphology. `ablaut`
therefore outputs *word forms with placement structure* (e.g.
`["stehe", "auf"]`), never sentences. Sentence realization is out of scope by
design.

## Why linguists should care

The architecture — a productive default rule plus a finite list of stored
exceptions — is precisely the **dual-mechanism model** of inflection
(Clahsen; Marcus, Pinker et al., *"German Inflection: The Exception That
Proves the Rule"*, Cognitive Psychology 1995), for which German weak vs.
strong morphology is the canonical battleground. `ablaut` is an executable
formalization of that model, validated exhaustively against
UniMorph/Wiktionary gold data, with every oracle disagreement adjudicated
against Duden and published.
