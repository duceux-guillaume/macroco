# Click-to-Info-Panel Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make chart lines clickable — click/tap a line to open its variable info panel.

**Architecture:** Add invisible wide hit-area SVG paths on top of visible lines. SVG handles hit-testing natively. On click, call existing `handleLegendSelect()` / `selectedHistoricalId.set()`. Desktop hover tooltip unchanged; mobile tap-on-background tooltip unchanged.

**Tech Stack:** Svelte 5, D3 v7, existing stores (`selectedVariableId`, `selectedHistoricalId`)

**REQ:** REQ-041

---

### Task 1: Add `fieldPath` to LineDatum interface

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte:119-127`

**Step 1: Add the field**

Add `fieldPath?: string` to the `LineDatum` interface at line 127 (before the closing `}`):

```typescript
interface LineDatum {
    id: string;
    color: string;
    points: Array<{ year: number; y: number }>;
    rawPoints: Array<{ year: number; value: number }>;
    label: string;
    format: string;
    historical?: boolean;
    fieldPath?: string;
}
```

**Step 2: Populate fieldPath in normal mode**

In the normal-mode line construction (~line 237), add `fieldPath: varConfig.fieldPath`:

```typescript
linesData.push({
    id: varConfig.id,
    color: varConfig.color,
    points: rawPoints.map((p) => ({ year: p.year, y: normalize(p.value) })),
    rawPoints,
    label: varConfig.shortLabel,
    format: varConfig.format,
    fieldPath: varConfig.fieldPath
});
```

Also add it to the historical overlay line (~line 248):

```typescript
linesData.push({
    id: `${HIST_LINE_PREFIX}${varConfig.id}`,
    color: varConfig.color,
    points: histVar.data.map((d) => ({ year: d.year, y: normalize(d.value) })),
    rawPoints: histVar.data.map((d) => ({ year: d.year, value: d.value })),
    label: `${varConfig.shortLabel} hist.`,
    format: varConfig.format,
    historical: true,
    fieldPath: varConfig.fieldPath
});
```

In compare mode (~line 161), add `fieldPath: varConfig.fieldPath` to each entry too.

**Step 3: Verify no regressions**

Run: `cd frontend && npm run check`
Expected: No type errors, no regressions.

**Step 4: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "refactor: add fieldPath to LineDatum for line click handling"
```

---

### Task 2: Add hit-area paths and click handler

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte:408-441` (after var-line join)

**Step 1: Add handleLineClick function**

After `handleItemClick` (~line 599), add:

```typescript
function handleLineClick(event: MouseEvent, d: LineDatum) {
    event.stopPropagation();
    if (d.historical) {
        selectedHistoricalId.set(get(selectedHistoricalId) ? null : 'historical');
    } else if (!_compareMode && d.fieldPath) {
        handleLegendSelect(d.fieldPath);
    }
}
```

**Step 2: Add hit-area paths after the var-line join**

After the `lines.join(...)` block (after line 441), insert:

```typescript
// Invisible hit-area paths for click/tap on lines
const hitLines = clipped.selectAll<SVGPathElement, LineDatum>('path.hit-line')
    .data(linesData, (d) => d.id);

hitLines.join(
    (enter) =>
        enter
            .append('path')
            .attr('class', 'hit-line')
            .attr('fill', 'none')
            .attr('stroke', 'transparent')
            .attr('stroke-width', 20)
            .attr('pointer-events', 'stroke')
            .attr('cursor', 'pointer')
            .attr('d', (d) => line(d.points))
            .on('click', handleLineClick),
    (update) =>
        update
            .attr('d', (d) => line(d.points)),
    (exit) => exit.remove()
);
```

**Step 3: Verify no regressions**

Run: `cd frontend && npm run check`
Expected: No type errors.

**Step 4: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: add invisible hit-area paths for click-to-info-panel (REQ-038)"
```

---

### Task 3: Update hit-area paths during zoom

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte:730-735` (inside `applyZoomTransform`)

**Step 1: Add hit-line update alongside var-line update**

After line 735 (`clipped.selectAll...('path.var-line').attr('d', ...)`), add:

```typescript
clipped.selectAll<SVGPathElement, LineDatum>('path.hit-line')
    .attr('d', (d) => zLine(d.points));
```

**Step 2: Verify zoom still works**

Run: `cd frontend && npm run check`
Expected: No errors.

**Step 3: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "fix: update hit-area paths during zoom transform"
```

---

### Task 4: Desktop hover feedback on hit-area paths

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte` (hit-line enter block from Task 2)

**Step 1: Add mouseenter/mouseleave handlers to hit-line enter**

Extend the hit-line enter chain (from Task 2) with hover handlers. Add after `.on('click', handleLineClick)`:

```typescript
.on('mouseenter', function (_event: MouseEvent, d: LineDatum) {
    if (isTouchDevice) return;
    clipped.selectAll<SVGPathElement, LineDatum>('path.var-line')
        .filter((ld) => ld.id === d.id)
        .attr('stroke-width', (ld) => getLineWidth(ld) + 1.5);
})
.on('mouseleave', function (_event: MouseEvent, _d: LineDatum) {
    if (isTouchDevice) return;
    clipped.selectAll<SVGPathElement, LineDatum>('path.var-line')
        .attr('stroke-width', (ld) => getLineWidth(ld));
})
```

**Step 2: Manual test**

- Desktop: hover over a line — it should thicken slightly. Move away — back to normal.
- Click a line — info panel opens. Click again — toggles off.
- Mobile (devtools device emulation): tap a line — info panel opens. Tap background — tooltip shows.

**Step 3: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: desktop hover feedback on chart lines"
```

---

### Task 5: Dismiss tooltip on line click (mobile)

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte` (handleLineClick from Task 2)

**Step 1: Add tooltip dismissal to handleLineClick**

Update `handleLineClick` to dismiss the tooltip when a line is tapped (so the info panel isn't competing with the tooltip):

```typescript
function handleLineClick(event: MouseEvent, d: LineDatum) {
    event.stopPropagation();
    dismissTooltip();
    if (d.historical) {
        selectedHistoricalId.set(get(selectedHistoricalId) ? null : 'historical');
    } else if (!_compareMode && d.fieldPath) {
        handleLegendSelect(d.fieldPath);
    }
}
```

**Step 2: Verify and commit**

Run: `cd frontend && npm run check && npm test`

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "fix: dismiss tooltip when clicking a chart line"
```

---

### Task 6: Final verification and squash

**Step 1: Run full frontend checks**

```bash
cd frontend && npm run check && npm test && npm run build
```

**Step 2: Manual smoke test**

1. Desktop: hover shows tooltip, click a line opens info panel, click legend still works, zoom/pan works
2. Mobile (devtools): tap line opens info panel, tap background shows tooltip, pinch zoom works
3. Escape key closes info panel
4. Number keys still select variables

**Step 3: Squash into a single commit**

```bash
git rebase -i HEAD~5
# Squash all into one commit:
# feat: click/tap chart lines to open info panel (REQ-038)
```

**Step 4: Update traceability (if tests added)**

```bash
python3 scripts/traceability.py
```
