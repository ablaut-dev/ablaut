# Tagalog gold-data oracles

Tagalog is ablaut's first Austronesian language and the first to need
**stem-internal morphology** — an infix and reduplication, not just
suffixes. It is shipped as a *work in progress*: the engine capability is
landed and the paradigm is 100% correct on the slots where the two
oracles agree, but that agreement set is small and the oracles disagree
often, so the honest coverage is partial. This document says exactly
what is and is not verified.

## The paradigm

A Tagalog verb is cited by its bare **root** (`sulat` "write", `kain`
"eat") and inflects on two axes:

- **aspect** — perfective (completed), imperfective (ongoing/habitual),
  and the contemplated aspect (not yet begun);
- **voice / focus / trigger** — which argument is the grammatical
  subject. Wiktionary and UniMorph tabulate two: **actor** focus
  (`AGFOC`, the doer is subject) and **patient/object** focus (`PFOC`,
  the undergoer is subject).

That is a 3 × 2 grid of six inflected cells, plus the root itself
(`V;NFIN`). UniMorph writes the contemplated aspect as
`V;<trigger>;LGSPEC1`; the other cells are `V;{PFV,IPFV};{AGFOC,PFOC}`.

Tagalog has further voices (locative, benefactive, instrumental) and
derivational families (causative `magpa-`, social `maki-`, potentive
`maka-`); the two oracles only tabulate the actor/patient pair
consistently, so that is the grid ablaut fills. The locative `-an` and
benefactive `i-` forms are folded into `PFOC`, since UniMorph lumps them
there.

## The new engine capability: infixation + reduplication

Every previous ablaut engine builds a form by attaching material to the
ends of a stem. Tagalog cannot be done that way, and `src/tgl.rs` adds
the missing operations as reusable helpers:

- **Infixation.** The actor `-um-` and the patient perfective `-in-` are
  infixes placed *after the root's first consonant*: sulat →
  s·**um**·ulat, tanggap → t·**in**·anggap. Vowel-initial roots take the
  affix as a prefix (abuso → **um**abuso). `infix(word, af)` does this.
- **CV-reduplication.** The imperfective and contemplated aspects copy
  the root's first consonant-plus-vowel: sulat → **su**·sulat, kuha →
  **ku**·kuha (the first vowel alone if the root is vowel-initial: abuso
  → **a**·abuso). `reduplicate(root)` does this, and it composes with the
  infix: imperfective actor = `infix(reduplicate(root), "um")`
  (s·um·**u**·sulat).

On top of these sit the smaller morphophonemic rules the oracles attest:
`mang-` **nasal assimilation** (the prefix nasal agrees with the root
onset, which itself fuses when it is p/t/k/s: kuha → na**ng**·uha), **o→u
raising** before a suffix (abot → ab**u**tin), an optional linking
**-h-** before a vowel-final suffix (basa → basa**h**in), and the two
productive allomorph variants the oracles spell out: intervocalic **d→r**
in reduplicated stems (dating → duma**r**ating) and the **ni-** shape of
the consonant-initial patient perfective (lagnat → **ni**lagnat).

## The lexicon: voice class is stored, morphology is derived

Which affix family a root takes — actor `-um-` / `mag-` / `mang-` /
`ma-`, and patient `-in` / `-an` / `i-` — is **lexical**. It is not
predictable from the root's shape, exactly as German strong-verb class is
not, so it is stored one row per root in `data/tgl/verbs.tsv`
(`root ⇥ actor ⇥ patient ⇥ h ⇥` six optional form overrides), mined by
`scripts/tgl/mine_verbs.py`. The engine supplies only the productive
morphophonology above, given the class; a root absent from the table
falls back to the commonest pattern (`-um-` actor, `-in` patient), so
novel verbs still conjugate, just not always into the class a speaker
would pick. A handful of genuinely irregular stems (kasal →
magpa**kasal**, kita → ma**kita**, both belonging to families outside the
tabulated grid) are stored as full-form overrides.

`scripts/tgl/tgl_rules.py` is a Python twin of the Rust rules; the miner
and the engine derive forms the same way, and the Rust side is the
shipped source of truth.

## Why this pair

Two machine-readable sources of independent provenance, per the project
criterion:

1. **UniMorph tgl** (`unimorph/tgl`, English-Wiktionary lineage,
   CC BY-SA). `scripts/tgl/fetch_unimorph.sh` (checksum-pinned) →
   `data/tgl/unimorph.tsv`: 344 verb roots, keyed on the bare root with
   the aspect × trigger bundles above.
2. **kaikki.org Tagalog** (Wiktextract of English Wiktionary, CC BY-SA).
   `scripts/tgl/fetch_kaikki.sh` → `data/tgl/kaikki.tsv`. kaikki keys
   each entry on an already-focus-marked head (`sumulat`, `sulatin`), not
   on the root, so `scripts/tgl/kaikki_to_tsv.py` **re-keys** it onto
   UniMorph's schema: it reconstructs the root from the entry's
   `tl-infl-*` inflection-template segmentation, reads the trigger from
   the template family, and takes the three aspect forms from the clean
   `tl-verb` head-template (sidestepping the Baybayin/error noise in the
   raw form list). This yields 1,160 roots.

The two share **English Wiktionary** as an ultimate source, so their
agreement is weaker evidence than a fully independent pair would give —
but the extraction pipelines, keying and segmentation are entirely
different, and the point where they most often *disagree* (see below) is
exactly the lexical voice-class choice, which is where an error would
hide. A genuinely independent third source (e.g. a Tagalog treebank) is
the natural next addition.

## Score

Overlap: the two oracles share **138 roots** and **660 (root, feature)
slots**. They **agree** on **469** of those slots and **disagree** on
**191**. The engine matches **469 / 469 = 100.00%** of the agreed slots
(root 138/138, actor voice 239/239, patient voice 92/92), 100% lemma
coverage of the agreed set.

Two honest caveats:

- **The agreement set is small.** 469 slots is one to three orders of
  magnitude below the other languages here. It is limited by the oracle
  overlap, not by the engine.
- **The oracles disagree a lot (191 slots).** Almost all of it is the
  lexical voice choice: given a root, UniMorph may tabulate its `mag-`
  actor form while kaikki tabulates the `-um-` or `mang-` one, or an
  `-in` object form against an `-an` locative one. These are genuine
  competing derivations of the same root, not engine errors; they are
  the adjudication corpus (`target/tgl_disagreements_todo.tsv`) and are
  left unresolved for now.

Measured with the *productive rules alone* — deriving each root's forms
from its stored voice class but **without** the two full-form overrides —
the engine reaches **98.5%** of the agreed slots; the stored classes and
the two overrides carry it to 100%. The reusable infix/reduplication
capability, not the score, is the deliverable here.
