import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { simulationResults, activeSimData } from './simulation';
import {
	scenarios,
	activeScenarioIds,
	focusedScenarioId,
	focusedScenario,
	scenarioColors
} from './scenarios';
import { paramsSchema, schemaBySector } from './schema';
import { makeWorldState } from '../test-helpers';
import type { ScenarioSummary, ParameterDescriptor } from '../types';

describe('activeSimData', () => {
	it('filters by activeScenarioIds', () => {
		const states = [makeWorldState({ time: 1970 }), makeWorldState({ time: 1980 })];
		simulationResults.set(
			new Map([
				['a', states],
				['b', [makeWorldState()]]
			])
		);
		activeScenarioIds.set(new Set(['a']));

		const data = get(activeSimData);
		expect(data.size).toBe(1);
		expect(data.has('a')).toBe(true);
		expect(data.has('b')).toBe(false);
		expect(data.get('a')).toHaveLength(2);
	});

	it('returns empty map for non-matching IDs', () => {
		simulationResults.set(new Map([['a', [makeWorldState()]]]));
		activeScenarioIds.set(new Set(['nonexistent']));

		const data = get(activeSimData);
		expect(data.size).toBe(0);
	});

	it('returns empty map when no scenarios active', () => {
		simulationResults.set(new Map([['a', [makeWorldState()]]]));
		activeScenarioIds.set(new Set());

		const data = get(activeSimData);
		expect(data.size).toBe(0);
	});
});

describe('focusedScenario', () => {
	const testScenarios: ScenarioSummary[] = [
		{
			id: 'bau',
			name: 'Business As Usual',
			description: 'Standard run',
			color_hex: '#ff0000',
			is_preset: true
		},
		{
			id: 'tech',
			name: 'Technology',
			description: 'Tech scenario',
			color_hex: '#00ff00',
			is_preset: true
		}
	];

	it('returns matching scenario', () => {
		scenarios.set(testScenarios);
		focusedScenarioId.set('bau');

		const focused = get(focusedScenario);
		expect(focused).not.toBeNull();
		expect(focused!.id).toBe('bau');
		expect(focused!.name).toBe('Business As Usual');
	});

	it('returns null for null ID', () => {
		scenarios.set(testScenarios);
		focusedScenarioId.set(null);

		expect(get(focusedScenario)).toBeNull();
	});

	it('returns null for non-existent ID', () => {
		scenarios.set(testScenarios);
		focusedScenarioId.set('nonexistent');

		expect(get(focusedScenario)).toBeNull();
	});
});

describe('scenarioColors', () => {
	it('maps only active scenario colors', () => {
		scenarios.set([
			{
				id: 'a',
				name: 'A',
				description: '',
				color_hex: '#ff0000',
				is_preset: true
			},
			{
				id: 'b',
				name: 'B',
				description: '',
				color_hex: '#00ff00',
				is_preset: false
			},
			{
				id: 'c',
				name: 'C',
				description: '',
				color_hex: '#0000ff',
				is_preset: false
			}
		]);
		activeScenarioIds.set(new Set(['a', 'c']));

		const colors = get(scenarioColors);
		expect(colors.size).toBe(2);
		expect(colors.get('a')).toBe('#ff0000');
		expect(colors.get('c')).toBe('#0000ff');
		expect(colors.has('b')).toBe(false);
	});
});

describe('schemaBySector', () => {
	it('groups parameters by sector name', () => {
		const testSchema: ParameterDescriptor[] = [
			{
				field: 'family_planning_year',
				label: 'FP Year',
				unit: 'year',
				min: 1970,
				max: 2100,
				default: 2000,
				step: 1,
				sector: 'Population',
				description: 'Start year'
			},
			{
				field: 'family_planning_efficacy',
				label: 'FP Efficacy',
				unit: '0-1',
				min: 0,
				max: 1,
				default: 0.5,
				step: 0.1,
				sector: 'Population',
				description: 'Efficacy'
			},
			{
				field: 'resource_efficiency',
				label: 'Resource Eff.',
				unit: 'multiplier',
				min: 1,
				max: 10,
				default: 1,
				step: 0.5,
				sector: 'Resources',
				description: 'Efficiency'
			}
		];

		paramsSchema.set(testSchema);
		const grouped = get(schemaBySector);

		expect(grouped.size).toBe(2);
		expect(grouped.get('Population')).toHaveLength(2);
		expect(grouped.get('Resources')).toHaveLength(1);
	});

	it('returns empty map for empty schema', () => {
		paramsSchema.set([]);
		expect(get(schemaBySector).size).toBe(0);
	});
});
