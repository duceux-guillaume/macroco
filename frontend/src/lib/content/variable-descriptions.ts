/** Structured content for chart variables, parameters, and feedback loops. */

export interface VariableInfo {
	name: string;
	unit: string;
	sector: string;
	isStock: boolean;
	beginner: string;
	expert: string;
	feedbackLoops: string[];
	relatedVariables: string[];
}

export interface ParameterInfo {
	name: string;
	unit: string;
	sector: string;
	beginner: string;
	expert: string;
}

export interface FeedbackLoopInfo {
	id: string;
	name: string;
	type: 'reinforcing' | 'stabilizing';
	description: string;
	chain: string[];
}

/** Variable descriptions keyed by ChartConfig.fieldPath */
export const variableDescriptions: Record<string, VariableInfo> = {
	'population.population': {
		name: 'World Population',
		unit: 'persons',
		sector: 'Population',
		isStock: true,
		beginner:
			'Total number of people alive on Earth. Grows when births exceed deaths, shrinks when deaths exceed births.',
		expert:
			'Sum of four age cohorts (0–14, 15–44, 45–64, 65+). Each cohort is an ODE stock with inflows (births or aging-in) and outflows (aging-out or deaths). Mortality is age-weighted: base_mort = 1/life_expectancy, with multipliers 0.8, 0.5, 1.0, 3.0 for the four cohorts.',
		feedbackLoops: ['demographic-transition', 'population-resource', 'food-population'],
		relatedVariables: [
			'population.life_expectancy',
			'population.birth_rate',
			'population.death_rate',
			'population.fertility_rate'
		]
	},
	'population.life_expectancy': {
		name: 'Life Expectancy',
		unit: 'years',
		sector: 'Population',
		isStock: false,
		beginner:
			'How long the average person lives. Depends on food availability, healthcare, crowding, and pollution. When conditions deteriorate, life expectancy falls and death rates rise.',
		expert:
			'life_expectancy = 20.0 × LEM_food(food_ratio) × LEM_health(health_services) × LEM_crowding(pop/3.6e9) × LEM_pollution(pollution_index). Product of four lookup-table multipliers on a 20-year base. Health services = service_output_per_capita × fraction_services_health.',
		feedbackLoops: ['food-population', 'pollution-food'],
		relatedVariables: [
			'population.population',
			'agriculture.food_per_capita',
			'pollution.pollution_index'
		]
	},
	'population.birth_rate': {
		name: 'Birth Rate',
		unit: 'per year',
		sector: 'Population',
		isStock: false,
		beginner:
			'How many babies are born per person per year. Falls as income rises (people choose smaller families) and with family planning programs.',
		expert:
			'birth_rate = births_per_year / population. births_per_year = (cohort_15_44 × 0.5) × total_fertility_rate / 30.0. TFR = desired_family_size(iopc) × fp_multiplier(efficacy × ramp) × food_fertility(food_ratio).',
		feedbackLoops: ['demographic-transition', 'food-population'],
		relatedVariables: ['population.population', 'population.fertility_rate']
	},
	'population.death_rate': {
		name: 'Death Rate',
		unit: 'per year',
		sector: 'Population',
		isStock: false,
		beginner:
			'How many people die per person per year. Rises when food, healthcare, or environmental conditions worsen.',
		expert:
			'death_rate = total_deaths / population. Deaths per cohort use age-weighted base mortality (1/life_expectancy × age_factor).',
		feedbackLoops: ['food-population', 'pollution-food'],
		relatedVariables: ['population.population', 'population.life_expectancy']
	},
	'population.fertility_rate': {
		name: 'Total Fertility Rate',
		unit: 'children/woman',
		sector: 'Population',
		isStock: false,
		beginner:
			'Average number of children per woman. Drops as income rises and family planning becomes available. Below ~2.1, population eventually declines.',
		expert:
			'TFR = desired_family_size(iopc) × family_planning_multiplier(efficacy × ramp) × food_fertility_multiplier(food_ratio). Desired family size ranges from 5.0 at iopc=0 to 1.9 at iopc=1600.',
		feedbackLoops: ['demographic-transition'],
		relatedVariables: ['population.population', 'capital.industrial_output_per_capita']
	},
	'capital.industrial_capital': {
		name: 'Industrial Capital',
		unit: 'USD (1975)',
		sector: 'Capital',
		isStock: true,
		beginner:
			'The total stock of factories, machines, and infrastructure. Grows when investment exceeds wear-and-tear (depreciation). Produces the industrial output that drives the economy.',
		expert:
			'd(IC)/dt = industrial_output × frac_to_investment − IC × depreciation_rate. Investment is the residual after consumption, services, and agriculture allocation.',
		feedbackLoops: ['resource-collapse', 'demographic-transition'],
		relatedVariables: [
			'capital.industrial_output',
			'capital.industrial_output_per_capita',
			'resources.fraction_remaining'
		]
	},
	'capital.industrial_output': {
		name: 'Industrial Output',
		unit: 'USD/yr (1975)',
		sector: 'Capital',
		isStock: false,
		beginner:
			'Total economic production per year. Split between investment (building more capital), services (health, education), and agriculture.',
		expert:
			'IO = productive_capital / ICOR. productive_capital = IC × (1 − capital_for_resources) × tech_multiplier. ICOR = 3.0 × COR_resources(fraction_remaining).',
		feedbackLoops: ['resource-collapse', 'pollution-food'],
		relatedVariables: ['capital.industrial_capital', 'capital.industrial_output_per_capita']
	},
	'capital.industrial_output_per_capita': {
		name: 'Industrial Output Per Capita',
		unit: '$/person/yr (1975)',
		sector: 'Capital',
		isStock: false,
		beginner:
			'Economic output divided by population — a rough measure of average income. When it rises, people choose smaller families. When it falls, the economy is contracting.',
		expert:
			'IOPC = industrial_output / population. Drives desired_family_size lookup and food_ratio calculations. Key indicator of economic health.',
		feedbackLoops: ['resource-collapse', 'demographic-transition'],
		relatedVariables: [
			'capital.industrial_output',
			'population.population',
			'population.fertility_rate'
		]
	},
	'capital.service_output_per_capita': {
		name: 'Service Output Per Capita',
		unit: '$/person/yr (1975)',
		sector: 'Capital',
		isStock: false,
		beginner:
			'How much is spent on services (healthcare, education) per person. Higher service output improves life expectancy through better health services.',
		expert:
			'SOPC = service_capital / SCOR / population. Feeds into health services calculation and service allocation fraction.',
		feedbackLoops: ['food-population'],
		relatedVariables: ['population.life_expectancy']
	},
	'agriculture.arable_land': {
		name: 'Arable Land',
		unit: 'hectares',
		sector: 'Agriculture',
		isStock: true,
		beginner:
			'Total farmland being cultivated. Increases when new land is developed, decreases from erosion. The best land was developed first, so new land costs more.',
		expert:
			'd(AL)/dt = development_rate − erosion_rate. Development limited by potentially_arable_land and investment. Erosion = AL × 0.002 × erosion_mult(yield_ratio) × (1 − land_protection).',
		feedbackLoops: ['pollution-food'],
		relatedVariables: ['agriculture.food', 'agriculture.food_per_capita', 'agriculture.land_yield']
	},
	'agriculture.food': {
		name: 'Total Food Production',
		unit: 'kg/yr',
		sector: 'Agriculture',
		isStock: false,
		beginner:
			'Total food produced worldwide. Equals arable land times yield per hectare. Both can change over time.',
		expert:
			'food = arable_land × land_yield. land_yield = 600 × yield_mult_capital(inputs/ha) × yield_mult_pollution(poll_index) × agri_technology.',
		feedbackLoops: ['pollution-food', 'food-population'],
		relatedVariables: ['agriculture.arable_land', 'agriculture.land_yield']
	},
	'agriculture.food_per_capita': {
		name: 'Food Per Capita',
		unit: 'kg/person/yr',
		sector: 'Agriculture',
		isStock: false,
		beginner:
			'How much food each person gets on average. When it drops below subsistence level (230 kg/yr), life expectancy falls and death rates rise. When it\'s adequate, people are healthier and may have more children.',
		expert:
			'FPC = food / population. food_ratio = FPC / subsistence_food (default 230 kg/yr). Drives life_exp_multiplier_food, food_fertility_multiplier, and industrial_fraction_to_agriculture.',
		feedbackLoops: ['pollution-food', 'food-population'],
		relatedVariables: [
			'agriculture.food',
			'population.population',
			'population.life_expectancy'
		]
	},
	'agriculture.land_yield': {
		name: 'Land Yield',
		unit: 'kg/hectare/yr',
		sector: 'Agriculture',
		isStock: false,
		beginner:
			'How much food each hectare of farmland produces. Increases with more fertilizer and machinery, decreases when pollution damages crops.',
		expert:
			'LY = 600 × LYMC(agri_inputs/ha) × LYMAP(pollution_index) × agricultural_technology. Base yield 600 kg/ha/yr (1900). LYMC ranges 1.0→6.9. LYMAP ranges 1.2→0.50.',
		feedbackLoops: ['pollution-food'],
		relatedVariables: ['agriculture.food', 'pollution.pollution_index']
	},
	'resources.nonrenewable_resources': {
		name: 'Non-Renewable Resources',
		unit: 'fraction of initial',
		sector: 'Resources',
		isStock: true,
		beginner:
			'The stock of oil, coal, metals, and minerals. Starts at 1.0 (100%) and is drawn down by industrial activity. Cannot be replenished.',
		expert:
			'd(NNR)/dt = −population × IOPC × 3.0e-15 / resource_efficiency. Calibrated so 1970 extraction ≈ 0.54%/yr of initial stock.',
		feedbackLoops: ['resource-collapse', 'population-resource'],
		relatedVariables: ['resources.fraction_remaining', 'capital.industrial_output_per_capita']
	},
	'resources.fraction_remaining': {
		name: 'Resources Remaining',
		unit: 'fraction (0–1)',
		sector: 'Resources',
		isStock: false,
		beginner:
			'What percentage of the original resource stock is left. As this drops, the economy must spend more and more effort on extraction, leaving less for everything else.',
		expert:
			'fraction_remaining = clamp(nonrenewable_resources, 0, 1). Feeds capital_output_ratio_resources (multiplier 0.50→4.0) and capital_fraction_resource_extraction (0.0→1.0). Breakeven ≈ 0.65.',
		feedbackLoops: ['resource-collapse'],
		relatedVariables: [
			'resources.nonrenewable_resources',
			'capital.industrial_output',
			'capital.industrial_capital'
		]
	},
	'pollution.persistent_pollution': {
		name: 'Persistent Pollution',
		unit: 'index units',
		sector: 'Pollution',
		isStock: true,
		beginner:
			'The accumulated level of long-lasting pollutants in the environment — CO2, heavy metals, persistent chemicals. Generated by industry and farming, slowly absorbed by the environment.',
		expert:
			'd(PP)/dt = generation − assimilation. generation = (gen_industry + gen_agriculture) × (1 − pollution_control). assimilation = PP / assimilation_time(PP).',
		feedbackLoops: ['pollution-tipping', 'pollution-food'],
		relatedVariables: ['pollution.pollution_index', 'capital.industrial_output']
	},
	'pollution.pollution_index': {
		name: 'Pollution Index',
		unit: '1970 = 1.0',
		sector: 'Pollution',
		isStock: false,
		beginner:
			'Pollution level relative to 1970. An index of 2.0 means twice the pollution of 1970. Above ~10, the environment struggles to absorb it; above ~30, assimilation effectively stops.',
		expert:
			'pollution_index = max(persistent_pollution, 0). Drives pollution_assimilation_time (20yr→480yr), life_exp_multiplier_pollution, and land_yield_multiplier_pollution.',
		feedbackLoops: ['pollution-tipping', 'pollution-food'],
		relatedVariables: [
			'pollution.persistent_pollution',
			'population.life_expectancy',
			'agriculture.land_yield'
		]
	}
};

