# Chart Zoom Design & Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the no-op brush with d3.zoom for X-axis zoom/pan on both desktop and mobile.

**Architecture:** Single-file change to `UnifiedChart.svelte`. Replace `d3.brushX` with `d3.zoom()`. The zoom transform rescales the X axis; all rendering (lines, axes, annotations, tooltip, now-line) uses the zoomed scale. Mobile tooltip switches from drag-follow to tap-to-pin.

**Tech Stack:** Svelte 5, D3 v7 (d3-zoom included), TypeScript

---

## Design

**Date:** 2026-03-05
**Status:** Approved

### Problem

Mobile UX is poor: 6 variables crammed into ~300px, can't inspect specific time periods, touch interactions (tooltip) feel broken. Desktop also benefits from zoom.

### Decision: Approach A — Replace Brush with d3.zoom

The existing `d3.brushX` is visual-only (highlights a year range, feeds no state). Replace it with `d3.zoom()` which handles wheel, drag, pinch, and double-click/tap across all devices.

#### Zoom Behavior

- **X-axis only zoom.** Y-axis auto-fits visible data (normalized mode stays [0,1]; compare mode recalculates min/max for visible year window).
- **Scale limits:** `scaleExtent([1, 20])` — 1x = full 1900–2100, 20x ≈ 10-year window.
- **Pan clamping:** `translateExtent` clamped to data year range — can't pan past edges.
- **Reset:** Double-click (desktop) / double-tap (mobile) → smooth 400ms transition to full view.
- **Transform storage:** Local variable. Zoomed X scale = `transform.rescaleX(baseXScale)`. All rendering uses zoomed scale.

#### Tooltip Handling

- **Desktop:** Hover shows tooltip — unchanged.
- **Mobile (pointer: coarse):** Tap to show tooltip at that year (pinned). Tap elsewhere moves it. Pinch/pan hides it. Tap outside or Escape dismisses.
- **Detection:** `window.matchMedia('(pointer: coarse)')` — correctly handles tablets, touchscreen laptops.

#### Visual Feedback

- **Reset button:** When zoomed (transform.k > 1), show "Reset zoom" button in top-right of chart area.
- **Axis animation:** 400ms transitions on axis ticks/labels during zoom/pan.
- **No minimap/scrollbar.**

### Alternatives Considered

#### B: Brush Becomes Zoom
Brush end → zoom into selected range. Pro: feels powerful on desktop. Con: no good pan story, not natural on mobile, mixing brush + zoom has D3 gesture conflicts.

#### C: Keep Both (Brush + Zoom)
Toggle between brush mode and zoom mode. Pro: preserves all functionality. Con: most complex, brush does nothing useful today, three gesture systems competing on mobile.

---

## Implementation Plan

All changes are in **one file**: `frontend/src/lib/charts/UnifiedChart.svelte` (890 lines). No new files, no new dependencies.

### Task 1: Remove Brush Code

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte:41-49,86-88,672-704`

**Step 1: Remove brush state variables and refs**

Delete these lines:

```typescript
// Line 41 — delete:
let isBrushing = false;

// Lines 47-49 — delete:
// Store brush reference for keyboard clear
let activeBrushGroup: d3.Selection<SVGGElement, null, null, undefined> | null = null;
let activeBrush: d3.BrushBehavior<null> | null = null;
```

**Step 2: Remove brush Escape handler**

In `handleKeydown()` (line 75), delete the brush-clearing block (lines 85-89):

```typescript
// Delete these lines:
			// Then clear brush
			if (activeBrushGroup && activeBrush) {
				activeBrushGroup.call(activeBrush.move, null);
				return;
			}
```

**Step 3: Remove brush setup in `$effect`**

Delete lines 672-704 (the entire brush section):

```typescript
// Delete from "// Brush (below tooltip overlay)" through the brushGroup styling
```

**Step 4: Remove `isBrushing` guard in `handlePointerMove`**

Delete line 728:

```typescript
// Delete:
if (isBrushing) return;
```

**Step 5: Verify it builds**

Run: `cd frontend && npm run check`
Expected: No type errors. Chart renders without brush.

**Step 6: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "refactor: remove unused brush from UnifiedChart"
```

