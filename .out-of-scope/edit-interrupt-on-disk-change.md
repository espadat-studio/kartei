# Edit interrupt on disk change

kartei does not interrupt an edit with a "file changed, reload?" prompt when the watcher reports a change to the file behind the open Card.

## Why this is out of scope

In the Edit form, the delete prompt, the conflict prompt and a raw edit in `$EDITOR`, a change on disk only marks the Address book stale. The reload runs on return to Browse (ADR-0003).

Every write already goes through the save-time conflict check: `vdir::conflict` compares the file's bytes against the loaded bytes before a save or delete. A Card changed by vdirsyncer mid-edit surfaces as the existing conflict prompt, so nothing is lost.

An interrupt would add a second prompt path for a race the conflict check already catches, plus the question of what happens to the half-edited form.

## When to reconsider

If deferred reloads cause lost work in practice, e.g. a long form edit that routinely ends in the conflict prompt, reopen this.

## Prior requests

- #53: "feat: prompt when a file changes during an edit"
