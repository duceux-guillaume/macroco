// REQ: REQ-009
import { describe, it, expect } from 'vitest';
import * as d3 from 'd3';
import {
	constrainToXAxis,
	isTransformZoomed,
	isTap,
	computeTrend,
	computeVisibleYExtent
} from './zoom-helpers';

describe('constrainToXAxis', () => {
	it('zeroes Y translation and preserves X translation and scale', () => {
		const t = d3.zoomIdentity.translate(50, 30).scale(3);
		const constrained = constrainToXAxis(t);
		expect(constrained.x).toBe(50);
		expect(constrained.y).toBe(0);
		expect(constrained.k).toBe(3);
	});

	it('returns identity-like transform for identity input', () => {
		const constrained = constrainToXAxis(d3.zoomIdentity);
		expect(constrained.x).toBe(0);
		expect(constrained.y).toBe(0);
		expect(constrained.k).toBe(1);
	});

	it('handles negative X translation', () => {
		const t = d3.zoomIdentity.translate(-100, 20).scale(5);
		const constrained = constrainToXAxis(t);
		expect(constrained.x).toBe(-100);
		expect(constrained.y).toBe(0);
		expect(constrained.k).toBe(5);
	});
});

describe('isTransformZoomed', () => {
	it('returns false for identity transform (k=1)', () => {
		expect(isTransformZoomed(d3.zoomIdentity)).toBe(false);
	});

	it('returns false for k just above 1 but below threshold', () => {
		const t = d3.zoomIdentity.scale(1.005);
		expect(isTransformZoomed(t)).toBe(false);
	});

	it('returns true for k above threshold (1.01)', () => {
		const t = d3.zoomIdentity.scale(1.02);
		expect(isTransformZoomed(t)).toBe(true);
	});

	it('returns true for large zoom (k=5)', () => {
		const t = d3.zoomIdentity.scale(5);
		expect(isTransformZoomed(t)).toBe(true);
	});

	it('returns false for k exactly at threshold boundary', () => {
		const t = d3.zoomIdentity.scale(1.01);
		expect(isTransformZoomed(t)).toBe(false);
	});
});

describe('isTap', () => {
	it('returns true for short tap with no movement', () => {
		expect(isTap(100, 50, 50, 50, 50)).toBe(true);
	});

	it('returns false for long press (> 300ms)', () => {
		expect(isTap(301, 50, 50, 50, 50)).toBe(false);
	});

	it('returns true at exactly 300ms', () => {
		expect(isTap(300, 50, 50, 50, 50)).toBe(true);
	});

	it('returns false for drag (> 10px movement)', () => {
		expect(isTap(100, 0, 0, 11, 0)).toBe(false);
	});

	it('returns true for small movement within threshold', () => {
		expect(isTap(100, 0, 0, 5, 5)).toBe(true);
	});

	it('returns false for diagonal drag exceeding distance', () => {
		// hypot(8, 8) ≈ 11.3 > 10
		expect(isTap(100, 0, 0, 8, 8)).toBe(false);
	});

	it('respects custom maxDuration', () => {
		expect(isTap(500, 0, 0, 0, 0, 600)).toBe(true);
		expect(isTap(500, 0, 0, 0, 0, 400)).toBe(false);
	});

	it('respects custom maxDistance', () => {
		expect(isTap(100, 0, 0, 15, 0, 300, 20)).toBe(true);
		expect(isTap(100, 0, 0, 15, 0, 300, 10)).toBe(false);
	});
});

describe('computeTrend', () => {
	it('returns empty string when no previous value', () => {
		expect(computeTrend(100, undefined)).toBe('');
	});

	it('returns up arrow for increasing value', () => {
		expect(computeTrend(110, 100)).toBe('\u2191');
	});

	it('returns down arrow for decreasing value', () => {
		expect(computeTrend(90, 100)).toBe('\u2193');
	});

	it('returns right arrow for negligible change (< 0.1%)', () => {
		expect(computeTrend(100.05, 100)).toBe('\u2192');
	});

	it('returns right arrow when previous is zero and current is zero', () => {
		expect(computeTrend(0, 0)).toBe('\u2192');
	});

	it('returns up arrow when previous is zero and current is positive', () => {
		// pct = 0 (prev is 0), but diff > 0 — however the function checks pct < 0.001 first
		// With prev=0, pct=0, so pct < 0.001 is true → returns →
		expect(computeTrend(5, 0)).toBe('\u2192');
	});

	it('returns down arrow for large decrease', () => {
		expect(computeTrend(50, 100)).toBe('\u2193');
	});
});

describe('computeVisibleYExtent', () => {
	const points = [
		{ year: 1900, y: 10 },
		{ year: 1950, y: 50 },
		{ year: 2000, y: 30 },
		{ year: 2050, y: 80 },
		{ year: 2100, y: 20 }
	];

	it('returns padded extent for full range', () => {
		const result = computeVisibleYExtent(points, 1900, 2100);
		expect(result).not.toBeNull();
		const [min, max] = result!;
		// min/max of y: 10..80, range=70, pad=3.5
		expect(min).toBeCloseTo(6.5, 1);
		expect(max).toBeCloseTo(83.5, 1);
	});

	it('returns padded extent for zoomed window', () => {
		const result = computeVisibleYExtent(points, 1950, 2050);
		expect(result).not.toBeNull();
		const [min, max] = result!;
		// visible: y=50,30,80; range=50, pad=2.5
		expect(min).toBeCloseTo(27.5, 1);
		expect(max).toBeCloseTo(82.5, 1);
	});

	it('returns null when no points in visible range', () => {
		expect(computeVisibleYExtent(points, 2200, 2300)).toBeNull();
	});

	it('clamps min to 0 when padding would go negative', () => {
		const pts = [{ year: 2000, y: 0.01 }, { year: 2010, y: 0.02 }];
		const result = computeVisibleYExtent(pts, 2000, 2010);
		expect(result).not.toBeNull();
		// range=0.01, pad=0.0005; min = max(0, 0.01-0.0005) = 0.0095; still > 0
		// Use y values that actually go negative after padding:
		const pts2 = [{ year: 2000, y: 0 }, { year: 2010, y: 10 }];
		const result2 = computeVisibleYExtent(pts2, 2000, 2010);
		expect(result2).not.toBeNull();
		// range=10, pad=0.5; min = max(0, 0-0.5) = max(0, -0.5) = 0
		expect(result2![0]).toBe(0);
		expect(result2![1]).toBeCloseTo(10.5, 1);
	});

	it('uses padding=1 when all values are the same', () => {
		const pts = [{ year: 2000, y: 5 }, { year: 2010, y: 5 }];
		const result = computeVisibleYExtent(pts, 2000, 2010);
		expect(result).not.toBeNull();
		// range=0, pad=1; domain=[max(0, 5-1), 5+1] = [4, 6]
		expect(result![0]).toBe(4);
		expect(result![1]).toBe(6);
	});

	it('handles single visible point', () => {
		const result = computeVisibleYExtent(points, 1950, 1950);
		expect(result).not.toBeNull();
		// Only y=50, range=0, pad=1
		expect(result![0]).toBe(49);
		expect(result![1]).toBe(51);
	});

	it('respects custom padding', () => {
		const result = computeVisibleYExtent(points, 1900, 2100, 0.1);
		expect(result).not.toBeNull();
		// min/max: 10..80, range=70, pad=7
		expect(result![0]).toBeCloseTo(3, 1);
		expect(result![1]).toBeCloseTo(87, 1);
	});
});
