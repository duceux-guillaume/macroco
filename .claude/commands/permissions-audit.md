---
description: Audit and improve .claude/settings.json permission rules
---

# Permission Audit

Review and optimize Claude Code permission settings to reduce friction while maintaining safety guardrails.

## Steps

1. **Read current settings:**
   Read `.claude/settings.json` and `.claude/settings.local.json`. Parse the `permissions.allow` and `permissions.deny` arrays.

2. **Check for issues:**
   - **Redundant rules:** Allow rules that are already covered by a broader allow (e.g., `Bash(cargo test*)` is redundant if `Bash(*)` exists)
   - **Conflicting rules:** Rules that appear in both allow and deny
   - **Duplicate rules:** Exact duplicates within the same list
   - **Stale local overrides:** Rules in `settings.local.json` that duplicate or conflict with `settings.json`

3. **Review deny-list coverage:**
   Check the deny-list against these common dangerous patterns. Suggest adding any that are missing:
   - File deletion: `rm`, `rm -rf`
   - Dependency changes: `npm install/add/remove`, `cargo add/remove`, `pip install`, `brew`
   - Destructive git: `git push --force`, `git reset --hard`, `git clean`, `git branch -D`
   - Deployment: `flyctl deploy`, `docker push`
   - CI changes: editing `.github/workflows/*`

4. **Session reflection (if end-of-session):**
   Think about the current session:
   - Which tool calls triggered permission prompts that the user approved? These may need allow rules.
   - Which tool calls were denied? These may need deny rules.
   - Were there patterns of friction (same type of command repeatedly prompted)?

5. **Present findings:**
   Output a summary report:
   - Current rule counts (allow/deny in each file)
   - Issues found (redundant, conflicting, duplicate, stale)
   - Missing deny patterns
   - Session-based suggestions (if applicable)
   - Proposed JSON changes (show exact before/after)

6. **Apply on approval:**
   If the user approves the changes, update `.claude/settings.json` and/or `.claude/settings.local.json` with the proposed modifications.
