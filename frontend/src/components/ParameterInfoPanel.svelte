<script lang="ts">
	import * as d3 from 'd3';
	import { onDestroy } from 'svelte';
	import { selectedParameterId, selectedVariableId } from '$lib/stores/info';
	import { simulationResults } from '$lib/stores/simulation';
	import { scenarios, activeScenarioIds } from '$lib/stores/scenarios';
	import {
		parameterDescriptions,
		feedbackLoops,
		variableDescriptions,
		type ParameterInfo,
		type FeedbackLoopInfo
	} from '$lib/content/variable-descriptions';
	import { extractSeries, type DataPoint } from '$lib/utils/extract';
	import type { WorldState } from '$lib/types';
	import InfoPanelShell from './InfoPanelShell.svelte';
	import FeedbackLoops from './FeedbackLoops.svelte';
	import RelatedVars from './RelatedVars.svelte';

	let parameterId = $state<string | null>(null);

	const unsub = selectedParameterId.subscribe((p) => {
		parameterId = p;
	});
	onDestroy(unsub);

	let info = $derived<ParameterInfo | null>(
		parameterId ? parameterDescriptions[parameterId] ?? null : null
	);

	let relatedLoops = $derived<FeedbackLoopInfo[]>(
		info
			? info.feedbackLoops
					.map((id) => feedbackLoops[id])
					.filter((l): l is FeedbackLoopInfo => l != null)
			: []
	);

	let relatedVars = $derived(
		info
			? info.relatedVariables
					.map((path) => {
						const desc = variableDescriptions[path];
						return desc ? { path, name: desc.name } : null;
					})
					.filter((v): v is { path: string; name: string } => v != null)
			: []
	);

	let sparklineEl = $state<HTMLDivElement | null>(null);

	// Find the BAU scenario ID
	let bauScenarioId = $derived.by(() => {
		const allScenarios = $scenarios;
		const bau = allScenarios.find(
			(s) => s.name.toLowerCase().includes('business as usual') || s.name === 'BAU'
		);
		return bau?.id ?? null;
	});

	// Get a focused scenario ID (any active non-BAU scenario, or BAU itself)
	let comparisonScenarioId = $derived.by(() => {
		const active = $activeScenarioIds;
		const bau = bauScenarioId;
		for (const id of active) {
			if (id !== bau) return id;
		}
		return bau;
	});

	$effect(() => {
		if (!sparklineEl || !info) return;
		const fieldPath = info.impact.sparklineVariable;
		const results = $simulationResults;

		const bauData = bauScenarioId ? results.get(bauScenarioId) : null;
		const compData = comparisonScenarioId ? results.get(comparisonScenarioId) : null;

		drawSparkline(sparklineEl, fieldPath, bauData ?? null, compData ?? null);
	});

	function drawSparkline(
		el: HTMLDivElement,
		fieldPath: string,
		bauStates: WorldState[] | null,
		compStates: WorldState[] | null
	) {
		const rect = el.getBoundingClientRect();
		const W = Math.max(120, Math.floor(rect.width));
		const H = 70;
		const m = { top: 4, right: 4, bottom: 4, left: 4 };
		const innerW = W - m.left - m.right;
		const innerH = H - m.top - m.bottom;

		const svg = d3
			.select(el)
			.selectAll<SVGSVGElement, null>('svg')
			.data([null])
			.join('svg')
			.attr('width', W)
			.attr('height', H);

		const g = svg
			.selectAll<SVGGElement, null>('g.spark')
			.data([null])
			.join('g')
			.attr('class', 'spark')
			.attr('transform', `translate(${m.left},${m.top})`);

		const allPoints: DataPoint[] = [];
		const bauPoints = bauStates ? extractSeries(bauStates, fieldPath) : [];
		const compPoints = compStates ? extractSeries(compStates, fieldPath) : [];
		allPoints.push(...bauPoints, ...compPoints);

		if (allPoints.length === 0) {
			g.selectAll('*').remove();
			return;
		}

		const xScale = d3
			.scaleLinear()
			.domain(d3.extent(allPoints, (d) => d.year) as [number, number])
			.range([0, innerW]);

		const yScale = d3
			.scaleLinear()
			.domain(d3.extent(allPoints, (d) => d.value) as [number, number])
			.nice()
			.range([innerH, 0]);

		const line = d3
			.line<DataPoint>()
			.x((d) => xScale(d.year))
			.y((d) => yScale(d.value));

		// BAU line (dimmed)
		g.selectAll<SVGPathElement, null>('path.bau-line')
			.data(bauPoints.length > 0 ? [bauPoints] : [])
			.join('path')
			.attr('class', 'bau-line')
			.attr('fill', 'none')
			.attr('stroke', 'var(--text-secondary)')
			.attr('stroke-width', 1)
			.attr('opacity', 0.4)
			.attr('d', (d) => line(d));

		// Current scenario line
		g.selectAll<SVGPathElement, null>('path.comp-line')
			.data(compPoints.length > 0 ? [compPoints] : [])
			.join('path')
			.attr('class', 'comp-line')
			.attr('fill', 'none')
			.attr('stroke', 'var(--accent)')
			.attr('stroke-width', 1.5)
			.attr('d', (d) => line(d));
	}

	function close() {
		selectedParameterId.set(null);
	}

	function selectVariable(path: string) {
		selectedVariableId.set(path);
	}
</script>

{#if info && parameterId}
	<InfoPanelShell
		title={info.name}
		meta="{info.sector} · {info.unit}"
		ariaLabel="Parameter information"
		beginner={info.beginner}
		expert={info.expert}
		onclose={close}
	>
		<section>
			<h3>Impact</h3>
			<div class="impact-card increase">
				<span class="impact-arrow">&#x2191;</span>
				<p>{info.impact.increase}</p>
			</div>
			<div class="impact-card decrease">
				<span class="impact-arrow">&#x2193;</span>
				<p>{info.impact.decrease}</p>
			</div>
			<div class="sparkline" bind:this={sparklineEl}></div>
		</section>

		<FeedbackLoops loops={relatedLoops} />
		<RelatedVars vars={relatedVars} onselect={selectVariable} />
	</InfoPanelShell>
{/if}

<style>
	.impact-card {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		background: var(--surface-hover);
		border-radius: 6px;
		padding: 10px;
		margin-bottom: 6px;
	}
	.impact-arrow {
		font-size: 16px;
		font-weight: 700;
		flex-shrink: 0;
		line-height: 1.2;
	}
	.impact-card.increase .impact-arrow {
		color: #86efac;
	}
	.impact-card.decrease .impact-arrow {
		color: #fca5a5;
	}
	.impact-card p {
		font-size: 12px;
		color: var(--text);
		line-height: 1.5;
		margin: 0;
	}
	.sparkline {
		margin-top: 4px;
		border-radius: 4px;
		background: var(--surface-hover);
		padding: 4px;
		display: flex;
		justify-content: center;
	}
</style>
