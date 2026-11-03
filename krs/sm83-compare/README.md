# SM83 comparator

Compares two SM83 observations, the (end class, event log, end state) of a run (`draft-personal/observer-design/design.md`), under a tier and named normalizations.
The vocabulary is `../sm83-events`; observations, tiers, normalizations and the comparison itself are `../sm83-observe`; this package is only the definition that runs a comparison.

A comparison is a case of four inputs: `left` and `right` (each `Run { end, log, state }`), `tier` (`Full` or `Writes`) and `normalizations` (a list, `CompressIdle` the only one so far).
`krs run --case FILE` prints the verdict as the computation cell, which the run's report calls stuck since no rule follows it: `equal`, `endDiffers(l, r)`, `logDiffers(position, left rest, right rest)` or `stateDiffers(field)`.
Collections inside an observation are written in tagged forms, `{"$list": [..]}` for the log and `{"$map": [[address, byte], ..]}` for memory, because a constructor field's element types are not declared to the decoder; the cases in `cases/` show the shape.

`./run-cases.sh` checks every case against `cases/expected.toml`.

Not yet established: that `CompressIdle` is idempotent and lossless (`Expand(Compress(l)) == l`) for every log.
Both are lemmas over lists, by induction on the left fold, which the reachability prover does not do; they are for the Lean side (k2lean), where the folds were written to make them `List.foldl_append` arguments.
