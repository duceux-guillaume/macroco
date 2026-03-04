// REQ: REQ-009
import { describe, it, expect } from 'vitest';
import { extractSeries } from './extract';
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

