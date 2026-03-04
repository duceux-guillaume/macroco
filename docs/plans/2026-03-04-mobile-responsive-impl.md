# Mobile Responsive Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the app usable on phones (375px+) by converting the sidebar to a drawer, adapting chart layout, and adding touch support.

**Architecture:** Single CSS breakpoint at 768px. Below it, the sidebar slides off-screen and becomes a drawer triggered by a hamburger button. Charts take full viewport width. Touch events added alongside mouse events. No new dependencies.

**Tech Stack:** SvelteKit 5 (runes), CSS media queries, D3 v7, existing ResizeObserver utility.

---

### Task 1: Add mobile breakpoint to app.css

**Files:**
- Modify: `frontend/src/app.css`

**Step 1: Add mobile media query to global CSS**

Add at the end of `frontend/src/app.css`:

```css
/* Mobile breakpoint */
@media (max-width: 767px) {
	:root {
		--sidebar-width: 0px;
	}
}
```

**Step 2: Verify no build errors**

Run: `cd frontend && npm run check`
Expected: No errors

**Step 3: Commit**

```bash
git add frontend/src/app.css
git commit -m "feat: add mobile breakpoint to global CSS"
```

---

### Task 2: Sidebar drawer on mobile — layout and hamburger

**Files:**
- Modify: `frontend/src/routes/+page.svelte`
- Modify: `frontend/src/components/Sidebar.svelte`

**Step 1: Add drawer state and hamburger to +page.svelte**

Replace the `<script>` and template in `frontend/src/routes/+page.svelte`:

```svelte
<script lang="ts">
	import Sidebar from '../components/Sidebar.svelte';
	import ScenarioBar from '../components/ScenarioBar.svelte';
	import ChartGrid from '../components/ChartGrid.svelte';
	import VariableInfoPanel from '../components/VariableInfoPanel.svelte';
	import ParameterInfoPanel from '../components/ParameterInfoPanel.svelte';
	import HistoricalInfoPanel from '../components/HistoricalInfoPanel.svelte';

	let sidebarOpen = $state(false);
</script>

<svelte:head>
	<title>Macroco — World 3 Simulator</title>
</svelte:head>

<button
	class="hamburger"
	onclick={() => (sidebarOpen = !sidebarOpen)}
	aria-label="Toggle sidebar"
>
	<svg width="20" height="20" viewBox="0 0 20 20" fill="none">
		<line x1="3" y1="5" x2="17" y2="5" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
		<line x1="3" y1="10" x2="17" y2="10" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
		<line x1="3" y1="15" x2="17" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
	</svg>
</button>

{#if sidebarOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="drawer-backdrop" onclick={() => (sidebarOpen = false)}></div>
{/if}

<div class="app-layout">
	<div class="sidebar-drawer" class:open={sidebarOpen}>
		<Sidebar />
	</div>
	<main class="main-content">
		<ScenarioBar />
		<ChartGrid />
	</main>
	<VariableInfoPanel />
	<ParameterInfoPanel />
	<HistoricalInfoPanel />
</div>
```

**Step 2: Add styles for drawer and hamburger in +page.svelte**

Replace the `<style>` block:

```css
<style>
	.app-layout {
		display: flex;
		height: 100vh;
		overflow: hidden;
	}
	.sidebar-drawer {
		display: contents;
	}
	.main-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 12px 16px;
		overflow-y: auto;
		min-width: 0;
	}
	.hamburger {
		display: none;
	}
	.drawer-backdrop {
		display: none;
	}

	@media (max-width: 767px) {
		.app-layout {
			overflow-y: auto;
		}
		.sidebar-drawer {
			display: block;
			position: fixed;
			top: 0;
			left: 0;
			z-index: 80;
			transform: translateX(-100%);
			transition: transform 0.3s ease;
		}
		.sidebar-drawer.open {
			transform: translateX(0);
		}
		.hamburger {
			display: flex;
			align-items: center;
			justify-content: center;
			position: fixed;
			top: 8px;
			left: 8px;
			z-index: 70;
			width: 36px;
			height: 36px;
			border-radius: 8px;
			border: 1px solid var(--border);
			background: rgba(15, 17, 23, 0.85);
			color: var(--text);
			cursor: pointer;
			backdrop-filter: blur(4px);
		}
		.drawer-backdrop {
			display: block;
			position: fixed;
			inset: 0;
			background: rgba(0, 0, 0, 0.5);
			z-index: 75;
		}
		.main-content {
			padding: 52px 8px 8px;
		}
	}
</style>
```

