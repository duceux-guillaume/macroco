import { writable } from 'svelte/store';

export interface HistoricalDataPoint {
	year: number;
	value: number;
}

export interface HistoricalVariable {
	variable: string;
	source: string;
	units: string;
	transformation: string;
	url: string;
	data: HistoricalDataPoint[];
}

/** Historical data keyed by variable ID (e.g., 'population', 'resources'). */
export const historicalData = writable<Map<string, HistoricalVariable>>(new Map());
