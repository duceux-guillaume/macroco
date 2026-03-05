<script lang="ts">
	import * as d3 from 'd3';
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { resize } from '../utils/resize';
	import { extractSeries } from '../utils/extract';
	import { formatBillions, formatPercent, formatDecimal, formatInteger } from '../utils/format';
	import { selectedVariableId, selectedHistoricalId, highlightedVariables, highlightHistoricalOnly } from '../stores/info';
	import { hoveredYear } from '../stores/simulation';
	import { compareMode, compareVariable, showHistorical, visibleVariables } from '../stores/chart-ui';
	import { historicalData } from '../stores/historical';
	import { getAnnotations } from '../content/chart-annotations';
	import { unifiedVariables, type UnifiedVariableConfig } from './unified-config';
	import type { WorldState } from '../types';

	const NOW_YEAR = 2026;
	const HIST_LINE_PREFIX = 'hist-';

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

	let margin = $derived(
		width < 500
			? { top: 24, right: 16, bottom: 80, left: 36 }
			: { top: 32, right: 200, bottom: 32, left: 48 }
	);
	let isBrushing = false;

	// Eye icon SVG paths (16x16 viewBox)
	const EYE_OPEN = 'M1.3,8 C4,2.7 12,2.7 14.7,8 C12,13.3 4,13.3 1.3,8 Z M8,6 a2,2 0 1,0 0.01,0';
	const EYE_CLOSED = 'M1.3,8 C4,2.7 12,2.7 14.7,8 C12,13.3 4,13.3 1.3,8 Z M2.7,2.7 L13.3,13.3';

	// Store brush reference for keyboard clear
	let activeBrushGroup: d3.Selection<SVGGElement, null, null, undefined> | null = null;
	let activeBrush: d3.BrushBehavior<null> | null = null;

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

	function handleLegendSelect(fieldPath: string) {
		if (get(selectedVariableId) === fieldPath) {
			selectedVariableId.set(null);
		} else {
			selectedVariableId.set(fieldPath);
		}
	}

	// Keyboard shortcuts
	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (get(selectedHistoricalId)) {
				selectedHistoricalId.set(null);
				return;
			}
			if (get(selectedVariableId)) {
				selectedVariableId.set(null);
				return;
			}
			// Then clear brush
			if (activeBrushGroup && activeBrush) {
				activeBrushGroup.call(activeBrush.move, null);
				return;
			}
		}
		// Number keys 1-6 select variables (only in normal mode)
		if (!$compareMode && e.key >= '1' && e.key <= '6') {
			const idx = parseInt(e.key) - 1;
			if (idx < unifiedVariables.length) {
				handleLegendSelect(unifiedVariables[idx].fieldPath);
			}
		}
	}

	onMount(() => {
		window.addEventListener('keydown', handleKeydown);
		return () => window.removeEventListener('keydown', handleKeydown);
	});

	// Shared line data type for both modes
	interface LineDatum {
		id: string;
		color: string;
		points: Array<{ year: number; y: number }>;
		rawPoints: Array<{ year: number; value: number }>;
		label: string;
		format: string;
		historical?: boolean;
	}

	$effect(() => {
		if (!containerEl || width <= 0 || height <= 0) return;

		const _data = data;
		const _colors = colors;
		const _focusedId = focusedScenarioId;
		const _compareMode = $compareMode;
		const _compareVariable = $compareVariable;
		const _showHistorical = $showHistorical;
		const _historicalData = $historicalData;
		const _visibleVars = $visibleVariables;

		const innerW = width - margin.left - margin.right;
		const innerH = height - margin.top - margin.bottom;
		if (innerW <= 0 || innerH <= 0) return;

		let linesData: LineDatum[] = [];
		let legendData: Array<{ id: string; label: string; color: string; fieldPath: string }> = [];
		let useNormalizedY = true;

		if (_compareMode) {
			const varConfig = unifiedVariables.find((v) => v.fieldPath === _compareVariable) ?? unifiedVariables[0];
			useNormalizedY = false;

			for (const [scenarioId, states] of _data) {
				const rawPoints = extractSeries(states, varConfig.fieldPath);
				if (rawPoints.length === 0) continue;
				const color = _colors.get(scenarioId) ?? '#888';
				linesData.push({
					id: scenarioId,
					color,
					points: rawPoints.map((p) => ({ year: p.year, y: p.value })),
					rawPoints,
					label: scenarioId.slice(0, 8),
					format: varConfig.format
				});
			}

			// Add historical overlay in compare mode
			if (_showHistorical) {
				const histVar = _historicalData.get(varConfig.id);
				if (histVar && histVar.data.length > 0) {
					linesData.push({
						id: `${HIST_LINE_PREFIX}${varConfig.id}`,
						color: '#9ca3af',
						points: histVar.data.map((d) => ({ year: d.year, y: d.value })),
						rawPoints: histVar.data.map((d) => ({ year: d.year, value: d.value })),
						label: 'Historical',
						format: varConfig.format,
						historical: true
					});
				}
			}

			for (const [scenarioId] of _data) {
				const color = _colors.get(scenarioId) ?? '#888';
				legendData.push({
					id: scenarioId,
					label: scenarioId.slice(0, 8),
					color,
					fieldPath: varConfig.fieldPath
				});
			}

			// Add historical toggle to legend in compare mode
			if (_historicalData.has(varConfig.id)) {
				legendData.push({
					id: 'historical',
					label: 'Historical',
					color: '#9ca3af',
					fieldPath: '__historical__'
				});
			}
		} else {
			const scenarioId = _focusedId ?? _data.keys().next().value;
			if (!scenarioId) return;
			const states = _data.get(scenarioId);
			if (!states || states.length === 0) return;

			// Extract series and compute combined sim+historical range for normalization
			for (const varConfig of unifiedVariables) {
				const rawPoints = extractSeries(states, varConfig.fieldPath);
				if (rawPoints.length === 0) continue;

				// Compute combined min/max including historical data
				let combinedMin = Infinity;
				let combinedMax = -Infinity;
				for (const p of rawPoints) {
					if (p.value < combinedMin) combinedMin = p.value;
					if (p.value > combinedMax) combinedMax = p.value;
				}

				const histVar = _showHistorical ? _historicalData.get(varConfig.id) : undefined;
				if (histVar && histVar.data.length > 0) {
					for (const d of histVar.data) {
						if (d.value < combinedMin) combinedMin = d.value;
						if (d.value > combinedMax) combinedMax = d.value;
					}
				}

				const range = combinedMax - combinedMin;
				const normalize = (v: number) => range > 0 ? (v - combinedMin) / range : 0.5;

				// Add simulation line
				linesData.push({
					id: varConfig.id,
					color: varConfig.color,
					points: rawPoints.map((p) => ({ year: p.year, y: normalize(p.value) })),
					rawPoints,
					label: varConfig.shortLabel,
					format: varConfig.format
				});

				// Add historical overlay line
				if (histVar && histVar.data.length > 0) {
					linesData.push({
						id: `${HIST_LINE_PREFIX}${varConfig.id}`,
						color: varConfig.color,
						points: histVar.data.map((d) => ({ year: d.year, y: normalize(d.value) })),
						rawPoints: histVar.data.map((d) => ({ year: d.year, value: d.value })),
						label: `${varConfig.shortLabel} hist.`,
						format: varConfig.format,
						historical: true
					});
				}
			}

			legendData = unifiedVariables.map((v) => ({
				id: v.id,
				label: v.label,
				color: v.color,
				fieldPath: v.fieldPath
			}));

			// Add historical toggle to legend
			if (_historicalData.size > 0) {
				legendData.push({
					id: 'historical',
					label: 'Historical',
					color: '#9ca3af',
					fieldPath: '__historical__'
				});
			}
		}

		// Filter hidden variables (normal mode only; historical handled by _showHistorical)
		if (!_compareMode) {
			linesData = linesData.filter((ld) => {
				if (ld.historical) return true;
				const varConfig = unifiedVariables.find((v) => v.id === ld.id);
				return varConfig ? _visibleVars.has(varConfig.fieldPath) : true;
			});
		}

		if (linesData.length === 0 && !_compareMode) return;

		const allYears = linesData.flatMap((l) => l.rawPoints.map((p) => p.year));
		if (allYears.length === 0) return;
		const xExtent = d3.extent(allYears) as [number, number];

		const xScale = d3.scaleLinear().domain(xExtent).range([0, innerW]);

		let yScale: d3.ScaleLinear<number, number>;
		let yTickFormat: (d: d3.NumberValue) => string;

		if (useNormalizedY) {
			yScale = d3.scaleLinear().domain([0, 1]).range([innerH, 0]);
			yTickFormat = d3.format('.1f');
		} else {
			const allVals = linesData.flatMap((l) => l.points.map((p) => p.y));
			const yExtent = d3.extent(allVals) as [number, number];
			const yPad = (yExtent[1] - yExtent[0]) * 0.05 || 1;
			yScale = d3.scaleLinear().domain([Math.max(0, yExtent[0] - yPad), yExtent[1] + yPad]).range([innerH, 0]);
			const varConfig = unifiedVariables.find((v) => v.fieldPath === _compareVariable) ?? unifiedVariables[0];
			const fmt = getFormatter(varConfig.format);
			yTickFormat = (d) => fmt(d as number);
		}

		const line = d3.line<{ year: number; y: number }>()
			.x((d) => xScale(d.year))
			.y((d) => yScale(d.y));

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
			.transition()
			.duration(400)
			.call(d3.axisBottom(xScale).tickFormat(d3.format('d')).ticks(Math.min(innerW / 80, 10)) as any);

		// Y axis
		g.selectAll<SVGGElement, null>('g.y-axis')
			.data([null])
			.join('g')
			.attr('class', 'y-axis')
			.transition()
			.duration(400)
			.call(d3.axisLeft(yScale).ticks(5).tickFormat(yTickFormat) as any);

		// Highlighting: dim lines not related to the selected parameter
		const highlighted = $highlightedVariables;
		const hasHighlight = highlighted.size > 0;
		const selectedVarFieldPath = $selectedVariableId;
		const histOnly = $highlightHistoricalOnly;

		function isLineHighlighted(d: LineDatum): boolean {
			if (!hasHighlight) return false;
			if (_compareMode) return false;
			const baseId = d.id.startsWith(HIST_LINE_PREFIX) ? d.id.slice(HIST_LINE_PREFIX.length) : d.id;
			const varConfig = unifiedVariables.find((v) => v.id === baseId);
			if (!varConfig || !highlighted.has(varConfig.fieldPath)) return false;
			if (histOnly && !d.historical) return false;
			return true;
		}

		function getLineOpacity(d: LineDatum): number {
			const base = d.historical ? 0.6 : 1;
			if (!hasHighlight) return base;
			return isLineHighlighted(d) ? base : 0.15;
		}

		function getLineWidth(d: LineDatum): number {
			const base = d.historical ? 1.5 : 2;
			if (!isLineHighlighted(d)) return base;
			return d.historical ? 2 : 2.5;
		}

		function getLegendOpacity(d: typeof legendData[number]): number {
			const isVisible = d.fieldPath === '__historical__'
				? _showHistorical
				: _visibleVars.has(d.fieldPath);
			if (!isVisible) return 0.35;
			if (histOnly) return d.fieldPath === '__historical__' ? 1 : 0.35;
			if (selectedVarFieldPath && d.fieldPath !== selectedVarFieldPath) return 0.35;
			return 1;
		}

		// Lines
		const lines = g.selectAll<SVGPathElement, LineDatum>('path.var-line')
			.data(linesData, (d) => d.id);

		lines.join(
			(enter) =>
				enter
					.append('path')
					.attr('class', 'var-line')
					.attr('fill', 'none')
					.attr('stroke-width', (d) => getLineWidth(d))
					.attr('stroke-dasharray', (d) => d.historical ? '6,3' : null)
					.attr('stroke', (d) => d.color)
					.attr('opacity', 0)
					.attr('d', (d) => line(d.points))
					.transition()
					.duration(400)
					.attr('opacity', (d) => getLineOpacity(d)),
			(update) =>
				update
					.attr('stroke-width', (d) => getLineWidth(d))
					.attr('stroke-dasharray', (d) => d.historical ? '6,3' : null)
					.transition()
					.duration(400)
					.attr('stroke', (d) => d.color)
					.attr('stroke-width', (d) => getLineWidth(d))
					.attr('opacity', (d) => getLineOpacity(d))
					.attr('d', (d) => line(d.points)),
			(exit) => exit
				.transition()
				.duration(200)
				.attr('opacity', 0)
				.remove()
		);

		// "Now" line (year 2026)
		const nowLineData = (NOW_YEAR >= xExtent[0] && NOW_YEAR <= xExtent[1]) ? [NOW_YEAR] : [];
		const nowGroup = g.selectAll<SVGGElement, number>('g.now-line')
			.data(nowLineData);

		nowGroup.join(
			(enter) => {
				const ng = enter.append('g').attr('class', 'now-line');
				ng.append('line')
					.attr('x1', (d) => xScale(d))
					.attr('x2', (d) => xScale(d))
					.attr('y1', 0)
					.attr('y2', innerH)
					.attr('stroke', 'var(--text-secondary)')
					.attr('stroke-width', 1)
					.attr('opacity', 0.5);
				ng.append('text')
					.attr('x', (d) => xScale(d) + 3)
					.attr('y', innerH - 4)
					.attr('fill', 'var(--text-secondary)')
					.attr('font-size', '9px')
					.attr('opacity', 0.6)
					.text('Now');
				return ng;
			},
			(update) => {
				update.select('line')
					.attr('x1', (d) => xScale(d))
					.attr('x2', (d) => xScale(d))
					.attr('y2', innerH);
				update.select('text')
					.attr('x', (d) => xScale(d) + 3)
					.attr('y', innerH - 4);
				return update;
			},
			(exit) => exit.remove()
		);

		// Annotations (only in normal mode, for visible variables)
		const annotations: Array<{ year: number; label: string; color: string }> = [];
		if (!_compareMode) {
			for (const ld of linesData) {
				const varConfig = unifiedVariables.find((v) => v.id === ld.id);
				if (!varConfig) continue;
				const anns = getAnnotations(varConfig.id, varConfig.fieldPath, _data, _focusedId, varConfig.shortLabel);
				for (const a of anns) {
					annotations.push({ year: a.year, label: a.label, color: varConfig.color });
				}
			}
		}

		// Stagger overlapping annotation labels: sort by year, offset Y for close ones
		annotations.sort((a, b) => a.year - b.year);
		const labelYPositions: number[] = [];
		const LABEL_SPACING = 14;
		for (let i = 0; i < annotations.length; i++) {
			let yPos = 10;
			// Check if close to a previous annotation (within 20px on x)
			for (let j = 0; j < i; j++) {
				const xDist = Math.abs(xScale(annotations[i].year) - xScale(annotations[j].year));
				if (xDist < 40) {
					yPos = labelYPositions[j] + LABEL_SPACING;
				}
			}
			labelYPositions.push(yPos);
		}

		const annotationGroup = g.selectAll<SVGGElement, null>('g.annotations')
			.data([null])
			.join('g')
			.attr('class', 'annotations');

		const annDataWithY = annotations.map((a, i) => ({ ...a, labelY: labelYPositions[i] }));

		const annSel = annotationGroup
			.selectAll<SVGGElement, typeof annDataWithY[number]>('g.annotation')
			.data(annDataWithY, (d) => `${d.year}-${d.label}`);

		annSel.join(
			(enter) => {
				const ann = enter.append('g').attr('class', 'annotation')
					.attr('opacity', 0);
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
					.attr('y', (d) => d.labelY)
					.attr('fill', (d) => d.color)
					.attr('font-size', '9px')
					.attr('opacity', 0.7)
					.text((d) => d.label);
				ann.transition().duration(400).attr('opacity', 1);
				return ann;
			},
			(update) => {
				update.transition().duration(400).attr('opacity', 1);
				update.select('line')
					.attr('x1', (d) => xScale(d.year))
					.attr('x2', (d) => xScale(d.year))
					.attr('y2', innerH)
					.attr('stroke', (d) => d.color);
				update.select('text')
					.attr('x', (d) => xScale(d.year) + 3)
					.attr('y', (d) => d.labelY)
					.attr('fill', (d) => d.color)
					.text((d) => d.label);
				return update;
			},
			(exit) => exit.transition().duration(200).attr('opacity', 0).remove()
		);

		// Legend (right side on wide screens, below chart on narrow)
		const isNarrow = width < 500;
		const legendGroup = svg.selectAll<SVGGElement, null>('g.legend')
			.data([null])
			.join('g')
			.attr('class', 'legend');

		if (isNarrow) {
			legendGroup.attr('transform', `translate(${margin.left}, ${margin.top + innerH + 24})`);
		} else {
			legendGroup.attr('transform', `translate(${margin.left + innerW + 16}, ${margin.top + 8})`);
		}

		const legendItems = legendGroup
			.selectAll<SVGGElement, typeof legendData[number]>('g.legend-item')
			.data(legendData, (d) => d.id);

		function handleEyeClick(event: MouseEvent, d: typeof legendData[number]) {
			event.stopPropagation();
			if (d.fieldPath === '__historical__') {
				showHistorical.update((v) => !v);
			} else {
				visibleVariables.update((set) => {
					const next = new Set(set);
					if (next.has(d.fieldPath)) next.delete(d.fieldPath);
					else next.add(d.fieldPath);
					return next;
				});
			}
		}

		function handleItemClick(_: MouseEvent, d: typeof legendData[number]) {
			if (d.fieldPath === '__historical__') {
				selectedHistoricalId.set(get(selectedHistoricalId) ? null : 'historical');
			} else if (!_compareMode) {
				handleLegendSelect(d.fieldPath);
			}
		}

		function getEyePath(d: typeof legendData[number]): string {
			const vis = d.fieldPath === '__historical__' ? _showHistorical : _visibleVars.has(d.fieldPath);
			return vis ? EYE_OPEN : EYE_CLOSED;
		}

		legendItems.join(
			(enter) => {
				const item = enter.append('g')
					.attr('class', 'legend-item')
					.attr('transform', (_, i) => {
						if (isNarrow) {
							const col = i % 3;
							const row = Math.floor(i / 3);
							return `translate(${col * Math.floor(innerW / 3)}, ${row * 18})`;
						}
						return `translate(0, ${i * 28})`;
					})
					.attr('cursor', 'pointer')
					.attr('opacity', getLegendOpacity)
					.on('click', handleItemClick);

				// Eye toggle icon
				const eyeG = item.append('g')
					.attr('class', 'eye-toggle')
					.attr('cursor', 'pointer');

				// Invisible hit area — the eye path is stroke-only (~14×11px),
				// so without this rect clicks only register on the thin strokes.
				eyeG.append('rect')
					.attr('x', -3)
					.attr('y', -1)
					.attr('width', 22)
					.attr('height', 18)
					.attr('fill', 'transparent');

				eyeG.append('path')
					.attr('d', getEyePath)
					.attr('fill', 'none')
					.attr('stroke', 'var(--text-secondary)')
					.attr('stroke-width', 1.2);

				eyeG.on('click', handleEyeClick);

				// Color swatch (at x=22)
				item.each(function(d) {
					const el = d3.select(this);
					if (d.fieldPath === '__historical__') {
						el.append('line')
							.attr('x1', 22).attr('y1', 8)
							.attr('x2', 38).attr('y2', 8)
							.attr('stroke', d.color)
							.attr('stroke-width', 2)
							.attr('stroke-dasharray', '4,2');
					} else {
						el.append('rect')
							.attr('class', 'swatch')
							.attr('x', 22)
							.attr('width', 16)
							.attr('height', 4)
							.attr('y', 7)
							.attr('rx', 2)
							.attr('fill', d.color);
					}
				});

				// Label text (at x=42)
				const labelText = item.append('text')
					.attr('x', 42)
					.attr('y', 12)
					.attr('fill', 'var(--text-secondary)')
					.attr('font-size', isNarrow ? '10px' : '13px')
					.text((d) => d.label);

				if (isNarrow) {
					const maxTextW = Math.floor(innerW / 3) - 46;
					labelText
						.attr('textLength', (d) => {
							// Only constrain if text would overflow
							const est = d.label.length * 6; // ~6px per char at 10px font
							return est > maxTextW ? maxTextW : null;
						})
						.attr('lengthAdjust', 'spacing');
				}

				return item;
			},
			(update) => {
				update
					.transition()
					.duration(400)
					.attr('transform', (_, i) => {
						if (isNarrow) {
							const col = i % 3;
							const row = Math.floor(i / 3);
							return `translate(${col * Math.floor(innerW / 3)}, ${row * 18})`;
						}
						return `translate(0, ${i * 28})`;
					})
					.attr('opacity', getLegendOpacity);
				update.select('text').text((d) => d.label);
				update.select('rect.swatch').attr('fill', (d) => d.color);
				update.select('line').attr('stroke', (d) => d.color);
				update.select('.eye-toggle path').attr('d', getEyePath);
				update.select('.eye-toggle').on('click', handleEyeClick);
				update.on('click', handleItemClick);
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
				if (!event.selection) return;
				const [x0, x1] = event.selection as [number, number];
				const yearStart = Math.round(xScale.invert(x0));
				const yearEnd = Math.round(xScale.invert(x1));
				if (yearEnd - yearStart < 5) {
					brushGroup.call(brush.move, null);
				}
			});

		brushGroup.call(brush);

		// Store refs for keyboard shortcut
		activeBrushGroup = brushGroup as any;
		activeBrush = brush;

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

		function handlePointerMove(event: MouseEvent | TouchEvent) {
			if (isBrushing) return;

			const [mx] = d3.pointer(event, overlay.node()!);
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
		}

		overlay
			.on('mousemove', handlePointerMove)
			.on('mouseleave', () => {
				tooltipLine.style('display', 'none');
				tooltipVisible = false;
				hoveredYear.set(null);
			})
			.on('touchstart', (event: TouchEvent) => {
				event.preventDefault();
				handlePointerMove(event);
			}, { passive: false } as any)
			.on('touchmove', (event: TouchEvent) => {
				event.preventDefault();
				handlePointerMove(event);
			}, { passive: false } as any)
			.on('touchend', () => {
				setTimeout(() => {
					tooltipLine.style('display', 'none');
					tooltipVisible = false;
					hoveredYear.set(null);
				}, 300);
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
	.unified-chart :global(.eye-toggle:hover path) {
		stroke: var(--accent) !important;
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
