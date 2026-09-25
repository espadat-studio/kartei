# kartei

Keyboard-driven editor for contacts stored as vCard files in a local vdir.

kartei lists, searches, views, edits and creates contacts in a directory of `.vcf` files or in a single `.vcf` Bundle, such as Thunderbird writes. It never talks to a server; sync with [vdirsyncer](https://vdirsyncer.pimutils.org/).

Docs: <https://kartei.espadat.com>

## Install

Download a binary for Linux (x86_64, aarch64) or macOS (Apple Silicon) from the [latest release](https://github.com/espadat-studio/kartei/releases/latest) and put it on your `PATH`.

Or install from crates.io:

```bash
cargo install kartei
```

Until the first release, build from source:

```bash
cargo install --git https://github.com/espadat-studio/kartei
```

## Setup

```text
kartei <dir-or-file.vcf>
```

Point kartei at the directory vdirsyncer syncs into:

```bash
kartei ~/.local/share/contacts
```

Or pass a single `.vcf` file, such as a Thunderbird Bundle: `kartei Contacts.vcf`. New Cards go after the file's last Card, with its line endings. In a directory, each new Card gets its own `<uid>.vcf`. See [Edit Thunderbird contacts](https://kartei.espadat.com/getting-started/#edit-thunderbird-contacts) for the export and import round trip.

Or set `KARTEI_DIR` once and run `kartei` alone. The argument wins over `KARTEI_DIR`. A missing or unreadable path, or one that is neither a directory nor a `.vcf` file, prints the error and exits with status 1.

In the vdirsyncer `filesystem` storage, use `fileext = ".vcf"` and `collections = null` so one address book lands flat in `path`. Run `vdirsyncer sync` before and after editing.

A `.vcf` holding several Cards (a Thunderbird export) lists each one; saving rewrites only the edited Card's lines. A Card that is not valid vCard is skipped at load, the rest of its file still loads. The status bar counts skipped Cards; `!` lists each file, the position of the Card inside a Bundle (`Contacts.vcf #4`), and the reason.

## Lossless edits

kartei rewrites only the lines an edit touches. Every other line is written back byte for byte, so Apple extras (`itemN.` groups, `X-ABLabel`, `PHOTO`, unknown `X-` properties) survive. Before saving, kartei checks that the file on disk still matches what it loaded.

## Keymap

Keys are fixed. The status bar shows the main ones; `?` lists them per mode.

### Browse

| Key                      | Action                      |
| ------------------------ | --------------------------- |
| `j` / `k`, `Down` / `Up` | Move down / up              |
| `g` / `G`                | Jump to top / bottom        |
| `/`                      | Search                      |
| `e` / `Enter`            | Edit selected card          |
| `n`                      | New card                    |
| `d`                      | Delete selected card        |
| `y`                      | Copy a value (if any)       |
| `R`                      | Reload all from disk        |
| `!`                      | List skipped cards (if any) |
| `?`                      | Help                        |
| `q`                      | Quit                        |

### Search

Typing filters live on display name, company, department, email or phone digits.

| Key         | Action                    |
| ----------- | ------------------------- |
| `Backspace` | Delete last character     |
| `Enter`     | Keep filter, back to list |
| `Esc`       | Clear filter              |

### Edit

| Key                    | Action                               |
| ---------------------- | ------------------------------------ |
| `Tab` / `Down`         | Next field                           |
| `Shift-Tab` / `Up`     | Previous field                       |
| `Left` / `Right`       | Move cursor                          |
| `Backspace` / `Delete` | Delete before / after cursor         |
| `Enter`                | Newline (note and street only)       |
| `Alt-a`                | Add a value (phone, email, address)  |
| `Alt-d`                | Remove the focused value             |
| `Alt-l`                | Cycle label: home, work, cell, other |
| `Ctrl-s`               | Save                                 |
| `Esc`                  | Cancel (asks first if changed)       |

Birthdays are typed as `YYYY-MM-DD`, or `--MM-DD` without a year. A birthday kartei cannot read stays read-only.

### Discard

| Key         | Action                  |
| ----------- | ----------------------- |
| `y`         | Discard unsaved changes |
| `n` / `Esc` | Keep editing            |

### Conflict

Shown when a save finds the file changed or deleted on disk.

| Key   | Action                                 |
| ----- | -------------------------------------- |
| `r`   | Reload from disk, drop your edit       |
| `o`   | Overwrite (or recreate) with your edit |
| `Esc` | Keep editing                           |

### Delete

`d` asks "Delete <name>? y/n". `y` deletes the card for good; any other key cancels. A card alone in its file removes the file. A card in a bundle drops only its own lines; the file is removed once no card is left, unless a skipped card remains. A `.vcf` file given as the address book is kept, even empty.

If the file changed on disk since it was loaded:

| Key   | Action                                 |
| ----- | -------------------------------------- |
| `r`   | Reload from disk, delete nothing       |
| `o`   | Delete anyway (single-card files only) |
| `Esc` | Cancel                                 |

In a changed bundle, `o` is refused: reload first. A card whose file is already gone is dropped from the list.

### Copy

`y` lists the card's phones, emails, addresses and URLs, numbered 1 to 9. The value is sent with OSC 52, so it works over SSH and in tmux (`set-clipboard on`).

| Key   | Action          |
| ----- | --------------- |
| `1-9` | Copy that value |
| `Esc` | Close           |

### Help and skipped list

Any key closes.

## License

[AGPL-3.0-or-later](LICENSE)
