# Zen Stdlib Status

The files in this directory are experimental and are not part of the implemented v1 surface in the rewrite baseline.

A stdlib module may be promoted only after tests prove the required gate: it
must parse, typecheck, and build through the same compiler path as user modules.
Until then, these files
are design material and implementation sketches covered by the gates in
[docs/V1_SPEC.md](../docs/V1_SPEC.md).

Do not document a stdlib API as implemented solely because a `.zen` file exists
here.
