import { writable, derived } from 'svelte/store';
import type { ParameterDescriptor } from '../types';

export const paramsSchema = writable<ParameterDescriptor[]>([]);

export const schemaBySector = derived(paramsSchema, ($schema) => {
	const grouped = new Map<string, ParameterDescriptor[]>();
	for (const desc of $schema) {
		const list = grouped.get(desc.sector) ?? [];
		list.push(desc);
		grouped.set(desc.sector, list);
	}
	return grouped;
});
