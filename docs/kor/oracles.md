# Korean gold-data oracles

The oracle pair for the Korean verification loop, chosen by the same
criterion as every other language: two machine-readable sources of
independent provenance that both list the same kind of object — a whole
conjugated word for a (lemma, slot) — so their agreement is strong
evidence and their disagreements form the adjudication corpus.

The hard part in Korean is *granularity*. Most Korean resources are
morpheme-segmented (a token 먹었어요 analysed as 먹 + 었 + 어요) or are
paradigm *generators* (KoParadigm, byunlp), which are rule engines like
ours and so cannot cross-check it. UniMorph ko is an English-Wiktionary
scrape and shares kaikki's lineage. The one genuinely independent source
of *whole-word* attested forms is the National Institute of Korean
Language's Basic Dictionary.

## Why this pair

1. **kaikki.org Korean** (en.wiktionary `ko-conj` templates via
   Wiktextract; CC BY-SA / GFDL). `scripts/kor/fetch_kaikki.sh` →
   `data/kor/kaikki.tsv`. 3,888 verb entries, 3,412 with a full
   conjugation table (88% expansion); each lists ~49 whole-word forms
   tagged by speech level and mood, plus the stem class.

2. **NIKL 한국어기초사전 / Korean Basic Dictionary** (국립국어원; CC-BY-SA
   2.0 KR, downloadable login-free from krdict.korean.go.kr; mirrored
   commit-pinned at `spellcheck-ko/korean-dict-nikl-krdict`).
   `scripts/kor/fetch_krdict.sh` → `data/kor/krdict.tsv`. Each 동사 entry
   carries a few curated **활용 (conjugation) WordForms** — whole,
   editorially maintained surface words, wholly independent of
   Wiktionary — the diagnostic forms that reveal a verb's irregular
   class.

## Feature schema

The four whole-word forms both oracles attest as single words, so they
can be intersected:

| feature       | ending        | 먹다   | 하다   | 살다   |
| ------------- | ------------- | ------ | ------ | ------ |
| `V;INTIMATE`  | `-아/어` (해체) | 먹어   | 해     | 살아   |
| `V;CONN;NI`   | `-(으)니`      | 먹으니 | 하니   | 사니   |
| `V;DET;PRS`   | `-는` (관형)    | 먹는   | 하는   | 사는   |
| `V;FORM;PRS`  | `-(스)ㅂ니다`   | 먹습니다 | 합니다 | 삽니다 |

`V;INTIMATE` needs one normalisation. The Basic Dictionary spells the
`-아/어` form uncontracted (꾸미어, 하여); kaikki contracts it (꾸며, 해).
`scripts/kor/krdict_to_tsv.py` emits both the raw form and its regular
contraction as variants of that slot, so the two oracles overlap on it
(the same variant-set trick as the Catalan diaeresis). kaikki's honorific
`-시-` doublets (하셔, 하시니) ride along as extra variants; agreement is by
overlap, so they never hurt.

## Two-oracle agreement

On the alignable slots the two oracles agree on **3,576 of 3,593** shared
(lemma, slot) pairs across **1,401 shared verbs — 99.53%**. The 17
disagreements are the adjudication corpus and are excluded from gold:
Basic-Dictionary mis-segmentations of a few compounds (찢어지다 → 찌저지어
rather than 찢어져), a couple of kaikki class errors (짝짓다 conjugated as
regular rather than ㅅ-irregular), and the 이르다 homograph, where the two
dictionaries picked different lemmas (kaikki the 르-irregular 일러 "to
say", krdict the 러-irregular 이르러 "to arrive").

## The engine

`src/kor.rs` composes forms at the hangul jamo level. Given the stem's
final jamo it applies the euphonic rules — vowel harmony, 으-insertion,
아/어 contraction (오 + 아 → 와, 되 + 어 → 돼), ㅡ-deletion (쓰 → 써),
ㄹ-drop before ㄴ/ㅂ (살 → 사니, 삽니다) and the predictable 하 → 여 (해)
rule — all of which are regular and stored for no verb. The eight
irregular classes that deviate before a vowel ending — ㅂ (도와), ㄷ (들어),
ㅅ (지어), 르 (몰라), 러 (이르러), 우 (퍼), ㅎ, and the small 그러다 → 그래
contraction — live in `data/kor/verbs.tsv` as `lemma⇥class`, mined by
`scripts/kor/mine_verbs.py`: for each agreed verb it picks the class
whose generated forms match the agreed gold, storing only the
non-regular ones (66 verbs). The class is thus attested, not guessed.

## Score

100.00% of the 3,576 agreed slots, 100.00% lemma coverage.
