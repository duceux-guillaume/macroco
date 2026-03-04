# User Feedback Mechanism — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add sidebar footer links that let users report bugs or request features via GitHub Issues, with pre-filled context.

**Architecture:** Two external links in a new `Sidebar.svelte` footer section + a `feedbackUrl.ts` utility for URL construction + GitHub issue templates. No backend changes.

**Tech Stack:** Svelte 5, TypeScript, GitHub Issues URL API

---

### Task 1: Create GitHub issue templates

**Files:**
- Create: `.github/ISSUE_TEMPLATE/bug_report.md`
- Create: `.github/ISSUE_TEMPLATE/feature_request.md`

**Step 1: Create bug report template**

Create `.github/ISSUE_TEMPLATE/bug_report.md`:

```markdown
---
name: Bug Report
about: Report a bug in Macroco
title: "[Bug] "
labels: bug
---

## Description

A clear description of the bug.

## Steps to Reproduce

1. Go to '...'
2. Click on '...'
3. See error

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened.

## Environment

- **Browser / OS:** (auto-filled if opened from app)
- **Active Preset:** (auto-filled if opened from app)

## Screenshots

If applicable, add screenshots.
```

**Step 2: Create feature request template**

Create `.github/ISSUE_TEMPLATE/feature_request.md`:

```markdown
---
name: Feature Request
about: Suggest a feature for Macroco
title: "[Feature] "
labels: enhancement
---

## Description

A clear description of the feature you'd like.

## Use Case

Why would this be useful? What problem does it solve?

## Alternatives Considered

Have you considered any alternative solutions or workarounds?
```

**Step 3: Commit**

```bash
git add .github/ISSUE_TEMPLATE/bug_report.md .github/ISSUE_TEMPLATE/feature_request.md
git commit -m "feat: add GitHub issue templates for bug reports and feature requests"
```

---

### Task 2: Create feedback URL utility

**Files:**
- Create: `frontend/src/lib/utils/feedback-url.ts`
- Create: `frontend/src/lib/utils/feedback-url.test.ts`

**Step 1: Write the test**

Create `frontend/src/lib/utils/feedback-url.test.ts`:

```typescript
// REQ: REQ-028
import { describe, it, expect } from 'vitest';
import { buildBugReportUrl, buildFeatureRequestUrl, REPO_URL } from './feedback-url';

describe('feedback URL builders', () => {
	it('builds bug report URL with encoded body', () => {
		const url = buildBugReportUrl('BAU', 'Mozilla/5.0 Test');
		expect(url).toContain(`${REPO_URL}/issues/new`);
		expect(url).toContain('template=bug_report.md');
		expect(url).toContain('labels=bug');
		// Body should contain preset and user agent
		const bodyParam = new URL(url).searchParams.get('body');
		expect(bodyParam).toContain('BAU');
		expect(bodyParam).toContain('Mozilla/5.0 Test');
	});

	it('builds feature request URL', () => {
		const url = buildFeatureRequestUrl();
		expect(url).toContain(`${REPO_URL}/issues/new`);
		expect(url).toContain('template=feature_request.md');
		expect(url).toContain('labels=enhancement');
	});

	it('handles null preset gracefully', () => {
		const url = buildBugReportUrl(null, 'Agent');
		const bodyParam = new URL(url).searchParams.get('body');
		expect(bodyParam).toContain('None');
	});
});
```

**Step 2: Run test to verify it fails**

Run: `cd frontend && npx vitest run src/lib/utils/feedback-url.test.ts`
Expected: FAIL — module not found

**Step 3: Write implementation**

Create `frontend/src/lib/utils/feedback-url.ts`:

```typescript
export const REPO_URL = 'https://github.com/duceux-guillaume/macroco';

export function buildBugReportUrl(presetName: string | null, userAgent: string): string {
	const body = [
		'## Environment',
		'',
		`- **Browser / OS:** ${userAgent}`,
		`- **Active Preset:** ${presetName ?? 'None'}`,
		'',
		'## Description',
		'',
		'',
		'## Steps to Reproduce',
		'',
		'1. ',
		'',
		'## Expected Behavior',
		'',
		'',
		'## Actual Behavior',
		''
	].join('\n');

	const params = new URLSearchParams({
		template: 'bug_report.md',
		labels: 'bug',
		body
	});
	return `${REPO_URL}/issues/new?${params.toString()}`;
}

export function buildFeatureRequestUrl(): string {
	const params = new URLSearchParams({
		template: 'feature_request.md',
		labels: 'enhancement'
	});
	return `${REPO_URL}/issues/new?${params.toString()}`;
}
```

**Step 4: Run test to verify it passes**

