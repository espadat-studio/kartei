---
title: "kartei"
description: "Keyboard-driven editor for contacts stored as vCard files in a local vdir."
---

> Read and edit your contacts from the terminal, one `.vcf` file at a time.

:::caution
kartei is pre-release. These pages describe the MVP being built; there is no release yet.
:::

kartei is a TUI for an address book kept as a vdir (a directory with one vCard file per contact) or as a single `.vcf` Bundle, such as Thunderbird writes. It lists, searches, views and edits contacts. It creates new ones too. Syncing is left to [vdirsyncer](https://github.com/pimutils/vdirsyncer).

## Key features

- **Lossless edits**: lines you did not touch are written back byte for byte, so Apple extras survive
- **Keyboard only**: vim-style movement, live search, a form editor
- **Conflict check**: a save refuses to clobber a file that changed on disk since it was loaded
- **No config**: point it at a directory or a `.vcf` file and go

## Quick example

```text
kartei <dir-or-file.vcf>
```

```bash
kartei ~/.local/share/contacts
kartei Contacts.vcf
```

See [Getting Started](/getting-started/) to install and set up an address book, then the [Keymap](/keymap/).
