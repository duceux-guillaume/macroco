# REQ-023: Simulation Controls — Test Coverage Design

**Date:** 2026-03-04
**REQ:** REQ-023 (Simulation controls)
**Status:** Uncovered Done requirement → adding tests

## Problem

`SimulationControls.svelte` has no test coverage. Its core logic — optimistic cache updates with debounced WebSocket sends — is embedded in the component's `<script>` block and untestable without component mounting.

## Approach

Extract the `updateField` logic into a standalone TypeScript module (`simulation-controls.ts`), then unit-test it directly. The Svelte component becomes a thin view layer that imports the extracted function.

This follows the project's existing pattern: stores in `.ts` files, tests in `.test.ts` files using `vitest` + `svelte/store` `get()`.

## Files

| File | Action |
|------|--------|
| `frontend/src/lib/stores/simulation-controls.ts` | **New** — extracted `updateSimField()`, debounce timer, `getSimParams()` |
| `frontend/src/components/SimulationControls.svelte` | **Edit** — import from new module, remove inline logic |
| `frontend/src/lib/stores/simulation-controls.test.ts` | **New** — 9 unit tests |

## Extracted API

```typescript
// frontend/src/lib/stores/simulation-controls.ts
import { get } from 'svelte/store';
import { focusedScenarioId, scenarioParamsCache } from './scenarios';
import { send } from '../ws';
import type { ScenarioParams } from '../types';

export const startYearOptions = [1900, 1950, 1970, 2000];
export const endYearOptions = [2050, 2100, 2150, 2200];
export const timeStepOptions = [0.25, 0.5, 1.0, 2.0];

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

export function getSimParams(): ScenarioParams | null {
  const id = get(focusedScenarioId);
  if (!id) return null;
  return get(scenarioParamsCache).get(id) ?? null;
}

export function updateSimField(field: string, value: number): void {
  const id = get(focusedScenarioId);
  if (!id) return;
  const params = getSimParams();
  if (!params) return;

  const updated = { ...params, [field]: value };
  scenarioParamsCache.update((cache) => {
    const next = new Map(cache);
    next.set(id, updated);
    return next;
  });

  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    send({ type: 'update_params', scenario_id: id, params: updated });
  }, 200);
}
```

## Test Plan

**File:** `frontend/src/lib/stores/simulation-controls.test.ts`
**REQ annotation:** `// REQ: REQ-023`

| # | Test | Assert |
|---|------|--------|
| 1 | `getSimParams` returns null when no focused scenario | `getSimParams()` is null |
| 2 | `getSimParams` returns null when ID has no cached params | `getSimParams()` is null |
| 3 | `getSimParams` returns params for focused scenario | `getSimParams()` equals cached params |
| 4 | `updateSimField` updates cache immediately | `get(scenarioParamsCache).get(id).start_year` equals new value synchronously |
| 5 | `updateSimField` sends WS after 200ms | After `vi.advanceTimersByTime(200)`, `send` called once with correct payload |
| 6 | Rapid updates debounce to single send | 3 calls in 50ms → advance 200ms → `send` called once with last value |
| 7 | No-op without focused scenario | `focusedScenarioId` is null → cache unchanged, `send` not called |
| 8 | No-op if params missing from cache | Focused ID set but cache empty → `send` not called |
| 9 | Each field updates independently | `start_year`, `end_year`, `time_step` each produce correct merged params |

**Mocking:**
- `vi.mock('$lib/ws')` — mock `send` function
- `vi.useFakeTimers()` — control debounce timing
- Store resets in `beforeEach` (same pattern as `stores.test.ts`)

## Component Refactor

`SimulationControls.svelte` changes from:
```svelte
<script>
  // inline getParams, updateField, debounceTimer, options
</script>
```
to:
```svelte
<script>
  import { focusedScenarioId, scenarioParamsCache } from '$lib/stores/scenarios';
  import { startYearOptions, endYearOptions, timeStepOptions, getSimParams, updateSimField } from '$lib/stores/simulation-controls';
</script>
```

No visual/behavioral changes. Same HTML, same CSS.
