import { writable, derived } from 'svelte/store';
import type { ScenarioSummary, ScenarioParams } from '../types';

/** All known scenarios (presets + custom). */
export const scenarios = writable<ScenarioSummary[]>([]);

/** IDs of scenarios currently visible on charts. */
export const activeScenarioIds = writable<Set<string>>(new Set());

/** ID of the scenario being edited in the sidebar. */
export const focusedScenarioId = writable<string | null>(null);

/** Cached params keyed by scenario ID. */
export const scenarioParamsCache = writable<Map<string, ScenarioParams>>(new Map());

/** The currently focused scenario summary (derived). */
export const focusedScenario = derived(
	[scenarios, focusedScenarioId],
	([$scenarios, $focusedId]) => {
		if (!$focusedId) return null;
		return $scenarios.find((s) => s.id === $focusedId) ?? null;
	}
);

/** Color map for active scenarios (derived). */
export const scenarioColors = derived(
	[scenarios, activeScenarioIds],
	([$scenarios, $activeIds]) => {
		const colors = new Map<string, string>();
		for (const s of $scenarios) {
			if ($activeIds.has(s.id)) {
				colors.set(s.id, s.color_hex);
			}
		}
		return colors;
	}
);
