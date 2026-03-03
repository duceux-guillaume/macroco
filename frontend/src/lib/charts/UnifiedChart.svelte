<script lang="ts">
	import * as d3 from 'd3';
	import { resize } from '../utils/resize';
	import { extractSeries, normalizeSeries, type NormalizedPoint } from '../utils/extract';
	import { formatBillions, formatPercent, formatDecimal, formatInteger } from '../utils/format';
	import { selectedVariableId } from '../stores/info';
	import { getAnnotations } from '../content/chart-annotations';
	import { unifiedVariables, type UnifiedVariableConfig } from './unified-config';
	import type { WorldState } from '../types';

	interface Props {
		data: Map<string, WorldState[]>;
		colors: Map<string, string>;
		focusedScenarioId?: string | null;
	}

	let { data, colors, focusedScenarioId = null }: Props = $props();

	let containerEl: HTMLDivElement;
	let width = $state(0);
	let height = $state(0);
	let tooltipVisible = $state(false);
	let tooltipX = $state(0);
	let tooltipY = $state(0);
	let tooltipYear = $state(0);
	let tooltipItems = $state<Array<{ label: string; color: string; rawValue: string; unit: string; trend: string }>>([]);

	const margin = { top: 32, right: 160, bottom: 32, left: 48 };

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

	function handleLegendClick(fieldPath: string) {
		selectedVariableId.set(fieldPath);
	}

	// Precompute series per variable for the focused scenario
	interface VariableSeries {
		config: UnifiedVariableConfig;
		normalizedPoints: NormalizedPoint[];
		rawPoints: Array<{ year: number; value: number }>;
	}

	$effect(() => {
		if (!containerEl || width <= 0 || height <= 0) return;

		const _data = data;
		const _focusedId = focusedScenarioId;

		const innerW = width - margin.left - margin.right;
		const innerH = height - margin.top - margin.bottom;
		if (innerW <= 0 || innerH <= 0) return;

		// Pick the focused scenario (or first available)
		const scenarioId = _focusedId ?? _data.keys().next().value;
		if (!scenarioId) return;
		const states = _data.get(scenarioId);
		if (!states || states.length === 0) return;

		// Extract and normalize all 6 variables
		const allVarSeries: VariableSeries[] = [];
		for (const varConfig of unifiedVariables) {
			const rawPoints = extractSeries(states, varConfig.fieldPath);
			if (rawPoints.length === 0) continue;
			const { points: normalizedPoints } = normalizeSeries(rawPoints);
			allVarSeries.push({ config: varConfig, normalizedPoints, rawPoints });
		}

		if (allVarSeries.length === 0) return;

		// X domain from first series (all share the same time range)
		const xExtent = d3.extent(allVarSeries[0].rawPoints, (d) => d.year) as [number, number];

		const xScale = d3.scaleLinear().domain(xExtent).range([0, innerW]);
		const yScale = d3.scaleLinear().domain([0, 1]).range([innerH, 0]);

		const line = d3.line<NormalizedPoint>()
			.x((d) => xScale(d.year))
			.y((d) => yScale(d.normalized));

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

		// Y axis (normalized 0-1)
		g.selectAll<SVGGElement, null>('g.y-axis')
			.data([null])
			.join('g')
			.attr('class', 'y-axis')
			.call(d3.axisLeft(yScale).ticks(5).tickFormat(d3.format('.1f')));

		// Lines — one per variable
		const linesData = allVarSeries.map((vs) => ({
			id: vs.config.id,
			color: vs.config.color,
			points: vs.normalizedPoints
		}));

		const lines = g.selectAll<SVGPathElement, typeof linesData[number]>('path.var-line')
			.data(linesData, (d) => d.id);

		lines.join(
			(enter) =>
				enter
					.append('path')
					.attr('class', 'var-line')
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

		// Annotations (from all visible variables)
		const annotations: Array<{ year: number; label: string; color: string }> = [];
		for (const vs of allVarSeries) {
			const anns = getAnnotations(vs.config.id, vs.config.fieldPath, _data, _focusedId);
			for (const a of anns) {
				annotations.push({ year: a.year, label: `${vs.config.shortLabel} ${a.label}`, color: vs.config.color });
			}
		}

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
					.attr('stroke', (d) => d.color)
					.attr('stroke-width', 1)
					.attr('stroke-dasharray', '4,3')
					.attr('opacity', 0.4);
				ann.append('text')
					.attr('x', (d) => xScale(d.year) + 3)
					.attr('y', 10)
					.attr('fill', (d) => d.color)
					.attr('font-size', '9px')
					.attr('opacity', 0.7)
					.attr('transform', (d) => `rotate(-45, ${xScale(d.year) + 3}, 10)`)
					.text((d) => d.label);
				return ann;
			},
			(update) => {
				update.select('line')
					.attr('x1', (d) => xScale(d.year))
					.attr('x2', (d) => xScale(d.year))
					.attr('y2', innerH)
					.attr('stroke', (d) => d.color);
				update.select('text')
					.attr('x', (d) => xScale(d.year) + 3)
					.attr('fill', (d) => d.color)
					.attr('transform', (d) => `rotate(-45, ${xScale(d.year) + 3}, 10)`)
					.text((d) => d.label);
				return update;
			},
			(exit) => exit.remove()
		);

		// Legend (right side, inside SVG)
		const legendGroup = svg.selectAll<SVGGElement, null>('g.legend')
			.data([null])
			.join('g')
			.attr('class', 'legend')
			.attr('transform', `translate(${margin.left + innerW + 16}, ${margin.top + 8})`);

		const legendItems = legendGroup
			.selectAll<SVGGElement, typeof allVarSeries[number]>('g.legend-item')
			.data(allVarSeries, (d) => d.config.id);

		legendItems.join(
			(enter) => {
				const item = enter.append('g')
					.attr('class', 'legend-item')
					.attr('transform', (_, i) => `translate(0, ${i * 22})`)
					.attr('cursor', 'pointer')
					.on('click', (_, d) => handleLegendClick(d.config.fieldPath));

				item.append('rect')
					.attr('width', 12)
					.attr('height', 3)
					.attr('y', 5)
					.attr('rx', 1.5)
					.attr('fill', (d) => d.config.color);

				item.append('text')
					.attr('x', 18)
					.attr('y', 10)
					.attr('fill', 'var(--text-secondary)')
					.attr('font-size', '11px')
					.text((d) => d.config.label);

				return item;
			},
			(update) => {
				update.attr('transform', (_, i) => `translate(0, ${i * 22})`);
				return update;
			},
			(exit) => exit.remove()
		);

		// Tooltip overlay
		const overlay = g.selectAll<SVGRectElement, null>('rect.overlay')
			.data([null])
			.join('rect')
			.attr('class', 'overlay')
			.attr('width', innerW)
			.attr('height', innerH)
			.attr('fill', 'none')
			.attr('pointer-events', 'all');

		// Vertical tooltip line
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

				// Build tooltip items from all variables
				const items: typeof tooltipItems = [];
				for (const vs of allVarSeries) {
					const fmt = getFormatter(vs.config.format);
					const idx = vs.rawPoints.findIndex((p) => Math.round(p.year) === year);
					if (idx >= 0) {
						const pt = vs.rawPoints[idx];
						let trend = '';
						if (idx > 0) {
							const prev = vs.rawPoints[idx - 1].value;
							const diff = pt.value - prev;
							const pct = prev !== 0 ? Math.abs(diff / prev) : 0;
							if (pct < 0.001) trend = '\u2192';
							else if (diff > 0) trend = '\u2191';
							else trend = '\u2193';
						}
						items.push({
							label: vs.config.shortLabel,
							color: vs.config.color,
							rawValue: fmt(pt.value),
							unit: vs.config.unit,
							trend
						});
					}
				}

				tooltipYear = year;
				tooltipItems = items;

				// Position tooltip
				const px = margin.left + xScale(year);
				tooltipX = px + 12;
				if (tooltipX + 200 > width) {
					tooltipX = px - 210;
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

<div class="unified-chart" bind:this={containerEl} use:resize={handleResize}>
	{#if tooltipVisible}
		<div
			class="tooltip-html"
			style="left: {tooltipX}px; top: {tooltipY}px;"
		>
			<div class="tooltip-year">{tooltipYear}</div>
			{#each tooltipItems as item}
				<div class="tooltip-row">
					<span class="tooltip-dot" style="background: {item.color}"></span>
					<span class="tooltip-label">{item.label}</span>
					<span class="tooltip-val">{item.rawValue}</span>
					{#if item.trend}
						<span class="tooltip-trend">{item.trend}</span>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.unified-chart {
		width: 100%;
		height: 100%;
		min-height: 200px;
		position: relative;
	}
	.unified-chart :global(.x-axis text),
	.unified-chart :global(.y-axis text) {
		fill: var(--text-secondary);
		font-size: 10px;
	}
	.unified-chart :global(.x-axis line),
	.unified-chart :global(.x-axis path),
	.unified-chart :global(.y-axis line),
	.unified-chart :global(.y-axis path) {
		stroke: var(--border);
	}
	.unified-chart :global(.legend-item text:hover) {
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
		min-width: 160px;
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
		line-height: 1.5;
	}
	.tooltip-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.tooltip-label {
		color: var(--text-secondary);
		font-size: 11px;
		min-width: 32px;
	}
	.tooltip-val {
		color: var(--text);
		font-variant-numeric: tabular-nums;
	}
	.tooltip-trend {
		color: var(--text-secondary);
		font-size: 11px;
	}
</style>
