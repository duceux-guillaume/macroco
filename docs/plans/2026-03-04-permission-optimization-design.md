# Permission Optimization Design

**Date:** 2026-03-04
**Goal:** Reduce permission friction during Claude Code sessions while maintaining guardrails for destructive/dependency/deployment actions.

## Problem

The current permission setup uses explicit allow-lists for specific commands, causing frequent permission prompts for routine operations (file edits, agent spawning, basic shell commands). This breaks flow without adding safety for the cases that actually matter.

## Design: Hybrid Broad-Allow + Deny-List + Audit Skill

### 1. Permission Settings Overhaul

Replace granular allow-lists with broad allows and a targeted deny-list.

**`.claude/settings.json`:**
```json
{
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

**`.claude/settings.local.json`:** Clear all rules (now redundant).

### 2. CLAUDE.md Behavioral Guardrails

Add a section to CLAUDE.md with semantic guardrails that glob patterns can't express:

- Deleting or renaming any file (even via Edit/Write)
- Adding, removing, or upgrading dependencies (Cargo.toml, package.json)
- Any destructive git operation (force push, reset, rebase, branch delete)
- Modifying CI/CD configuration (.github/workflows/*)
- Changing .claude/settings.json permissions
- Running commands that send data to external services (deploy, publish, curl POST)

These require explicit user confirmation regardless of tool permissions.

### 3. `/permissions-audit` Skill

A slash command at `.claude/commands/permissions-audit.md` with two modes:

**On-demand** (`/permissions-audit`):
1. Read `.claude/settings.json` and `.claude/settings.local.json`
2. Identify redundant, duplicate, or conflicting rules
3. Suggest deny rules for common dangerous patterns not yet covered
4. Present proposed changes, apply on approval

**End-of-session:**
1. Reflect on session — which tool calls triggered permission prompts?
2. For approved prompts: suggest adding to allow-list or relaxing deny rules
3. For denied prompts: suggest adding to deny-list if it represents a pattern
4. Present changes, apply on approval

## Implementation Steps

1. Update `.claude/settings.json` with broad allow + deny-list
2. Clear `.claude/settings.local.json`
3. Add "Permission & Autonomy Guidelines" section to CLAUDE.md
4. Create `.claude/commands/permissions-audit.md` skill
5. Test by running a typical session workflow
