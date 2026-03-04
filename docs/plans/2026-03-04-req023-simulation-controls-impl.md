# REQ-023 Simulation Controls Test Coverage — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add test coverage for REQ-023 (Simulation Controls) by extracting component logic into a testable module and writing 6 unit tests.

**Architecture:** Extract `updateField`/`getParams`/options from `SimulationControls.svelte` into `frontend/src/lib/stores/simulation-controls.ts`. Test the extracted module directly using vitest with fake timers and mocked WS `send`. Refactor the Svelte component to import from the new module.

**Tech Stack:** TypeScript, vitest, svelte/store `get()`, `vi.useFakeTimers()`, `vi.mock()`

---

### Task 1: Create the extracted module

**Files:**
- Create: `frontend/src/lib/stores/simulation-controls.ts`

**Step 1: Write the module**

```typescript
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

**Step 2: Verify it compiles**

Run: `cd frontend && npx tsc --noEmit src/lib/stores/simulation-controls.ts 2>&1 || true`

Note: TypeScript path aliases (`$lib/`) won't resolve with bare `tsc`. The real check is in Step 4 of Task 2 when we run vitest. Just verify no obvious syntax errors here.

**Step 3: Commit**

```bash
git add frontend/src/lib/stores/simulation-controls.ts
git commit -m "feat: extract simulation controls logic into testable module"
```

---

### Task 2: Write the test file

**Files:**
- Create: `frontend/src/lib/stores/simulation-controls.test.ts`

**Step 1: Write all 6 tests**

The test file mocks `$lib/ws` (so `send` is a `vi.fn()`), uses `vi.useFakeTimers()` to control debounce, and resets stores in `beforeEach`. It uses dynamic `import()` after `vi.resetModules()` so the module picks up the mocked `send`.

```typescript
// REQ: REQ-023
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { focusedScenarioId, scenarioParamsCache } from './scenarios';
import type { ScenarioParams } from '../types';

// Mock ws module before importing simulation-controls
vi.mock('$lib/ws', () => ({
	send: vi.fn(),
	connect: vi.fn(),
	disconnect: vi.fn(),
	connectionState: { subscribe: vi.fn() },
	onServerMessage: vi.fn()
}));

// Dynamic import so mock is in place
let simControls: typeof import('./simulation-controls');
let mockSend: ReturnType<typeof vi.fn>;

function makeParams(overrides: Partial<ScenarioParams> = {}): ScenarioParams {
	return {
		meta: {
			id: 'test-id',
			name: 'Test',
			description: 'Test scenario',
			color_hex: '#ff0000',
			created_at: '2024-01-01T00:00:00Z'
		},
		family_planning_year: 2000,
		family_planning_efficacy: 0.5,
		health_investment_multiplier: 1.0,
		industrial_depreciation_rate: 0.05,
		service_depreciation_rate: 0.05,
		technology_growth_rate: 0.01,
		agricultural_technology: 1.0,
		land_protection_fraction: 0.1,
		subsistence_food_per_capita: 230,
		resource_efficiency: 1.0,
		initial_nnr_fraction: 1.0,
		pollution_control: 1.0,
		start_year: 1900,
		end_year: 2100,
		time_step: 1.0,
		...overrides
	};
}

beforeEach(async () => {
	vi.useFakeTimers();
	vi.resetModules();

	// Reset stores to clean state
	focusedScenarioId.set(null);
	scenarioParamsCache.set(new Map());

	// Re-import to get fresh module state (debounceTimer reset)
	simControls = await import('./simulation-controls');
	const ws = await import('../ws');
	mockSend = ws.send as ReturnType<typeof vi.fn>;
	mockSend.mockClear();
});

afterEach(() => {
	vi.useRealTimers();
});

describe('getSimParams', () => {
	it('returns null when no focused scenario', () => {
		expect(simControls.getSimParams()).toBeNull();
	});

	it('returns null when focused ID has no cached params', () => {
		focusedScenarioId.set('missing-id');
		expect(simControls.getSimParams()).toBeNull();
	});

	it('returns params for focused scenario', () => {
		const params = makeParams();
		focusedScenarioId.set('bau');
		scenarioParamsCache.set(new Map([['bau', params]]));
		expect(simControls.getSimParams()).toEqual(params);
	});
});

