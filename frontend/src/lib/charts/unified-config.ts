import type { ChartConfig } from './chart-config';

export interface UnifiedVariableConfig {
	id: string;
	fieldPath: string;
	label: string;
	shortLabel: string;
	color: string;
	format: ChartConfig['format'];
	unit: string;
}

/** The 6 variables for the unified overview chart, with colorblind-safe palette. */
export const unifiedVariables: UnifiedVariableConfig[] = [
	{
		id: 'population',
		fieldPath: 'population.population',
		label: 'Population',
		shortLabel: 'Pop.',
		color: '#60a5fa',
		format: 'billions',
		unit: 'Billions'
	},
	{
		id: 'resources',
		fieldPath: 'resources.fraction_remaining',
		label: 'Resources Remaining',
		shortLabel: 'Res.',
		color: '#34d399',
		format: 'percent',
		unit: 'Fraction'
	},
	{
		id: 'food',
		fieldPath: 'agriculture.food_per_capita',
		label: 'Food Per Capita',
		shortLabel: 'Food',
		color: '#fb923c',
		format: 'integer',
		unit: 'kg/person/yr'
	},
	{
		id: 'industrial',
		fieldPath: 'capital.industrial_output_per_capita',
		label: 'Industrial Output / Capita',
		shortLabel: 'Ind.',
		color: '#a78bfa',
		format: 'integer',
		unit: '$/person/yr'
	},
	{
		id: 'pollution',
		fieldPath: 'pollution.pollution_index',
		label: 'Pollution Index',
		shortLabel: 'Poll.',
		color: '#f87171',
		format: 'decimal',
		unit: 'Index'
	},
	{
		id: 'life-expectancy',
		fieldPath: 'population.life_expectancy',
		label: 'Life Expectancy',
		shortLabel: 'Life',
		color: '#2dd4bf',
		format: 'decimal',
		unit: 'Years'
	}
];
