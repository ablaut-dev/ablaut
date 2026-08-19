# Japanese gold-data oracles

The oracle pair for the Japanese verification loop, chosen by the same
criterion as every other language: two machine-readable sources of
independent provenance, so their agreement is strong evidence and their
disagreements form the adjudication corpus.

## The granularity problem, and its solution

Japanese has no European-style finite paradigm; a "conjugated" word like
書きます (kakimasu) is the verb's 連用形 stem 書き plus an agglutinated
auxiliary ます. A non-Wiktionary morphological lexicon tokenises that as
two units (書き | ます) and never stores the whole word, so the only
surface strings **both** kinds of source attest are the bare **katsuyou-kei**
(活用形) — the inflectional stems themselves.

kaikki.org expands, per verb head, three top-level forms that are exactly
such bare stems:

| slot     | 活用形            | kaikki tag | example (書く) |
|----------|-------------------|------------|----------------|
| `V;NFIN` | 終止形 (terminal) | the headword | 書く |
| `V;CONT` | 連用形 (continuative / masu-stem) | `stem` | 書き |
| `V;PST`  | plain past (た-form) | `past` | 書いた |

These three are attested for **12,322** of kaikki's 13,868 Japanese verb
entries — a broad, modern base. (kaikki's richer per-verb *tables* are
either the classical/bungo katsuyou table, in pre-modern orthography —
会は vs modern 会わ — present for ~900 verbs, or the full modern
auxiliary table, present for only ~120; neither is a broad modern source,
so we take only the three top-level stems.)

## Why this pair

UniMorph ja is out for the usual reason: it is an English-Wiktionary
scrape, sharing kaikki's lineage, so its agreement would be circular.

1. **kaikki.org Japanese** (en.wiktionary `ja-verb` head template via
   Wiktextract; CC BY-SA / GFDL). `scripts/jpn/fetch_kaikki.sh` →
   `data/jpn/kaikki.tsv`: 37,380 rows, 13,362 lemmas.
2. **mecab-ipadic 2.7.0** (the IPADIC morphological dictionary; NAIST /
   Kyoto-corpus lineage — LGPL-style terms, not a Wiktionary derivative).
   Each row is a surface token tagged with its 活用型 (conjugation type,
   e.g. 五段・カ行イ音便) and 活用形 (form, e.g. 連用形). We read the
   終止形/基本形, the 連用形, and reconstruct the plain past from the
   連用タ接続 onbin stem plus the voiced/plain auxiliary.
   `scripts/jpn/fetch_ipadic.sh` → `data/jpn/ipadic.tsv`: 42,747 rows,
   14,569 lemmas.

## Agreement

On the **3,289** lemmas the two oracles share:

| slot     | shared slots | agree  |
|----------|--------------|--------|
| `V;NFIN` | 3,289        | 100.00% |
| `V;CONT` | 3,080        | 99.68% |
| `V;PST`  | 3,080        | 99.61% |

Comfortably past the ≥ 95%-on-≥ 1000-verbs feasibility bar. The ~22
disagreements are a clean adjudication corpus, all genuine:

- **special-ラ行 verbs** (なさる/くださる/いらっしゃる/ござる): kaikki gives
  the regular 連用形 なさり, IPADIC the euphonic なさい.
- **class-ambiguous verbs** that one oracle files godan and the other
  ichidan (湿気る, すぐる, …).
- the **する / 来る godan homographs** IPADIC lists under 摩る / 繰る.

They are excluded from gold, not scored.

## The engine

Japanese verb morphology is almost exceptionless *once the class is
known*, but the class is the one fact the dictionary form cannot reveal
(着る "wear" is ichidan → 着, 切る "cut" is godan → 切り). So the single
stored lexical fact per verb is its **inflection class**, mined from
IPADIC's 活用型 over the two-oracle agreement set:
`scripts/jpn/mine_verbs.py` → `data/jpn/verbs.tsv` (3,283 verbs). Classes:
`ichidan`, `godan` (row inferred from the final kana; standard onbin),
`godan_iku` (行く-family, past 行った), `godan_u` (問う-family, literary
問うた), `rspecial` (なさる), and the three irregulars `suru` / `zuru` /
`kuru`. Everything else — the row shifts (書か/書き/書く/書け) and the
past-tense onbin (書いた, 泳いだ, 死んだ, 待った) — is rule, in `src/jpn.rs`.

Because the whole lexicon is the class list rather than an
irregular-only patch, there is no `capture_irregulars.sh` step (the
Catalan pattern where the engine handles regulars and only exceptions are
stored does not apply: no Japanese -る verb's class is derivable).

## Score

`cargo run --release --bin golden_jpn`: **100.00%** of the 9,421 agreed
slots (terminal 3,283/3,283, continuative 3,070/3,070, past 3,068/3,068),
**99.82%** lemma coverage — the six unsupported lemmas are classical-only
classes (下二・/上二・/四段・) that mine_verbs deliberately skips.
