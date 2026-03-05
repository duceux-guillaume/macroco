# Chart Zoom Design (REQ-TBD)

**Date:** 2026-03-05
**Status:** Approved

## Problem

Mobile UX is poor: 6 variables crammed into ~300px, can't inspect specific time periods, touch interactions (tooltip) feel broken. Desktop also benefits from zoom.

## Decision: Approach A — Replace Brush with d3.zoom

The existing `d3.brushX` is visual-only (highlights a year range, feeds no state). Replace it with `d3.zoom()` which handles wheel, drag, pinch, and double-click/tap across all devices.

### Zoom Behavior

- **X-axis only zoom.** Y-axis auto-fits visible data (normalized mode stays [0,1]; compare mode recalculates min/max for visible year window).
- **Scale limits:** `scaleExtent([1, 20])` — 1x = full 1900–2100, 20x ≈ 10-year window.
- **Pan clamping:** `translateExtent` clamped to data year range — can't pan past edges.
- **Reset:** Double-click (desktop) / double-tap (mobile) → smooth 400ms transition to full view.
- **Transform storage:** Local variable. Zoomed X scale = `transform.rescaleX(baseXScale)`. All rendering uses zoomed scale.

### Tooltip Handling

- **Desktop:** Hover shows tooltip — unchanged.
- **Mobile (pointer: coarse):** Tap to show tooltip at that year (pinned). Tap elsewhere moves it. Pinch/pan hides it. Tap outside or Escape dismisses.
- **Detection:** `window.matchMedia('(pointer: coarse)')` — correctly handles tablets, touchscreen laptops.

### Visual Feedback

- **Reset button:** When zoomed (transform.k > 1), show "Reset zoom" button in top-right of chart area.
- **Axis animation:** 400ms transitions on axis ticks/labels during zoom/pan.
- **No minimap/scrollbar.**

## Implementation Scope

**Single file changed:** `UnifiedChart.svelte`

- Remove: brush setup (~30 lines), `activeBrush`/`activeBrushGroup` refs, brush Escape handling
- Add: `d3.zoom()` setup with `rescaleX`, zoom transform state, reset button
- Modify: tooltip handler — desktop hover, mobile tap
- Modify: Y-axis recalculation in compare mode for visible year window

No new files, no new dependencies (D3 v7 includes d3-zoom), no store changes.

## Alternatives Considered

### B: Brush Becomes Zoom
Brush end → zoom into selected range. Pro: feels powerful on desktop. Con: no good pan story, not natural on mobile, mixing brush + zoom has D3 gesture conflicts.

### C: Keep Both (Brush + Zoom)
Toggle between brush mode and zoom mode. Pro: preserves all functionality. Con: most complex, brush does nothing useful today, three gesture systems competing on mobile.
