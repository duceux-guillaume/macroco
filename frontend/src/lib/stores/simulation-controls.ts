import { get } from 'svelte/store';
import { focusedScenarioId, scenarioParamsCache } from './scenarios';
import { send } from '../ws';
import type { ScenarioParams } from '../types';

export const startYearOptions = [1900, 1950, 1970, 2000];
export const endYearOptions = [2050, 2100, 2150, 2200];
export const timeStepOptions = [0.25, 0.5, 1.0, 2.0];

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

export function getSimParams(): ScenarioParams | null {
	const id = get(focusedScenarioId);
	if (!id) return null;
	return get(scenarioParamsCache).get(id) ?? null;
}

export function updateSimField(field: string, value: number): void {
	const id = get(focusedScenarioId);
	if (!id) return;
	const params = getSimParams();
	if (!params) return;

	const updated = { ...params, [field]: value };
	scenarioParamsCache.update((cache) => {
		const next = new Map(cache);
		next.set(id, updated);
		return next;
	});

	if (debounceTimer) clearTimeout(debounceTimer);
	debounceTimer = setTimeout(() => {
		send({ type: 'update_params', scenario_id: id, params: updated });
	}, 200);
}