Run: `cd frontend && npx vitest run src/lib/utils/feedback-url.test.ts`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add frontend/src/lib/utils/feedback-url.ts frontend/src/lib/utils/feedback-url.test.ts
git commit -m "feat: add feedback URL builder utility with tests"
```

---

### Task 3: Add feedback footer to Sidebar

**Files:**
- Modify: `frontend/src/components/Sidebar.svelte`

**Step 1: Add script imports and reactive URL construction**

In the `<script lang="ts">` block of `Sidebar.svelte`, add:

```typescript
import { buildBugReportUrl, buildFeatureRequestUrl } from '$lib/utils/feedback-url';
import { focusedScenario } from '$lib/stores/scenarios';

const featureUrl = buildFeatureRequestUrl();
// Bug URL is reactive because it depends on the focused scenario
let bugUrl = $derived(
	buildBugReportUrl(
		$focusedScenario?.name ?? null,
		typeof navigator !== 'undefined' ? navigator.userAgent : 'SSR'
	)
);
```

**Step 2: Add footer HTML after the scrollable section**

After the closing `</div>` of `.sidebar-section.scrollable`, before `</aside>`, add:

```svelte
<div class="sidebar-divider"></div>
<div class="sidebar-footer">
	<a href={bugUrl} target="_blank" rel="noopener noreferrer" class="feedback-link">
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
			<path d="M8 2l1.88 1.88"/>
			<path d="M14.12 3.88 16 2"/>
			<path d="M9 7.13v-1a3.003 3.003 0 1 1 6 0v1"/>
			<path d="M12 20c-3.3 0-6-2.7-6-6v-3a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v3c0 3.3-2.7 6-6 6"/>
			<path d="M12 20v-9"/>
			<path d="M6.53 9C4.6 8.8 3 7.1 3 5"/>
			<path d="M6 13H2"/>
			<path d="M3 21c0-2.1 1.7-3.9 3.8-4"/>
			<path d="M20.97 5c0 2.1-1.6 3.8-3.5 4"/>
			<path d="M22 13h-4"/>
			<path d="M17.2 17c2.1.1 3.8 1.9 3.8 4"/>
		</svg>
		Report a bug
	</a>
	<a href={featureUrl} target="_blank" rel="noopener noreferrer" class="feedback-link">
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
			<path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/>
			<path d="M9 18h6"/>
			<path d="M10 22h4"/>
		</svg>
		Request a feature
	</a>
</div>
```

**Step 3: Add footer styles**

In the `<style>` block, add:

```css
.sidebar-footer {
	padding: 8px 16px 12px;
	display: flex;
	gap: 12px;
}
.feedback-link {
	display: inline-flex;
	align-items: center;
	gap: 4px;
	font-size: 11px;
	color: var(--text-secondary);
	text-decoration: none;
	transition: color 0.15s;
}
.feedback-link:hover {
	color: var(--accent);
}
```

**Step 4: Verify visually**

Run: `cd frontend && npm run dev`
Check: sidebar footer shows two links, hover turns them blue, clicking opens GitHub in new tab with pre-filled body.

**Step 5: Commit**

```bash
git add frontend/src/components/Sidebar.svelte
git commit -m "feat: add feedback links to sidebar footer"
```

---

### Task 4: Add REQ-028 to product requirements

**Files:**
- Modify: `docs/product-requirements.md`

**Step 1: Add REQ-028 entry**

In the `## Planned` section (or create it if needed), add:

```markdown
- [ ] **REQ-028: User feedback mechanism**
  - *Context:* Users need a way to report bugs and request features from within the app. A sidebar footer with GitHub Issues links provides a lightweight, zero-backend feedback channel.
  - *Priority:* medium
  - *Components:* `frontend/src/components/Sidebar.svelte`, `frontend/src/lib/utils/feedback-url.ts`, `.github/ISSUE_TEMPLATE/`
```

**Step 2: Commit**

```bash
git add docs/product-requirements.md
git commit -m "docs: add REQ-028 user feedback mechanism to product requirements"
```

---

### Task 5: Run full frontend checks

**Step 1: Run type check**

Run: `cd frontend && npm run check`
Expected: PASS — no type errors

**Step 2: Run all tests**

Run: `cd frontend && npm test`
Expected: PASS — all tests including new feedback-url tests

**Step 3: Build**

Run: `cd frontend && npm run build`
Expected: PASS — successful build

**Step 4: Mark REQ-028 as Done if all pass**

Update `docs/product-requirements.md` to move REQ-028 to Done section with checkbox checked.

**Step 5: Final commit**

```bash
git add docs/product-requirements.md
git commit -m "docs: mark REQ-028 as done"
```
