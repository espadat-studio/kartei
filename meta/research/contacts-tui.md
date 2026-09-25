# Keyboard-driven contacts on the laptop against Baïkal CardDAV

2026-09-25. Goal: replace Thunderbird for reading and editing the vCards on the self-hosted Baïkal
(`ansible/roles/baikal`, pinned `0.12.1` in `ansible/playbooks/baikal.meta.yml`). Fast, keyboard-first,
TUI preferred. Two layers: a syncer (CardDAV to a local vdir) and a client (reads/edits the vdir).

## TL;DR

**Use vdirsyncer + khard now.** vdirsyncer 0.21.0 is already installed and speaks Digest. khard is the
only maintained vCard client that fits: CLI plus `$EDITOR` YAML editing, and it keeps fields it doesn't
know. Add an `fzf` picker if you want it to feel like a TUI. **Plan to move to pimsync** (vdirsyncer's
successor, much more active) once Arch ships ≥0.6, or once Baïkal switches to Basic auth. **No real
maintained TUI contact manager exists.** Every ratatui/bubbletea candidate is dead, vibe-coded, not
vCard-based, or depends on a private library. Main data-loss risk: khard silently rewrites `BDAY`
(1900 placeholder year, BDAY params dropped). A second risk is Baïkal's own vCard 4.0 to 3.0
conversion on read.

## Local state (read-only probe, 2026-09-25)

