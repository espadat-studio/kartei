---
title: "kartei"
description: "Terminal address book for vCard files. Edits touch only the lines you change, so Apple and CardDAV extras survive the next sync. Works with vdirsyncer folders, Thunderbird exports and aerc or mutt completion."
---

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
- [`kartei query`](/getting-started/#complete-addresses-in-aerc-and-mutt) gives aerc and mutt the same search for address completion
- Opens [Thunderbird's one-file export](/getting-started/#edit-thunderbird-contacts) and writes it back ready to import
- `y` copies a phone or email with OSC 52, which also works over SSH and in tmux
- No config file, no server, no account

## Quick start

```bash
mise use -g github:espadat-studio/kartei
kartei ~/.local/share/contacts
```

[Getting Started](/getting-started/) covers the other installs, Thunderbird and vdirsyncer. The [Keymap](/keymap/) lists every key.
