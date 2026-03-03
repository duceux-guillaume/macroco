import { describe, it, expect } from 'vitest';
import { unifiedVariables } from './unified-config';
import { variableDescriptions } from '../content/variable-descriptions';

describe('unifiedVariables', () => {
	it('has 6 unified variable configs', () => {
		expect(unifiedVariables).toHaveLength(6);
	});

	it('has no duplicate IDs', () => {
		const ids = unifiedVariables.map((v) => v.id);
		expect(new Set(ids).size).toBe(ids.length);
	});

	it.each(unifiedVariables.map((v) => [v.id, v.fieldPath]))(
		'config "%s" fieldPath "%s" exists in variableDescriptions',
		(_id, fieldPath) => {
			expect(variableDescriptions[fieldPath]).toBeDefined();
		}
	);

	it.each(unifiedVariables.map((v) => [v.id, v.color]))(
		'config "%s" color "%s" is valid hex',
		(_id, color) => {
			expect(color).toMatch(/^#[0-9a-f]{6}$/i);
		}
	);

	it('all configs have non-empty label and shortLabel', () => {
		for (const v of unifiedVariables) {
			expect(v.label.length).toBeGreaterThan(0);
			expect(v.shortLabel.length).toBeGreaterThan(0);
		}
	});

	it('all formats are valid', () => {
		for (const v of unifiedVariables) {
			expect(['billions', 'percent', 'decimal', 'integer']).toContain(v.format);
		}
	});
});
