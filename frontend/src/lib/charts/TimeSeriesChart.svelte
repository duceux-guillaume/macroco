<script lang="ts">
	import * as d3 from 'd3';
	import { resize } from '../utils/resize';
	import { extractSeries } from '../utils/extract';
	import { formatBillions, formatPercent, formatDecimal, formatInteger } from '../utils/format';
	import { selectedVariableId } from '../stores/info';
	import { variableDescriptions } from '../content/variable-descriptions';
	import { getAnnotations } from '../content/chart-annotations';
	import type { ChartConfig } from './chart-config';
	import type { WorldState } from '../types';

	interface Props {
		config: ChartConfig;
		data: Map<string, WorldState[]>;
		colors: Map<string, string>;
		focusedScenarioId?: string | null;
	}

	let { config, data, colors, focusedScenarioId = null }: Props = $props();

	let containerEl: HTMLDivElement;
	let tooltipEl = $state<HTMLDivElement>();
	let width = $state(0);
	let height = $state(0);
	let tooltipVisible = $state(false);
	let tooltipX = $state(0);
	let tooltipY = $state(0);
	let tooltipYear = $state(0);
	let tooltipItems = $state<Array<{ color: string; value: string; trend: string }>>([]);

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

	function handleTitleClick() {
		selectedVariableId.set(config.fieldPath);
	}

	// Get the beginner description for this variable
	const varInfo = $derived(variableDescriptions[config.fieldPath]);
	const shortDesc = $derived(varInfo?.beginner ?? '');

	$effect(() => {
		if (!containerEl || width <= 0 || height <= 0) return;

		const _config = config;
		const _data = data;
		const _colors = colors;
		const _focusedId = focusedScenarioId;

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
		const yPad = (yExtent[1] - yExtent[0]) * 0.05 || 1;

		const xScale = d3.scaleLinear().domain(xExtent).range([0, innerW]);
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

		// Title (clickable)
		const titleEl = svg.selectAll<SVGTextElement, null>('text.title')
			.data([null])
			.join('text')
			.attr('class', 'title')
			.attr('x', margin.left + innerW / 2)
			.attr('y', 16)
			.attr('text-anchor', 'middle')
			.attr('fill', 'var(--text)')
			.attr('font-size', '13px')
			.attr('font-weight', '600')
			.attr('cursor', 'pointer')
			.text(_config.title);

		titleEl.on('click', handleTitleClick);

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

		// Annotations
		const annotations = getAnnotations(_config.id, _config.fieldPath, _data, _focusedId);
		const annotationGroup = g.selectAll<SVGGElement, null>('g.annotations')
			.data([null])
			.join('g')
			.attr('class', 'annotations');

		const annSel = annotationGroup
			.selectAll<SVGGElement, typeof annotations[number]>('g.annotation')
			.data(annotations, (d) => `${d.year}-${d.label}`);

		annSel.join(
			(enter) => {
				const ann = enter.append('g').attr('class', 'annotation');
				ann.append('line')
					.attr('x1', (d) => xScale(d.year))
					.attr('x2', (d) => xScale(d.year))
					.attr('y1', 0)
					.attr('y2', innerH)
					.attr('stroke', 'var(--text-secondary)')
					.attr('stroke-width', 1)
					.attr('stroke-dasharray', '4,3')
					.attr('opacity', 0.5);
				ann.append('text')
					.attr('x', (d) => xScale(d.year) + 3)
					.attr('y', 10)
					.attr('fill', 'var(--text-secondary)')
					.attr('font-size', '9px')
					.attr('transform', (d) => `rotate(-45, ${xScale(d.year) + 3}, 10)`)
					.text((d) => d.label);
				return ann;
			},
			(update) => {
				update.select('line')
					.attr('x1', (d) => xScale(d.year))
					.attr('x2', (d) => xScale(d.year))
					.attr('y2', innerH);
				update.select('text')
					.attr('x', (d) => xScale(d.year) + 3)
					.attr('transform', (d) => `rotate(-45, ${xScale(d.year) + 3}, 10)`)
					.text((d) => d.label);
				return update;
			},
			(exit) => exit.remove()
		);

		// Tooltip overlay (invisible rect for mouse events)
		const overlay = g.selectAll<SVGRectElement, null>('rect.overlay')
			.data([null])
			.join('rect')
			.attr('class', 'overlay')
			.attr('width', innerW)
			.attr('height', innerH)
			.attr('fill', 'none')
			.attr('pointer-events', 'all');

		// SVG vertical line for tooltip
		const tooltipLine = g.selectAll<SVGLineElement, null>('line.tooltip-line')
			.data([null])
			.join('line')
			.attr('class', 'tooltip-line')
			.attr('stroke', 'var(--border)')
			.attr('stroke-dasharray', '3,3')
			.attr('y1', 0)
			.attr('y2', innerH)
			.style('display', 'none');

		overlay
			.on('mousemove', (event: MouseEvent) => {
				const [mx] = d3.pointer(event);
				const year = Math.round(xScale.invert(mx));

				tooltipLine
					.style('display', null)
					.attr('x1', xScale(year))
					.attr('x2', xScale(year));

				// Compute tooltip items
				const items: typeof tooltipItems = [];
				for (const series of allSeries) {
					const idx = series.points.findIndex((p) => Math.round(p.year) === year);
					if (idx >= 0) {
						const pt = series.points[idx];
						let trend = '';
						if (idx > 0) {
							const prev = series.points[idx - 1].value;
							const diff = pt.value - prev;
							const pct = prev !== 0 ? Math.abs(diff / prev) : 0;
							if (pct < 0.001) trend = '→';
							else if (diff > 0) trend = '↑';
							else trend = '↓';
						}
						items.push({ color: series.color, value: fmt(pt.value), trend });
					}
				}

				tooltipYear = year;
				tooltipItems = items;

				// Position HTML tooltip
				const rect = containerEl.getBoundingClientRect();
				const px = margin.left + xScale(year);
				tooltipX = px + 12;
				// Flip to left side if near right edge
				if (tooltipX + 160 > width) {
					tooltipX = px - 170;
				}
				tooltipY = margin.top + 8;
				tooltipVisible = true;
			})
			.on('mouseleave', () => {
				tooltipLine.style('display', 'none');
				tooltipVisible = false;
			});
	});
