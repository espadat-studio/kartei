# kartei

Keyboard-driven editor for contacts stored as vCard files in a local vdir.

## Language

**Card**:
One `.vcf` file in the vdir holding exactly one contact.
_Avoid_: entry, record, file

**Address book**:
The vdir directory kartei reads and writes; one Card per file.
_Avoid_: collection, database

**Content line**:
One logical vCard line (`group.NAME;params:value`) after unfolding, kept with its original bytes.
_Avoid_: row, field line

**Group**:
Content lines sharing a prefix like `item1.`; Apple uses it to attach a label to a value.

**Label**:
Human name for a value's kind, from `TYPE=` or a grouped `X-ABLabel`.
_Avoid_: tag, category

**Structured name**:
The Card's `N`: family, given, additional, prefixes, suffixes. Used for sorting.
_Avoid_: full name

**Display name**:
The Card's `FN`: free text shown to humans. Linked when it equals the name derived from the Structured name, custom otherwise.
_Avoid_: formatted name, full name
