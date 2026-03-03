import { describe, it, expect } from 'vitest';
import { extractSeries, normalizeSeries } from './extract';
import { makeWorldState, makeTimeSeries } from '../test-helpers';

const ALL_FIELD_PATHS = [
	'population.population',
	'population.birth_rate',
	'population.death_rate',
	'population.life_expectancy',
	'population.fertility_rate',
	'capital.industrial_capital',
	'capital.service_capital',
	'capital.industrial_output',
	'capital.industrial_output_per_capita',
	'capital.service_output_per_capita',
	'agriculture.arable_land',
	'agriculture.food',
	'agriculture.food_per_capita',
	'agriculture.land_yield',
	'resources.nonrenewable_resources',
	'resources.fraction_remaining',
	'pollution.persistent_pollution',
	'pollution.pollution_index'
];

describe('extractSeries', () => {
	const states = makeTimeSeries(
		[1970, 1980, 1990],
		(year) => ({
			population: { population: 3.6e9 + (year - 1970) * 1e8 }
		})
	);

	it('extracts a valid field path as DataPoint[]', () => {
		const series = extractSeries(states, 'population.population');
		expect(series).toHaveLength(3);
		expect(series[0]).toEqual({ year: 1970, value: 3.6e9 });
		expect(series[1]).toEqual({ year: 1980, value: 4.6e9 });
		expect(series[2]).toEqual({ year: 1990, value: 5.6e9 });
	});

	it.each(ALL_FIELD_PATHS)('extracts field path "%s" successfully', (path) => {
		const result = extractSeries(states, path);
		expect(result).toHaveLength(3);
		for (const point of result) {
			expect(typeof point.year).toBe('number');
			expect(typeof point.value).toBe('number');
		}
	});

	it('returns empty array for invalid path', () => {
		expect(extractSeries(states, 'nonexistent.field')).toEqual([]);
	});

	it('preserves year ordering', () => {
		const series = extractSeries(states, 'population.population');
		for (let i = 1; i < series.length; i++) {
			expect(series[i].year).toBeGreaterThan(series[i - 1].year);
		}
	});

	it('handles empty states array', () => {
		expect(extractSeries([], 'population.population')).toEqual([]);
	});
});

describe('normalizeSeries', () => {
	it('returns empty for empty array', () => {
		const result = normalizeSeries([]);
		expect(result.points).toEqual([]);
		expect(result.min).toBe(0);
		expect(result.max).toBe(0);
	});

	it('normalizes single point to 0.5', () => {
		const result = normalizeSeries([{ year: 2000, value: 42 }]);
		expect(result.points).toHaveLength(1);
		expect(result.points[0].normalized).toBe(0.5);
		expect(result.points[0].raw).toBe(42);
	});

	it('normalizes range [0, 100] to [0.0, 1.0]', () => {
		const result = normalizeSeries([
			{ year: 2000, value: 0 },
			{ year: 2050, value: 50 },
			{ year: 2100, value: 100 }
		]);
		expect(result.min).toBe(0);
		expect(result.max).toBe(100);
		expect(result.points[0].normalized).toBeCloseTo(0.0);
		expect(result.points[1].normalized).toBeCloseTo(0.5);
		expect(result.points[2].normalized).toBeCloseTo(1.0);
	});

	it('handles constant values (all same) as 0.5', () => {
		const result = normalizeSeries([
			{ year: 2000, value: 5 },
			{ year: 2010, value: 5 },
			{ year: 2020, value: 5 }
		]);
		for (const p of result.points) {
			expect(p.normalized).toBe(0.5);
		}
	});

	it('preserves raw values', () => {
		const input = [
			{ year: 2000, value: 10 },
			{ year: 2010, value: 20 }
		];
		const result = normalizeSeries(input);
		expect(result.points[0].raw).toBe(10);
		expect(result.points[1].raw).toBe(20);
	});
});
