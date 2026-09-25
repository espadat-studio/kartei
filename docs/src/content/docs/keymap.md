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
| `n`                      | New card                    |
| `y`                      | Copy a value (if any)       |
| `R`                      | Reload all from disk        |
| `!`                      | List skipped cards (if any) |
| `?`                      | Help                        |
| `q`                      | Quit                        |

## Search

Typing filters the list live. The match is a case-insensitive substring of the display name, company, department, email or phone digits.

| Key         | Action                    |
| ----------- | ------------------------- |
| `Backspace` | Delete last character     |
| `Enter`     | Keep filter, back to list |
| `Esc`       | Clear filter              |

## Edit

| Key                    | Action                               |
| ---------------------- | ------------------------------------ |
| `Tab` / `Down`         | Next field                           |
| `Shift-Tab` / `Up`     | Previous field                       |
| `Left` / `Right`       | Move cursor                          |
| `Backspace` / `Delete` | Delete before / after cursor         |
| `Alt-a`                | Add a value (phone, email, address)  |
| `Alt-d`                | Remove the focused value             |
| `Alt-l`                | Cycle label: home, work, cell, other |
| `Ctrl-s`               | Save                                 |
| `Esc`                  | Cancel (asks first if changed)       |

In the note and street fields, `Enter` inserts a newline. Birthdays are typed as `YYYY-MM-DD`, or `--MM-DD` without a year. A birthday kartei cannot read stays read-only.

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
