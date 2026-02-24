import { writable, derived } from 'svelte/store';
import type { WorldState } from '../types';
import { activeScenarioIds } from './scenarios';

/** Completed simulation results: scenario_id → WorldState[]. */
export const simulationResults = writable<Map<string, WorldState[]>>(new Map());

/** In-progress streaming buffer: scenario_id → WorldState[]. */
export const streamingBuffer = writable<Map<string, WorldState[]>>(new Map());

/** The scenario ID currently being simulated via WebSocket. */
export const simulatingScenarioId = writable<string | null>(null);

/** Active scenario data for charts (completed results filtered by active IDs). */
export const activeSimData = derived(
	[simulationResults, activeScenarioIds],
	([$results, $activeIds]) => {
		const data = new Map<string, WorldState[]>();
		for (const id of $activeIds) {
			const states = $results.get(id);
			if (states) {
				data.set(id, states);
			}
		}
		return data;
	}
);
