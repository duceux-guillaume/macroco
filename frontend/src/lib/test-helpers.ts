import type { WorldState } from './types';

/** Create a complete WorldState with realistic defaults. Override any nested field. */
export function makeWorldState(overrides: Partial<{
	time: number;
	population: Partial<WorldState['population']>;
	capital: Partial<WorldState['capital']>;
	agriculture: Partial<WorldState['agriculture']>;
	resources: Partial<WorldState['resources']>;
	pollution: Partial<WorldState['pollution']>;
}> = {}): WorldState {
	return {
		time: overrides.time ?? 1970,
		population: {
			population: 3.6e9,
			cohort_0_14: 1.3e9,
			cohort_15_44: 1.4e9,
			cohort_45_64: 0.6e9,
			cohort_65_plus: 0.3e9,
			birth_rate: 0.030,
			death_rate: 0.012,
			life_expectancy: 53,
			fertility_rate: 4.5,
			...overrides.population
		},
		capital: {
			industrial_capital: 2.1e11,
			service_capital: 1.4e11,
			industrial_output: 7.0e10,
			industrial_output_per_capita: 190,
			service_output_per_capita: 65,
			...overrides.capital
		},
		agriculture: {
			arable_land: 0.9e9,
			potentially_arable_land: 2.3e9,
			food: 3.0e12,
			food_per_capita: 830,
			land_yield: 600,
			agricultural_inputs_per_hectare: 20,
			...overrides.agriculture
		},
		resources: {
			nonrenewable_resources: 0.95,
			fraction_remaining: 0.95,
			...overrides.resources
		},
		pollution: {
			persistent_pollution: 1.0,
			pollution_index: 1.0,
			generation_rate: 0.05,
			assimilation_rate: 0.04,
			...overrides.pollution
		}
	};
}

/** Create an array of WorldStates forming a time series. */
export function makeTimeSeries(
	years: number[],
	fieldOverrides?: (year: number, index: number) => Partial<Parameters<typeof makeWorldState>[0]>
): WorldState[] {
	return years.map((year, i) =>
		makeWorldState({ time: year, ...fieldOverrides?.(year, i) })
	);
}
