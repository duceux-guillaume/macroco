// REQ: REQ-023
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { get } from 'svelte/store';
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
let stores: typeof import('./scenarios');
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
		agricultural_technology_growth_rate: 0.005,
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

	// Re-import all modules so they share the same store instances
	stores = await import('./scenarios');
	simControls = await import('./simulation-controls');
	const ws = await import('../ws');
	mockSend = ws.send as ReturnType<typeof vi.fn>;
	mockSend.mockClear();

	// Reset stores to clean state
	stores.focusedScenarioId.set(null);
	stores.scenarioParamsCache.set(new Map());
});

afterEach(() => {
	vi.useRealTimers();
});

describe('getSimParams', () => {
	it('returns null when no focused scenario', () => {
		expect(simControls.getSimParams()).toBeNull();
	});

	it('returns null when focused ID has no cached params', () => {
		stores.focusedScenarioId.set('missing-id');
		expect(simControls.getSimParams()).toBeNull();
	});

	it('returns params for focused scenario', () => {
		const params = makeParams();
		stores.focusedScenarioId.set('bau');
		stores.scenarioParamsCache.set(new Map([['bau', params]]));
		expect(simControls.getSimParams()).toEqual(params);
	});
});

describe('updateSimField', () => {
	it('updates cache immediately', () => {
		const params = makeParams({ start_year: 1900 });
		stores.focusedScenarioId.set('bau');
		stores.scenarioParamsCache.set(new Map([['bau', params]]));

		simControls.updateSimField('start_year', 1950);

		const cached = get(stores.scenarioParamsCache).get('bau')!;
		expect(cached.start_year).toBe(1950);
		// Other fields unchanged
		expect(cached.end_year).toBe(2100);
	});

	it('sends WS message after 200ms debounce', () => {
		const params = makeParams({ start_year: 1900 });
		stores.focusedScenarioId.set('bau');
		stores.scenarioParamsCache.set(new Map([['bau', params]]));

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
		stores.focusedScenarioId.set('bau');
		stores.scenarioParamsCache.set(new Map([['bau', params]]));

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
		expect(get(stores.scenarioParamsCache).size).toBe(0);
	});

	it('no-ops if params missing from cache', () => {
		stores.focusedScenarioId.set('bau');
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
		stores.focusedScenarioId.set('bau');
		stores.scenarioParamsCache.set(new Map([['bau', params]]));

		simControls.updateSimField('start_year', 1970);
		const after1 = get(stores.scenarioParamsCache).get('bau')!;
		expect(after1.start_year).toBe(1970);
		expect(after1.end_year).toBe(2100);
		expect(after1.time_step).toBe(1.0);

		simControls.updateSimField('end_year', 2200);
		const after2 = get(stores.scenarioParamsCache).get('bau')!;
		expect(after2.start_year).toBe(1970);
		expect(after2.end_year).toBe(2200);
		expect(after2.time_step).toBe(1.0);

		simControls.updateSimField('time_step', 0.5);
		const after3 = get(stores.scenarioParamsCache).get('bau')!;
		expect(after3.start_year).toBe(1970);
		expect(after3.end_year).toBe(2200);
		expect(after3.time_step).toBe(0.5);
	});
});
