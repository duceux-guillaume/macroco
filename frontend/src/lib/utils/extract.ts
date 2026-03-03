import type { WorldState } from '../types';

export interface DataPoint {
	year: number;
	value: number;
}

/** Extract a field path from a WorldState, matching backend extract_field paths. */
function extractField(state: WorldState, path: string): number | null {
	switch (path) {
		case 'population.population':
			return state.population.population;
		case 'population.birth_rate':
			return state.population.birth_rate;
		case 'population.death_rate':
			return state.population.death_rate;
		case 'population.life_expectancy':
			return state.population.life_expectancy;
		case 'population.fertility_rate':
			return state.population.fertility_rate;
		case 'capital.industrial_capital':
			return state.capital.industrial_capital;
		case 'capital.service_capital':
			return state.capital.service_capital;
		case 'capital.industrial_output':
			return state.capital.industrial_output;
		case 'capital.industrial_output_per_capita':
			return state.capital.industrial_output_per_capita;
		case 'capital.service_output_per_capita':
			return state.capital.service_output_per_capita;
		case 'agriculture.arable_land':
			return state.agriculture.arable_land;
		case 'agriculture.food':
			return state.agriculture.food;
		case 'agriculture.food_per_capita':
			return state.agriculture.food_per_capita;
		case 'agriculture.land_yield':
			return state.agriculture.land_yield;
		case 'resources.nonrenewable_resources':
			return state.resources.nonrenewable_resources;
		case 'resources.fraction_remaining':
			return state.resources.fraction_remaining;
		case 'pollution.persistent_pollution':
			return state.pollution.persistent_pollution;
		case 'pollution.pollution_index':
			return state.pollution.pollution_index;
		default:
			return null;
	}
}

/** Extract a time series from an array of WorldState for a given field path. */
export function extractSeries(states: WorldState[], path: string): DataPoint[] {
	const result: DataPoint[] = [];
	for (const s of states) {
		const value = extractField(s, path);
		if (value !== null) {
			result.push({ year: s.time, value });
		}
	}
	return result;
}

export interface NormalizedPoint {
	year: number;
	normalized: number;
	raw: number;
}

/** Normalize a series to 0-1 range using min-max normalization. */
export function normalizeSeries(points: DataPoint[]): { points: NormalizedPoint[]; min: number; max: number } {
	if (points.length === 0) return { points: [], min: 0, max: 0 };

	let min = Infinity;
	let max = -Infinity;
	for (const p of points) {
		if (p.value < min) min = p.value;
		if (p.value > max) max = p.value;
	}

	const range = max - min;
	const normalized: NormalizedPoint[] = points.map((p) => ({
		year: p.year,
		normalized: range > 0 ? (p.value - min) / range : 0.5,
		raw: p.value
	}));

	return { points: normalized, min, max };
}
