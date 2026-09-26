---
title: "Keymap"
---

Keys are fixed. The status bar shows the main ones; `?` lists them per mode.

## Browse

| Key                      | Action                      |
| ------------------------ | --------------------------- |
| `j` / `k`, `Down` / `Up` | Move down / up              |
| `g` / `G`                | Jump to top / bottom        |
| `/`                      | Search                      |
| `e` / `Enter`            | Edit selected card          |
| `E`                      | Edit raw vCard in `$VISUAL` |
| `n`                      | New card                    |
| `d`                      | Delete selected card        |
| `y`                      | Copy a value (if any)       |
| `R`                      | Reload all from disk        |
| `!`                      | List skipped cards (if any) |
| `?`                      | Help                        |
| `q`                      | Quit                        |

## Search

Typing filters the list live with a case-insensitive fuzzy match on display name, company, department and email, so `jhn smth` finds John Smith. Best matches come first and the top one is selected; ties keep the usual order. A query of digits also matches phone numbers as a substring, ignoring spaces, `+`, `-` and parentheses, and those hits come after fuzzy ones.

| Key         | Action                    |
| ----------- | ------------------------- |
| `Backspace` | Delete last character     |
| `Enter`     | Keep filter, back to list |
| `Esc`       | Clear filter              |

## Edit

| Key                    | Action                                                |
| ---------------------- | ----------------------------------------------------- |
| `Tab` / `Down`         | Next field                                            |
| `Shift-Tab` / `Up`     | Previous field                                        |
| `Left` / `Right`       | Move cursor                                           |
| `Backspace` / `Delete` | Delete before / after cursor                          |
| `Alt-a`                | Add a value (phone, email, URL, address)              |
| `Alt-d`                | Remove the focused value                              |
| `Alt-l`                | Cycle label: home, work, cell, other (URLs skip cell) |
| `Ctrl-s`               | Save                                                  |
| `Esc`                  | Cancel (asks first if changed)                        |

In the note and street fields, `Enter` inserts a newline. Birthdays are typed as `YYYY-MM-DD`, or `--MM-DD` without a year. A birthday kartei cannot read stays read-only.

## Delete

`d` asks "Delete <name>? y/n". `y` deletes the card for good; any other key cancels. There is no undo. The next card is selected, or the previous one if you deleted the last, and the filter is kept.

- A card alone in its file removes the file, so vdirsyncer propagates the deletion.
- A card in a bundle drops only its own lines. The other cards stay byte for byte.
- A file is removed once no card is left, unless it still holds a skipped card.
- A `.vcf` file given as the address book is never removed, even when empty.

If the file changed on disk since it was loaded, kartei deletes nothing and asks:

| Key   | Action                                 |
| ----- | -------------------------------------- |
| `r`   | Reload from disk                       |
| `o`   | Delete anyway (single-card files only) |
| `Esc` | Cancel                                 |

In a changed bundle, `o` is refused, because the card may have moved: reload first. A card whose file is already gone is dropped from the list.

## Raw edit

`E` opens the selected card's vCard text in `$VISUAL`, else `$EDITOR`, run through `sh` so arguments work. Inside a Bundle, only that card's text is opened. With neither variable set, the status bar shows an error. The text goes to a temp file readable only by you, deleted once the editor exits.

Saving an empty buffer aborts. Saving unchanged text writes nothing. Otherwise the text must hold exactly one valid card, then it goes through the same conflict check as a save and is written exactly as your editor saved it. Inside a Bundle, a missing final newline is added so the next card stays apart. The line-preserving promise ([ADR-0001](https://github.com/espadat-studio/kartei/blob/master/meta/adr/0001-line-preserving-card-edits.md)) does not hold for raw edits.

If the text is not one valid card, kartei shows why and asks:

| Key | Action                         |
| --- | ------------------------------ |
| `e` | Reopen the editor on your text |
| `d` | Discard the raw edit           |

On a conflict, `Esc` reopens the editor on your text.

## Copy

`y` lists the card's phones, emails, addresses and URLs, numbered 1 to 9. The value is sent through the terminal with OSC 52, so it works over SSH and in tmux (with `set-clipboard on`). The status shows "copied" as soon as the sequence is sent, even if the terminal ignores it.

| Key   | Action          |
| ----- | --------------- |
| `1-9` | Copy that value |
| `Esc` | Close the list  |

## Prompts

The help and skipped-files lists close on any key.

A prompt asks one question and waits for a key. Cancelling an edit with unsaved changes asks to confirm: `y` discards, `n` or `Esc` keeps editing. A failed save shows the error in red and keeps the form open.

When a save finds the file changed or deleted on disk, kartei keeps your edit and asks:

| Key   | Action                                 |
| ----- | -------------------------------------- |
| `r`   | Reload from disk, drop your edit       |
| `o`   | Overwrite (or recreate) with your edit |
| `Esc` | Keep editing                           |
