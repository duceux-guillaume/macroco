import * as d3 from 'd3';

/** Constrain a d3 zoom transform to X-axis only (zero Y translation, unit Y scale). */
export function constrainToXAxis(t: d3.ZoomTransform): d3.ZoomTransform {
	return d3.zoomIdentity.translate(t.x, 0).scale(t.k);
}

/** Whether a transform represents a zoomed-in state (k > 1 with small tolerance). */
export function isTransformZoomed(t: d3.ZoomTransform): boolean {
	return t.k > 1.01;
}

/** Detect whether a touch interaction was a tap (not a drag or long-press). */
export function isTap(
	elapsed: number,
	startX: number, startY: number,
	endX: number, endY: number,
	maxDuration = 300,
	maxDistance = 10
): boolean {
	if (elapsed > maxDuration) return false;
	const dist = Math.hypot(endX - startX, endY - startY);
	return dist <= maxDistance;
}

/** Compute tooltip trend arrow for a data point relative to the previous point. */
export function computeTrend(currentValue: number, previousValue: number | undefined): string {
	if (previousValue === undefined) return '';
	const diff = currentValue - previousValue;
	const pct = previousValue !== 0 ? Math.abs(diff / previousValue) : 0;
	if (pct < 0.001) return '\u2192'; // →
	if (diff > 0) return '\u2191'; // ↑
	return '\u2193'; // ↓
}

/** Filter data points to those within a visible year window and compute Y extent with padding. */
export function computeVisibleYExtent(
	points: Array<{ year: number; y: number }>,
	visibleYearStart: number,
	visibleYearEnd: number,
	padding = 0.05
): [number, number] | null {
	const visibleVals = points
		.filter((p) => p.year >= visibleYearStart && p.year <= visibleYearEnd)
		.map((p) => p.y);
	if (visibleVals.length === 0) return null;
	const [min, max] = d3.extent(visibleVals) as [number, number];
	const pad = (max - min) * padding || 1;
	return [Math.max(0, min - pad), max + pad];
}
