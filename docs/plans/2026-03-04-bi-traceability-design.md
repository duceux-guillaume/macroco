# Bi-Traceability: REQ ↔ Test Coverage System

**Date:** 2026-03-04
**Goals:** Coverage visibility + impact analysis
**Approach:** Comment-based annotations in test code + CI script

## Problem

Product requirements (`docs/product-requirements.md`) link forward to architecture components, and `docs/architecture.md` links back to REQs. But there is no link between requirements and tests. We can't answer:
- Which "Done" requirements have no tests?
- Which tests break if we change REQ-005?
- What's the overall test coverage of our requirements?

## Annotation Format

Tests annotate which REQs they cover with a `// REQ:` comment.

**Rust** — on `mod tests` or individual `#[test]` functions:
```rust
// REQ: REQ-001, REQ-002
#[cfg(test)]
mod tests {
    #[test]
    fn rk4_solver_converges() { ... }
}
```

**TypeScript** — on `describe()` or `it()` blocks:
```typescript
// REQ: REQ-009, REQ-020
describe('variable-descriptions', () => {
    it('has descriptions for all variables', () => { ... });
});
```

Convention: `// REQ:` prefix, comma-separated REQ IDs. Can appear at module level (covers all tests in the module) or per-test function (finer grained).

## CI Script (`scripts/traceability.py`)

A Python script (no external dependencies) that:

1. **Parses `docs/product-requirements.md`** — extracts all REQ IDs and their status (Done / In Progress / Planned).
2. **Scans test files** — greps for `// REQ:` annotations in:
   - `crates/*/src/**/*.rs` (Rust `mod tests` blocks)
   - `frontend/src/**/*.test.ts` (frontend test files)
3. **Builds coverage maps:**
   - Forward: REQ → list of test files/functions that reference it
   - Reverse: test file → list of REQs it covers
4. **Outputs:**
   - Markdown traceability matrix to `docs/traceability-matrix.md` (auto-generated, committed)
   - Console summary with covered/uncovered counts
   - Exit code 1 if any "Done" REQ has zero test references

## Coverage Rules

| REQ Status  | Rule                                |
|-------------|-------------------------------------|
| Done        | Must have ≥1 test reference (CI fails otherwise) |
| In Progress | Warning if no tests, no CI failure  |
| Planned     | No tests expected, no warning       |

## Impact Analysis

Once annotations exist, impact analysis is a simple grep:
- **"I'm changing REQ-005"** → `grep -r '// REQ:.*REQ-005' crates/ frontend/` → shows all affected tests
- **"I'm changing `agriculture.rs`"** → the `// REQ:` comment in its test module tells you which requirements are affected

The CI script outputs both forward (REQ → tests) and reverse (test → REQs) indexes.

## CI Integration

New `traceability` step in `.github/workflows/ci.yml`, runs after the test jobs:

```yaml
traceability:
  runs-on: ubuntu-latest
  needs: [test, frontend-test]
  steps:
    - uses: actions/checkout@v4
    - run: python3 scripts/traceability.py
```

## File Layout

```
scripts/traceability.py          # CI + local traceability checker
docs/traceability-matrix.md      # Auto-generated coverage matrix (committed)
.github/workflows/ci.yml         # New 'traceability' job step
```

## Incremental Adoption

Annotations are added incrementally — existing tests get `// REQ:` comments as part of this work. The CI script starts in warning mode (exit 0) until all Done REQs are annotated, then switches to enforcement mode (exit 1).

## Out of Scope

- ARCH-NNN identifiers (architecture traceability uses existing Components field)
- Proc-macro or build-time enforcement
- Test coverage percentage (this is requirement coverage, not line coverage)
