# Mobile Responsive Design — Sidebar-to-Drawer

**Date:** 2026-03-04
**REQ:** TBD (new REQ needed in product-requirements.md)
**Status:** Approved

## Problem

The app has zero responsive CSS. The fixed 320px sidebar leaves ~55px for charts on phones (375px viewport), making the app unusable on mobile. Tooltips are mouse-only. Chart legends consume 180px of right margin.

## Decision

Approach A: Sidebar-to-Drawer. Single breakpoint at 768px. Below this, the sidebar becomes a slide-out drawer, charts take full width, and touch events are added.

**Target:** Modern phones 375px+. View-first experience — charts front and center, controls accessible via drawer.

**Constraints:** No new components, stores, or dependencies. Pure CSS breakpoints (no server-side UA detection). Same SvelteKit codebase.

## Design

### Breakpoint & Layout Switch

- Single breakpoint: `@media (max-width: 767px)`
- `>= 768px`: Current side-by-side layout unchanged
- `< 768px`: Sidebar hidden, charts full-width, hamburger button top-left
- Root `overflow: hidden` replaced with `overflow-y: auto` on mobile

### Drawer Mechanics

- Sidebar: `position: fixed; left: -320px; transition: transform 0.3s ease`
- Open state: `transform: translateX(320px)`
- Semi-transparent backdrop (`rgba(0,0,0,0.5)`), click-to-dismiss
- `sidebarOpen` state in `+page.svelte` (local `$state` rune, not a store)
- Hamburger button: fixed top-left, semi-transparent background

### Chart Adaptations

- **UnifiedChart legend:** When container width < 500px, legend moves from right margin to below the chart as a flex-wrap row of chips. Right margin shrinks from 180px to 16px.
- **Chart margins on mobile:** `margin.left` 48→36px, `margin.right` 180→16px
- **ChartGrid:** Vertical scroll allowed on mobile (no fixed overflow)
- **Sparkline:** Derive dimensions from container width instead of hardcoded 280x70

### Touch Support

- Add `touchstart`/`touchmove`/`touchend` handlers on chart SVG overlay rects
- Use `d3.pointer(event)` (works for both mouse and touch)
- On `touchend`: hide tooltip after 300ms delay
- No pinch-to-zoom — tap/drag scrubs the year crosshair

### Info Panels

- On mobile: `width: 100vw` (full-screen slide-over)
- Larger close button for touch targets
- Hamburger hides when info panel is open

## Changes by File

| File | Change |
|---|---|
| `+page.svelte` | `sidebarOpen` state, hamburger button, media query |
| `Sidebar.svelte` | Fixed position, slide transform, backdrop |
| `app.css` | Mobile media query for sidebar-width, root overflow |
| `UnifiedChart.svelte` | Width-based legend relocation, smaller margins, touch events |
| `TimeSeriesChart.svelte` | Tighter margins, touch events |
| `ChartGrid.svelte` | Remove fixed overflow on mobile |
| `ParameterInfoPanel.svelte` | Responsive sparkline sizing |
| `InfoPanelShell.svelte` | Full-width on mobile, larger close button |

## Not Changing

- No new components, stores, or dependencies
- Scenario bar (already wraps with `flex-wrap`)
- Simulation controls (already flex within sidebar)
- Backend (no changes)
