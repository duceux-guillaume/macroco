<script lang="ts">
	import * as d3 from 'd3';
	import { resize } from '../utils/resize';
	import { extractSeries } from '../utils/extract';
	import { formatBillions, formatPercent, formatDecimal, formatInteger } from '../utils/format';
	import { selectedVariableId } from '../stores/info';
	import { hoveredYear } from '../stores/simulation';
	import { getAnnotations } from '../content/chart-annotations';
	import type { UnifiedVariableConfig } from './unified-config';
	import type { WorldState } from '../types';

	interface Props {
		config: UnifiedVariableConfig;
		data: Map<string, WorldState[]>;
		focusedScenarioId?: string | null;
		xDomain?: [number, number] | null;
	}

	let { config, data, focusedScenarioId = null, xDomain = null }: Props = $props();

	let containerEl: HTMLDivElement;
	let width = $state(0);
	let height = $state(0);

	const margin = { top: 16, right: 8, bottom: 20, left: 40 };

	function getFormatter(format: string): (v: number) => string {
		switch (format) {
			case 'billions': return formatBillions;
			case 'percent': return formatPercent;
			case 'decimal': return formatDecimal;
			case 'integer': return formatInteger;
			default: return formatDecimal;
		}
	}

	function handleResize(w: number, h: number) {
		width = w;
		height = h;
	}

	function handleTitleClick() {
		selectedVariableId.set(config.fieldPath);
	}

	$effect(() => {
		if (!containerEl || width <= 0 || height <= 0) return;

		const _config = config;
		const _data = data;
		const _focusedId = focusedScenarioId;
		const _hoveredYear = $hoveredYear;
		const _xDomain = xDomain;

		const innerW = width - margin.left - margin.right;
		const innerH = height - margin.top - margin.bottom;
		if (innerW <= 0 || innerH <= 0) return;

		const fmt = getFormatter(_config.format);

		// Pick the focused scenario (or first available)
		const scenarioId = _focusedId ?? _data.keys().next().value;
		if (!scenarioId) return;
		const states = _data.get(scenarioId);
		if (!states || states.length === 0) return;

		const points = extractSeries(states, _config.fieldPath);
		if (points.length === 0) return;

		const xExtent = d3.extent(points, (d) => d.year) as [number, number];
		const effectiveXDomain: [number, number] = _xDomain ?? xExtent;

		// Filter points to visible range for dynamic Y rescaling
		const visiblePoints = _xDomain
			? points.filter((p) => p.year >= _xDomain[0] && p.year <= _xDomain[1])
			: points;
		const ySource = visiblePoints.length > 0 ? visiblePoints : points;
		const yExtent = d3.extent(ySource, (d) => d.value) as [number, number];
		const yPad = (yExtent[1] - yExtent[0]) * 0.05 || 1;

		const xScale = d3.scaleLinear().domain(effectiveXDomain).range([0, innerW]);
		const yScale = d3.scaleLinear().domain([Math.max(0, yExtent[0] - yPad), yExtent[1] + yPad]).range([innerH, 0]);

		const line = d3.line<{ year: number; value: number }>()
			.x((d) => xScale(d.year))
			.y((d) => yScale(d.value));

		// SVG setup
		const svg = d3.select(containerEl)
			.selectAll<SVGSVGElement, null>('svg')
			.data([null])
			.join('svg')
			.attr('width', width)
			.attr('height', height);

		const g = svg
			.selectAll<SVGGElement, null>('g.chart')
			.data([null])
			.join('g')
			.attr('class', 'chart')
			.attr('transform', `translate(${margin.left},${margin.top})`);

		// X axis (minimal)
		g.selectAll<SVGGElement, null>('g.x-axis')
			.data([null])
			.join('g')
			.attr('class', 'x-axis')
			.attr('transform', `translate(0,${innerH})`)
			.call(d3.axisBottom(xScale).tickFormat(d3.format('d')).ticks(Math.min(innerW / 60, 5)));

		// Y axis (2-3 ticks)
		g.selectAll<SVGGElement, null>('g.y-axis')
			.data([null])
			.join('g')
			.attr('class', 'y-axis')
			.call(d3.axisLeft(yScale).tickFormat((d) => fmt(d as number)).ticks(3));

		// Title (clickable, top-left)
		svg.selectAll<SVGTextElement, null>('text.spark-title')
			.data([null])
			.join('text')
			.attr('class', 'spark-title')
			.attr('x', margin.left + 2)
			.attr('y', 11)
			.attr('fill', 'var(--text-secondary)')
			.attr('font-size', '9px')
			.attr('font-weight', '600')
			.attr('cursor', 'pointer')
			.text(_config.shortLabel)
			.on('click', handleTitleClick);

		// Line (with transition)
		g.selectAll<SVGPathElement, null>('path.spark-line')
			.data([null])
			.join(
				(enter) => enter.append('path')
					.attr('class', 'spark-line')
					.attr('fill', 'none')
					.attr('stroke', _config.color)
					.attr('stroke-width', 1.5)
					.attr('d', line(points)),
				(update) => update
					.transition()
					.duration(200)
					.attr('stroke', _config.color)
					.attr('d', line(points)),
				(exit) => exit.remove()
			);

		// Per-chart annotations (peaks, thresholds)
		const annotations = getAnnotations(_config.id, _config.fieldPath, _data, _focusedId);
		const annGroup = g.selectAll<SVGGElement, null>('g.spark-annotations')
			.data([null])
			.join('g')
			.attr('class', 'spark-annotations');

		const annSel = annGroup
			.selectAll<SVGGElement, typeof annotations[number]>('g.spark-ann')
			.data(annotations, (d) => `${d.year}-${d.label}`);

		annSel.join(
			(enter) => {
				const ann = enter.append('g').attr('class', 'spark-ann');
				ann.append('line')
					.attr('x1', (d) => xScale(d.year))
					.attr('x2', (d) => xScale(d.year))
					.attr('y1', 0)
					.attr('y2', innerH)
					.attr('stroke', _config.color)
					.attr('stroke-width', 0.5)
					.attr('stroke-dasharray', '3,2')
					.attr('opacity', 0.35);
				ann.append('text')
					.attr('x', (d) => xScale(d.year) + 2)
					.attr('y', 8)
					.attr('fill', _config.color)
					.attr('font-size', '7px')
					.attr('opacity', 0.6)
					.text((d) => d.label);
				return ann;
			},
			(update) => {
				update.select('line')
					.attr('x1', (d) => xScale(d.year))
					.attr('x2', (d) => xScale(d.year))
					.attr('y2', innerH);
				update.select('text')
					.attr('x', (d) => xScale(d.year) + 2)
					.text((d) => d.label);
				return update;
			},
			(exit) => exit.remove()
		);

		// Synced hover indicator from overview
		const hoverLine = g.selectAll<SVGLineElement, null>('line.hover-line')
			.data([null])
			.join('line')
			.attr('class', 'hover-line')
			.attr('stroke', 'var(--text-secondary)')
			.attr('stroke-width', 1)
			.attr('stroke-dasharray', '2,2')
			.attr('y1', 0)
			.attr('y2', innerH)
			.attr('opacity', 0.6);

		const effectiveHoverDomain = effectiveXDomain;
		if (_hoveredYear !== null && _hoveredYear >= effectiveHoverDomain[0] && _hoveredYear <= effectiveHoverDomain[1]) {
			hoverLine
				.style('display', null)
				.attr('x1', xScale(_hoveredYear))
				.attr('x2', xScale(_hoveredYear));
		} else {
			hoverLine.style('display', 'none');
		}
	});
</script>

<div
	class="sparkline-chart"
	style="border-left-color: {config.color}"
	bind:this={containerEl}
	use:resize={handleResize}
>
</div>

<style>
	.sparkline-chart {
		width: 100%;
		height: 100%;
		min-height: 60px;
		position: relative;
		border-left: 3px solid transparent;
	}
	.sparkline-chart :global(.x-axis text),
	.sparkline-chart :global(.y-axis text) {
		fill: var(--text-secondary);
		font-size: 9px;
	}
	.sparkline-chart :global(.x-axis line),
	.sparkline-chart :global(.x-axis path),
	.sparkline-chart :global(.y-axis line),
	.sparkline-chart :global(.y-axis path) {
		stroke: var(--border);
	}
	.sparkline-chart :global(.spark-title:hover) {
		fill: var(--accent) !important;
	}
</style>
