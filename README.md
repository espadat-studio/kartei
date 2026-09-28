<p align="center">
  <h1 align="center"><a href="https://kartei.espadat.com">kartei</a></h1>
</p>
<p align="center">
  <a href="https://crates.io/crates/kartei">
    <img src="https://img.shields.io/crates/v/kartei" alt="Crates.io">
  </a>
</p>

> Edit your contacts in the terminal without losing what your phone put in them.

kartei is a keyboard-driven address book for the `.vcf` files on your disk: a folder that [vdirsyncer](https://vdirsyncer.pimutils.org/) syncs from CardDAV, or the single file Thunderbird exports. You browse, search, edit, add and delete contacts. On save, kartei rewrites only the lines you changed, and every other byte goes back as it was. The labels, photos and `X-` fields that Apple Contacts or another client wrote are still there after the next sync. The same search also completes addresses in aerc and mutt. A Kartei is German for a card index, the box of contact cards on a desk.

```text
┌Cards─────────────────────────┐┌──────────────────────────────────────────────┐
│Margaret Hamilton             ││Ada Lovelace                                  │
│Grace Hopper                  ││Company  Analytical Engines                   │
│Hedy Lamarr                   ││                                              │
│Ada Lovelace                  ││Phone (cell)  +44 20 7946 0001                │
│Linus Torvalds                ││Email (work)  ada@engines.example             │
│Alan Turing                   ││URL (blog)  https://ada.example               │
└──────────────────────────────┘└──────────────────────────────────────────────┘
 / search  e edit  E raw edit  n new  d delete  y copy  ? help  q quit
```

## Features

- Saves touch only the lines you edited, so Apple `itemN.` groups, `X-ABLabel` and `PHOTO` survive
- Each save first checks the file on disk, so a sync that landed meanwhile is never overwritten without asking
- Files changed by vdirsyncer reload on their own, and the reload waits while you edit
- Fuzzy search on name, company and email: `jhn smth` finds John Smith, digits find phone numbers
- `kartei query` gives aerc and mutt the same search for address completion
- Opens Thunderbird's one-file export and writes it back ready to import
- `y` copies a phone or email with OSC 52, which also works over SSH and in tmux
- No config file, no server, no account

## Install

Download a binary for Linux (x86_64, aarch64) or macOS (Apple Silicon) from the [latest release](https://github.com/espadat-studio/kartei/releases/latest) and put it on your `PATH`, or install from crates.io:

```bash
cargo install kartei
```

## Quick start

Point kartei at a folder of `.vcf` files or at a single `.vcf` file:

```bash
kartei ~/.local/share/contacts
kartei Contacts.vcf
```

Or set `KARTEI_DIR` once and run `kartei` alone. `?` lists the keys of the current screen.

For address completion in aerc, add this to `aerc.conf`:

```ini
[compose]
address-book-cmd = kartei query "%s"
```

```console
$ kartei query lov
ada@engines.example	Ada Lovelace	work
linus@kernel.example	Linus Torvalds	work
```

## Documentation

Full documentation lives at [kartei.espadat.com](https://kartei.espadat.com):

- [Getting started](https://kartei.espadat.com/getting-started/): install, choose the address book, exit codes
- [Edit Thunderbird contacts](https://kartei.espadat.com/getting-started/#edit-thunderbird-contacts): the export and import round trip, and its two traps
- [Complete addresses in aerc and mutt](https://kartei.espadat.com/getting-started/#complete-addresses-in-aerc-and-mutt): the `query` output format and mutt mode
- [Sync with vdirsyncer](https://kartei.espadat.com/getting-started/#sync-with-vdirsyncer): a working config for one CardDAV address book
- [Keymap](https://kartei.espadat.com/keymap/): every key per screen, plus delete, raw edit and conflict rules

## Develop

`./setup` installs mise and the git hooks, `mise run` lists the tasks. [CONTEXT.md](./CONTEXT.md) holds the vocabulary, [meta/adr/](./meta/adr/) the decisions.

## License

[AGPL-3.0-or-later](LICENSE)