---

### Task 2: Add d3.zoom with X-axis Rescaling

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte`

**Step 1: Add zoom transform state**

After line 34 (tooltipItems), add:

```typescript
let currentTransform = $state(d3.zoomIdentity);
let isZoomed = $derived(currentTransform.k > 1.01);
```

**Step 2: Store base X scale and create zoomed scale**

Inside the `$effect`, after the `xScale` creation (line 276), add:

```typescript
const baseXScale = xScale.copy();
const zoomedX = currentTransform.rescaleX(baseXScale);
```

**Step 3: Replace all `xScale` references with `zoomedX`**

In the `$effect` body, replace every `xScale(...)` and `xScale.invert(...)` with `zoomedX(...)` and `zoomedX.invert(...)`. This affects:
- Line generator (line 294-296)
- X axis call (line 321)
- Now-line position (lines 414-416, 432-434)
- Annotation positions (lines 464, 489, 497, 509, 515)
- Tooltip line position (lines 735-736)
- Tooltip year inversion (line 731)
- Tooltip px calculation (line 767)

Keep `baseXScale` for the zoom behavior's internal scale reference.

**Step 4: Set up d3.zoom behavior**

Where the brush code used to be (after the annotation section, before the overlay), add:

```typescript
// Zoom behavior (X-axis only)
const zoom = d3.zoom<SVGSVGElement, null>()
    .scaleExtent([1, 20])
    .translateExtent([[0, 0], [innerW, innerH]])
    .extent([[0, 0], [innerW, innerH]])
    .filter((event: Event) => {
        // Allow wheel, touch, and mouse drag. Block double-click (handled separately for reset).
        if (event.type === 'dblclick') return false;
        return true;
    })
    .on('zoom', (event: d3.D3ZoomEvent<SVGSVGElement, null>) => {
        // Constrain to X-axis only: keep y translation at 0, y scale at 1
        const t = event.transform;
        const constrainedT = d3.zoomIdentity.translate(t.x, 0).scale(t.k);
        currentTransform = constrainedT;
    });

svg.call(zoom);

// Sync stored transform back to SVG (for when data/size changes re-run the effect)
svg.call(zoom.transform, currentTransform);
```

**Step 5: Add double-click/double-tap reset**

After the zoom setup, add:

```typescript
svg.on('dblclick.zoom', null); // Remove d3.zoom's default dblclick handler
svg.on('dblclick', () => {
    svg.transition().duration(400).call(zoom.transform, d3.zoomIdentity);
});
```

**Step 6: Verify it builds and zoom works**

Run: `cd frontend && npm run check`
Run: `cd frontend && npm run dev` — test mouse wheel zoom and drag pan in browser.
Expected: X-axis zooms, lines clip to visible range, axes update.

**Step 7: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: add d3.zoom with X-axis rescaling to UnifiedChart"
```

---

### Task 3: Add SVG Clip Path for Zoomed Lines

When zoomed in, lines extend beyond the chart area. Add a clip path.

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte`

**Step 1: Add clipPath definition to SVG**

After the `svg` join (around line 304), add:

```typescript
// Clip path so lines don't overflow chart area when zoomed
const clipId = 'chart-clip';
const defs = svg.selectAll('defs').data([null]).join('defs');
defs.selectAll(`clipPath#${clipId}`).data([null]).join('clipPath')
    .attr('id', clipId)
    .selectAll('rect').data([null]).join('rect')
    .attr('x', 0).attr('y', 0)
    .attr('width', innerW).attr('height', innerH);
```

**Step 2: Apply clip path to chart group**

On the `g.chart` group, add the clip path:

```typescript
g.attr('clip-path', `url(#${clipId})`);
```

Note: the legend is rendered in a separate `g.legend` group directly under `svg` (not inside `g.chart`), so it won't be clipped. The axes are inside `g.chart` and will be clipped, which is correct — ticks shouldn't overflow either.

**Step 3: Verify clip works**

Run: `cd frontend && npm run dev` — zoom in and verify lines are clipped at chart boundaries.

**Step 4: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: add clip path to prevent line overflow when zoomed"
```

---

