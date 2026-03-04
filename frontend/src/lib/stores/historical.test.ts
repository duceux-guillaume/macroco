import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { historicalData } from './historical';
import { showHistorical } from './chart-ui';
import type { HistoricalVariable } from './historical';

beforeEach(() => {
	historicalData.set(new Map());
	showHistorical.set(true);
});

describe('historicalData store', () => {
	it('starts empty', () => {
		expect(get(historicalData).size).toBe(0);
	});

	it('stores historical variables by id', () => {
		const testVar: HistoricalVariable = {
			variable: 'population',
			source: 'World Bank',
			units: 'persons',
			transformation: 'none',
			data: [
				{ year: 1960, value: 3.034e9 },
				{ year: 1970, value: 3.700e9 }
			]
		};

		historicalData.update((m) => {
			const next = new Map(m);
			next.set('population', testVar);
			return next;
		});

		const data = get(historicalData);
		expect(data.size).toBe(1);
		expect(data.get('population')?.data).toHaveLength(2);
		expect(data.get('population')?.source).toBe('World Bank');
	});
});

describe('showHistorical store', () => {
	it('defaults to true', () => {
		expect(get(showHistorical)).toBe(true);
	});

	it('can be toggled off and on', () => {
		showHistorical.set(false);
		expect(get(showHistorical)).toBe(false);
		showHistorical.update((v) => !v);
		expect(get(showHistorical)).toBe(true);
	});
});