- Installed: `vdirsyncer 0.21.0`, `khal 0.14.1`, `neomutt 20260616`, `abook 0.6.2`, `thunderbird 155.0`, `fzf`, `nvim`.
- Not installed: `pimsync`, `khard`, `aerc`.
- Arch `extra` has `khard 0.21.0`, `aerc 0.22.0` and `pimsync 0.5.7`. pimsync was flagged out-of-date on 2026-05-09 ([archlinux.org JSON API](https://archlinux.org/packages/search/json/?name=pimsync)). The local sync DB still lists aerc 0.21.0, so it is stale.
- `~/.config/vdirsyncer/config` already has a `contacts` pair. It points at a retired Radicale server; the radicale role is gone from auberge's `ansible/roles`.
- `~/.local/share/contacts/` is empty. The `contacts.items` status DB has 0 rows, last written 2026-01-09. No stale-status deletion risk.
- All 4 existing storages already use `password.fetch = ["command", ...]`. The same pattern carries over.

## Server facts that constrain the choice

- **Baïkal is Digest-only.** `baikal.yaml.j2` sets `dav_auth_type: Digest`. With that setting Baïkal wires only `Sabre\DAV\Auth\Backend\PDO` (digest) ([Server.php](https://github.com/sabre-io/Baikal/blob/0.12.1/Core/Frameworks/Baikal/Core/Server.php#L133-L141)). Basic-only clients get 401.
- Switching to Basic needs no password reset. `PDOBasicAuth` checks `md5(user:realm:pass)` against the same `digesta1` column ([PDOBasicAuth.php](https://github.com/sabre-io/Baikal/blob/0.12.1/Core/Frameworks/Baikal/Core/PDOBasicAuth.php#L66-L78)).
- **No `/.well-known/carddav` redirect under Caddy.** Baïkal ships the `Redirect 308 /.well-known/carddav /dav.php` only in `.htaccess`, which is Apache-only ([html/.htaccess](https://github.com/sabre-io/Baikal/blob/0.12.1/html/.htaccess)). `Caddyfile.j2` has no equivalent, so point clients at `https://<baikal_domain>/dav.php/` explicitly.
- URL shape: base `dav.php/` ([html/dav.php](https://github.com/sabre-io/Baikal/blob/0.12.1/html/dav.php)). Baïkal creates each user's address book with uri `default` ([User.php](https://github.com/sabre-io/Baikal/blob/0.12.1/Core/Frameworks/Baikal/Model/User.php#L165-L175)). The collection is `https://<baikal_domain>/dav.php/addressbooks/<user>/default/`.
- Repo health: latest release 0.12.1 on 2026-08-05, last push 2026-08-13, 117 open issues+PRs (`gh api repos/sabre-io/Baikal`). Baïkal pins `sabre/dav ~4.7.1` ([composer.json](https://github.com/sabre-io/Baikal/blob/0.12.1/composer.json)).

### sabre/dav rewrites vCards (data-loss relevant)

- **On PUT:** runs `validate(PROFILE_CARDDAV | REPAIR)` unless the client sends `Prefer: handling=strict`. On a warning it **re-serializes the card** and withholds the ETag ([CardDAV/Plugin.php L308-L386](https://github.com/sabre-io/dav/blob/4.7.1/lib/CardDAV/Plugin.php#L308-L386)). The stored bytes can differ from what the client sent. The syncer then re-downloads, which is safe but shows up as a "change".
- **On GET/REPORT:** content negotiation **defaults to vCard 3.0**. A card stored as 4.0 is converted with `VObject\Document::convert(VCARD30)` unless the client asks for `text/vcard; version=4.0` ([L756-L791](https://github.com/sabre-io/dav/blob/4.7.1/lib/CardDAV/Plugin.php#L756-L791), [L803-L845](https://github.com/sabre-io/dav/blob/4.7.1/lib/CardDAV/Plugin.php#L803-L845)). Version conversion is a lossy path for 4.0-only properties.
- vdirsyncer only asks for 4.0 when `use_vcard_4 = true` ([docs/config.rst](https://github.com/pimutils/vdirsyncer/blob/main/docs/config.rst)). pimsync's `libdav 0.11.0` sends only `text/vcard` (`dav/mod.rs`, crate source), so it gets 3.0.
- RFC 6352 §3 requires servers to support vCard 3; v4 is only SHOULD ([RFC 6352](https://www.rfc-editor.org/rfc/rfc6352#section-3)). **Standardise on 3.0 end to end.** That is khard's default (`preferred_version = 3.0`) and vdirsyncer's.

### Repo-specific coupling

- `baikal-nudge-sync.py` reads cadence tokens from `NOTE`. `baikal-birthday-sync.py` reads `BDAY`, accepting `--MM-DD`, `YYYY-MM-DD` and `YYYYMMDD`. Any client that rewrites these two fields changes server-side behaviour (see khard below).

## Sync layer

### vdirsyncer — installed, recommended now

- Repo [pimutils/vdirsyncer](https://github.com/pimutils/vdirsyncer): tag v0.21.0 on 2026-09-03, last commit 2026-09-03, 203 open issues + 22 PRs. 27 commits in the last 12 months (WhyNotHugo 19). Lifetime contributors: untitaker 1867, WhyNotHugo 150.
- Changelog 0.21.0: retries transient network errors, Python 3.14 support ([CHANGELOG.rst](https://github.com/pimutils/vdirsyncer/blob/main/CHANGELOG.rst)).
- Digest auth since 0.19.3 (`auth = "digest"`). `guess` is refused with a UserError ([vdirsyncer/http.py](https://github.com/pimutils/vdirsyncer/blob/main/vdirsyncer/http.py)).
- Two-way sync. `conflict_resolution`: `null` (default: error, no change), `"a wins"`, `"b wins"`, or `["command", "nvim", "-d"]` ([docs/config.rst](https://github.com/pimutils/vdirsyncer/blob/main/docs/config.rst)).
- Whole-item conflicts only. Per-property merge has been open since 2016 ([#521](https://github.com/pimutils/vdirsyncer/issues/521)).
- Safety: raises `StorageEmpty` if one side got fully emptied since the last sync. Override with `--force-delete` ([sync/**init**.py](https://github.com/pimutils/vdirsyncer/blob/main/vdirsyncer/sync/__init__.py)).
- Items are opaque: they are hashed for change detection and never re-serialized. Known wart: files are written with LF, not CRLF ([#1128](https://github.com/pimutils/vdirsyncer/issues/1128)). The maintainer points to pimsync as the fix.
- Baïkal: "continuously tested against the latest version of Baikal" ([docs/tutorials/baikal.rst](https://github.com/pimutils/vdirsyncer/blob/main/docs/tutorials/baikal.rst)). Open [#896](https://github.com/pimutils/vdirsyncer/issues/896) (Baïkal test 500s, 2021) is unresolved.
- Secrets: `password.fetch = ["command", ...]`, already in use locally.
- Mode: one-shot `vdirsyncer sync`, via a timer or by hand. No daemon.

### pimsync — the successor, recommended next

- Same author. "successor and reimplementation of vdirsyncer", Rust, sync logic in the `vstorage` crate ([README](https://git.sr.ht/~whynothugo/pimsync)). The maintainer says NLnet funds it ([vdirsyncer#1128 comment, 2026-03-06](https://github.com/pimutils/vdirsyncer/issues/1128)).
- Repo health from a clone of `git.sr.ht/~whynothugo/pimsync`: tag v0.6.0 on 2026-09-05, last commit 2026-09-21. 200 commits in the last 12 months (Hugo Osvaldo Barrera 193, 7 others 1–2 each), 12 authors ever. Bus factor 1. todo.sr.ht returned 502, so the open-ticket count is unverified.
- Digest auth landed in **v0.5.10** ("Implement digest auth") ([changelog](https://pimsync.whynothugo.nl/)). v0.6.0 renamed `auth` to `auth_method` (breaking) and added reusable `auth {}` blocks.
- `pimsync-migration(7)` still lists digest as missing (dated 2025-04-14, stale). `pimsync.conf(5)` and `src/parseconf.rs` confirm `auth_method basic | digest`.
- **Arch `extra` pimsync 0.5.7 predates digest, so it cannot log in to this Baïkal.** Options: `cargo install` from source, wait for the package, or switch Baïkal to Basic.
- Conflicts: `conflict_resolution cmd nvim -d | keep a | keep b`, plus an interactive `pimsync resolve-conflicts <pair>` ([pimsync.conf(5)](https://pimsync.whynothugo.nl/pimsync.conf.5.html), `pimsync(1)`).
- Safety: `on_empty skip` by default, so an emptied side does not propagate. `on_delete sync` applies only to already-empty collections.
- `pimsync daemon` monitors the vdir (inotify, debounced) and syncs incrementally. `interval N` is the remote poll period ([pimsync(1)](https://pimsync.whynothugo.nl/pimsync.1.html)).
- Normalises LF to CRLF (maintainer, [#1128](https://github.com/pimutils/vdirsyncer/issues/1128)). Relative paths are rejected (0.5.11).
- Secrets: `password { cmd … }` or `password { shell … }`. vdirsyncer's `fetch` syntax is not accepted ([pimsync-migration(7)](https://pimsync.whynothugo.nl/pimsync-migration.7.html)).
- Status DBs are not shared with vdirsyncer. Use a fresh `status_path`.

### Others

- None maintained that target a vdir. cardamum (below) talks to CardDAV directly, with no local mirror unless you pair it with the Pimalaya sync engine.

## Client layer

### khard — recommended

- Repo [lucc/khard](https://github.com/lucc/khard): v0.21.0 on 2026-06-15, last commit 2026-09-07, 30 open issues + 5 PRs. 33 commits in the last 12 months (lucc 29). Bus factor 1. Arch `extra` 0.21.0.
- CLI over a vdir. `list`, `show`, `new`, `edit`, `merge`, `email`, `phone`, `birthdays`. Field queries like `name:`, `emails:`, `uid:` ([commandline.rst](https://github.com/lucc/khard/blob/main/doc/source/commandline.rst), [query.py](https://github.com/lucc/khard/blob/main/khard/query.py)).
- Editing: `khard edit` renders the contact as YAML in `$EDITOR` and parses it back. `merge` opens `merge_editor` (vimdiff).
- vCard 3.0 default, 4.0 optional (`preferred_version`) ([khard.conf.example](https://github.com/lucc/khard/blob/main/doc/source/examples/khard.conf.example)).
- Round-trip: `update()` deletes and rewrites only the fields in its YAML schema: N, NICKNAME, ORG, X-ABSHOWAS, KIND, ROLE, TITLE, TEL, EMAIL, ADR, CATEGORIES, URL, ANNIVERSARY/X-ANNIVERSARY, BDAY, configured `X-<private>`, NOTE ([contacts.py L1091-L1284](https://github.com/lucc/khard/blob/main/khard/contacts.py#L1091-L1284)).
  - **PHOTO, IMPP, MEMBER, X-SOCIALPROFILE and other unknown properties survive untouched.** Photo editing is unsupported ([#107](https://github.com/lucc/khard/issues/107), open since 2017).
  - Apple `itemN.X-ABLABEL` labels are removed together with their grouped field, then re-added from the YAML labels ([_delete_vcard_object](https://github.com/lucc/khard/blob/main/khard/contacts.py#L162-L180)).
  - Groups: KIND/MEMBER are not modelled ([todo.txt](https://github.com/lucc/khard/blob/main/todo.txt)) but are preserved. CATEGORIES is editable.
- **Data-loss risk, BDAY:**
  - Every edit deletes and re-adds BDAY, even if you didn't touch it, so any BDAY parameter is dropped (e.g. an Apple `X-APPLE-OMIT-YEAR`) ([L1252-L1254](https://github.com/lucc/khard/blob/main/khard/contacts.py#L1252-L1254), [birthday setter](https://github.com/lucc/khard/blob/main/khard/contacts.py#L322-L335)).
  - On 3.0 a year-less `--MM-DD` is read as year 1900 (`DEFAULT_YEAR`) and written back as `1900-MM-DD`. Typing `--MM-DD` into the YAML is rejected on 3.0 ([_set_date](https://github.com/lucc/khard/blob/main/khard/contacts.py#L1059-L1084), [yaml_anniversary](https://github.com/lucc/khard/blob/main/khard/helpers/__init__.py#L163-L182)).
  - Effect here: `baikal-birthday-sync` then sees year 1900 instead of "no year". The event still recurs yearly, but "year unknown" is lost.
- NOTE multiline: `|` blocks come back as a `"…\n…"` string ([#328](https://github.com/lucc/khard/issues/328), open). This is cosmetic in YAML only; the vCard NOTE value is unchanged. `@30d` nudge tokens survive.
- Speed: Python + vobject parse every `.vcf` per call. `search_in_source_files = yes` greps raw files first. Not benchmarked here because installing is out of scope.
- Keyboard UX: plain tables, no interactive UI. An `fzf` wrapper fills the gap (sketch below).
- Mail integration: neomutt `set query_command = "khard email --parsable %s"` ([scripting.rst](https://github.com/lucc/khard/blob/main/doc/source/scripting.rst)). aerc: `address-book-cmd = khard email --remove-first-line --parsable %s` ([aerc-config(5)](https://git.sr.ht/~rjarry/aerc/tree/master/item/doc/aerc-config.5.scd)).

### aerc — address book integration only

- Repo `git.sr.ht/~rjarry/aerc` (clone): 0.22.0 on 2026-07-28, last commit 2026-09-25. 218 commits in the last 12 months across ≥5 regular authors (Robin Jarry 80).
- Contacts are completion only, via `address-book-cmd` (tab-delimited `email\tname`). No viewer or editor.
- The bundled `carddav-query` queries CardDAV directly but sends only `Authorization: Basic` (`contrib/carddav-query` L89). **It won't work against Digest Baïkal.** Use khard over the vdir instead.

### Real TUIs — none viable (GitHub/crates.io search, 2026-09-25)

| Project                                                                                           | State                                                                                    | Verdict                     |
| ------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | --------------------------- |
| [kenianbei/vcard_tui](https://github.com/kenianbei/vcard_tui) (Rust)                              | 4 commits, last 2024-01-20, v0.1.2, 4★                                                   | dead                        |
| [verdigris12/rldx](https://github.com/verdigris12/rldx) (Rust, vdir)                              | 77 commits in 2 days (2026-01-10..11), 0★, README: "vibe coded … not even in alpha"      | do not trust with data      |
| [popplestones/rs-rolodex](https://github.com/popplestones/rs-rolodex) (ratatui)                   | 1 day of activity 2025-07-16, reads `contacts.json`                                      | not vCard                   |
| [mrusme/addrb](https://github.com/mrusme/addrb) (Go, CardDAV)                                     | archived 2026-07-22, lookup-only                                                         | superseded                  |
| [mrusme/inca](https://github.com/mrusme/inca) (Go, CalDAV+CardDAV)                                | active (28 commits since 2026-07-22), README: depends on a private "Maya" go-webdav fork | unusable today              |
| [nikiroo/jvcard](https://github.com/nikiroo/jvcard) (Java)                                        | last push 2019-03-03                                                                     | dead                        |
| [uriel1998/ppl_virdirsyncer_addysearch](https://github.com/uriel1998/ppl_virdirsyncer_addysearch) | shell viewer/searcher over vcards, 4★                                                    | viewer only                 |
| abook (installed)                                                                                 | own native format, vCard only via `--convert` (`man abook`)                              | lossy for vCard round-trips |

### Direct-CardDAV CLI: cardamum

- [pimalaya/cardamum](https://github.com/pimalaya/cardamum): v0.2.0 on 2026-08-24, last commit 2026-09-01, 43★, soywod 85 of 86 commits. README: "v0.x: expect breaking changes".
- Raw-vCard editing: `card update -i` opens a composer (`$EDITOR`) on the temp `.vcf`, validates it, and sends `If-Match` so it won't clobber concurrent writes ([CHANGELOG.md](https://github.com/pimalaya/cardamum/blob/master/CHANGELOG.md), unreleased section).
- Auth documented as `carddav.auth.basic.*` and bearer only ([config.sample.toml](https://github.com/pimalaya/cardamum/blob/master/config.sample.toml)). No digest, so the Baïkal auth switch applies here too. Worth watching; not ready.

### GUI fallbacks (one line each)

- GNOME Contacts: CardDAV via the GNOME Online Accounts WebDAV provider (GNOME 46+) ([release notes](https://release.gnome.org/46/)). Pulls in EDS.
- [KAddressBook](https://apps.kde.org/kaddressbook/): Akonadi DAV resource. Heavy stack under Hyprland.

## Recommended stack

1. `pacman -S khard` (user action). Keep vdirsyncer 0.21.0.
2. Replace the dead radicale `contacts` pair with a Baïkal pair. Use a new pair name so the old status DB is ignored. Set `auth = "digest"`, no `use_vcard_4`, and `conflict_resolution = ["command", "nvim", "-d"]`.
3. Run `vdirsyncer discover contacts_baikal && vdirsyncer sync contacts_baikal` by hand, or on a user systemd timer ([docs/tutorials/systemd-timer.rst](https://github.com/pimutils/vdirsyncer/blob/main/docs/tutorials/systemd-timer.rst)).
4. Use khard for editing. Bind a Hyprland key to an `fzf` picker for the TUI feel.
5. Later: switch Baïkal `dav_auth_type` to `Basic`. TLS is already enforced by Caddy, and passwords need no reset. This unblocks Arch pimsync 0.5.7, aerc `carddav-query` and cardamum. Then move to `pimsync daemon`.

## Config sketch (placeholders, no secrets)

`~/.config/vdirsyncer/config` (append):

```ini
[pair contacts_baikal]
a = "baikal_contacts"
b = "local_contacts_baikal"
collections = ["from a"]
conflict_resolution = ["command", "nvim", "-d"]

[storage baikal_contacts]
type = "carddav"
url = "https://<baikal_domain>/dav.php/"
username = "<baikal_user>"
password.fetch = ["command", "sh", "-c", "<secret-command> | head -1"]
auth = "digest"

[storage local_contacts_baikal]
type = "filesystem"
path = "~/.local/share/contacts/"
fileext = ".vcf"
```

Equivalent `~/.config/pimsync/pimsync.conf` (needs pimsync ≥0.6.0 while Baïkal is on Digest):

```
status_path "~/.local/share/pimsync/status/"

pair contacts {
	storage_a contacts_local
	storage_b contacts_baikal
	collections all
	conflict_resolution cmd nvim -d
}

storage contacts_local {
	type vdir/vcard
	path ~/.local/share/contacts/
	fileext vcf
}

storage contacts_baikal {
	type carddav
	url https://<baikal_domain>/dav.php/
	username <baikal_user>
	password {
		shell <secret-command> | head -1
	}
	auth_method digest
	interval 300
}
```

`~/.config/khard/khard.conf`:

```ini
[addressbooks]
[[baikal]]
path = ~/.local/share/contacts/default/

[general]
default_action = list
editor = nvim
merge_editor = nvim, -d

[contact table]
display = formatted_name
sort = last_name
show_uids = yes
localize_dates = no

[vcard]
preferred_version = 3.0
search_in_source_files = yes
skip_unparsable = no
```

fzf picker (`khard list -p` prints `uid\tname\taddress_book` per [cli.py](https://github.com/lucc/khard/blob/main/khard/cli.py)):

```sh
khard list -p | fzf --delimiter='\t' --with-nth=2 \
  --preview 'khard show uid:{1}' \
  --bind 'enter:execute(khard edit uid:{1})+reload(khard list -p)' \
  --bind 'ctrl-n:execute(khard new)+reload(khard list -p)'
```

## Open questions

- Which vCard versions are on the server now? Thunderbird wrote most of them. Read-only check on the host: `sqlite3 -readonly /opt/baikal/Specific/db/db.sqlite "SELECT substr(CAST(carddata AS TEXT), instr(CAST(carddata AS TEXT),'VERSION:'), 11) v, count(*) FROM cards GROUP BY v"`. Any 4.0 cards come back as sabre-converted 3.0.
- Does an iPhone/macOS client also write to this Baïkal? If yes, khard edits drop Apple BDAY params and rewrite `itemN.X-ABLABEL` groups.
- Year-less birthdays: accept `1900-MM-DD` from khard, or keep BDAY edits out of khard? Another option: teach `baikal-birthday-sync.py` that 1900 means "no year".
- Switch Baïkal to Basic auth? It is one line in `baikal.yaml.j2`, and every device's client must follow. It unblocks pimsync from Arch, aerc `carddav-query` and cardamum.
- Add a Caddy `redir /.well-known/carddav /dav.php/ 308` (RFC 6764) so discovery-only clients work?
- `use_vcard_4`: leave off unless the server inventory is mostly 4.0 and it is confirmed that no 3.0-only client writes.
- pimsync open-issue count and any data-loss tickets: todo.sr.ht returned 502 on 2026-09-25. Re-check before migrating.