/** Parameter descriptions keyed by ScenarioParams field name */
export const parameterDescriptions: Record<string, ParameterInfo> = {
	family_planning_year: {
		name: 'Family Planning Start Year',
		unit: 'year',
		sector: 'Population',
		beginner:
			'The year when family planning programs become fully effective. Earlier = earlier fertility decline.',
		expert:
			'Controls the ramp function: fp_ramp = clamp((time − 1900) / (fp_year − 1900), 0, 1). Multiplied by efficacy to get effective family planning input.'
	},
	family_planning_efficacy: {
		name: 'Family Planning Efficacy',
		unit: '0–1',
		sector: 'Population',
		beginner:
			'How effective family planning programs are at reducing birth rates. 0 = no effect, 1 = maximum effect.',
		expert:
			'Scales the family_planning_multiplier lookup input. At efficacy=1.0 and full ramp, fertility multiplier ≈ 0.40.'
	},
	health_investment_multiplier: {
		name: 'Health Investment Multiplier',
		unit: 'multiplier',
		sector: 'Population',
		beginner:
			'How much the economy invests in healthcare. Higher values mean better health services and longer life expectancy.',
		expert: 'Scales service_output_per_capita input to life_exp_multiplier_health lookup.'
	},
	industrial_depreciation_rate: {
		name: 'Industrial Depreciation Rate',
		unit: 'fraction/yr',
		sector: 'Capital',
		beginner:
			'How fast factories and machines wear out. Higher = capital decays faster, requiring more investment just to maintain.',
		expert:
			'Used in d(IC)/dt = investment − IC × depreciation_rate. Default 0.05 = 20-year average capital lifetime.'
	},
	service_depreciation_rate: {
		name: 'Service Depreciation Rate',
		unit: 'fraction/yr',
		sector: 'Capital',
		beginner:
			'How fast service infrastructure (hospitals, schools) wears out.',
		expert:
			'Used in d(SC)/dt = service_investment − SC × depreciation_rate. Default 0.05.'
	},
	technology_growth_rate: {
		name: 'Technology Growth Rate',
		unit: 'fraction/yr',
		sector: 'Capital',
		beginner:
			'Annual improvement in how efficiently capital produces output. Compounds over time — even small rates have big long-term effects.',
		expert:
			'tech_multiplier = (1 + rate)^max(time−1970, 0). Applied to productive capital before ICOR division.'
	},
	agricultural_technology: {
		name: 'Agricultural Technology',
		unit: 'multiplier',
		sector: 'Agriculture',
		beginner:
			'Multiplier on crop yields from improved farming techniques — better seeds, irrigation, precision agriculture.',
		expert: 'Direct multiplier on land_yield: LY = 600 × LYMC × LYMAP × agri_tech.'
	},
	land_protection_fraction: {
		name: 'Land Protection',
		unit: 'fraction (0–0.5)',
		sector: 'Agriculture',
		beginner:
			'How much farmland is protected from erosion through conservation practices. 0 = no protection, 0.5 = half of erosion prevented.',
		expert:
			'Reduces erosion: erosion × (1 − land_protection_fraction). Clamped to [0, 0.5].'
	},
	subsistence_food_per_capita: {
		name: 'Subsistence Food Level',
		unit: 'kg/person/yr',
		sector: 'Agriculture',
		beginner:
			'The minimum food per person needed for basic health. Below this level, life expectancy drops sharply.',
		expert:
			'Denominator in food_ratio = FPC / subsistence_food. Drives multiple lookup tables. Default 230 kg/yr.'
	},
	resource_efficiency: {
		name: 'Resource Efficiency',
		unit: 'multiplier',
		sector: 'Resources',
		beginner:
			'How efficiently resources are used. Higher values mean the economy gets more output per unit of resource consumed. Technology preset uses 4x.',
		expert:
			'Divides extraction rate: extraction = pop × IOPC × coeff / resource_efficiency.'
	},
	initial_nnr_fraction: {
		name: 'Initial Resource Level',
		unit: 'fraction (0–1)',
		sector: 'Resources',
		beginner:
			'Starting level of non-renewable resources. 1.0 = full initial endowment. Lower values simulate a world where resources are already partially depleted.',
		expert: 'Initial condition for nonrenewable_resources ODE stock.'
	},
	pollution_control: {
		name: 'Pollution Control',
		unit: 'fraction (0–1)',
		sector: 'Pollution',
		beginner:
			'How much pollution is prevented at the source. 0 = no control, 0.8 = 80% of pollution eliminated before it enters the environment.',
		expert:
			'generation = (gen_industry + gen_agriculture) × (1 − pollution_control). Clamped to [0, 1].'
	}
};

