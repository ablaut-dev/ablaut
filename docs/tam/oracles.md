# Tamil gold-data oracles

The oracle pair for the Tamil verification loop, chosen by the same
criterion as every other language: two machine-readable sources of
independent provenance, so their agreement is strong evidence and their
disagreements form the adjudication corpus. Tamil targets the
**literary / written** register (செந்தமிழ்), which is what both sources
describe; the spoken register (பேச்சுத்தமிழ்) is a separate variety and
out of scope.

## Why this pair

There is no UniMorph for Tamil, so the second oracle is **generated from
a finite-state transducer** rather than taken off the shelf.

1. **kaikki.org Tamil** (English Wiktionary via Wiktextract, CC BY-SA).
   `scripts/tam/fetch_kaikki.sh` → `data/tam/kaikki.tsv`: 1,394 verb
   lemmas, ~40 slots each, converted by `scripts/tam/kaikki_to_tsv.py`.
   This is the Wiktionary leg.

2. **ThamizhiMorph** (K. Sarveswaran, University of Jaffna; Apache-2.0):
   a foma finite-state morphological analyser–generator for literary
   Tamil built on a paradigm of 18 verb classes (Graul's classification),
   with no Wiktionary lineage. `scripts/tam/fetch_thamizhi.sh`
   (commit-pinned) clones the repo, **recompiles the class FSTs from the
   lexc + meta-morph sources** shipped in `foma/ThamizhiMorph-Verbs.zip`
   (not from the checked-in binaries), and drives them as a *generator*
   via `scripts/tam/thamizhi_gen.py` → `data/tam/thamizhi.tsv`.

   ThamizhiMorph ships as an *analyser*, so generation is the extra work.
   Each verb class has its own FST (`verb-c3`, `verb-c4`, `verb-c11`,
   `verb-c12`, `verb-c62`, and `verb-c-rest` for classes 1–2, 5–10,
   13–18). A class's paradigm is a fixed list of upper-side analysis
   strings — `<root>+verb+fin+sim+strong+past=த்+3sgm=ஆன்` — the same for
   every root in the class; the per-root surface (செய்தான்) is produced
   by the FST's rewrite rules. So for every kaikki lemma the generator
   prepends the lemma to each class's analysis strings and applies the
   FST downward (`flookup -i`); a lemma not in a class yields `+?` and is
   dropped. This generates the paradigm **independently** of kaikki (not
   by analysing kaikki's forms), so the agreement is genuine
   cross-validation rather than an echo. **Requires foma**
   (`brew install foma` / `apt-get install foma-bin`).

## The overlap and the score

Generating over the 1,394 kaikki lemmas, ThamizhiMorph covers **923** of
them (its lexicon is the intersection). Over those 923 shared lemmas the
two oracles share **32,809** slots and agree on **97%** of them raw; the
agreed gold is **31,829** (lemma, feature) slots. The engine matches
**100.00%** of them, at 100% lemma coverage and every one of the 37 slot
types covered (`cargo run --bin golden_tam`).

The **980** slots where the two oracles disagree are excluded from the
score (they are the disagreement corpus, `target/tam_disagreements_todo.tsv`).
They are not engine errors — the engine follows kaikki, the literary
reference — but genuine oracle splits, almost all of one of two kinds:
a **lexical strong/weak or class split** (படி is listed as both weak
படிந்த் and strong படித்த், அடி's future neuter as both அடியும் and
அடிக்கும்), and a handful of suppletive stems where ThamizhiMorph
over-generates (it derives *வாகிறேன்* for வா, whose present is வருகிறேன்).

## Feature schema

`lemma ⇥ form ⇥ features`, a small UniMorph-style schema shared verbatim
by both converters:

- the finite grid `V;{PST,PRS,FUT};{PNG}` for the ten PNG cells
  1SG, 1PL, 2SG, 2PL, 3SGM, 3SGF, 3SGH (honorific), 3SGN (neuter),
  3PLE (epicene) and 3PLN (neuter plural);
- the imperatives `V;IMP;{SG,PL}`;
- the infinitive `V;INF` (செய்ய), the adverbial (verbal) participle
  `V;CVB` (செய்து), the three relative (adjectival) participles
  `V;PTCP;{PST,PRS,FUT}` (செய்த, செய்கிற, செய்யும்) and the conditional
  `V;COND` (செய்தால்).

Two cells are deliberately **out of the shared set**. **`V;FUT;3PLN`**
is a convention split rather than an error — kaikki has a distinct
future neuter plural (செய்வன) while ThamizhiMorph reuses the -உம்
neuter singular — so it is emitted by neither converter. The **negatives
and the perfect / progressive / modal periphrases** are analytic
(auxiliary verbs and the negator இல்லை); they are out of the synthetic
core for the engine and for both oracles.

## Mechanical-slip corrections

As with the Armenian uniparser oracle, a few of ThamizhiMorph's
mechanical slips are corrected during generation (documented, and only
where kaikki and standard grammar concur), so the agreement reflects the
morphology rather than a spelling artifact of the raw FST output:

- **the -இன் past class** (ஓடு, அகற்று …): ThamizhiMorph keeps -ன்
  before the அ-initial neuter and adjectival endings (ஓடினது, ஓடின),
  where literary Tamil takes the -இ stem with a ய glide (ஓடியது, ஓடிய)
  and a bare stem for the neuter plural (ஓடின). Corrected off the
  `past=இன்` analysis tag;
- **the present adjectival participle**: ThamizhiMorph spells it with the
  கின்ற allomorph only (செய்கின்ற), kaikki with கிற (செய்கிற); both are
  standard and the finite present carries both, so both are emitted;
- **the plural imperative glide**: on a vowel-final root ThamizhiMorph
  glues -உங்கள் straight onto the vowel (அணிுங்கள்); the glide is
  restored (அணியுங்கள் / அண்ணாவுங்கள்);
- a general Unicode cleanup dropping a pulli left before a vowel sign.

## The engine

`src/tam.rs` stacks a person-number-gender ending onto a tense stem.
Tamil's conjugation class is lexical — the root does not tell you the
past marker (செய்த், வந்த், ஓடின், படித்த்) or whether a verb is strong
or weak — so, exactly as a German strong verb stores its ablaut
principal parts, the stems are stored in `data/tam/verbs.tsv` (923 rows,
mined by `scripts/tam/mine_verbs.py` from the oracle agreement) and the
engine supplies the productive part: the shared PNG ending set, the
கிற்/கின்ற present allomorphy, the -இன் class's -இ stem, and the Tamil
orthographic sandhi at the suffix boundary (a stem-final pulli drops and
the ending's initial vowel is written as its dependent sign: செய்த் +
ஏன் → செய்தேன்).

Two suppletive-stem verbs (ஆகு, நூல்) whose present neuter-plural kaikki
spells on the full root while the engine takes the short stem — both
standard forms — are ruled in [`adjudications.tsv`](adjudications.tsv).
