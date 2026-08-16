# Design notes

How ablaut models German conjugation. The model has four layers; the code
refers to them by these names. Feature terminology follows the
[UniMorph schema](https://unimorph.github.io/); the verb classes are the
traditional ones (Grimm 1819; Duden, *Die Grammatik*; Eisenberg, *Grundriss
der deutschen Grammatik*).

## Layer A: the lexicon

Some properties of a verb cannot be derived from the infinitive and must be
stored:

| Property | Values | Example |
|---|---|---|
| Inflection class | weak, strong, mixed, preterite-present, suppletive | *kaufen*, *singen*, *denken*, *können*, *sein* |
| Principal parts (non-weak) | preterite and Konjunktiv II stems, participle, stem alternations | *singen*: *sang*, *säng-*, *gesungen* |
| Perfect auxiliary | *haben*, *sein* (some verbs allow both) | *laufen*: *sein* |
| Prefix behavior | separable, inseparable, fused, per-lexeme rulings | *aufstehen*, *verstehen*, *lobsingen*, *übersetzen* |

The lexicon stays small because most of German is regular or derived:

- Weak verbs, the open productive class, need no entry at all. New verbs
  (*googeln*, *downloaden*) conjugate correctly without one.
- Strong verbs come to roughly 200 base lemmas. Their vowel gradation falls
  into the seven classes Grimm described, which still hold for modern German.
- Mixed verbs (about nine: *denken*, *brennen*, *kennen*, *nennen*, *rennen*,
  *senden*, *wenden*, *bringen*, plus verbs with a weak preterite and strong
  participle such as *hauen* and *mahlen*) take weak endings on a changed
  stem.
- The preterite-presents (the modals and *wissen*) inflect their present
  singular like a strong preterite (*ich kann*) and have no imperative.
- *sein*, *werden* and *tun* resist decomposition and are stored in full.
- Prefixed verbs are derived, not stored: *aufstehen* conjugates like
  *stehen*. Only the prefix behavior is lexical, and dual-behavior prefixes
  (*durch-, über-, um-, unter-, wieder-*) need per-lexeme rulings because
  the same surface form can go either way (*übersetzen* "translate" is
  inseparable, "ferry across" is separable). A few compounds never split at
  all but still take the *ge-* and *zu* infixes (*lobsingen*: *lobsang*,
  *lobgesungen*, *lobzusingen*); multiword lemmas (*Rad fahren*) keep their
  particle as a free word.

## Layer B: the feature space

A form is a point in a small grid: person (1/2/3), number (sg/pl), synthetic
tense (present/preterite), mood (indicative, Konjunktiv I, Konjunktiv II),
plus the non-finite forms (infinitive, *zu*-infinitive, both participles)
and the imperative, which exists only in the second person (the polite *Sie*
and adhortative *wir* forms borrow Konjunktiv I). The grid is encoded as
Rust enums, so a request for a form that does not exist, such as a
third-person imperative, does not compile.

## Layer C: synthetic and analytic forms

Only a small core of German is synthetic, expressed in one word: Präsens,
Präteritum, Konjunktiv I and II, the imperative, the participles and the
infinitive. Everything else is analytic, built from an auxiliary (conjugated
by the same synthetic core) plus a participle or infinitive:

| Analytic form | Recipe |
|---|---|
| Perfekt | *haben/sein* (present) + Partizip II |
| Plusquamperfekt | *haben/sein* (preterite) + Partizip II |
| Futur I | *werden* (present) + infinitive |
| Futur II | *werden* (present) + Partizip II + *haben/sein* |
| *würde*-form | Futur I with *werden* in Konjunktiv II |
| Processual passive | *werden* + Partizip II |
| Statal passive | *sein* + Partizip II |

The engine is therefore two parts of very different size: a morphological
core producing the synthetic forms, and a thin compositional layer deriving
the rest. Almost all of the correctness work is in the core.

## Layer D: orthographic surface rules

Adjustments applied when an ending attaches to a stem. The main ones:

1. *e*-epenthesis before *-st/-t/-te* endings: stems in *d/t*
   (*du arbeitest*), stems in *m/n* after an obstruent (*du atmest*,
   *du rechnest*, but *du lernst*, *du wohnst*), *-mn* clusters
   (*du bewillkommnest*) and consonant + *w* (*verwitwet*).
2. *s*-coalescence: the 2sg *-st* loses its *s* after *s/ss/ß/x/z*
   (*du heißt*, *du sitzt*). This is a present-tense rule; strong preterites
   take *-est* instead (*du saßest*).
3. Schwa stems (*-eln/-ern*): elision in the 1sg (*ich sammle*), full
   endings in Konjunktiv I (*du sammelest*).
4. Umlaut in the Konjunktiv II of strong verbs (*kam*, *käme*).
5. *e/i* stem alternation in the 2/3sg present and the imperative of some
   strong verbs (*du sprichst*, *sprich!*, never \**spreche!*).
6. The past participle takes no *ge-* after inseparable prefixes
   (*verstanden*) or in Latinate *-ieren* verbs (*studiert*). Native verbs
   that merely end in *-ieren* keep it (*geschmiert*): the two are told
   apart by whether a vowel precedes *-ier-*.
7. Separable prefixes infix *ge-* and *zu* (*aufgestanden*, *aufzustehen*).

This inventory grew out of testing against gold data; several rules
(the *-mn* cluster, the Latinate distinction, the *lädt*/*hält* asymmetry
in stem-changing 3sg forms) were found that way rather than planned.

## Input handling

`Verb::from_infinitive` normalizes common messy input before parsing:
surrounding and repeated whitespace is trimmed and collapsed, a leading
capital or stray casing is lowercased (German infinitives are entirely
lowercase), and a free *zu* particle is stripped (*zu gehen*,
*Rad zu fahren*). Multiword lemmas keep their noun's capital
(*Rad fahren*).

Reflexive lemmas (*sich freuen*) are recognized: the pronoun agrees with
the subject in finite and analytic forms (*freue mich*, *hast dich
gefreut*, *stellt sich vor*) and stays *sich* in citation forms
(*sich gefreut*, *sich zu freuen*). The accusative pronoun is always
used; the handful of dative-reflexive verbs (*sich etwas merken*: *ich
merke mir*) would need valence data to tell apart.

Three input kinds are deliberately *not* guessed, because doing so is
ambiguous: umlaut transcriptions typed without a German keyboard
(*ueben* for *üben*, *schoen* for *schön*), conjugated forms given
instead of the infinitive (*ging*, *gegangen*), and one-word
zu-infinitives with an infixed *zu* (*aufzustehen*). The first would need
a dictionary to disambiguate from legitimate letter sequences (*neu*,
*Steuer*); the second is lemmatization, a separate direction; the third
cannot be told apart from lemmas whose stem begins with *zu-*
(*aufzucken* is *auf|zucken*, not *auf-zu-cken*).

## Scope: morphology, not syntax

Separable prefixes split in verb-second clauses (*ich stehe auf*). Where
the finite verb and the particle land in a full sentence is syntax and out
of scope: ablaut returns the forms themselves (`"stehe auf"`), never
sentences.

## Background

A productive default rule plus a stored exception list is the shape that
the dual-mechanism model of inflection describes (Clahsen; Marcus, Pinker
et al., "German Inflection: The Exception That Proves the Rule", Cognitive
Psychology 1995), and German weak versus strong morphology is the case that
literature is largely about. How the correctness of the implementation is
measured, and the errors found in the gold data along the way, are covered
in the [README](../README.md) and [`adjudications-deu.tsv`](adjudications-deu.tsv).
