import { writable, derived } from 'svelte/store';
import { parameterDescriptions } from '$lib/content/variable-descriptions';

/** The field path of the currently selected variable for the info panel (null = closed). */
export const selectedVariableId = writable<string | null>(null);

/** The field name of the currently selected parameter for the info panel (null = closed). */
export const selectedParameterId = writable<string | null>(null);

// Mutual exclusion: setting one clears the other.
// The null guards prevent infinite loops: A sets B→null, which triggers B's
// subscriber, but since null is set it stops (guard fails). Without these
// guards, the cascade would loop indefinitely.
selectedVariableId.subscribe((v) => {
	if (v !== null) selectedParameterId.set(null);
});
selectedParameterId.subscribe((p) => {
	if (p !== null) selectedVariableId.set(null);
});

/** Set of variable field paths to highlight on the chart. */
export const highlightedVariables = derived(
	[selectedParameterId, selectedVariableId],
	([$paramId, $varId]) => {
		if ($varId) return new Set<string>([$varId]);
		if (!$paramId) return new Set<string>();
		const info = parameterDescriptions[$paramId];
		return new Set(info?.relatedVariables ?? []);
	}
);
