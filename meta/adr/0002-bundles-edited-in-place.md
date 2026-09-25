# Bundles are edited in place

kartei reads a Bundle (a `.vcf` file holding several Cards, as Thunderbird exports write) by splitting it before each `BEGIN:VCARD` into chunks that concatenate back to the exact file bytes, and saves an edit by rebuilding the whole file with that Card swapped in. A Card in a Bundle is identified by its file and position; any change to the file on disk since load is a conflict. A defective chunk is skipped on its own and its bytes are kept, so saving a neighbour writes it back untouched. A file given as the Address book receives new Cards at its end; a directory Address book gives each new Card its own `<uid>.vcf`.

## Considered Options

- Split Bundles into one file per Card on disk: rejected, it leaves two copies that drift and needs a manual rebundle step before handing the file back to Thunderbird.
- Per-Card conflict detection inside a Bundle: rejected, matching a Card across a changed file needs positions or UIDs that shift or may be missing; whole-file conflicts reuse the existing reload/overwrite prompt.
- Read-only Bundles: rejected, a contact that can be seen but not edited is not useful.
