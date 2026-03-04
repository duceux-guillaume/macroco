// REQ: REQ-009, REQ-021
import { describe, it, expect } from 'vitest';
import { formatBillions, formatPercent, formatDecimal, formatInteger, formatAuto } from './format';

describe('formatBillions', () => {
	it('formats large numbers as billions', () => {
		expect(formatBillions(8.1e9)).toBe('8.1B');
	});

	it('formats zero', () => {
		expect(formatBillions(0)).toBe('0.0B');
	});

	it('formats sub-billion values', () => {
		expect(formatBillions(5e8)).toBe('0.5B');
	});
});

describe('formatPercent', () => {
	it('formats fraction as percent', () => {
		expect(formatPercent(0.73)).toBe('73%');
	});

	it('formats 1.0 as 100%', () => {
		expect(formatPercent(1.0)).toBe('100%');
	});

	it('formats zero', () => {
		expect(formatPercent(0)).toBe('0%');
	});
});

describe('formatDecimal', () => {
	it('formats with one decimal place', () => {
		expect(formatDecimal(3.14)).toBe('3.1');
	});

	it('rounds correctly', () => {
		expect(formatDecimal(3.95)).toBe('4.0');
	});
});

describe('formatInteger', () => {
	it('rounds to integer', () => {
		expect(formatInteger(3.7)).toBe('4');
	});

	it('rounds down', () => {
		expect(formatInteger(3.2)).toBe('3');
	});
});

describe('formatAuto', () => {
	it('formats billions', () => {
		expect(formatAuto(1.2e9)).toBe('1.2B');
	});

	it('formats millions', () => {
		expect(formatAuto(3.4e6)).toBe('3.4M');
	});

	it('formats thousands', () => {
		expect(formatAuto(5.6e3)).toBe('5.6K');
	});

	it('formats small numbers with one decimal', () => {
		expect(formatAuto(42)).toBe('42.0');
	});

	it('formats fractional numbers with 3 decimals', () => {
		expect(formatAuto(0.123)).toBe('0.123');
	});

	it('formats zero with one decimal', () => {
		expect(formatAuto(0)).toBe('0.0');
	});

	it('handles negative billions', () => {
		expect(formatAuto(-2e9)).toBe('-2.0B');
	});
});
