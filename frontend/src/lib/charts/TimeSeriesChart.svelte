<script lang="ts">
	import * as d3 from 'd3';
	import { resize } from '../utils/resize';
	import { extractSeries } from '../utils/extract';
	import { formatBillions, formatPercent, formatDecimal, formatInteger } from '../utils/format';
	import type { ChartConfig } from './chart-config';
	import type { WorldState } from '../types';

	interface Props {
		config: ChartConfig;
		data: Map<string, WorldState[]>;
		colors: Map<string, string>;
	}

	let { config, data, colors }: Props = $props();

	let containerEl: HTMLDivElement;
	let width = $state(0);
	let height = $state(0);

	const margin = { top: 24, right: 16, bottom: 32, left: 56 };

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

	$effect(() => {
		if (!containerEl || width <= 0 || height <= 0) return;

		// Access reactive dependencies
		const _config = config;
		const _data = data;
		const _colors = colors;

		const innerW = width - margin.left - margin.right;
		const innerH = height - margin.top - margin.bottom;
		if (innerW <= 0 || innerH <= 0) return;

		const fmt = getFormatter(_config.format);

		// Extract all series
		const allSeries: Array<{ id: string; points: Array<{ year: number; value: number }>; color: string }> = [];
		for (const [id, states] of _data) {
			const points = extractSeries(states, _config.fieldPath);
			if (points.length > 0) {
				allSeries.push({ id, points, color: _colors.get(id) ?? '#888' });
			}
		}

		// Compute domains
		const allPoints = allSeries.flatMap((s) => s.points);
		if (allPoints.length === 0) {
			d3.select(containerEl).selectAll('svg').remove();
			return;
		}

		const xExtent = d3.extent(allPoints, (d) => d.year) as [number, number];
		const yExtent = d3.extent(allPoints, (d) => d.value) as [number, number];
		// Add 5% padding to y domain
		const yPad = (yExtent[1] - yExtent[0]) * 0.05 || 1;

		const xScale = d3.scaleLinear().domain(xExtent).range([0, innerW]);
		const yScale = d3.scaleLinear().domain([Math.max(0, yExtent[0] - yPad), yExtent[1] + yPad]).range([innerH, 0]);

		const line = d3.line<{ year: number; value: number }>()
			.x((d) => xScale(d.year))
			.y((d) => yScale(d.value));

		// SVG setup with join pattern
		const svg = d3.select(containerEl)
			.selectAll<SVGSVGElement, null>('svg')
			.data([null])
			.join('svg')
			.attr('width', width)
			.attr('height', height);

		// Chart group
		const g = svg
			.selectAll<SVGGElement, null>('g.chart')
			.data([null])
			.join('g')
			.attr('class', 'chart')
			.attr('transform', `translate(${margin.left},${margin.top})`);

		// X axis
		g.selectAll<SVGGElement, null>('g.x-axis')
			.data([null])
			.join('g')
			.attr('class', 'x-axis')
			.attr('transform', `translate(0,${innerH})`)
			.call(d3.axisBottom(xScale).tickFormat(d3.format('d')).ticks(Math.min(innerW / 80, 10)));

		// Y axis
		g.selectAll<SVGGElement, null>('g.y-axis')
			.data([null])
			.join('g')
			.attr('class', 'y-axis')
			.call(d3.axisLeft(yScale).tickFormat((d) => fmt(d as number)).ticks(6));

		// Y label
		g.selectAll<SVGTextElement, null>('text.y-label')
			.data([null])
			.join('text')
			.attr('class', 'y-label')
			.attr('transform', 'rotate(-90)')
			.attr('x', -innerH / 2)
			.attr('y', -42)
			.attr('text-anchor', 'middle')
			.attr('fill', 'var(--text-secondary)')
			.attr('font-size', '11px')
			.text(_config.yLabel);

		// Title
		svg.selectAll<SVGTextElement, null>('text.title')
			.data([null])
			.join('text')
			.attr('class', 'title')
			.attr('x', margin.left + innerW / 2)
			.attr('y', 16)
			.attr('text-anchor', 'middle')
			.attr('fill', 'var(--text)')
			.attr('font-size', '13px')
			.attr('font-weight', '600')
			.text(_config.title);

		// Lines
		const lines = g.selectAll<SVGPathElement, typeof allSeries[number]>('path.line')
			.data(allSeries, (d) => d.id);

		lines.join(
			(enter) =>
				enter
					.append('path')
					.attr('class', 'line')
					.attr('fill', 'none')
					.attr('stroke-width', 2)
					.attr('stroke', (d) => d.color)
					.attr('d', (d) => line(d.points)),
			(update) =>
				update
					.transition()
					.duration(300)
					.attr('stroke', (d) => d.color)
					.attr('d', (d) => line(d.points)),
			(exit) => exit.remove()
		);

		// Tooltip overlay
		let overlay = g.selectAll<SVGRectElement, null>('rect.overlay')
			.data([null])
			.join('rect')
			.attr('class', 'overlay')
			.attr('width', innerW)
			.attr('height', innerH)
			.attr('fill', 'none')
			.attr('pointer-events', 'all');

		let tooltip = g.selectAll<SVGGElement, null>('g.tooltip')
			.data([null])
			.join('g')
			.attr('class', 'tooltip')
			.style('display', 'none');

		let tooltipLine = tooltip.selectAll<SVGLineElement, null>('line')
			.data([null])
			.join('line')
			.attr('stroke', 'var(--border)')
			.attr('stroke-dasharray', '3,3')
			.attr('y1', 0)
			.attr('y2', innerH);

		overlay
			.on('mousemove', (event: MouseEvent) => {
				const [mx] = d3.pointer(event);
				const year = Math.round(xScale.invert(mx));
				tooltip.style('display', null);
				tooltipLine.attr('x1', xScale(year)).attr('x2', xScale(year));

				// Update tooltip text
				tooltip.selectAll('text.tooltip-text').remove();
				let ty = -4;
				for (const series of allSeries) {
					const pt = series.points.find((p) => Math.round(p.year) === year);
					if (pt) {
						tooltip
							.append('text')
							.attr('class', 'tooltip-text')
							.attr('x', xScale(year) + 6)
							.attr('y', ty)
							.attr('fill', series.color)
							.attr('font-size', '11px')
							.text(`${fmt(pt.value)}`);
						ty -= 14;
					}
				}
				// Year label
				tooltip
					.append('text')
					.attr('class', 'tooltip-text')
					.attr('x', xScale(year) + 6)
					.attr('y', ty)
					.attr('fill', 'var(--text)')
					.attr('font-size', '11px')
					.attr('font-weight', '600')
					.text(String(year));
			})
			.on('mouseleave', () => {
				tooltip.style('display', 'none');
			});
	});
</script>

<div class="chart-container" bind:this={containerEl} use:resize={handleResize}></div>

<style>
	.chart-container {
		width: 100%;
		height: 100%;
		min-height: 200px;
	}
	.chart-container :global(.x-axis text),
	.chart-container :global(.y-axis text) {
		fill: var(--text-secondary);
		font-size: 10px;
	}
	.chart-container :global(.x-axis line),
	.chart-container :global(.x-axis path),
	.chart-container :global(.y-axis line),
	.chart-container :global(.y-axis path) {
		stroke: var(--border);
	}
</style>