**Step 3: Add mobile styles to Sidebar.svelte**

Add this media query inside the `<style>` block of `frontend/src/components/Sidebar.svelte`, after the existing `.scrollable` rule:

```css
@media (max-width: 767px) {
	.sidebar {
		width: 320px;
		min-width: 320px;
	}
}
```

This ensures on mobile the sidebar retains its 320px width (since the CSS variable is set to 0px globally, but the sidebar itself needs its real width when shown as a drawer).

**Step 4: Verify build and visual test**

Run: `cd frontend && npm run check`
Expected: No errors

Run: `cd frontend && npm run build`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add frontend/src/routes/+page.svelte frontend/src/components/Sidebar.svelte
git commit -m "feat: sidebar-to-drawer on mobile with hamburger button"
```

---

### Task 3: UnifiedChart — mobile legend and margins

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte`

**Step 1: Make margins responsive to container width**

In `UnifiedChart.svelte`, replace the fixed `margin` constant (line 34):

```typescript
const margin = { top: 32, right: 180, bottom: 32, left: 48 };
```

With a derived value based on width:

```typescript
let margin = $derived(
	width < 500
		? { top: 24, right: 16, bottom: 80, left: 36 }
		: { top: 32, right: 180, bottom: 32, left: 48 }
);
```

The `bottom: 80` reserves space for the legend below the chart on mobile.

**Step 2: Move legend below chart when narrow**

In the `$effect` block, find the legend group positioning (around line 520-524):

```typescript
const legendGroup = svg.selectAll<SVGGElement, null>('g.legend')
	.data([null])
	.join('g')
	.attr('class', 'legend')
	.attr('transform', `translate(${margin.left + innerW + 16}, ${margin.top + 8})`);
```

Replace with width-aware positioning:

```typescript
const isNarrow = width < 500;
const legendGroup = svg.selectAll<SVGGElement, null>('g.legend')
	.data([null])
	.join('g')
	.attr('class', 'legend');

if (isNarrow) {
	// Legend below chart: horizontally laid out
	legendGroup.attr('transform', `translate(${margin.left}, ${margin.top + innerH + 24})`);
} else {
	legendGroup.attr('transform', `translate(${margin.left + innerW + 16}, ${margin.top + 8})`);
}
```

**Step 3: Horizontal legend layout on mobile**

Find the legend items join (around line 557). In the enter handler, replace the fixed vertical layout:

```typescript
.attr('transform', (_, i) => `translate(0, ${i * 22})`)
```

With width-aware layout:

```typescript
.attr('transform', (_, i) => {
	if (isNarrow) {
		const col = i % 3;
		const row = Math.floor(i / 3);
		return `translate(${col * Math.floor(innerW / 3)}, ${row * 18})`;
	}
	return `translate(0, ${i * 22})`;
})
```

And in the update handler, apply the same transform logic:

```typescript
.attr('transform', (_, i) => {
	if (isNarrow) {
		const col = i % 3;
		const row = Math.floor(i / 3);
		return `translate(${col * Math.floor(innerW / 3)}, ${row * 18})`;
	}
	return `translate(0, ${i * 22})`;
})
```

**Step 4: Verify build**

Run: `cd frontend && npm run check`
Expected: No errors

**Step 5: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: responsive chart margins and legend layout"
```

---

### Task 4: Touch support for chart tooltips

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte`

**Step 1: Add touch event handlers to the overlay**

In the `$effect` block, find the overlay event handlers (around line 682-735). After the `.on('mouseleave', ...)` handler, add touch handlers:

```typescript
		.on('touchstart', (event: TouchEvent) => {
			event.preventDefault();  // prevent scroll while interacting with chart
			handlePointerMove(event);
		}, { passive: false })
		.on('touchmove', (event: TouchEvent) => {
			event.preventDefault();
			handlePointerMove(event);
		}, { passive: false })
		.on('touchend', () => {
			setTimeout(() => {
				tooltipLine.style('display', 'none');
				tooltipVisible = false;
				hoveredYear.set(null);
			}, 300);
		});
```

**Step 2: Extract pointer logic into shared function**

Refactor the `mousemove` handler body into a shared function. Before the overlay event binding, add:

```typescript
	function handlePointerMove(event: MouseEvent | TouchEvent) {
		if (isBrushing) return;

		const [mx] = d3.pointer(event, overlay.node()!);
		const year = Math.round(xScale.invert(mx));

		tooltipLine
			.style('display', null)
			.attr('x1', xScale(year))
			.attr('x2', xScale(year));

		const items: typeof tooltipItems = [];
		for (const ld of linesData) {
			const fmt = getFormatter(ld.format);
			const idx = ld.rawPoints.findIndex((p) => Math.round(p.year) === year);
			if (idx >= 0) {
				const pt = ld.rawPoints[idx];
				let trend = '';
				if (idx > 0) {
					const prev = ld.rawPoints[idx - 1].value;
					const diff = pt.value - prev;
					const pct = prev !== 0 ? Math.abs(diff / prev) : 0;
					if (pct < 0.001) trend = '\u2192';
					else if (diff > 0) trend = '\u2191';
					else trend = '\u2193';
				}
				items.push({
					label: ld.label,
					color: ld.color,
					rawValue: fmt(pt.value),
					unit: '',
					trend
				});
			}
		}

		tooltipYear = year;
		tooltipItems = items;
		hoveredYear.set(year);

		const px = margin.left + xScale(year);
		tooltipX = px + 12;
		if (tooltipX + 200 > width) {
			tooltipX = px - 210;
		}
		tooltipY = margin.top + 8;
		tooltipVisible = true;
	}
```

Then simplify the mousemove handler to:

```typescript
	overlay
		.on('mousemove', handlePointerMove)
		.on('mouseleave', () => {
			tooltipLine.style('display', 'none');
			tooltipVisible = false;
			hoveredYear.set(null);
		})
```

**Step 3: Verify build**

Run: `cd frontend && npm run check`
Expected: No errors

**Step 4: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: add touch support for chart tooltips"
```

---

### Task 5: ChartGrid mobile scroll

**Files:**
- Modify: `frontend/src/components/ChartGrid.svelte`

**Step 1: Add mobile media query**

Add a media query to the `<style>` block of `ChartGrid.svelte`:

```css
@media (max-width: 767px) {
	.chart-section {
		min-height: 250px;
	}
}
```

**Step 2: Commit**

```bash
git add frontend/src/components/ChartGrid.svelte
git commit -m "feat: reduce chart min-height on mobile"
```

---

### Task 6: Responsive sparkline in ParameterInfoPanel

**Files:**
- Modify: `frontend/src/components/ParameterInfoPanel.svelte`

**Step 1: Make sparkline responsive**

In `ParameterInfoPanel.svelte`, replace the hardcoded dimensions in `drawSparkline()` (line 88-89):

```typescript
	const W = 280,
		H = 70;
```

With container-aware sizing:

```typescript
	const rect = el.getBoundingClientRect();
	const W = Math.max(120, Math.floor(rect.width));
	const H = 70;
```

**Step 2: Verify build**

Run: `cd frontend && npm run check`
Expected: No errors

**Step 3: Commit**

```bash
git add frontend/src/components/ParameterInfoPanel.svelte
git commit -m "feat: responsive sparkline dimensions"
```

---

### Task 7: InfoPanelShell mobile full-width

**Files:**
- Modify: `frontend/src/components/InfoPanelShell.svelte`

**Step 1: Add mobile styles**

Add a media query to the `<style>` block of `InfoPanelShell.svelte`:

```css
@media (max-width: 767px) {
	.info-panel {
		width: 100vw;
		max-width: 100vw;
	}
	.close-btn {
		font-size: 28px;
		padding: 4px 8px;
	}
}
```

**Step 2: Commit**

```bash
git add frontend/src/components/InfoPanelShell.svelte
git commit -m "feat: full-width info panels on mobile"
```

---

### Task 8: Final verification

**Step 1: Run full frontend checks**

Run: `cd frontend && npm run check`
Expected: No errors

Run: `cd frontend && npm run build`
Expected: Build succeeds

Run: `cd frontend && npm test`
Expected: All tests pass

**Step 2: Manual test (if dev server available)**

Run: `cd frontend && npm run dev`
Open browser devtools → responsive mode → iPhone 14 (390px wide).
Verify:
- Charts take full width
- Hamburger button visible top-left
- Clicking hamburger opens sidebar drawer with backdrop
- Clicking backdrop closes drawer
- Chart tooltips work with touch/click
- Info panels open full-width
- Scenario bar chips wrap correctly

**Step 3: Commit any remaining fixes, then final commit**

```bash
git add -A
git commit -m "feat: mobile responsive layout complete"
```