/** Major feedback loops connecting the sectors */
export const feedbackLoops: Record<string, FeedbackLoopInfo> = {
	'resource-collapse': {
		id: 'resource-collapse',
		name: 'Resource Depletion → Economic Collapse',
		type: 'reinforcing',
		description:
			'As resources deplete, extraction costs rise. More capital is diverted to extraction, leaving less for productive output. Lower output means less investment, so capital shrinks, output falls further.',
		chain: [
			'resources.fraction_remaining',
			'capital.industrial_output',
			'capital.industrial_capital',
			'resources.fraction_remaining'
		]
	},
	'pollution-food': {
		id: 'pollution-food',
		name: 'Pollution → Agricultural Decline',
		type: 'reinforcing',
		description:
			'Industrial output generates pollution. Pollution reduces crop yields, leading to less food per capita. More industrial output is diverted to agriculture, but the pollution persists.',
		chain: [
			'capital.industrial_output',
			'pollution.pollution_index',
			'agriculture.land_yield',
			'agriculture.food_per_capita'
		]
	},
	'demographic-transition': {
		id: 'demographic-transition',
		name: 'Demographic Transition',
		type: 'stabilizing',
		description:
			'As income rises, people choose smaller families. Lower birth rates slow population growth, easing pressure on resources and food. This is the major stabilizing feedback in the model.',
		chain: [
			'capital.industrial_output_per_capita',
			'population.fertility_rate',
			'population.population',
			'capital.industrial_output_per_capita'
		]
	},
	'population-resource': {
		id: 'population-resource',
		name: 'Population → Resource Pressure',
		type: 'reinforcing',
		description:
			'More people consume more resources. Faster depletion raises extraction costs, eventually causing economic contraction and rising death rates.',
		chain: [
			'population.population',
			'resources.fraction_remaining',
			'capital.industrial_output_per_capita',
			'population.life_expectancy'
		]
	},
	'pollution-tipping': {
		id: 'pollution-tipping',
		name: 'Pollution Tipping Point',
		type: 'reinforcing',
		description:
			"Once pollution exceeds the environment's absorption capacity, assimilation time grows dramatically. Pollution accumulates faster, further overwhelming the environment. A classic tipping-point dynamic.",
		chain: [
			'pollution.persistent_pollution',
			'pollution.pollution_index',
			'pollution.persistent_pollution'
		]
	},
	'food-population': {
		id: 'food-population',
		name: 'Food–Population Balance',
		type: 'stabilizing',
		description:
			'Adequate food lowers mortality and slightly raises fertility, growing population. But more people means less food per capita, eventually raising mortality to balance growth.',
		chain: [
			'agriculture.food_per_capita',
			'population.life_expectancy',
			'population.population',
			'agriculture.food_per_capita'
		]
	}
};
