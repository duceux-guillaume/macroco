---
description: Run phased quality gates before creating or updating a PR
---

# Refine PR

Run quality gates in 3 phases before creating or updating a PR. Stop on hard failures in Phase 1. Report changes in Phase 2. Report findings in Phase 3.

**Prerequisites:** This command invokes `/simplify`, `/requesting-code-review`, `/security-review`, `/permissions-audit`, and `/claude-md-management:revise-claude-md`. If any skill is unavailable, skip that step and note it in the report.

## Phase 1 — Validate

Run these checks. If ANY fail, stop and fix before proceeding.

1. **Backend tests:**
   Run `cargo test --workspace`

2. **Clippy:**
   Run `cargo clippy --workspace -- -D warnings`

3. **Frontend checks:**
   Run `cd frontend && npm run check && npm test && npm run build`

4. **Traceability:**
   Run `python3 scripts/traceability.py`

5. **Model documentation sync:**
   Run `/audit-model-doc --diff`

If all pass, report: "Phase 1 passed. Proceeding to Phase 2." and continue.
If any fail, report which failed and stop. Do NOT proceed to Phase 2.

## Phase 2 — Refine

These steps may modify files. Run all of them, then report changes.

1. **Simplify:**
   Invoke the `/simplify` skill. Review changed files for reuse, quality, and efficiency. Apply fixes.

2. **Fold plan content:**
   Check if any `docs/plans/` files exist on disk for this feature/branch. If so:
   - Identify findings, decisions, and structural insights worth preserving
   - Fold them into the appropriate permanent docs: `CLAUDE.md`, `docs/architecture.md`, `docs/product-requirements.md`, or other relevant docs
   - Do NOT delete the plan files (they are gitignored and stay local)
   - Do NOT add plan files to git

After both steps, show a summary of all file changes made in Phase 2.
Ask: "Commit these refinement changes? [y/n]"
If yes, commit with a conventional-commit message summarizing the actual refinements made.

## Phase 3 — Review

These steps are advisory — they produce findings for review but do not auto-apply changes.

1. **Code review:**
   Invoke the `/requesting-code-review` skill. Present the review summary.

2. **Permissions audit:**
   Invoke the `/permissions-audit` skill. Present findings.

3. **Security review:**
   Invoke the `/security-review` command. Present findings.

4. **CLAUDE.md revision:**
   Invoke the `/claude-md-management:revise-claude-md` skill. Present proposed updates for review. Do NOT auto-apply — wait for user approval.

## Final Step

After all phases complete, ask:
"All quality gates complete. Ready to create/update PR? [y/n]"

If yes, proceed with PR creation or update using the `superpowers:finishing-a-development-branch` skill.
