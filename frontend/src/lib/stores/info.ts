import { writable } from 'svelte/store';

/** The field path of the currently selected variable for the info panel (null = closed). */
export const selectedVariableId = writable<string | null>(null);
