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

	let showExpert = $state(false);
	let parameterId = $state<string | null>(null);

	const unsub = selectedParameterId.subscribe((p) => {
		parameterId = p;
		showExpert = false;
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
		const W = 280,
			H = 70;
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

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if info && parameterId}
	<InfoPanelShell
		title={info.name}
		meta="{info.sector} · {info.unit}"
		ariaLabel="Parameter information"
		onclose={close}
	>
		<section>
			<p class="description">{info.beginner}</p>
		</section>

		<section>
			<button class="toggle-btn" onclick={() => (showExpert = !showExpert)}>
				{showExpert ? '▾' : '▸'} Technical Detail
			</button>
			{#if showExpert}
				<p class="expert">{info.expert}</p>
			{/if}
		</section>

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

		{#if relatedLoops.length > 0}
			<section>
				<h3>Feedback Loops</h3>
				{#each relatedLoops as loop}
					<div class="loop-card">
						<div class="loop-header">
							<span class="loop-type" class:reinforcing={loop.type === 'reinforcing'} class:stabilizing={loop.type === 'stabilizing'}>
								{loop.type === 'reinforcing' ? '+' : '−'}
							</span>
							<strong>{loop.name}</strong>
						</div>
						<p class="loop-desc">{loop.description}</p>
					</div>
				{/each}
			</section>
		{/if}

		{#if relatedVars.length > 0}
			<section>
				<h3>Related Variables</h3>
				<div class="related-list">
					{#each relatedVars as v}
						<button class="related-btn" onclick={() => selectVariable(v.path)}>
							{v.name}
						</button>
					{/each}
				</div>
			</section>
		{/if}
	</InfoPanelShell>
{/if}

<style>
	section h3 {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-secondary);
		margin: 0 0 8px;
	}
	.description {
		font-size: 13px;
		color: var(--text);
		line-height: 1.6;
	}
	.toggle-btn {
		background: none;
		border: none;
		color: var(--accent);
		cursor: pointer;
		font-size: 13px;
		padding: 0;
		text-align: left;
	}
	.toggle-btn:hover {
		text-decoration: underline;
	}
	.expert {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.6;
		margin-top: 8px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		white-space: pre-wrap;
	}
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
	.loop-card {
		background: var(--surface-hover);
		border-radius: 6px;
		padding: 10px;
		margin-bottom: 6px;
	}
	.loop-header {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 4px;
		font-size: 13px;
		color: var(--text);
	}
	.loop-type {
		width: 18px;
		height: 18px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 12px;
		font-weight: 700;
		flex-shrink: 0;
	}
	.loop-type.reinforcing {
		background: #7f1d1d;
		color: #fca5a5;
	}
	.loop-type.stabilizing {
		background: #14532d;
		color: #86efac;
	}
	.loop-desc {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.5;
	}
	.related-list {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.related-btn {
		background: var(--surface-hover);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 4px 8px;
		font-size: 12px;
		color: var(--accent);
		cursor: pointer;
		transition: background 0.1s;
	}
	.related-btn:hover {
		background: var(--surface-active);
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
