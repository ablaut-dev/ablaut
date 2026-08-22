# Kannada gold-data oracles

Kannada (ಕನ್ನಡ, `kan`) is a Dravidian language written in a clean
abugida. Its verbs are agglutinative and richly suffixing: a finite
form marks person, number and — in the third person — a three-way
gender (masculine, feminine, neuter), plus tense, on a stem built off
the vowel-final root.

## The oracle pair

Unlike Telugu — whose Wiktextract could not map its conjugation-table
headers, leaving it effectively single-oracle — the Kannada extraction
is clean, so the two sources form a genuine two-oracle *agreement*.

1. **UniMorph kan** (English-Wiktionary lineage, CC BY-SA 3.0).
   `scripts/kan/fetch_unimorph.sh` (commit-pinned, sha256-checked) →
   `data/kan/unimorph.tsv`. Its **41 verb lemmas** carry a full literary
   paradigm: person, number, three-way third-person gender, and the
   past (`PST`), present (`PRS`) and future (`FUT`) tenses plus the
   imperative (`IMP`). UniMorph fills the negative / potential /
   emphatic and tense-less bare-person columns with a *pronoun*
   placeholder (ಅವನು "he", ನೀವು "you", …) rather than a verb form; the
   converter drops those cells, keeping only the tensed finite bundles
   and the imperative.

2. **kaikki.org Kannada** (Wiktextract of English Wiktionary, CC BY-SA).
   `scripts/kan/fetch_kaikki.sh` → `data/kan/kaikki.tsv`. About **165
   verbs** carry a full conjugation table with clean
   person/number/gender/tense tags — the same schema UniMorph uses. The
   converter keeps the finite past/present/future and imperative and
   drops two families of noise: the separate negative / contingent
   (dubitative) / participle / infinitive / cohortative / optative /
   conditional columns, and malformed extractions — template residue
   (`{{{204}}}`) and forms with two adjacent Kannada dependent vowel
   signs (e.g. ಆಗಮಿಸುುವೆ), which is not valid Kannada orthography (a
   Wiktionary template that failed to drop a stem-final vowel before a
   vowel-initial suffix).

**Overlap.** All **41** UniMorph lemmas are also tabulated by kaikki
(full lemma overlap), and the two agree on **411 person/tense slots** —
a real agreement surface, not the handful Telugu had. The engine is
scored on those 411.

## The agreement surface, and the disagreements

Intersecting the two oracles leaves **5 slots** where they cover the
same cell but disagree; the harness excludes these from the scored gold
and they are catalogued in [`disagreements.tsv`](disagreements.tsv):

- four ಆಯಿಸು "cause to happen" past slots where kaikki dropped the
  `-ಇದ-` past marker (malformed ಆಯಿಸೆ for UniMorph's correct ಆಯಿಸಿದೆ) —
  resolved **o1** (UniMorph right);
- ನಡೆ "walk" 2sg imperative, where the bare-root ನಡೆ (kaikki) and the
  euphonic ನಡೆಯು (UniMorph) are both standard — resolved **variant**.

## Feature schema

`lemma ⇥ form ⇥ features`, UniMorph bundles:

- the **past** `V;{1,2};{SG,PL};PST` and `V;3;{MASC,FEM,NEUT};{SG,PL};PST`
  (ಮಾಡಿದೆನು "I did", ಮಾಡಿತು "it did");
- the **present** `…;PRS` (ಮಾಡುತ್ತೇನೆ "I do");
- the **future** `…;FUT` (ಮಾಡುವೆನು "I will do");
- the **imperative** `V;2;{SG,PL};IMP` (ಮಾಡು, ಮಾಡಿರಿ).

The third person distinguishes masculine (human male, `-ನు`/`-ಾನೆ`),
feminine (human female, `-ಳು`/`-ಾಳೆ`) and neuter (everything else); the
plural human masculine and feminine share one ending, so `3;PL;MASC` and
`3;PL;FEM` coincide.

## The engine

`src/kan.rs` builds three tense stems off the citation root and attaches
one set of personal endings. For a `-ు` root (with `base` the root minus
`-ు`): past stem `base + -ಇದ-` (neuter 3sg `base + -ಇತು`), present stem
`base + -ುತ್ತ-`, future stem `base + -ುವ-`. A root ending in another
vowel (ನಡೆ, ಕುಡಿ, ಮೀ) inserts the glide `-ಯ-` before the present/future
markers and forms its past on `root + -ದ-` (neuter 3sg `root + -ಯಿತು`).

What deviates lives in `data/kan/verbs.tsv` (9 rows): the suppletive or
contracted past stems of ಆಗು → ಆದ- (neuter ಆಯಿತు), ಹೋಗು → ಹೋದ-
(ಹೋಯಿತు), ಕೊಡು → ಕೊಟ್ಟ-, ಕೊಲ್ಲు → ಕೊಂದ-, ಗೆಲ್ಲು → ಗೆದ್ದ-, ಹೊರಡು →
ಹೊರಟ್ಟ-, ಮೀ → ಮಿಂದ-.

## Score

**100.00%** of the 411 agreed forms, 41 of 41 lemmas covered, every
paradigm slot type covered, and all five oracle disagreements resolved.

## Reproducing

```sh
./scripts/kan/fetch_unimorph.sh          # → data/kan/unimorph.tsv
./scripts/kan/fetch_kaikki.sh            # → data/kan/kaikki.tsv
cargo run --release --bin golden_kan -- data/kan/unimorph.tsv data/kan/kaikki.tsv --check
```
