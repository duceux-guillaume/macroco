# User Feedback Mechanism — Design

**Date:** 2026-03-04
**REQ:** REQ-028
**Status:** Approved

## Problem

Users have no way to report bugs or request features from within the app. For an open source educational tool, a lightweight feedback channel increases community engagement and surfaces issues early.

## Decision

Sidebar footer with direct links to GitHub Issues, pre-filled with context. No backend changes, no new dependencies.

## Design

### Sidebar Footer

Add a footer area to `Sidebar.svelte` (flex-end, sticky to bottom):

- **"Report a bug"** link → `https://github.com/duceux-guillaume/macroco/issues/new?template=bug_report.md&labels=bug&body=<prefilled>`
- **"Request a feature"** link → `https://github.com/duceux-guillaume/macroco/issues/new?template=feature_request.md&labels=enhancement&body=<prefilled>`

Both open in new tab (`target="_blank"`, `rel="noopener noreferrer"`).

### Styling

- Font size: 11–12px, `--text-secondary` color
- Hover: transition to `--accent`
- Small inline SVG icons (bug, lightbulb) — no icon library
- Separator line above footer to distinguish from parameter controls

### URL Pre-fill

The `body` query parameter encodes:
- Current preset name (from Svelte store)
- Browser user agent (`navigator.userAgent`)
- App context (simulation year range if running)

### GitHub Issue Templates

Two templates in `.github/ISSUE_TEMPLATE/`:

**bug_report.md:**
- Description
- Steps to reproduce
- Expected vs actual behavior
- Browser / OS
- Current preset

**feature_request.md:**
- Description
- Use case
- Alternatives considered

### Mobile

Links appear in the sidebar drawer footer, same position. Accessible when drawer is open.

## Rejected Alternatives

| Approach | Why rejected |
|----------|-------------|
| In-app mini-form | More code for marginal UX gain; still opens GitHub |
| API-based issue creation | Backend changes, auth token management, security surface |
| Floating action button | Overlaps charts, takes screen real estate |

## Components Affected

- `frontend/src/routes/+page.svelte` (sidebar layout)
- `frontend/src/lib/components/Sidebar.svelte` (footer section)
- `.github/ISSUE_TEMPLATE/bug_report.md` (new)
- `.github/ISSUE_TEMPLATE/feature_request.md` (new)
- `docs/product-requirements.md` (REQ-028)
