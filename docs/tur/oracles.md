# Turkish gold-data oracles

The oracle pair for the Turkish verification loop, chosen by the same
criterion as every other language: two machine-readable sources of
independent provenance, so their agreement is strong evidence and their
disagreements form the adjudication corpus.

## The pair

1. **UniMorph tur** (CC BY-SA). `scripts/tur/fetch_unimorph.sh`
   (commit-pinned, sha256-checked) → `data/tur/unimorph.tsv`. UniMorph
   Turkish is generated from **TRmorph**, Çağrı Çöltekin's finite-state
   morphological analyzer for Turkish, and is native-verified — it has no
   Wiktionary lineage, so pairing it with kaikki is not circular. It
   enumerates a very large paradigm: voice × TAM × person × number ×
   polarity × interrogative × copular stacking runs to ~413k verb forms
   over **588 lemmas**.
2. **kaikki.org Turkish** (Wiktextract, CC BY-SA).
   `scripts/tur/fetch_kaikki.sh` → `data/tur/kaikki.tsv`, from the
   `{{tr-conj}}` conjugation tables of English Wiktionary. 1,860 verb
   lemmas carry a conjugation table.

The two share **513 lemmas** (every lemma UniMorph has that kaikki also
conjugates; UniMorph's 588 is the ceiling). On those, the engine is
scored against the slots the two **agree** on.

### The kaikki caveat, and why the method absorbs it

wiktextract mis-handles the Turkish tables in two ways. First, the
**person/number tags are scrambled** — `giderim` (aorist 1sg) comes out
tagged `singular;third-person`, a fixed rotation that makes the tags
useless for person. Second, the tables are the big *combined* layout, so
the *-ebil-* potential and the negative rows are interleaved with the
simple paradigm under the same tense tag.

`scripts/tur/kaikki_to_tsv.py` handles this by reading the **tense** from
the tags (which are reliable) and the **person** from the cell's
**position** within its tense block: Wiktionary lays each tense out as six
cells in the canonical order 1sg…3pl, and the first six cells seen for a
(tense, polarity) are the simple paradigm; later repeats (the potential
stacks) are dropped. A handful of cells still land on the wrong slot, but
that is harmless: the harness scores only where the two oracles *agree*,
so a mis-slotted kaikki cell drops out of the gold instead of corrupting
it.

## The bound: the single-word synthetic paradigm

The full UniMorph paradigm is not a sensible scoring target — most of it
is periphrastic (`gelecek olacaktı`) or the interrogative particle
(`gelir mi`), both of which are syntax. Both converters therefore keep
only the **single-word** forms (no spaces) in the **declarative**
(`;DECL`, non-interrogative) mood, which is exactly the synthetic verb.
The scored cell set is:

- six **base TAM categories** — aorist `V;IND;PRS;HAB` (gelir), present
  progressive `V;IND;PRS;PROG` (geliyor), future `V;IND;FUT` (gelecek),
  definite past `V;IND;PST` (geldi), evidential `V;INFR;PST` (gelmiş),
  necessitative `V;OBLIG;PRS` (gelmeli);
- the seven **single-word copular stacks** that carry the past or
  evidential copula on top of a base tense — `V;IND;PST;HAB` (gelirdi),
  `V;INFR;PRS;HAB` (gelirmiş), `V;IND;PST;PROG` (geliyordu),
  `V;INFR;PRS;PROG` (geliyormuş), `V;IND;PST;PROSP` (gelecekti),
  `V;INFR;FUT` (gelecekmiş), `V;INFR;PST;PFV` (gelmişti);
- each of the above across person (1/2/3) × number (sg/pl) ×
  polarity (POS/NEG);
- the **imperative** `V;IMP;2;{SG,PL}` and its formal plural
  (`geliniz`), and the **infinitive** `V;NFIN`.

## The engine

`src/tur.rs` builds every form productively from the stem: a
tense/aspect suffix and a personal ending, with **four-way and two-way
vowel harmony**, a **buffer `y`** between vowels (gelmeli → gelmeliyim),
**k→ğ softening** in the future ending (gelecek → geleceğim) and **d→t
assimilation** of the past copula after a voiceless consonant
(gelecek → gelecekti). The aorist's default is `-Ar` for monosyllables
and `-Ir` for polysyllables, with a `-r` after a vowel.

The exception table `data/tur/verbs.tsv` is small (21 rows), as befits a
highly regular language: the closed class of monosyllables whose aorist
is `-Ir` (al → alır, gel → gelir, az → azır), the verbs that voice a
final t→d before a vowel (git → gider, keşfet → keşfeder) and the
suppletive stems of demek/yemek (diyor, diyecek). It is mined from the
two-oracle agreement by `scripts/tur/mine_verbs.py`, restricted to the
lemmas the rules miss (`scripts/tur/capture_irregulars.sh`), and seeded
with `scripts/tur/manual.tsv` for the core irregulars (gitmek, etmek,
yemek) that the shared lemma set does not attest.

A cell is mined only where the two oracles intersect on a **single**
form, which is what lets the loop sidestep UniMorph's one systematic
generation slip: its necessitative does not apply back-harmony
(`almeli`, `yazmeli` for the correct `almalı`, `yazmalı`). kaikki spells
those correctly, so on back-vowel verbs the two disagree and the slot
leaves the gold; the engine produces the correct `-malı`.

## Score

**100.00%** of the **44,156** agreed forms, across **513** shared lemmas
and all five slot categories (infinitive, aorist, base tenses, copular
stacks, imperative), with 100% lemma coverage. 10,763 oracle-disagreement
slots — dominated by the UniMorph back-harmony necessitative bug — are
excluded from the gold, as the method prescribes.