### Task 4: Mobile Tooltip — Tap-to-Pin

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte`

**Step 1: Add touch-device detection**

Near the top of the script (after imports), add:

```typescript
import { browser } from '$app/environment';

const isTouchDevice = $derived(
    browser && window.matchMedia('(pointer: coarse)').matches
);
```

**Step 2: Replace touch event handlers on the overlay**

Replace the current overlay touch/mouse event binding (lines 776-797) with:

```typescript
overlay
    .on('mousemove', (event: MouseEvent) => {
        if (!isTouchDevice) handlePointerMove(event);
    })
    .on('mouseleave', () => {
        if (!isTouchDevice) {
            tooltipLine.style('display', 'none');
            tooltipVisible = false;
            hoveredYear.set(null);
        }
    });

// On touch devices: tap to pin tooltip (don't interfere with zoom gestures)
if (isTouchDevice) {
    // d3.zoom handles pinch/pan. We listen for taps (short touches) on the overlay.
    let touchStartTime = 0;
    let touchStartPos = { x: 0, y: 0 };

    svg.on('touchstart.tooltip', (event: TouchEvent) => {
        if (event.touches.length === 1) {
            touchStartTime = Date.now();
            touchStartPos = { x: event.touches[0].clientX, y: event.touches[0].clientY };
        }
    }, { passive: true } as any);

    svg.on('touchend.tooltip', (event: TouchEvent) => {
        const elapsed = Date.now() - touchStartTime;
        if (elapsed > 300) return; // not a tap
        if (event.changedTouches.length !== 1) return;
        const endPos = event.changedTouches[0];
        const dist = Math.hypot(endPos.clientX - touchStartPos.x, endPos.clientY - touchStartPos.y);
        if (dist > 10) return; // was a drag, not a tap

        // Convert to chart coordinates
        const svgNode = svg.node()!;
        const rect = svgNode.getBoundingClientRect();
        const mx = endPos.clientX - rect.left - margin.left;
        const my = endPos.clientY - rect.top - margin.top;

        // Check if tap is inside chart area
        if (mx >= 0 && mx <= innerW && my >= 0 && my <= innerH) {
            const year = Math.round(zoomedX.invert(mx));
            // Reuse handlePointerMove logic but with computed position
            tooltipLine.style('display', null)
                .attr('x1', zoomedX(year))
                .attr('x2', zoomedX(year));

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
                    items.push({ label: ld.label, color: ld.color, rawValue: fmt(pt.value), unit: '', trend });
                }
            }

            tooltipYear = year;
            tooltipItems = items;
            hoveredYear.set(year);
            const px = margin.left + zoomedX(year);
            tooltipX = px + 12;
            if (tooltipX + 200 > width) tooltipX = px - 210;
            tooltipY = margin.top + 8;
            tooltipVisible = true;
        } else {
            // Tap outside chart area — dismiss tooltip
            tooltipLine.style('display', 'none');
            tooltipVisible = false;
            hoveredYear.set(null);
        }
    }, { passive: true } as any);
}
```

**Step 3: Hide tooltip during zoom gestures**

In the zoom `on('zoom', ...)` handler, add tooltip dismissal:

```typescript
.on('zoom', (event: d3.D3ZoomEvent<SVGSVGElement, null>) => {
    const t = event.transform;
    const constrainedT = d3.zoomIdentity.translate(t.x, 0).scale(t.k);
    currentTransform = constrainedT;
    // Dismiss tooltip during zoom/pan
    tooltipVisible = false;
    hoveredYear.set(null);
})
```

**Step 4: Verify builds**

Run: `cd frontend && npm run check`
Expected: No errors.

**Step 5: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: tap-to-pin tooltip on mobile, hover on desktop"
```

---

