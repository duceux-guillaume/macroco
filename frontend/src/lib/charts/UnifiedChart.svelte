<script lang="ts">
	import * as d3 from 'd3';
	import { resize } from '../utils/resize';
	import { extractSeries, normalizeSeries, type NormalizedPoint } from '../utils/extract';
	import { formatBillions, formatPercent, formatDecimal, formatInteger } from '../utils/format';
	import { selectedVariableId } from '../stores/info';
	import { hoveredYear, brushedXDomain } from '../stores/simulation';
	import { visibleVariables, compareMode, compareVariable } from '../stores/chart-ui';
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
	let isBrushing = false;

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

	function handleLegendToggle(fieldPath: string) {
		visibleVariables.update((set) => {
			const next = new Set(set);
			if (next.has(fieldPath)) {
				// Don't allow hiding all variables
				if (next.size > 1) next.delete(fieldPath);
			} else {
				next.add(fieldPath);
			}
			return next;
		});
	}

	function handleLegendInfoClick(e: MouseEvent, fieldPath: string) {
		e.stopPropagation();
		selectedVariableId.set(fieldPath);
	}

	// Shared line data type for both modes
	interface LineDatum {
		id: string;
		color: string;
		points: Array<{ year: number; normalized: number }>;
		rawPoints: Array<{ year: number; value: number }>;
		label: string;
		format: string;
	}

	$effect(() => {
		if (!containerEl || width <= 0 || height <= 0) return;

		const _data = data;
		const _colors = colors;
		const _focusedId = focusedScenarioId;
		const _compareMode = $compareMode;
		const _compareVariable = $compareVariable;
		const _visibleVars = $visibleVariables;

		const innerW = width - margin.left - margin.right;
		const innerH = height - margin.top - margin.bottom;
		if (innerW <= 0 || innerH <= 0) return;

		let linesData: LineDatum[] = [];
		let legendData: Array<{ id: string; label: string; color: string; fieldPath: string; visible: boolean }> = [];
		let useNormalizedY = true;

		if (_compareMode) {
			// Compare mode: 1 variable x N scenarios (native scale, scenario colors)
			const varConfig = unifiedVariables.find((v) => v.fieldPath === _compareVariable) ?? unifiedVariables[0];
			const fmt = getFormatter(varConfig.format);
			useNormalizedY = false;

			for (const [scenarioId, states] of _data) {
				const rawPoints = extractSeries(states, varConfig.fieldPath);
				if (rawPoints.length === 0) continue;
				const color = _colors.get(scenarioId) ?? '#888';
				linesData.push({
					id: scenarioId,
					color,
					points: rawPoints.map((p) => ({ year: p.year, normalized: p.value })),
					rawPoints,
					label: scenarioId.slice(0, 8),
					format: varConfig.format
				});
			}

			// Legend shows scenarios
			for (const [scenarioId] of _data) {
				const color = _colors.get(scenarioId) ?? '#888';
				legendData.push({
					id: scenarioId,
					label: scenarioId.slice(0, 8),
					color,
					fieldPath: varConfig.fieldPath,
					visible: true
				});
			}
		} else {
			// Normal mode: all variables for focused scenario, normalized
			const scenarioId = _focusedId ?? _data.keys().next().value;
			if (!scenarioId) return;
			const states = _data.get(scenarioId);
			if (!states || states.length === 0) return;

			for (const varConfig of unifiedVariables) {
				const rawPoints = extractSeries(states, varConfig.fieldPath);
				if (rawPoints.length === 0) continue;
				const { points: normalizedPoints } = normalizeSeries(rawPoints);
				const visible = _visibleVars.has(varConfig.fieldPath);
				if (visible) {
					linesData.push({
						id: varConfig.id,
						color: varConfig.color,
						points: normalizedPoints,
						rawPoints,
						label: varConfig.shortLabel,
						format: varConfig.format
					});
				}
			}

			// Legend always shows all 6 variables (with visibility state)
			legendData = unifiedVariables.map((v) => ({
				id: v.id,
				label: v.label,
				color: v.color,
				fieldPath: v.fieldPath,
				visible: _visibleVars.has(v.fieldPath)
			}));
		}

		if (linesData.length === 0 && !_compareMode) return;

		// X domain from all lines
		const allYears = linesData.flatMap((l) => l.rawPoints.map((p) => p.year));
		if (allYears.length === 0) return;
		const xExtent = d3.extent(allYears) as [number, number];

		const xScale = d3.scaleLinear().domain(xExtent).range([0, innerW]);

		// Y scale
		let yScale: d3.ScaleLinear<number, number>;
		let yTickFormat: (d: d3.NumberValue) => string;

		if (useNormalizedY) {
			yScale = d3.scaleLinear().domain([0, 1]).range([innerH, 0]);
			yTickFormat = d3.format('.1f');
		} else {
			const allVals = linesData.flatMap((l) => l.points.map((p) => p.normalized));
			const yExtent = d3.extent(allVals) as [number, number];
			const yPad = (yExtent[1] - yExtent[0]) * 0.05 || 1;
			yScale = d3.scaleLinear().domain([Math.max(0, yExtent[0] - yPad), yExtent[1] + yPad]).range([innerH, 0]);
			const varConfig = unifiedVariables.find((v) => v.fieldPath === _compareVariable) ?? unifiedVariables[0];
			const fmt = getFormatter(varConfig.format);
			yTickFormat = (d) => fmt(d as number);
		}

		const line = d3.line<{ year: number; normalized: number }>()
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

		// Y axis
		g.selectAll<SVGGElement, null>('g.y-axis')
			.data([null])
			.join('g')
			.attr('class', 'y-axis')
			.call(d3.axisLeft(yScale).ticks(5).tickFormat(yTickFormat));

		// Lines
		const lines = g.selectAll<SVGPathElement, LineDatum>('path.var-line')
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

		// Annotations (only in normal mode, for visible variables)
		const annotations: Array<{ year: number; label: string; color: string }> = [];
		if (!_compareMode) {
			for (const ld of linesData) {
				const varConfig = unifiedVariables.find((v) => v.id === ld.id);
				if (!varConfig) continue;
				const anns = getAnnotations(varConfig.id, varConfig.fieldPath, _data, _focusedId);
				for (const a of anns) {
					annotations.push({ year: a.year, label: `${varConfig.shortLabel} ${a.label}`, color: varConfig.color });
				}
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
			.selectAll<SVGGElement, typeof legendData[number]>('g.legend-item')
			.data(legendData, (d) => d.id);

		legendItems.join(
			(enter) => {
				const item = enter.append('g')
					.attr('class', 'legend-item')
					.attr('transform', (_, i) => `translate(0, ${i * 22})`)
					.attr('cursor', 'pointer')
					.attr('opacity', (d) => d.visible ? 1 : 0.35)
					.on('click', (_, d) => {
						if (!_compareMode) handleLegendToggle(d.fieldPath);
					});

				item.append('rect')
					.attr('width', 12)
					.attr('height', 3)
					.attr('y', 5)
					.attr('rx', 1.5)
					.attr('fill', (d) => d.color);

				item.append('text')
					.attr('x', 18)
					.attr('y', 10)
					.attr('fill', 'var(--text-secondary)')
					.attr('font-size', '11px')
					.text((d) => d.label);

				return item;
			},
			(update) => {
				update
					.attr('transform', (_, i) => `translate(0, ${i * 22})`)
					.attr('opacity', (d) => d.visible ? 1 : 0.35);
				update.select('text').text((d) => d.label);
				update.select('rect').attr('fill', (d) => d.color);
				// Rebind click handler for current mode
				update.on('click', (_, d) => {
					if (!_compareMode) handleLegendToggle(d.fieldPath);
				});
				return update;
			},
			(exit) => exit.remove()
		);

		// Brush (below tooltip overlay)
		const brushGroup = g.selectAll<SVGGElement, null>('g.brush')
			.data([null])
			.join('g')
			.attr('class', 'brush');

		const brush = d3.brushX<null>()
			.extent([[0, 0], [innerW, innerH]])
			.on('start', () => {
				isBrushing = true;
			})
			.on('end', (event: d3.D3BrushEvent<null>) => {
				isBrushing = false;
				if (!event.selection) {
					brushedXDomain.set(null);
					return;
				}
				const [x0, x1] = event.selection as [number, number];
				const yearStart = Math.round(xScale.invert(x0));
				const yearEnd = Math.round(xScale.invert(x1));
				if (yearEnd - yearStart < 5) {
					brushGroup.call(brush.move, null);
					brushedXDomain.set(null);
					return;
				}
				brushedXDomain.set([yearStart, yearEnd]);
			});

		brushGroup.call(brush);

		brushGroup.select('.selection')
			.attr('fill', 'var(--accent)')
			.attr('fill-opacity', 0.15)
			.attr('stroke', 'var(--accent)')
			.attr('stroke-opacity', 0.4);

		// Tooltip overlay (above brush so mousemove still works)
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
				if (isBrushing) return;

				const [mx] = d3.pointer(event);
				const year = Math.round(xScale.invert(mx));

				tooltipLine
					.style('display', null)
					.attr('x1', xScale(year))
					.attr('x2', xScale(year));

				const items: typeof tooltipItems = [];
				for (const ld of linesData) {
					const fmt = getFormatter(ld.format);
					const idx = ld.rawPoints.findIndex((p) => Math.round(p.year) === year);
					if (idx >= 0) {
						const pt = ld.rawPoints[idx];
						let trend = '';
						if (idx > 0) {
							const prev = ld.rawPoints[idx - 1].value;
							const diff = pt.value - prev;
							const pct = prev !== 0 ? Math.abs(diff / prev) : 0;
							if (pct < 0.001) trend = '\u2192';
							else if (diff > 0) trend = '\u2191';
							else trend = '\u2193';
						}
						items.push({
							label: ld.label,
							color: ld.color,
							rawValue: fmt(pt.value),
							unit: '',
							trend
						});
					}
				}

				tooltipYear = year;
				tooltipItems = items;
				hoveredYear.set(year);

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
				hoveredYear.set(null);
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