describe('updateSimField', () => {
	it('updates cache immediately', () => {
		const params = makeParams({ start_year: 1900 });
		focusedScenarioId.set('bau');
		scenarioParamsCache.set(new Map([['bau', params]]));

		simControls.updateSimField('start_year', 1950);

		const cached = get(scenarioParamsCache).get('bau')!;
		expect(cached.start_year).toBe(1950);
		// Other fields unchanged
		expect(cached.end_year).toBe(2100);
	});

	it('sends WS message after 200ms debounce', () => {
		const params = makeParams({ start_year: 1900 });
		focusedScenarioId.set('bau');
		scenarioParamsCache.set(new Map([['bau', params]]));

		simControls.updateSimField('start_year', 1950);

		// Not sent yet
		expect(mockSend).not.toHaveBeenCalled();

		// Advance past debounce
		vi.advanceTimersByTime(200);

		expect(mockSend).toHaveBeenCalledOnce();
		expect(mockSend).toHaveBeenCalledWith({
			type: 'update_params',
			scenario_id: 'bau',
			params: expect.objectContaining({ start_year: 1950 })
		});
	});

	it('debounces rapid updates to single WS send', () => {
		const params = makeParams({ end_year: 2100 });
		focusedScenarioId.set('bau');
		scenarioParamsCache.set(new Map([['bau', params]]));

		simControls.updateSimField('end_year', 2050);
		vi.advanceTimersByTime(50);
		simControls.updateSimField('end_year', 2150);
		vi.advanceTimersByTime(50);
		simControls.updateSimField('end_year', 2200);

		// Advance past debounce from last call
		vi.advanceTimersByTime(200);

		expect(mockSend).toHaveBeenCalledOnce();
		expect(mockSend).toHaveBeenCalledWith({
			type: 'update_params',
			scenario_id: 'bau',
			params: expect.objectContaining({ end_year: 2200 })
		});
	});

	it('no-ops without focused scenario', () => {
		// focusedScenarioId is null (from beforeEach)
		simControls.updateSimField('start_year', 1950);

		vi.advanceTimersByTime(200);
		expect(mockSend).not.toHaveBeenCalled();
		expect(get(scenarioParamsCache).size).toBe(0);
	});

	it('no-ops if params missing from cache', () => {
		focusedScenarioId.set('bau');
		// scenarioParamsCache is empty (from beforeEach)

		simControls.updateSimField('start_year', 1950);

		vi.advanceTimersByTime(200);
		expect(mockSend).not.toHaveBeenCalled();
	});

	it('updates each field independently', () => {
		const params = makeParams({
			start_year: 1900,
			end_year: 2100,
			time_step: 1.0
		});
		focusedScenarioId.set('bau');
		scenarioParamsCache.set(new Map([['bau', params]]));

		simControls.updateSimField('start_year', 1970);
		const after1 = get(scenarioParamsCache).get('bau')!;
		expect(after1.start_year).toBe(1970);
		expect(after1.end_year).toBe(2100);
		expect(after1.time_step).toBe(1.0);

		simControls.updateSimField('end_year', 2200);
		const after2 = get(scenarioParamsCache).get('bau')!;
		expect(after2.start_year).toBe(1970);
		expect(after2.end_year).toBe(2200);
		expect(after2.time_step).toBe(1.0);

		simControls.updateSimField('time_step', 0.5);
		const after3 = get(scenarioParamsCache).get('bau')!;
		expect(after3.start_year).toBe(1970);
		expect(after3.end_year).toBe(2200);
		expect(after3.time_step).toBe(0.5);
	});
});
```

**Step 2: Run the tests — expect all 9 to pass**

Run: `cd frontend && npx vitest run src/lib/stores/simulation-controls.test.ts`

Expected: 9 tests pass (3 in `getSimParams`, 6 in `updateSimField`).

**Step 3: Commit**

```bash
git add frontend/src/lib/stores/simulation-controls.test.ts
git commit -m "test: add REQ-023 simulation controls unit tests (9 tests)"
```

---

### Task 3: Refactor the Svelte component

**Files:**
- Modify: `frontend/src/components/SimulationControls.svelte`

**Step 1: Replace inline logic with imports**

The `<script>` block changes to:

```svelte
<script lang="ts">
	import { focusedScenarioId, scenarioParamsCache } from '$lib/stores/scenarios';
	import {
		startYearOptions,
		endYearOptions,
		timeStepOptions,
		getSimParams,
		updateSimField
	} from '$lib/stores/simulation-controls';
</script>
```

The template changes `getParams()` → `getSimParams()` and `updateField(` → `updateSimField(` (3 occurrences each). Everything else (HTML, CSS, `{#if}` block) stays identical.

**Step 2: Verify existing frontend tests still pass**

Run: `cd frontend && npx vitest run`

Expected: All existing tests pass + 9 new tests pass.

**Step 3: Verify frontend type-checks**

Run: `cd frontend && npm run check`

Expected: No errors.

**Step 4: Commit**

```bash
git add frontend/src/components/SimulationControls.svelte
git commit -m "refactor: SimulationControls uses extracted module"
```

---

### Task 4: Update traceability matrix

**Step 1: Run traceability script**

Run: `cd /Users/guillaume/Documents/macroco/.claude/worktrees/real-more-coverage && python3 scripts/traceability.py`

Expected: REQ-023 now shows test coverage from `simulation-controls.test.ts`. Uncovered count drops from 3 to 2.

**Step 2: Commit**

```bash
git add docs/traceability-matrix.md
git commit -m "docs: update traceability matrix with REQ-023 coverage"
```
