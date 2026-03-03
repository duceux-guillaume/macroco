import { describe, it, expect } from 'vitest';
import { getAnnotations } from './chart-annotations';
import { makeWorldState } from '../test-helpers';
import type { WorldState } from '../types';

function makeDataMap(states: WorldState[]): Map<string, WorldState[]> {
	return new Map([['scenario-1', states]]);
}

/** Build a bell-shaped population curve peaking at `peakYear`. */
function makeBellCurve(
	start: number,
	end: number,
	peakYear: number,
	peakValue: number,
	baseValue: number
): WorldState[] {
	const states: WorldState[] = [];
	for (let y = start; y <= end; y += 10) {
		const dist = Math.abs(y - peakYear);
		const value = baseValue + (peakValue - baseValue) * Math.exp(-(dist * dist) / 2000);
		states.push(makeWorldState({ time: y, population: { population: value } }));
	}
	return states;
}

describe('getAnnotations', () => {
	it('population chart includes static "LtG published" annotation', () => {
		const data = makeDataMap([makeWorldState({ time: 1970 })]);
		const annotations = getAnnotations('population', 'population.population', data, null);
		expect(annotations.some((a) => a.label === 'LtG published' && a.year === 1972)).toBe(true);
	});

	it('returns empty for unknown chart ID', () => {
		const data = makeDataMap([makeWorldState()]);
		const annotations = getAnnotations('unknown-chart', 'population.population', data, null);
		expect(annotations).toEqual([]);
	});

	it('applies prefix to all labels', () => {
		const data = makeDataMap([makeWorldState({ time: 1970 })]);
		const annotations = getAnnotations(
			'population',
			'population.population',
			data,
			null,
			'Pop.'
		);
		for (const a of annotations) {
			expect(a.label).toMatch(/^Pop\. /);
		}
	});
});

describe('findPeakYear (via getAnnotations)', () => {
	it('finds peak in bell curve data', () => {
		const states = makeBellCurve(1900, 2100, 2030, 8e9, 2e9);
		const data = makeDataMap(states);
		const annotations = getAnnotations('population', 'population.population', data, null);
		const peak = annotations.find((a) => a.type === 'dynamic');
		expect(peak).toBeDefined();
		expect(peak!.year).toBe(2030);
		expect(peak!.label).toContain('Peak');
	});

	it('filters peak at start of data (within first 5 years)', () => {
		// Peak at year 1900, start is 1900 → within ±5 of start → should be filtered
		const states = makeBellCurve(1900, 2100, 1900, 8e9, 2e9);
		const data = makeDataMap(states);
		const annotations = getAnnotations('population', 'population.population', data, null);
		const peak = annotations.find((a) => a.type === 'dynamic');
		expect(peak).toBeUndefined();
	});

	it('filters peak at end of data (within last 5 years)', () => {
		const states = makeBellCurve(1900, 2100, 2100, 8e9, 2e9);
		const data = makeDataMap(states);
		const annotations = getAnnotations('population', 'population.population', data, null);
		const peak = annotations.find((a) => a.type === 'dynamic');
		expect(peak).toBeUndefined();
	});

	it('returns no peak for monotonically increasing data', () => {
		const states: WorldState[] = [];
		for (let y = 1900; y <= 2100; y += 10) {
			states.push(makeWorldState({ time: y, population: { population: y * 1e6 } }));
		}
		const data = makeDataMap(states);
		const annotations = getAnnotations('population', 'population.population', data, null);
		const peak = annotations.find((a) => a.type === 'dynamic');
		expect(peak).toBeUndefined();
	});
});

describe('findThresholdCrossing (via getAnnotations)', () => {
	it('detects resources crossing below 0.5', () => {
		const states: WorldState[] = [];
		for (let y = 1900; y <= 2100; y += 10) {
			const frac = 1.0 - ((y - 1900) / 200) * 0.8; // 1.0 → 0.2
			states.push(makeWorldState({ time: y, resources: { fraction_remaining: frac } }));
		}
		const data = makeDataMap(states);
		const annotations = getAnnotations(
			'resources',
			'resources.fraction_remaining',
			data,
			null
		);
		const crossing = annotations.find((a) => a.label === '50% depleted');
		expect(crossing).toBeDefined();
		expect(crossing!.type).toBe('dynamic');
		// Should be around year 2025 (frac goes from ~0.5 to ~0.46 between 2020-2030)
		expect(crossing!.year).toBeGreaterThanOrEqual(2000);
		expect(crossing!.year).toBeLessThanOrEqual(2060);
	});

	it('detects pollution crossing above 5x', () => {
		const states: WorldState[] = [];
		for (let y = 1900; y <= 2100; y += 10) {
			const poll = ((y - 1900) / 200) * 15; // 0 → 15
			states.push(makeWorldState({ time: y, pollution: { pollution_index: poll } }));
		}
		const data = makeDataMap(states);
		const annotations = getAnnotations(
			'pollution',
			'pollution.pollution_index',
			data,
			null
		);
		const crossing = annotations.find((a) => a.label === '5× 1970 level');
		expect(crossing).toBeDefined();
	});

	it('returns no crossing when threshold is never reached', () => {
		const states: WorldState[] = [];
		for (let y = 1900; y <= 2100; y += 10) {
			states.push(makeWorldState({ time: y, resources: { fraction_remaining: 0.9 } }));
		}
		const data = makeDataMap(states);
		const annotations = getAnnotations(
			'resources',
			'resources.fraction_remaining',
			data,
			null
		);
		const crossing = annotations.find((a) => a.label === '50% depleted');
		expect(crossing).toBeUndefined();
	});
});
