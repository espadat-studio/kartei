# kartei MVP plan

Keyboard-driven Rust TUI to read and edit vCard contacts in a local vdir.

## Goal

Smallest tool used daily instead of Thunderbird for contacts.

## Decisions

- D1 scope: browse, search, view, edit existing, create new. No delete (v1.1), no merge, no contact lists (CATEGORIES/KIND:group), no sync (vdirsyncer owns network).
- D2 write strategy: line-preserving edit (ADR-0001). Untouched lines emitted byte-for-byte; edits replace/append only touched lines. Crates ruled out (vCard 3.0 / Apple support mandatory). Full re-serialize model rejected (diff noise, weak lossless guarantee).
- D3 editable fields: N, FN, TEL, EMAIL, ORG, NOTE, BDAY, ADR. Everything else preserved; URL shown read-only. No raw/$EDITOR editing (users break syntax).
- D4 N/FN: FN linked to N while FN == derive(N) at load; else FN custom. Editing N updates linked FN only. UI shows linked/custom. New card: FN from N, else ORG.
- D5 labels: write fixed set via TYPE= only (home, work, cell, other). Read/display TYPE and X-ABLabel (decode _$!<Mobile>!$_). Editing grouped value keeps its group. Removing value deletes whole group. No itemN allocation in v1.
- D6 BDAY yearless write: keep Card's existing form; if none, VERSION 3.0 -> `X-APPLE-OMIT-YEAR=1604:1604-MM-DD`, 4.0 -> `--MMDD`. Read all forms.
- D7 new Card: VERSION 3.0, UID = UUIDv4 (`uuid` crate), filename `<UID>.vcf`, CRLF, fold 75 octets, no PRODID. Lines: BEGIN, VERSION, UID, N, FN, filled fields, END.
- D7b all writes atomic: temp file in same dir + rename.
- D8 conflicts: keep loaded bytes; at save re-read and compare full bytes. Mismatch/deleted -> refuse, keep edit, prompt [r] reload (discard) / [o] overwrite (or recreate) / [esc] keep editing. Reload-all key. No fs watch, no merge.
- D9 modes + keys (hardcoded):
  - Browse: j/k/arrows move, g/G top/bottom, / search, e/Enter edit, n new, y copy a value (numbered 1-9 prompt, #18), R reload all, ? help, q quit
  - Search: live filter, Enter keep, Esc clear
  - Edit form: Tab/S-Tab/Up/Down move, typing edits, Alt-a add value, Alt-d remove value, Alt-l cycle Label, Ctrl-s save, Esc cancel (confirm if dirty). NOTE: Enter = newline
  - Prompt: modal choice (conflicts, discard, errors)
- D10 layout: fixed 40/60 two-pane (list | detail/edit form) + bottom status bar (key hints; errors red). No responsive collapse.
  - Sort by N (family, given), fallback FN. List label "Family, Given" or FN if custom.
  - Search: case-insensitive substring over FN, ORG, EMAIL, TEL digits. No fuzzy.
- D11 config: none. Address book = positional arg, else `KARTEI_DIR`, else error. Single flat dir (user's vdirsyncer: Radicale, `collections = null`, `~/.local/share/contacts/`). Multi-book out of scope.
- D12 errors:
  - startup fatal (bad path/perms): stderr, exit 1, no TUI
  - bad Card at load (no BEGIN/END, invalid UTF-8): skip only that Card, even inside a Bundle (ADR-0002); status "N cards skipped"; `!` lists path + reason
  - save failure: red status + Prompt; form stays open
  - bad field value (e.g. BDAY): show raw, field read-only, never rewritten
  - panic hook restores terminal. No log file.
- D13 tests (runner: cargo nextest):
  - fixture corpus `tests/fixtures/*.vcf`, synthetic only (public repo): apple-grouped-labels, apple-omit-year, v4-yearless, company, folded-photo, lf-endings, utf8-fold-midchar, escaped-adr, fn-custom, bad-*; byte-identity + expected view
  - proptest (dev-dep): round-trip identity, edit locality (changed lines ⊆ field/Group), fold ≤75 octets + UTF-8 safe; default 256 cases
  - unit tests co-located: derive_fn, BDAY parse/write, Label decode, conflict check
  - TUI: ratatui TestBackend, key events -> state + few buffer asserts. No insta, no PTY e2e, no fuzz.
- D14 CI/release/tooling: mirror ~/code/{dublette,auberge,colporteur}; split scaffolding into several tickets.
  - workflows: master (check + test, no ffmpeg), semantic-pr lint-title verbatim, release (release-plz + crates.io OIDC trusted publishing + binaries draft->publish), build-binaries, docs (Astro/Starlight on Pages)
  - binaries: x86_64/aarch64 linux, aarch64 macOS. No Windows.
  - GH App: reuse org app stan-s-stanman (`GH_APP_ID`, `GH_APP_PRIVATE_KEY`)
  - crates.io: `kartei` 0.0.0 placeholder owned by sripwoud -> configure trusted publisher for release.yml
  - tooling: mise tools+tasks, hk, dprint, convco, renovate, zizmor hash-pin, setup script
- D15 clipboard: `y` writes OSC 52 to terminal; status "copied" optimistic. Fallback (shell out wl-copy/pbcopy) only if it fails in practice.
- D16 deps: runtime `ratatui` (crossterm backend), `uuid` (v4). Dev `proptest`. Hand-roll: text input (single-line + NOTE multi-line, via ratatui's unicode-width), base64 for OSC 52, arg parsing, one `Error` enum. No clap/serde/anyhow/tui-textarea.
- D17 structured forms: ADR = street, postal code, city, region, country (street multi-line). ORG = company, department. Hidden components (PO box, extended, deeper units) and X-ABADR preserved as loaded; new values write them empty.
- D18 layout: lib + bin. `card/` pure (bytes in/out), `vdir` owns fs, `app` pure state, `main`/`ui` only terminal code.
  - src/main.rs, lib.rs, card/{mod,line,bday,label}.rs, vdir.rs, app.rs, ui.rs, input.rs, osc52.rs
  - tests/fixtures/*.vcf, tests/roundtrip.rs, tests/app.rs

## Open questions

None.

## Scaffolding tickets (mirror ~/code/{dublette,auberge,colporteur})

- T1 (#2) `chore: scaffold cargo crate and local tooling`: lib+bin skeleton, edition 2024, AGPL, mise tools/tasks (build, test, check-*), dprint, convco, hk pre-commit, setup script. Check: `mise r check` + `cargo nextest run` green on empty crate.
- T2 (#3) `ci: add check, test and pr title workflows`: master.yml (changed-files, _test nextest, test shim, check), semantic-pr.yml lint-title verbatim, zizmor hash-pin. Check: PR runs all 3 green.
- T3 (#4) `ci: release with release-plz and binaries`: release.yml (stan-s-stanman app token, crates.io OIDC, draft -> binaries x86_64/aarch64 linux + aarch64 macOS -> publish, tag recovery), build-binaries.yml, release-plz.toml. Manual: configure trusted publisher for `kartei` on crates.io. Check: release PR opens on first feat.
- T4 (#5) `chore: add renovate config`: cargo deps `build(deps)`, npm `chore(deps)`, pin action digests, hk regex manager.
- T5 (#6) `docs: add starlight docs site`: Astro/Starlight + @espadat/docs-theme, docs.yml Pages deploy, CNAME. Check: docs.yml build green.

## Commits

Phase 1 card (pure)

1. `feat(card): parse and write content lines losslessly` - line.rs unfold/parse, raw bytes kept; card/mod.rs Card; fixtures folded-photo, lf-endings, utf8-fold-midchar. Tests: fixture byte identity; proptest `write(parse(x)) == x`.
2. `feat(card): fold and escape edited values` - line.rs fold 75 octets, escape/unescape. Tests: proptest fold <=75 + UTF-8 safe; escape round-trip units.
3. `feat(card): read name, contact and note fields` - view N, FN, TEL, EMAIL, ORG, NOTE, URL; label.rs decode TYPE + X-ABLabel. Fixtures apple-grouped-labels, company, fn-custom. Tests: expected views; label decode units.
4. `feat(card): parse birthday forms` - bday.rs read full, Apple omit-year, `--MMDD`; bad value -> raw read-only. Fixtures apple-omit-year, v4-yearless. Tests: units per form.
5. `feat(card): parse address components` - ADR view (D17). Fixture escaped-adr. Tests: components + escapes.
6. `feat(card): edit single-value fields with linked display name` - set N/FN/ORG/NOTE; derive_fn; linked/custom (D4). Tests: derive_fn units; proptest edit locality.
7. `feat(card): add and remove multi-value fields` - TEL/EMAIL/ADR add (TYPE fixed set, D5), edit keeps Group, remove deletes Group. Tests: group removal on apple fixture; edit locality.
8. `feat(card): write birthdays per card convention` - D6. Tests: 3.0 -> Apple form, 4.0 -> `--MMDD`, existing form kept.
9. `feat(card): create new cards` - uuid dep; D7 template. Tests: required lines present, round-trips, FN from N else ORG.

Phase 2 vdir
10. `feat(vdir): load address book and report skipped cards` - D12 skip reasons. Fixtures bad-*. Tests: temp dir under `std::env::temp_dir()`; counts + reasons.
11. `feat(vdir): save atomically with conflict check` - temp + rename; byte compare (D8). Tests: unchanged -> saved; changed/deleted -> Conflict.

Phase 3 TUI
12. `feat(app): browse, sort and search cards` - app.rs Browse/Search state (D9, D10). Tests: sort order, search over FN/ORG/EMAIL/TEL digits.
13. `feat(ui): render two-pane layout` - ratatui dep; ui.rs; main.rs arg/`KARTEI_DIR`, panic hook, loop. Tests: TestBackend buffer lines; missing path -> exit 1.
14. `feat(input): add text input` - input.rs single + multi-line, UTF-8 cursor. Tests: insert/delete/move units incl. wide chars.
15. `feat(app): edit and save cards` - Edit mode, Alt-a/d/l, Ctrl-s, Esc dirty confirm. Tests: TestBackend "e, type, Ctrl-s" writes expected bytes.
16. `feat(app): prompt on conflicts and errors` - Prompt mode, `!` skipped list, R reload. Tests: external change -> prompt; reload/overwrite paths.
17. `feat(app): create cards from browse` - `n`. Tests: new file `<UID>.vcf` appears in list.
18. `feat(app): copy focused value via osc 52` - osc52.rs base64 + sequence. Tests: base64 vectors, sequence bytes.
19. `docs: document keymap and usage` - README + docs pages.

Final

- PR review of branch as another engineer.
- Decide which recommendations to apply; apply; tests green.
- Remove unnecessary code comments.