### Task 5: Reset Zoom Button

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte`

**Step 1: Add reset button to the template**

In the template section (around line 801), after the tooltip div and before the closing `</div>`, add:

```svelte
{#if isZoomed}
    <button
        class="zoom-reset"
        onclick={() => {
            const svg = d3.select(containerEl).select<SVGSVGElement>('svg');
            const zoom = d3.zoom<SVGSVGElement, null>();
            svg.transition().duration(400).call(zoom.transform, d3.zoomIdentity);
        }}
    >
        Reset zoom
    </button>
{/if}
```

Note: The button's zoom reset needs access to the zoom behavior. To make this work, store the zoom behavior in a module-level variable:

At the top (state section), add:

```typescript
let zoomBehavior: d3.ZoomBehavior<SVGSVGElement, null> | null = null;
```

In the `$effect`, after creating `zoom`, store it:

```typescript
zoomBehavior = zoom;
```

Then the button becomes:

```svelte
{#if isZoomed}
    <button
        class="zoom-reset"
        onclick={() => {
            if (!zoomBehavior) return;
            const svg = d3.select(containerEl).select<SVGSVGElement>('svg');
            svg.transition().duration(400).call(zoomBehavior.transform, d3.zoomIdentity);
        }}
    >
        Reset zoom
    </button>
{/if}
```

**Step 2: Add styles**

```css
.zoom-reset {
    position: absolute;
    top: 8px;
    right: 8px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    z-index: 5;
    transition: color 0.15s, border-color 0.15s;
}
.zoom-reset:hover {
    color: var(--text);
    border-color: var(--accent);
}
```

**Step 3: Update Escape key handler**

In `handleKeydown`, add zoom reset to the Escape chain (after clearing selectedVariableId):

```typescript
if (e.key === 'Escape') {
    if (get(selectedHistoricalId)) {
        selectedHistoricalId.set(null);
        return;
    }
    if (get(selectedVariableId)) {
        selectedVariableId.set(null);
        return;
    }
    // Reset zoom
    if (isZoomed && zoomBehavior) {
        const svg = d3.select(containerEl).select<SVGSVGElement>('svg');
        svg.transition().duration(400).call(zoomBehavior.transform, d3.zoomIdentity);
        return;
    }
}
```

**Step 4: Verify**

Run: `cd frontend && npm run check`
Run: `cd frontend && npm run dev` — zoom in, verify button appears, click it, verify reset.

**Step 5: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: add reset zoom button and Escape key support"
```

---

### Task 6: Compare Mode Y-axis Auto-fit for Visible Window

When zoomed in compare mode, the Y-axis should auto-fit to the visible data range, not the full 1900–2100 range.

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte`

**Step 1: Filter Y extent to visible year window**

In the `$effect`, where `yScale` is computed for compare mode (around line 285), replace:

```typescript
const allVals = linesData.flatMap((l) => l.points.map((p) => p.y));
```

with:

```typescript
const [visibleYearStart, visibleYearEnd] = zoomedX.domain();
const allVals = linesData.flatMap((l) =>
    l.points.filter((p) => p.year >= visibleYearStart && p.year <= visibleYearEnd).map((p) => p.y)
);
```

This makes the Y-axis tighten to the zoomed window, showing more detail.

**Step 2: Verify**

Run: `cd frontend && npm run check`
Run: `cd frontend && npm run dev` — switch to compare mode, zoom in, verify Y-axis adjusts.

**Step 3: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: auto-fit Y-axis to visible window in compare mode"
```

---

### Task 7: Final Polish & Manual Testing

**Step 1: Run all checks**

```bash
cd frontend && npm run check && npm test && npm run build
```

**Step 2: Manual test matrix**

Test in browser (desktop + mobile emulation in DevTools):

| Test | Desktop | Mobile (emulated) |
|------|---------|-------------------|
| Mouse wheel zoom | zoom in/out | N/A |
| Click-drag pan | pans X axis | N/A |
| Pinch-to-zoom | N/A (trackpad) | zoom in/out |
| Single-finger pan | N/A | pans X axis |
| Double-click reset | resets to full | N/A |
| Double-tap reset | N/A | resets to full |
| Hover tooltip | follows mouse | N/A |
| Tap tooltip | N/A | pins at tapped year |
| Escape key | resets zoom | N/A |
| Reset button | appears when zoomed, click resets | tap resets |
| Lines clip | don't overflow chart area | same |
| Axes update | ticks match visible range | same |
| Compare mode Y-fit | Y-axis tightens to window | same |
| Legend | unaffected by zoom | same |

**Step 3: Commit final state if any polish needed**

```bash
git add -A
git commit -m "polish: chart zoom final adjustments"
```
