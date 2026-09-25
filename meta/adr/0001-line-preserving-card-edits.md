# Line-preserving card edits

kartei keeps each card as its original content lines and rewrites only the lines an edit touches; everything else is written back byte-for-byte. We rejected parsing into a full model and re-serializing, because it rewrites every line on save (folding, order, param casing, escaping), makes lossless round-trip unprovable for properties we don't model, and risks breaking Apple `itemN.` label pairings. Existing crates were ruled out because they reject or normalize vCard 3.0 as emitted by Apple/iCloud.

Consequence: typed fields are a read view over lines; edits must address the line(s) they own, including grouped properties.
