# Legend Scale-Up Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the chart legend larger — bigger text, icons, and swatches.

**Architecture:** Single-file edit to `UnifiedChart.svelte`. Scale all legend dimension constants per the approved design in `docs/plans/2026-03-04-legend-scale-up-design.md`.

**Tech Stack:** Svelte 5, D3 v7

---

### Task 1: Scale up legend dimensions

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte`

This is a single coordinated edit — all constants must change together to stay visually consistent.

**Step 1: Update right margin (line 34)**

Change:
```typescript
const margin = { top: 32, right: 180, bottom: 32, left: 48 };
```
To:
```typescript
const margin = { top: 32, right: 200, bottom: 32, left: 48 };
```

**Step 2: Scale eye icon SVG paths from 12×12 to 16×16 viewBox (lines 37-39)**

The current paths are drawn for a 12×12 coordinate space. Scale to 16×16 by multiplying all coordinates by 16/12 ≈ 1.333:

Change:
```typescript
// Eye icon SVG paths (12x12 viewBox)
const EYE_OPEN = 'M1,6 C3,2 9,2 11,6 C9,10 3,10 1,6 Z M6,4.5 a1.5,1.5 0 1,0 0.01,0';
const EYE_CLOSED = 'M1,6 C3,2 9,2 11,6 C9,10 3,10 1,6 Z M2,2 L10,10';
```
To:
```typescript
// Eye icon SVG paths (16x16 viewBox)
const EYE_OPEN = 'M1.3,8 C4,2.7 12,2.7 14.7,8 C12,13.3 4,13.3 1.3,8 Z M8,6 a2,2 0 1,0 0.01,0';
const EYE_CLOSED = 'M1.3,8 C4,2.7 12,2.7 14.7,8 C12,13.3 4,13.3 1.3,8 Z M2.7,2.7 L13.3,13.3';
```

**Step 3: Update row spacing from 22 to 28 (lines 561, 614)**

In the `enter` callback (line 561), change:
```typescript
.attr('transform', (_, i) => `translate(0, ${i * 22})`)
```
To:
```typescript
.attr('transform', (_, i) => `translate(0, ${i * 28})`)
```

In the `update` callback (line 614), same change:
```typescript
.attr('transform', (_, i) => `translate(0, ${i * 22})`)
```
To:
```typescript
.attr('transform', (_, i) => `translate(0, ${i * 28})`)
```

**Step 4: Update eye icon stroke-width (line 575)**

Change:
```typescript
.attr('stroke-width', 1);
```
To:
```typescript
.attr('stroke-width', 1.2);
```

**Step 5: Update color swatch positions and sizes (lines 580-598)**

Change the historical line swatch (x1=18→22, x2=30→38, y=6→7):
```typescript
el.append('line')
    .attr('x1', 18).attr('y1', 6)
    .attr('x2', 30).attr('y2', 6)
    .attr('stroke', d.color)
    .attr('stroke-width', 2)
    .attr('stroke-dasharray', '3,2');
```
To:
```typescript
el.append('line')
    .attr('x1', 22).attr('y1', 8)
    .attr('x2', 38).attr('y2', 8)
    .attr('stroke', d.color)
    .attr('stroke-width', 2)
    .attr('stroke-dasharray', '4,2');
```

Change the simulation rect swatch (x=18→22, width=12→16, height=3→4, y=5→7, rx=1.5→2):
```typescript
el.append('rect')
    .attr('x', 18)
    .attr('width', 12)
    .attr('height', 3)
    .attr('y', 5)
    .attr('rx', 1.5)
    .attr('fill', d.color);
```
To:
```typescript
el.append('rect')
    .attr('x', 22)
    .attr('width', 16)
    .attr('height', 4)
    .attr('y', 7)
    .attr('rx', 2)
    .attr('fill', d.color);
```

**Step 6: Update label text position and font size (lines 601-606)**

Change:
```typescript
item.append('text')
    .attr('x', 34)
    .attr('y', 10)
    .attr('fill', 'var(--text-secondary)')
    .attr('font-size', '11px')
    .text((d) => d.label);
```
To:
```typescript
item.append('text')
    .attr('x', 42)
    .attr('y', 12)
    .attr('fill', 'var(--text-secondary)')
    .attr('font-size', '13px')
    .text((d) => d.label);
```

**Step 7: Run frontend checks**

Run: `cd frontend && npm run check`
Expected: No errors.

**Step 8: Visual test**

Run: `cd frontend && npm run dev`
Verify legend is visually larger in the browser. Check:
- Text is readable at 13px
- Eye icons are proportioned correctly
- Swatches align with text
- No overlap between legend items (28px spacing)
- Legend doesn't overflow the SVG area
- Compare mode legend still looks correct

**Step 9: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: scale up chart legend for readability"
```
