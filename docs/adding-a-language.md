# Adding a language

Adding a language is a **two-repo** operation: the engine lives in this
crate, but a language only reaches users when its UI entry also lands in
`ablaut-web`. The second half is easy to forget — Eastern Armenian
shipped in the engine (0.8.x) but was invisible on the site for a
release because its web entry was skipped. This checklist exists so that
can't happen again.

## 1. Engine (this crate)

- [ ] `src/<lang>.rs` — the rule engine (a productive default rule + a
      finite exception table; see [`design.md`](design.md)).
- [ ] `data/<lang>/verbs.tsv` — mined exceptions. `data/*/*` is
      gitignored, so `git add -f`.
- [ ] `src/bin/golden_<lang>.rs` — scores the engine on the slots where
      the two oracles agree.
- [ ] `scripts/<lang>/` — fetch scripts for both oracles + adapters to
      the common `lemma\tform\tfeatures` TSV. Make `.sh` files
      executable (`git update-index --chmod=+x`) — CI runs them as `./`.
- [ ] `docs/<lang>/oracles.md`, `disagreements.tsv`, `adjudications.tsv`.
- [ ] **Register** the language in every shared file: `src/lib.rs`
      (`Lang` enum), `src/reverse.rs` (grow **both** `INDEXES` and `SETS`
      and add the `ord()` + enumerate arms — ordinals must stay unique),
      `src/python.rs`, `src/wasm.rs`, `Cargo.toml` (`[[bin]]` include),
      `.github/workflows/ci.yml` (fetch/cache + the agreement gate; add
      `foma` if the second oracle is an FST), and `scripts/correctness.py`
      (`NAMES`).
- [ ] Do **not** hardcode a language count anywhere. The README headline
      is count-free ("20+ languages") on purpose — exact counts collide
      on every parallel language PR. The per-language list/table stay;
      the table is generated, don't hand-maintain a number.
- [ ] `cargo build && cargo test && cargo clippy --all-targets -D warnings`
      and `cargo run --bin golden_<lang> -- … --check` all green.

## 2. Website (`ablaut-web`) — **do not skip**

- [ ] Rebuild the wasm: `wasm-pack build --target web -- --features wasm`,
      copy `pkg/` → `ablaut-web/public/pkg/`, and bump the cache-buster
      `v` in `app/HomeClient.tsx`.
- [ ] `app/langData.ts` — add to the `Lang` union; add `LANG_DATA`,
      `ISO1`, `LANG_SEARCH`, `FOOTER`, `UNKNOWN_VERB` entries. Group only
      tense keys the engine actually returns.
- [ ] `app/langConfig.ts` — flag, label, status.
- [ ] `app/seo.ts`, `app/verbs.ts`, `app/conjugateServer.ts` — the
      exhaustive `Record<Lang, …>` maps (tsc fails without them).
- [ ] `app/accuracy/data.ts` — regenerate from this crate's
      `docs/correctness.json` + `docs/correctness-history.json`.
- [ ] `npx tsc --noEmit && npm run build` green; the new `/<iso>` page
      renders a real table.

## 3. Release (libs)

- [ ] Bump `Cargo.toml` and the wasm package `package.json` versions.
- [ ] Regenerate `docs/correctness.{json,md}` + `correctness-history.json`.
- [ ] `cargo publish`; `npm publish` for `@v4nn4/ablaut` (needs the
      browser passkey in a real TTY); cut a GitHub Release to trigger the
      PyPI wheels workflow. `api.ablaut.dev` / MCP redeploy off the crate.
