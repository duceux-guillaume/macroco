// REQ: REQ-020
import { describe, it, expect } from 'vitest';
import {
	variableDescriptions,
	parameterDescriptions,
	feedbackLoops,
	type VariableInfo,
	type ParameterInfo
} from './variable-descriptions';

const REQUIRED_VARIABLE_FIELDS: (keyof VariableInfo)[] = [
	'name',
	'unit',
	'sector',
	'isStock',
	'beginner',
	'expert',
	'feedbackLoops',
	'relatedVariables'
];

const REQUIRED_PARAM_FIELDS: (keyof ParameterInfo)[] = [
	'name',
	'unit',
	'sector',
	'beginner',
	'expert',
	'feedbackLoops',
	'relatedVariables',
	'impact'
];

/** All field paths that extractSeries supports (from extract.ts switch). */
const EXTRACT_PATHS = [
	'population.population',
	'population.birth_rate',
	'population.death_rate',
	'population.life_expectancy',
	'population.fertility_rate',
	'population.perceived_le',
	'capital.industrial_capital',
	'capital.service_capital',
	'capital.perceived_iopc',
	'capital.industrial_output',
	'capital.industrial_output_per_capita',
	'capital.service_output_per_capita',
	'agriculture.arable_land',
	'agriculture.urban_industrial_land',
	'agriculture.land_fertility',
	'agriculture.food',
	'agriculture.food_per_capita',
	'agriculture.food_per_capita_smooth',
	'agriculture.land_yield',
	'resources.nonrenewable_resources',
	'resources.fraction_remaining',
	'pollution.persistent_pollution',
	'pollution.pollution_appearance_buffer',
	'pollution.pollution_index'
];

describe('variableDescriptions', () => {
	it.each(Object.entries(variableDescriptions))(
		'"%s" has all required fields',
		(_key, info) => {
			for (const field of REQUIRED_VARIABLE_FIELDS) {
				expect(info[field], `missing field: ${field}`).toBeDefined();
			}
		}
	);

	it('all keys are valid extractSeries field paths', () => {
		for (const key of Object.keys(variableDescriptions)) {
			expect(EXTRACT_PATHS, `"${key}" not in extractSeries paths`).toContain(key);
		}
	});

	it('all relatedVariables reference existing entries', () => {
		const validKeys = new Set(Object.keys(variableDescriptions));
		for (const [key, info] of Object.entries(variableDescriptions)) {
			for (const ref of info.relatedVariables) {
				expect(validKeys.has(ref), `${key} references unknown variable: ${ref}`).toBe(true);
			}
		}
	});

	it('all feedbackLoop IDs reference existing feedbackLoops', () => {
		const validLoopIds = new Set(Object.keys(feedbackLoops));
		for (const [key, info] of Object.entries(variableDescriptions)) {
			for (const loopId of info.feedbackLoops) {
				expect(
					validLoopIds.has(loopId),
					`${key} references unknown feedback loop: ${loopId}`
				).toBe(true);
			}
		}
	});
});

describe('parameterDescriptions', () => {
	it.each(Object.entries(parameterDescriptions))(
		'"%s" has all required fields',
		(_key, info) => {
			for (const field of REQUIRED_PARAM_FIELDS) {
				expect(info[field], `missing field: ${field}`).toBeDefined();
			}
		}
	);

	it('has at least 10 parameters', () => {
		expect(Object.keys(parameterDescriptions).length).toBeGreaterThanOrEqual(10);
	});

	it('all feedbackLoop IDs reference existing feedbackLoops', () => {
		const validLoopIds = new Set(Object.keys(feedbackLoops));
		for (const [key, info] of Object.entries(parameterDescriptions)) {
			for (const loopId of info.feedbackLoops) {
				expect(
					validLoopIds.has(loopId),
					`${key} references unknown feedback loop: ${loopId}`
				).toBe(true);
			}
		}
	});

	it('all relatedVariables reference existing variableDescriptions', () => {
		const validKeys = new Set(Object.keys(variableDescriptions));
		for (const [key, info] of Object.entries(parameterDescriptions)) {
			for (const ref of info.relatedVariables) {
				expect(validKeys.has(ref), `${key} references unknown variable: ${ref}`).toBe(true);
			}
		}
	});

	it('all impact.sparklineVariable reference existing variableDescriptions', () => {
		const validKeys = new Set(Object.keys(variableDescriptions));
		for (const [key, info] of Object.entries(parameterDescriptions)) {
			expect(
				validKeys.has(info.impact.sparklineVariable),
				`${key} impact.sparklineVariable references unknown variable: ${info.impact.sparklineVariable}`
			).toBe(true);
		}
	});
});

describe('feedbackLoops', () => {
	it.each(Object.entries(feedbackLoops))(
		'"%s" chain entries reference valid variable paths',
		(_key, loop) => {
			const validPaths = new Set(EXTRACT_PATHS);
			for (const path of loop.chain) {
				expect(validPaths.has(path), `chain entry "${path}" not a valid path`).toBe(true);
			}
		}
	);

	it.each(Object.entries(feedbackLoops))(
		'"%s" type is reinforcing or stabilizing',
		(_key, loop) => {
			expect(['reinforcing', 'stabilizing']).toContain(loop.type);
		}
	);

	it('all loops have id matching their key', () => {
		for (const [key, loop] of Object.entries(feedbackLoops)) {
			expect(loop.id).toBe(key);
		}
	});
});