</script>

<div class="chart-container" bind:this={containerEl} use:resize={handleResize}>
	{#if tooltipVisible}
		<div
			class="tooltip-html"
			bind:this={tooltipEl}
			style="left: {tooltipX}px; top: {tooltipY}px;"
		>
			<div class="tooltip-year">{tooltipYear}</div>
			{#each tooltipItems as item}
				<div class="tooltip-row">
					<span class="tooltip-dot" style="background: {item.color}"></span>
					<span class="tooltip-val">{item.value}</span>
					{#if item.trend}
						<span class="tooltip-trend">{item.trend}</span>
					{/if}
				</div>
			{/each}
			{#if shortDesc}
				<div class="tooltip-desc">{shortDesc}</div>
			{/if}
			<div class="tooltip-hint">Click title for more</div>
		</div>
	{/if}
</div>

<style>
	.chart-container {
		width: 100%;
		height: 100%;
		min-height: 200px;
		position: relative;
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
	.chart-container :global(text.title:hover) {
		fill: var(--accent) !important;
		text-decoration: underline;
	}
	.tooltip-html {
		position: absolute;
		pointer-events: none;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 8px 10px;
		font-size: 12px;
		z-index: 10;
		min-width: 120px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
	}
	.tooltip-year {
		font-weight: 600;
		color: var(--text);
		margin-bottom: 4px;
	}
	.tooltip-row {
		display: flex;
		align-items: center;
		gap: 5px;
		line-height: 1.4;
	}
	.tooltip-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.tooltip-val {
		color: var(--text);
	}
	.tooltip-trend {
		color: var(--text-secondary);
		font-size: 11px;
	}
	.tooltip-desc {
		margin-top: 6px;
		padding-top: 6px;
		border-top: 1px solid var(--border);
		color: var(--text-secondary);
		font-size: 11px;
		line-height: 1.4;
	}
	.tooltip-hint {
		margin-top: 4px;
		color: var(--accent);
		font-size: 10px;
		opacity: 0.7;
	}
</style>
