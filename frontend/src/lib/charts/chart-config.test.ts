import { describe, it, expect } from 'vitest';
import { chartConfigs } from './chart-config';
import { variableDescriptions } from '../content/variable-descriptions';

describe('chartConfigs', () => {
	it('has 6 chart configurations', () => {
		expect(chartConfigs).toHaveLength(6);
	});

	it('has no duplicate IDs', () => {
		const ids = chartConfigs.map((c) => c.id);
		expect(new Set(ids).size).toBe(ids.length);
	});

	it.each(chartConfigs.map((c) => [c.id, c.fieldPath]))(
		'config "%s" fieldPath "%s" exists in variableDescriptions',
		(_id, fieldPath) => {
			expect(variableDescriptions[fieldPath]).toBeDefined();
		}
	);

	it.each(chartConfigs.map((c) => [c.id, c.format]))(
		'config "%s" format "%s" is valid',
		(_id, format) => {
			expect(['billions', 'percent', 'decimal', 'integer']).toContain(format);
		}
	);

	it('all configs have non-empty title and yLabel', () => {
		for (const c of chartConfigs) {
			expect(c.title.length).toBeGreaterThan(0);
			expect(c.yLabel.length).toBeGreaterThan(0);
		}
	});
});
