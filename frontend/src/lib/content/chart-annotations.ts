import type { WorldState } from '../types';
import { extractSeries } from '../utils/extract';

export interface ChartAnnotation {
	year: number;
	label: string;
	type: 'static' | 'dynamic';
}

/** Static annotations per chart ID — fixed historical markers */
const staticAnnotations: Record<string, ChartAnnotation[]> = {
	population: [
		{ year: 1972, label: 'LtG published', type: 'static' }
	],
	resources: [
		{ year: 1972, label: 'LtG published', type: 'static' }
	],
	food: [],
	industrial: [],
	pollution: [],
	'life-expectancy': []
};

/** Compute dynamic annotations from simulation data */
function findPeakYear(
	data: Map<string, WorldState[]>,
	fieldPath: string,
	focusedId: string | null
): ChartAnnotation | null {
	const id = focusedId ?? data.keys().next().value;
	if (!id) return null;
	const states = data.get(id);
	if (!states || states.length === 0) return null;

	const points = extractSeries(states, fieldPath);
	if (points.length < 3) return null;

	let maxVal = -Infinity;
	let maxYear = 0;
	for (const p of points) {
		if (p.value > maxVal) {
			maxVal = p.value;
			maxYear = p.year;
		}
	}

	// Only annotate if peak is not at the start or end (i.e. it's a real peak)
	const firstYear = points[0].year;
	const lastYear = points[points.length - 1].year;
	if (maxYear <= firstYear + 5 || maxYear >= lastYear - 5) return null;

	return { year: maxYear, label: `Peak ~${Math.round(maxYear)}`, type: 'dynamic' };
}

function findThresholdCrossing(
	data: Map<string, WorldState[]>,
	fieldPath: string,
	threshold: number,
	direction: 'below' | 'above',
	label: string,
	focusedId: string | null
): ChartAnnotation | null {
	const id = focusedId ?? data.keys().next().value;
	if (!id) return null;
	const states = data.get(id);
	if (!states || states.length === 0) return null;

	const points = extractSeries(states, fieldPath);
	for (let i = 1; i < points.length; i++) {
		const prev = points[i - 1].value;
		const curr = points[i].value;
		if (direction === 'below' && prev >= threshold && curr < threshold) {
			return { year: Math.round(points[i].year), label, type: 'dynamic' };
		}
		if (direction === 'above' && prev <= threshold && curr > threshold) {
			return { year: Math.round(points[i].year), label, type: 'dynamic' };
		}
	}
	return null;
}

/** Get all annotations for a chart (static + dynamic) */
export function getAnnotations(
	chartId: string,
	fieldPath: string,
	data: Map<string, WorldState[]>,
	focusedId: string | null
): ChartAnnotation[] {
	const annotations: ChartAnnotation[] = [...(staticAnnotations[chartId] ?? [])];

	switch (chartId) {
		case 'population':
			{
				const peak = findPeakYear(data, fieldPath, focusedId);
				if (peak) annotations.push(peak);
			}
			break;
		case 'resources':
			{
				const half = findThresholdCrossing(
					data,
					fieldPath,
					0.5,
					'below',
					'50% depleted',
					focusedId
				);
				if (half) annotations.push(half);
			}
			break;
		case 'industrial':
			{
				const peak = findPeakYear(data, fieldPath, focusedId);
				if (peak) annotations.push(peak);
			}
			break;
		case 'pollution':
			{
				const high = findThresholdCrossing(
					data,
					fieldPath,
					5,
					'above',
					'5× 1970 level',
					focusedId
				);
				if (high) annotations.push(high);
			}
			break;
		case 'food':
			{
				const peak = findPeakYear(data, fieldPath, focusedId);
				if (peak) annotations.push(peak);
			}
			break;
		case 'life-expectancy':
			{
				const peak = findPeakYear(data, fieldPath, focusedId);
				if (peak) annotations.push(peak);
			}
			break;
	}

	return annotations;
}
