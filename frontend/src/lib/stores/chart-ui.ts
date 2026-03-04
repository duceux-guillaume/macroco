import { writable } from 'svelte/store';
import { unifiedVariables } from '../charts/unified-config';

/** Which variables are visible on the unified chart (set of fieldPaths). */
export const visibleVariables = writable<Set<string>>(
	new Set(unifiedVariables.map((v) => v.fieldPath))
);

/** Whether the chart is in "compare scenarios" mode (one variable across all scenarios). */
export const compareMode = writable<boolean>(false);

/** Which variable to show across scenarios in compare mode. */
export const compareVariable = writable<string>(unifiedVariables[0].fieldPath);

/** Whether to show historical data overlay on charts. */
export const showHistorical = writable<boolean>(true);
