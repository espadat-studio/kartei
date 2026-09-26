# Reloads are deferred during edits

kartei watches the Address book and reloads it without asking while the user browses, searches or reads help. In the Edit form, a prompt, or a raw edit in `$EDITOR`, a change on disk only marks the book stale, and the reload runs on return to Browse. The save-time conflict check (the file's bytes against the loaded bytes) stays the only guard on writes, so a Card changed by vdirsyncer mid-edit surfaces as the existing conflict prompt, not a new interruption.

## Considered Options

- Interrupt the edit with a "file changed, reload?" prompt: rejected, it adds a second prompt path for a race the conflict check already catches. See `.out-of-scope/edit-interrupt-on-disk-change.md` (#53).
- Reload in place under the open form: rejected, the form's Card and chunk position would silently stop matching the disk.
