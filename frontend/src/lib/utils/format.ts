/** Format a number as billions (e.g. 8.1B). */
export function formatBillions(value: number): string {
	return (value / 1e9).toFixed(1) + 'B';
}

/** Format a number as a percentage (e.g. 73%). */
export function formatPercent(value: number): string {
	return (value * 100).toFixed(0) + '%';
}

/** Format with 1 decimal place. */
export function formatDecimal(value: number): string {
	return value.toFixed(1);
}

/** Format with no decimal places. */
export function formatInteger(value: number): string {
	return value.toFixed(0);
}

/** Auto-format based on magnitude. */
export function formatAuto(value: number): string {
	if (Math.abs(value) >= 1e9) return formatBillions(value);
	if (Math.abs(value) >= 1e6) return (value / 1e6).toFixed(1) + 'M';
	if (Math.abs(value) >= 1e3) return (value / 1e3).toFixed(1) + 'K';
	if (Math.abs(value) < 1 && value !== 0) return value.toFixed(3);
	return value.toFixed(1);
}
