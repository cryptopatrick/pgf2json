The goal is to parse @grammars/Food/Food.pgf and for the output json file to be identical with the file @grammars/Food/gf_make_generated.json.

## Technical Notes

### JSON Field Ordering Issue
`serde_json` 1.0 uses `BTreeMap` internally by default, which orders JSON object keys lexicographically rather than by insertion order. This causes field ordering differences from the target output:

- Current order: `"funs"`, `"name"`, `"startcat"` (lexicographic)
- Target order: `"name"`, `"startcat"`, `"funs"` (insertion order)

This lexicographic ordering may also affect other parts of the JSON generation where HashMap/BTreeMap collections are used.

### Potential Solutions
1. Use `indexmap::IndexMap` instead of `std::collections::HashMap` to preserve insertion order
2. Upgrade to `serde_json` with `preserve_order` feature enabled
3. Manually control serialization order using `serde_json::Map` with careful insertion order
4. Use ordered JSON construction patterns throughout the codebase