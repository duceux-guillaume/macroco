# Permission Optimization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce permission friction to near-zero while maintaining guardrails for destructive, dependency, and deployment actions.

**Architecture:** Three-layer approach — broad allow/deny in settings.json, semantic guardrails in CLAUDE.md, and a /permissions-audit slash command for continuous improvement.

**Tech Stack:** Claude Code settings (JSON), CLAUDE.md (Markdown), slash commands (Markdown)

---

### Task 1: Update `.claude/settings.json` with broad allow + deny-list

**Files:**
- Modify: `.claude/settings.json`

**Step 1: Replace the permissions block**

Replace the entire `"permissions"` key in `.claude/settings.json` with:

```json
{
  "enabledPlugins": {
    "frontend-design@claude-plugins-official": true,
    "context7@claude-plugins-official": true,
    "superpowers@claude-plugins-official": true,
    "github@claude-plugins-official": true,
    "claude-md-management@claude-plugins-official": true
  },
  "permissions": {
    "allow": [
      "Bash(*)",
      "Edit(*)",
      "Write(*)",
      "Agent(*)"
    ],
    "deny": [
      "Bash(rm *)",
      "Bash(rm -rf *)",
      "Bash(npm install*)",
      "Bash(npm add*)",
      "Bash(npm remove*)",
      "Bash(cargo add*)",
      "Bash(cargo remove*)",
      "Bash(brew *)",
      "Bash(pip install*)",
      "Bash(git push --force*)",
      "Bash(git reset --hard*)",
      "Bash(git clean*)",
      "Bash(git branch -D*)"
    ]
  }
}
```

**Step 2: Verify JSON is valid**

Run: `cat .claude/settings.json | python3 -m json.tool > /dev/null && echo "Valid JSON"`
Expected: `Valid JSON`

**Step 3: Commit**

```bash
git add .claude/settings.json
git commit -m "chore: switch to broad allow + deny-list permissions"
```

---

### Task 2: Clear `.claude/settings.local.json`

**Files:**
- Modify: `.claude/settings.local.json`

**Step 1: Replace with empty permissions**

Write `.claude/settings.local.json` with:

```json
{
  "permissions": {
    "allow": [],
    "deny": []
  }
}
```

**Step 2: Commit**

```bash
git add .claude/settings.local.json
git commit -m "chore: clear redundant local permission overrides"
```

Note: `settings.local.json` is typically gitignored, so this commit may be a no-op. If so, skip the commit.

---

### Task 3: Add Permission & Autonomy Guidelines to CLAUDE.md

**Files:**
- Modify: `CLAUDE.md` (insert new section after "## Developer Conventions" and before "### Simulation Engine")

**Step 1: Insert the new section**

Insert the following block in `CLAUDE.md` right before the `### Simulation Engine` line:

```markdown
## Permission & Autonomy Guidelines

Claude has broad tool permissions in this project. The deny-list in `.claude/settings.json` blocks dangerous shell commands. In addition, ALWAYS ask the user before:

- Deleting or renaming any file (even via Edit/Write — not just `rm`)
- Adding, removing, or upgrading dependencies (Cargo.toml, package.json)
- Any destructive git operation (force push, reset, rebase, branch delete)
- Modifying CI/CD configuration (.github/workflows/*)
- Changing .claude/settings.json permissions
- Running commands that send data to external services (deploy, publish, curl POST)

These actions require explicit user confirmation regardless of tool permissions.

Run `/permissions-audit` to review and improve permission settings.
```

**Step 2: Verify CLAUDE.md is well-formed**

Run: `head -80 CLAUDE.md` — confirm the new section appears in the right place and doesn't break surrounding sections.

**Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add permission & autonomy guidelines to CLAUDE.md"
```

---

### Task 4: Create `/permissions-audit` slash command

**Files:**
- Create: `.claude/commands/permissions-audit.md`

**Step 1: Write the slash command file**

Create `.claude/commands/permissions-audit.md` with this content:

```markdown
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
```

**Step 2: Verify the file exists and has correct frontmatter**

Run: `head -5 .claude/commands/permissions-audit.md`
Expected: Shows the `---` frontmatter block with `description:`.

**Step 3: Commit**

```bash
git add .claude/commands/permissions-audit.md
git commit -m "feat: add /permissions-audit slash command"
```

---

### Task 5: Smoke test

**Step 1: Verify all files are consistent**

Run: `cat .claude/settings.json | python3 -m json.tool` — confirm valid JSON with allow + deny structure.

Run: `ls .claude/commands/` — confirm both `audit-tables.md` and `permissions-audit.md` exist.

Run: `grep -n "Permission & Autonomy" CLAUDE.md` — confirm the new section exists.

**Step 2: Final commit (if any unstaged changes)**

```bash
git status
# If clean, done. If changes, stage and commit.
```
