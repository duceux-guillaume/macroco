// REQ: REQ-007, REQ-009
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock $env/static/public before importing api module
vi.mock('$env/static/public', () => ({
	PUBLIC_API_BASE: 'http://localhost:8080/api/v1'
}));

import {
	getParamsSchema,
	getPresets,
	getScenarios,
	createScenario,
	getScenario,
	updateParams,
	deleteScenario,
	runScenario,
	getHistoricalData
} from './api';
import type { ScenarioParams, ScenarioMeta } from './types';

const BASE = 'http://localhost:8080/api/v1';

function mockFetch(data: unknown, ok = true, status = 200) {
	return vi.fn().mockResolvedValue({
		ok,
		status,
		json: () => Promise.resolve(data),
		text: () => Promise.resolve(JSON.stringify(data))
	});
}

const testMeta: ScenarioMeta = {
	id: 'test-id',
	name: 'Test',
	description: 'Test scenario',
	color_hex: '#ff0000',
	created_at: '2024-01-01T00:00:00Z'
};

const testParams: ScenarioParams = {
	meta: testMeta,
	family_planning_year: 2000,
	family_planning_efficacy: 0.5,
	health_investment_multiplier: 1.0,
	industrial_depreciation_rate: 0.05,
	service_depreciation_rate: 0.05,
	technology_growth_rate: 0.0,
	agricultural_technology: 1.0,
	agricultural_technology_growth_rate: 0.005,
	land_protection_fraction: 0.0,
	subsistence_food_per_capita: 230,
	resource_efficiency: 1.0,
	initial_nnr_fraction: 1.0,
	pollution_control: 0.0,
	start_year: 1900,
	end_year: 2100,
	time_step: 1.0
};

beforeEach(() => {
	vi.restoreAllMocks();
});

describe('getParamsSchema', () => {
	it('calls correct URL with GET', async () => {
		const fetch = mockFetch([]);
		vi.stubGlobal('fetch', fetch);

		await getParamsSchema();

		expect(fetch).toHaveBeenCalledWith(`${BASE}/params/schema`, expect.objectContaining({
			headers: { 'Content-Type': 'application/json' }
		}));
	});
});

describe('getPresets', () => {
	it('calls /presets', async () => {
		const fetch = mockFetch([]);
		vi.stubGlobal('fetch', fetch);

		await getPresets();

		expect(fetch).toHaveBeenCalledWith(`${BASE}/presets`, expect.anything());
	});
});

describe('getScenarios', () => {
	it('calls /scenarios', async () => {
		const fetch = mockFetch([]);
		vi.stubGlobal('fetch', fetch);

		await getScenarios();

		expect(fetch).toHaveBeenCalledWith(`${BASE}/scenarios`, expect.anything());
	});
});

describe('createScenario', () => {
	it('sends POST with JSON body', async () => {
		const fetch = mockFetch({ params: testParams, is_preset: false, last_output: null });
		vi.stubGlobal('fetch', fetch);

		await createScenario(testParams);

		expect(fetch).toHaveBeenCalledWith(
			`${BASE}/scenarios`,
			expect.objectContaining({
				method: 'POST',
				body: JSON.stringify(testParams)
			})
		);
	});
});

describe('getScenario', () => {
	it('encodes ID in URL', async () => {
		const fetch = mockFetch({ params: testParams, is_preset: false, last_output: null });
		vi.stubGlobal('fetch', fetch);

		await getScenario('id with spaces/special');

		expect(fetch).toHaveBeenCalledWith(
			`${BASE}/scenarios/${encodeURIComponent('id with spaces/special')}`,
			expect.anything()
		);
	});
});

describe('updateParams', () => {
	it('sends PUT with body', async () => {
		const fetch = mockFetch({ params: testParams, is_preset: false, last_output: null });
		vi.stubGlobal('fetch', fetch);

		await updateParams('test-id', testParams);

		expect(fetch).toHaveBeenCalledWith(
			`${BASE}/scenarios/test-id/params`,
			expect.objectContaining({
				method: 'PUT',
				body: JSON.stringify(testParams)
			})
		);
	});
});

describe('deleteScenario', () => {
	it('uses DELETE method', async () => {
		const fetch = mockFetch(null);
		vi.stubGlobal('fetch', fetch);

		await deleteScenario('test-id');

		expect(fetch).toHaveBeenCalledWith(
			`${BASE}/scenarios/test-id`,
			expect.objectContaining({ method: 'DELETE' })
		);
	});
});

describe('runScenario', () => {
	it('sends POST to /run', async () => {
		const fetch = mockFetch({});
		vi.stubGlobal('fetch', fetch);

		await runScenario('test-id');

		expect(fetch).toHaveBeenCalledWith(
			`${BASE}/scenarios/test-id/run`,
			expect.objectContaining({ method: 'POST' })
		);
	});
});

describe('getHistoricalData', () => {
	it('calls /historical', async () => {
		const fetch = mockFetch([]);
		vi.stubGlobal('fetch', fetch);

		await getHistoricalData();

		expect(fetch).toHaveBeenCalledWith(`${BASE}/historical`, expect.anything());
	});
});

describe('apiFetch error handling', () => {
	it('throws on non-OK response', async () => {
		const fetch = mockFetch('Not found', false, 404);
		vi.stubGlobal('fetch', fetch);

		await expect(getParamsSchema()).rejects.toThrow('API error 404');
	});
});
