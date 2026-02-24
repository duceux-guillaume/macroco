// TypeScript types mirroring Rust backend (snake_case to match serde JSON output)

// ---------------------------------------------------------------------------
// WorldState sector sub-interfaces
// ---------------------------------------------------------------------------

export interface PopulationState {
	population: number;
	cohort_0_14: number;
	cohort_15_44: number;
	cohort_45_64: number;
	cohort_65_plus: number;
	birth_rate: number;
	death_rate: number;
	life_expectancy: number;
	fertility_rate: number;
}

export interface CapitalState {
	industrial_capital: number;
	service_capital: number;
	industrial_output: number;
	industrial_output_per_capita: number;
	service_output_per_capita: number;
}

export interface AgricultureState {
	arable_land: number;
	potentially_arable_land: number;
	food: number;
	food_per_capita: number;
	land_yield: number;
	agricultural_inputs_per_hectare: number;
}

export interface ResourceState {
	nonrenewable_resources: number;
	fraction_remaining: number;
}

export interface PollutionState {
	persistent_pollution: number;
	pollution_index: number;
	generation_rate: number;
	assimilation_rate: number;
}

export interface WorldState {
	time: number;
	population: PopulationState;
	capital: CapitalState;
	agriculture: AgricultureState;
	resources: ResourceState;
	pollution: PollutionState;
}

// ---------------------------------------------------------------------------
// Scenario types
// ---------------------------------------------------------------------------

export interface ScenarioMeta {
	id: string;
	name: string;
	description: string;
	color_hex: string;
	created_at: string;
}

export interface ScenarioParams {
	meta: ScenarioMeta;
	family_planning_year: number;
	family_planning_efficacy: number;
	health_investment_multiplier: number;
	industrial_depreciation_rate: number;
	service_depreciation_rate: number;
	technology_growth_rate: number;
	investment_rate: number;
	agricultural_technology: number;
	land_protection_fraction: number;
	subsistence_food_per_capita: number;
	resource_efficiency: number;
	initial_nnr_fraction: number;
	pollution_control: number;
	start_year: number;
	end_year: number;
	time_step: number;
}

export interface Scenario {
	params: ScenarioParams;
	is_preset: boolean;
	last_output: SimulationOutput | null;
}

export interface ScenarioSummary {
	id: string;
	name: string;
	description: string;
	color_hex: string;
	is_preset: boolean;
}

// ---------------------------------------------------------------------------
// Simulation output
// ---------------------------------------------------------------------------

export interface SimulationOutput {
	scenario_id: string;
	scenario_name: string;
	timeline: number[];
	states: WorldState[];
	params: ScenarioParams;
	computed_at: string;
}

// ---------------------------------------------------------------------------
// Parameter schema
// ---------------------------------------------------------------------------

export interface ParameterDescriptor {
	field: string;
	label: string;
	unit: string;
	min: number;
	max: number;
	default: number;
	step: number;
	sector: string;
	description: string;
}

// ---------------------------------------------------------------------------
// WebSocket messages (discriminated unions, tagged on "type", snake_case)
// ---------------------------------------------------------------------------

export type WsClientMsg =
	| { type: 'start_simulation'; scenario_id: string; params?: ScenarioParams }
	| { type: 'update_params'; scenario_id: string; params: ScenarioParams }
	| { type: 'stop_simulation' };

export type WsServerMsg =
	| { type: 'sim_step'; year: number; state: WorldState }
	| { type: 'sim_complete'; scenario_id: string; total_steps: number }
	| { type: 'sim_error'; message: string }
	| { type: 'params_ack'; scenario_id: string };
