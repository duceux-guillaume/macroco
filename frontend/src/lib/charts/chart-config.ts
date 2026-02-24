export interface ChartConfig {
	id: string;
	title: string;
	fieldPath: string;
	yLabel: string;
	/** Formatter name for y-axis values */
	format: 'billions' | 'percent' | 'decimal' | 'integer';
}

/** The 6 chart panels reproducing the classic Meadows World 3 visualization. */
export const chartConfigs: ChartConfig[] = [
	{
		id: 'population',
		title: 'Population',
		fieldPath: 'population.population',
		yLabel: 'Billions',
		format: 'billions'
	},
	{
		id: 'resources',
		title: 'Resources Remaining',
		fieldPath: 'resources.fraction_remaining',
		yLabel: 'Fraction',
		format: 'percent'
	},
	{
		id: 'food',
		title: 'Food Per Capita',
		fieldPath: 'agriculture.food_per_capita',
		yLabel: 'kg/person/yr',
		format: 'integer'
	},
	{
		id: 'industrial',
		title: 'Industrial Output / Capita',
		fieldPath: 'capital.industrial_output_per_capita',
		yLabel: '$/person/yr',
		format: 'integer'
	},
	{
		id: 'pollution',
		title: 'Pollution Index',
		fieldPath: 'pollution.pollution_index',
		yLabel: 'Index (1970 = 1)',
		format: 'decimal'
	},
	{
		id: 'life-expectancy',
		title: 'Life Expectancy',
		fieldPath: 'population.life_expectancy',
		yLabel: 'Years',
		format: 'decimal'
	}
];
