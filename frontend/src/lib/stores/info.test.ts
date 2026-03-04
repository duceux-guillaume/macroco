// REQ: REQ-022
import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
	selectedVariableId,
	selectedParameterId,
	selectedHistoricalId,
	highlightedVariables
} from './info';
import { getRelatedParameters } from '$lib/content/variable-descriptions';

beforeEach(() => {
	selectedVariableId.set(null);
	selectedParameterId.set(null);
	selectedHistoricalId.set(null);
});

describe('selectedVariableId', () => {
	it('defaults to null', () => {
		expect(get(selectedVariableId)).toBeNull();
	});

	it('can be set and read back', () => {
		selectedVariableId.set('population.population');
		expect(get(selectedVariableId)).toBe('population.population');
	});
});

describe('selectedHistoricalId', () => {
	it('defaults to null', () => {
		expect(get(selectedHistoricalId)).toBeNull();
	});

	it('can be set and read back', () => {
		selectedHistoricalId.set('population');
		expect(get(selectedHistoricalId)).toBe('population');
	});
});

describe('mutual exclusion', () => {
	it('setting variable clears parameter and historical', () => {
		selectedParameterId.set('resource_efficiency');
		selectedHistoricalId.set('population');
		selectedVariableId.set('population.population');
		expect(get(selectedVariableId)).toBe('population.population');
		expect(get(selectedParameterId)).toBeNull();
		expect(get(selectedHistoricalId)).toBeNull();
	});

	it('setting parameter clears variable and historical', () => {
		selectedVariableId.set('population.population');
		selectedHistoricalId.set('population');
		selectedParameterId.set('resource_efficiency');
		expect(get(selectedParameterId)).toBe('resource_efficiency');
		expect(get(selectedVariableId)).toBeNull();
		expect(get(selectedHistoricalId)).toBeNull();
	});

	it('setting historical clears variable and parameter', () => {
		selectedVariableId.set('population.population');
		selectedParameterId.set('resource_efficiency');
		selectedHistoricalId.set('population');
		expect(get(selectedHistoricalId)).toBe('population');
		expect(get(selectedVariableId)).toBeNull();
		expect(get(selectedParameterId)).toBeNull();
	});
});

describe('highlightedVariables with variable selected', () => {
	it('returns single-element set with the variable ID', () => {
		selectedVariableId.set('capital.industrial_output');
		const highlighted = get(highlightedVariables);
		expect(highlighted.size).toBe(1);
		expect(highlighted.has('capital.industrial_output')).toBe(true);
	});

	it('returns empty set when deselected', () => {
		selectedVariableId.set('capital.industrial_output');
		selectedVariableId.set(null);
		expect(get(highlightedVariables).size).toBe(0);
	});
});

describe('getRelatedParameters', () => {
	it('returns parameters that reference a known variable', () => {
		const params = getRelatedParameters('resources.nonrenewable_resources');
		expect(params.length).toBeGreaterThan(0);
		const paths = params.map((p) => p.path);
		expect(paths).toContain('resource_efficiency');
	});

	it('returns empty array for unknown variable path', () => {
		const params = getRelatedParameters('nonexistent.variable');
		expect(params).toEqual([]);
	});

	it('returns same reference on repeated calls (cached)', () => {
		const first = getRelatedParameters('pollution.pollution_index');
		const second = getRelatedParameters('pollution.pollution_index');
		expect(first).toBe(second);
	});
});
