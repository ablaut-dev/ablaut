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

Current coverage — all synthetic forms (Präsens, Präteritum,
Konjunktiv I/II, imperative, both participles) for all five inflection
classes, plus the full compositional layer of analytic tenses (Perfekt,
Plusquamperfekt, Futur I/II, the *würde*-form as Futur I Konjunktiv II,
and both passives) with per-lexeme perfect auxiliaries:

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

### Correctness, measured

The golden harness diffs every generated form against **two independent
oracles**: [UniMorph deu](https://github.com/unimorph/deu)
(`scripts/fetch_unimorph.sh`; 6.6k lemmas, 194k forms) and the
[kaikki.org Wiktextract extraction](https://kaikki.org/dictionary/German/)
(`scripts/fetch_kaikki.sh`; 10.2k lemmas, **794k forms** — including the
perfect auxiliary and all analytic tenses, which the harness scores too).
Current baseline:

| oracle | overall | auxiliary | analytic tenses |
|---|---|---|---|
| UniMorph deu (194k synthetic forms) | 97.5% | — | — |
| kaikki.org (794k forms) | 96.5% | 99.2–99.99% | 94.5–97.3% |

`scripts/cross_oracle.py` cross-examines the oracles: they agree on 98.9%
of their 195k shared slots; the 2,137 disagreements are data-quality
findings in their own right. Triaging our mismatches against oracle
agreement separates real bug candidates (both oracles against us — down to
~1,500, dominated by archaic-orthography lemmas and rare dual-prefix
senses) from adjudication cases (the oracles split). One caveat both
oracles share: they are extracted from the same upstream (Wiktionary), so
"both agree" is strong but not fully independent evidence.

Every disagreement with the gold data is ruled on in
[`docs/adjudications.tsv`](docs/adjudications.tsv) with a reference —
including genuine UniMorph errors found so far (*nennte* given as *nannte*;
*wisse!* marked nonexistent; corrupt paradigms auto-excluded when gold's own
infinitive contradicts its lemma). The lexicon now holds ~180 strong/mixed lemmas
(mined from the gold data itself and curated), plus per-lexeme dual-prefix
rulings (*umarmen* inseparable, *untertauchen* separable) and whole-word
guards against false decompositions (*bereiten* ≠ be+reiten). The remaining
gap is a long tail: dual-prefix senses, denominal weak verbs shadowed by
strong bases, and archaic gold lemmas.

### WebAssembly

The whole engine — rules, exception lexicon, analytic layer — compiles to a
**106 KB** wasm binary (≈45 KB gzipped) and produces a full 66-form
conjugation table in ~12 µs:

```sh
wasm-pack build --target web -- --features wasm
```

```js
import init, { conjugation_table } from "./pkg/ablaut.js";
await init();
const t = conjugation_table("aufstehen");
t.present[0];        // "stehe auf"
t.perfect[0];        // "bin aufgestanden"
t.zuInfinitive;      // "aufzustehen"
t.imperative;        // ["steh auf", "steht auf"] (undefined for modals)
```

Next, in order:

- [ ] Demo web app (separate repo: ablaut-demo) consuming the wasm package
- [ ] Work the bug-candidate list toward zero; adjudicate the oracle splits
- [ ] Python (PyO3) bindings for the linguistics audience
- [ ] Lemmatization (form → lemma) by inverting the paradigm tables

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
