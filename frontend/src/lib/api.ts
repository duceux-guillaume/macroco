import { getApiBase } from './env';
import type {
	ParameterDescriptor,
	Scenario,
	ScenarioParams,
	ScenarioSummary,
	SimulationOutput
} from './types';
import type { HistoricalVariable } from './stores/historical';

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(`${getApiBase()}${path}`, {
		headers: { 'Content-Type': 'application/json' },
		...init
	});
	if (!res.ok) {
		const text = await res.text();
		throw new Error(`API error ${res.status}: ${text}`);
	}
	return res.json();
}

export function getParamsSchema(): Promise<ParameterDescriptor[]> {
	return apiFetch('/params/schema');
}

export function getPresets(): Promise<ScenarioSummary[]> {
	return apiFetch('/presets');
}

export function getScenarios(): Promise<ScenarioSummary[]> {
	return apiFetch('/scenarios');
}

export function createScenario(params: ScenarioParams): Promise<Scenario> {
	return apiFetch('/scenarios', {
		method: 'POST',
		body: JSON.stringify(params)
	});
}

export function getScenario(id: string): Promise<Scenario> {
	return apiFetch(`/scenarios/${encodeURIComponent(id)}`);
}

export function updateParams(id: string, params: ScenarioParams): Promise<Scenario> {
	return apiFetch(`/scenarios/${encodeURIComponent(id)}/params`, {
		method: 'PUT',
		body: JSON.stringify(params)
	});
}

export function deleteScenario(id: string): Promise<void> {
	return apiFetch(`/scenarios/${encodeURIComponent(id)}`, { method: 'DELETE' });
}

export function runScenario(id: string): Promise<SimulationOutput> {
	return apiFetch(`/scenarios/${encodeURIComponent(id)}/run`, { method: 'POST' });
}

export function getHistoricalData(): Promise<HistoricalVariable[]> {
	return apiFetch('/historical');
}
