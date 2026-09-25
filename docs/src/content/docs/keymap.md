---
title: "Keymap"
---

Keys are fixed. The status bar shows the ones that apply in the current mode.

## Browse

| Key                | Action               |
| ------------------ | -------------------- |
| `j` / `k` / arrows | Move down / up       |
| `g` / `G`          | Jump to top / bottom |
| `/`                | Search               |
| `e` / `Enter`      | Edit focused contact |
| `n`                | New contact          |
| `y`                | Copy a value         |
| `R`                | Reload all from disk |
| `!`                | List skipped files   |
| `?`                | Help                 |
| `q`                | Quit                 |

`y` lists the contact's phones, emails, addresses and URLs, numbered 1 to 9. Pressing a digit copies that value through the terminal with OSC 52, so it works over SSH and in tmux (with `set-clipboard on`). `Esc` closes the list. The status shows "copied" as soon as the sequence is sent, even if the terminal ignores it.

## Search

Typing filters the list live. The match is a case-insensitive substring of the display name, organization, email or phone digits.

| Key     | Action                    |
| ------- | ------------------------- |
| `Enter` | Keep filter, back to list |
| `Esc`   | Clear filter              |

## Edit

| Key                 | Action                               |
| ------------------- | ------------------------------------ |
| `Tab` / `Shift-Tab` | Next / previous field                |
| `Up` / `Down`       | Previous / next field                |
| `Alt-a`             | Add a value (phone, email, address)  |
| `Alt-d`             | Remove the focused value             |
| `Alt-l`             | Cycle label: home, work, cell, other |
| `Ctrl-s`            | Save                                 |
| `Esc`               | Cancel (asks first if changed)       |

In the note and street fields, `Enter` inserts a newline. Birthdays are typed as `YYYY-MM-DD`, or `--MM-DD` without a year.

## Prompts

A prompt asks one question and waits for a key. Cancelling an edit with unsaved changes asks to confirm: `y` discards, `n` or `Esc` keeps editing. A failed save shows the error in red and keeps the form open.

When a save finds the file changed or deleted on disk, kartei keeps your edit and asks:

| Key   | Action                                 |
| ----- | -------------------------------------- |
| `r`   | Reload from disk, drop your edit       |
| `o`   | Overwrite (or recreate) with your edit |
| `Esc` | Keep editing                           |
