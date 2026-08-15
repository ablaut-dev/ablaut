# ablaut

A fast, correct German verb conjugator.

The name is the German term for the vowel gradation in *singen, sang,
gesungen*: Jacob Grimm formalized its seven classes in 1819, and they still
describe every strong verb in the language.

## Highlights

- **Measured correctness.** Every generated form is validated against two
  machine-readable gold standards: 99.2% agreement with
  [UniMorph deu](https://github.com/unimorph/deu) (194k forms) and 98.0%
  with the [Wiktextract extraction](https://kaikki.org/dictionary/German/)
  (794k forms, including analytic tenses and the perfect auxiliary). Where
  the two sources agree with each other, ablaut has **zero known errors**;
  every remaining disagreement is ruled on in a published
  [adjudication log](docs/adjudications.tsv) with references to standard
  grammars.
- **Generalizes to unseen verbs.** A rule engine with a curated exception
  lexicon, not a lookup table: novel verbs (*googeln*, *downloaden*)
  conjugate correctly. On verbs outside its lexicon, agreement with UniMorph
  is 99.55%.
- **Fast and small.** No I/O, no runtime data files, no dependencies in the
  core crate. A complete conjugation table (100+ forms) takes microseconds;
  the entire engine compiles to a 106 KB WebAssembly binary (about 45 KB
  gzipped).
- **Permissively licensed.** MIT OR Apache-2.0, including the lexicon: no
  ShareAlike obligations.

## Usage

### Rust

```sh
cargo add ablaut
```

```rust
use ablaut::{AnalyticTense, Mood, Number, Person, Tense, Verb};

let v = Verb::from_infinitive("aufstehen")?;

v.conjugate(Tense::Present, Mood::Indicative, Person::First, Number::Singular);
// "stehe auf"
v.conjugate(Tense::Preterite, Mood::Indicative, Person::Third, Number::Singular);
// "stand auf"
v.analytic(AnalyticTense::Perfect, Mood::Indicative, Person::First, Number::Singular);
// "bin aufgestanden"
v.past_participle();          // "aufgestanden"
v.zu_infinitive();            // "aufzustehen"
v.imperative(Number::Singular); // Some("steh auf"); None for modals
```

The full paradigm as one struct: `ablaut::table::Table::build(&v)`.

### Python

```sh
pip install ablaut
```

```python
import ablaut

t = ablaut.conjugation_table("aufstehen")
t["present"][0]     # "stehe auf"
t["perfect"][0]     # "bin aufgestanden"
t["auxiliary"]      # "sein"
t["imperative"]     # [None, None] for modals
```

Wheels are abi3 (Python 3.9+); no Rust toolchain needed.

### WebAssembly

```sh
wasm-pack build --target web -- --features wasm
```

```js
import init, { conjugation_table } from "./pkg/ablaut.js";
await init();
const t = conjugation_table("aufstehen");
t.present[0];       // "stehe auf"
t.zuInfinitive;     // "aufzustehen"
```

## Coverage

All synthetic and analytic forms of standard German:

- **Tenses and moods**: Präsens, Präteritum, Perfekt, Plusquamperfekt,
  Futur I/II, Konjunktiv I and II in all tenses (the *würde*-form is
  Futur I Konjunktiv II), processual and statal passive
- **Non-finite and imperative**: both participles, *zu*-infinitive,
  perfect infinitive, imperatives including the adhortative
  (*stehen wir auf!*) and polite (*stehen Sie auf!*) forms
- **All five inflection classes**: weak (the productive default), strong
  (ablaut, with 2/3sg stem alternation: *sprichst*, *hältst*), mixed
  (*dachte*), preterite-present (the modals and *wissen*: *ich kann*, no
  imperative), and suppletive (*sein*, *werden*, *tun*)
- **The prefix system**: separable (*stehe auf*, *aufgestanden*,
  *aufzustehen*), inseparable (*verstanden*), fused compounds that never
  split (*lobsingen*: *lobsang*, *lobgesungen*), multiword lemmas
  (*Rad fahren*: *fahre Rad*, *Rad gefahren*), and per-lexeme rulings for
  dual-behavior prefixes (*umarmen* vs *untertauchen*)
- **Orthographic surface rules**: *e*-epenthesis (*arbeitest*,
  *bewillkommnest*), *s*-coalescence (*du lässt*), schwa elision
  (*ich sammle*), the Latinate *-ieren* participle (*studiert* but
  *geschmiert*), and per-lexeme perfect auxiliaries (*aufstehen*: *sein*)

The exception lexicon ([`data/verbs.tsv`](data/verbs.tsv), about 900
human-readable rulings) is compiled into the binary.

## How correctness is verified

`src/bin/golden.rs` diffs every generated form against the gold data
(`scripts/fetch_unimorph.sh`, `scripts/fetch_kaikki.sh`) and CI fails if
accuracy regresses below pinned thresholds. `scripts/cross_oracle.py`
cross-examines the two oracles against each other: they agree on 98.9% of
their 195k shared slots, and triaging ablaut's mismatches against that
agreement separates genuine bugs from gold-data errors.

The method has found real errors in the gold data itself, documented in
[`docs/adjudications.tsv`](docs/adjudications.tsv): UniMorph gives the
Konjunktiv II of *nennen* as *nannte* (correct: *nennte*) and marks
*wisse!* as nonexistent; Wiktionary assigns *wachen* the auxiliary *sein*
and conjugates *entgelten* weak. One caveat: both oracles derive from
Wiktionary, so their agreement is strong but not fully independent
evidence.

## Design

The architecture is a productive default rule plus a finite table of stored
exceptions, which is precisely the dual-mechanism model of inflection
(Marcus, Pinker et al., 1995) for which German morphology is the canonical
test case. Features follow the [UniMorph schema](https://unimorph.github.io/).
[`docs/ontology.md`](docs/ontology.md) maps the whole domain in four layers
and doubles as the design spec.

## License

MIT OR Apache-2.0.

**Data provenance**: the exception lexicon is independently curated factual
data about standard German (principal parts, prefix behavior, auxiliaries),
hand-verified row by row. UniMorph (CC BY-SA 3.0) and Wiktextract
(CC BY-SA) are used as test oracles only: fetched at development time,
never shipped.
