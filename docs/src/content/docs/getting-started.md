---
title: "Getting Started"
---

## Install

Download a binary for Linux (x86_64, aarch64) or macOS (Apple Silicon) from the [latest release](https://github.com/espadat-studio/kartei/releases/latest). Extract it somewhere on your `PATH`, e.g. `~/.local/bin`.

Or install from crates.io:

```bash
cargo install kartei
```

Until the first release, build from source:

```bash
cargo install --git https://github.com/espadat-studio/kartei
```

## Choose the address book

kartei reads one flat directory of `.vcf` files, or a single `.vcf` file. Pass it as the only argument:

```text
kartei <dir-or-file.vcf>
```

For example:

```bash
kartei ~/.local/share/contacts
```

Or set `KARTEI_DIR` once and run `kartei` alone:

```bash
export KARTEI_DIR=~/.local/share/contacts
kartei
```

A single file, such as a Thunderbird export, works the same way: `kartei Contacts.vcf`. New contacts go after the file's last one, with its line endings. In a directory, each new contact gets its own `<uid>.vcf`.

The argument wins over `KARTEI_DIR`. With neither, or with a path that is missing, unreadable, or neither a directory nor a `.vcf` file, kartei prints the error and exits with status 1.

## Edit Thunderbird contacts

Thunderbird exports each of its address books as one `.vcf` Bundle. kartei edits it in place, and Thunderbird imports it back:

1. In Thunderbird's Address Book, right-click the Thunderbird address book and choose **Export**, then the vCard format. This writes e.g. `Contacts.vcf`.
2. Open the Bundle and edit:

   ```bash
   kartei Contacts.vcf
   ```

3. In Thunderbird, choose **Tools > Import**, pick the vCard file, and import it into the **same** Thunderbird address book you exported.

Thunderbird matches Cards by `UID` and replaces the existing contact. Cards added in kartei have new UIDs, so they are imported as new contacts.

:::caution
Import replaces **every** Card in the file, not only the ones you edited. A change made in Thunderbird after the export is lost. Export again right before editing.
:::

:::caution
Dragging contacts between Thunderbird address books gives them new UIDs. A Bundle exported from one Thunderbird address book then no longer matches the contacts in the other. Importing it creates duplicates.
:::

## Sync with vdirsyncer

kartei never talks to a server. Use [vdirsyncer](https://vdirsyncer.pimutils.org/) to mirror a CardDAV address book into the directory. A minimal config for one address book:

```ini
[general]
status_path = "~/.local/share/vdirsyncer/status/"

[pair contacts]
a = "contacts_local"
b = "contacts_remote"
collections = null

[storage contacts_local]
type = "filesystem"
path = "~/.local/share/contacts/"
fileext = ".vcf"

[storage contacts_remote]
type = "carddav"
url = "https://dav.example.com/"
username = "you"
password.fetch = ["command", "pass", "dav"]
```

`collections = null` syncs a single address book straight into `path`, which is the flat layout kartei expects. Run `vdirsyncer sync` before and after editing.

## Lossless edits

kartei rewrites only the lines an edit touches. Every other line is written back byte for byte, so Apple extras (`itemN.` groups, `X-ABLabel`, `PHOTO`, unknown `X-` properties) survive. Before saving, kartei checks that the file on disk still matches what it loaded; see [Prompts](/keymap/#prompts).

## Cards kartei skips

A `.vcf` may hold several Cards, as Thunderbird exports write; each is listed and edited on its own, and saving rewrites only that Card's lines. Any change to the file on disk since load prompts before saving.

A Card without `BEGIN:VCARD`/`END:VCARD` or with invalid UTF-8 is skipped at load; the other Cards in its file still load, and its bytes are written back untouched. The status bar shows how many were skipped; press `!` to list each path, the position of the Card inside a Bundle (`Contacts.vcf #4`), and the reason.
