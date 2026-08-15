# ablaut

A fast, correct German verb conjugator in Rust.

Named after *Ablaut* — the systematic vowel gradation (*singen–sang–gesungen*)
whose seven classes, formalized by Jacob Grimm in 1819, still describe every
strong verb in modern German.

## Goals

1. **Correct** — validated exhaustively against multiple gold standards
   (UniMorph `deu`, Wiktextract/kaikki.org, DWDSMOR), with every oracle
   disagreement adjudicated against Duden and published. "100% correct" means
   100% agreement with an adjudicated gold standard, adjudication log included.
2. **Fast** — allocation-light rule engine plus a compiled-in exception
   lexicon of ~200 strong verbs; conjugation in nanoseconds, no I/O, no
   runtime data files.
3. **Legible to linguists, not just engineers** — the design is an executable
   formalization of the dual-mechanism model of inflection (productive rule +
   stored exceptions; Marcus, Pinker et al. 1995), with features expressed in
   the UniMorph schema.

Start with [`docs/ontology.md`](docs/ontology.md) — the four-layer map of the
domain that doubles as the design spec.

## Status

Early. Current coverage — all synthetic forms (Präsens, Präteritum,
Konjunktiv I/II, imperative, both participles) for all five inflection
classes:

- **weak** (the productive default) with the full orthographic rule set
  (*e*-epenthesis, *s*-coalescence, *-eln/-ern* schwa elision, *-ieren*
  participles)
- **strong** (ablaut) with 2/3sg stem alternation (*sprichst*, *fährt*,
  *hältst* without epenthesis)
- **mixed** (*dachte*, *sandte*)
- **preterite-present** — the *Präteritopräsentia*: modals + *wissen*
  (*ich kann*, no imperative)
- **suppletive** (*sein*, *werden*, *tun*), stored outright

The exception lexicon lives in [`data/verbs.tsv`](data/verbs.tsv)
(~70 seed verbs), human-readable and compiled into the binary.

Next, in order:

- [ ] Golden-test harness against UniMorph `deu` (6,810 verbs) and kaikki.org;
      grow the lexicon to the full ~200 strong lemmas from its diff
- [ ] Prefix system (separable / inseparable / dual)
- [ ] Compositional layer: analytic tenses (Perfekt, Futur, passives)
- [ ] WASM + Python (PyO3) bindings

## Example

```rust
use ablaut::{Verb, Tense, Mood, Person, Number};

let v = Verb::weak("sammeln")?;
assert_eq!(
    v.conjugate(Tense::Present, Mood::Indicative, Person::First, Number::Singular),
    "sammle"
);
assert_eq!(v.past_participle(), "gesammelt");
```

## License

MIT OR Apache-2.0
