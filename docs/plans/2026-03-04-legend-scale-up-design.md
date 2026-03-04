# Legend Scale-Up Design

**Date:** 2026-03-04
**Status:** Approved

## Problem

The chart legend in UnifiedChart.svelte is too small — text, icons, and swatches all feel undersized.

## Approach

Scale all legend dimensions up in-place (Approach A). No layout restructuring.

## Changes

**File:** `frontend/src/lib/charts/UnifiedChart.svelte`

| Element              | Current | New  |
|----------------------|---------|------|
| Font size            | 11px    | 13px |
| Eye icon             | 12×12   | 16×16 |
| Color swatch width   | 12px    | 16px |
| Color swatch height  | 3px     | 4px  |
| Row spacing          | 22px    | 28px |
| Eye-to-swatch offset | 18px    | 22px |
| Swatch-to-label offset | 34px  | 42px |
| Right margin         | 180px   | 200px |
| Swatch border radius | 1.5px   | 2px  |

## Scope

- Single file change (UnifiedChart.svelte)
- No new components
- All interactivity preserved (eye toggle, click-to-select, keyboard shortcuts, hover styles)
- Compare mode layout unchanged
