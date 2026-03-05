import { writable, derived } from 'svelte/store';
import { parameterDescriptions } from '$lib/content/variable-descriptions';
import { unifiedVariables } from '$lib/charts/unified-config';

/** The field path of the currently selected variable for the info panel (null = closed). */
export const selectedVariableId = writable<string | null>(null);

/** The field name of the currently selected parameter for the info panel (null = closed). */
export const selectedParameterId = writable<string | null>(null);

/** Whether the historical data info panel is open (null = closed). */
export const selectedHistoricalId = writable<string | null>(null);

// Mutual exclusion: setting any one clears the other two.
// The null guards prevent infinite loops.
selectedVariableId.subscribe((v) => {
	if (v !== null) {
		selectedParameterId.set(null);
		selectedHistoricalId.set(null);
	}
});
selectedParameterId.subscribe((p) => {
	if (p !== null) {
		selectedVariableId.set(null);
		selectedHistoricalId.set(null);
	}
});
selectedHistoricalId.subscribe((h) => {
	if (h !== null) {
		selectedVariableId.set(null);
		selectedParameterId.set(null);
	}
});

/** Set of variable field paths to highlight on the chart. */
export const highlightedVariables = derived(
	[selectedParameterId, selectedVariableId, selectedHistoricalId],
	([$paramId, $varId, $histId]) => {
		if ($histId) return new Set(unifiedVariables.map((v) => v.fieldPath));
		if ($varId) return new Set<string>([$varId]);
		if (!$paramId) return new Set<string>();
		const info = parameterDescriptions[$paramId];
		return new Set(info?.relatedVariables ?? []);
	}
);

/** Whether highlighting targets historical lines only (when "Historical" legend item is clicked). */
export const highlightHistoricalOnly = derived(
	selectedHistoricalId,
	($histId) => $histId !== null
);
