<script lang="ts">
	import { onDestroy } from 'svelte';
	import { selectedHistoricalId } from '$lib/stores/info';
	import { historicalData, type HistoricalVariable } from '$lib/stores/historical';
	import InfoPanelShell from './InfoPanelShell.svelte';

	let histId = $state<string | null>(null);
	const unsub = selectedHistoricalId.subscribe((v) => {
		histId = v;
	});
	onDestroy(unsub);

	let histMap = $state<Map<string, HistoricalVariable>>(new Map());
	const unsub2 = historicalData.subscribe((m) => {
		histMap = m;
	});
	onDestroy(unsub2);

	let sources = $derived(histId ? Array.from(histMap.values()) : []);

	let dateRange = $derived.by(() => {
		if (sources.length === 0) return '';
		let minYear = Infinity;
		let maxYear = -Infinity;
		for (const s of sources) {
			for (const d of s.data) {
				if (d.year < minYear) minYear = d.year;
				if (d.year > maxYear) maxYear = d.year;
			}
		}
		return `${minYear}\u2013${maxYear}`;
	});

	function close() {
		selectedHistoricalId.set(null);
	}
</script>

{#if histId}
	<InfoPanelShell
		title="Historical Data"
		meta={dateRange}
		ariaLabel="Historical data information"
		beginner="The dashed lines on the chart represent real-world observations from official sources like the World Bank and UN. They show how actual measurements compare to the simulation's projections, helping validate whether the model captures real trends."
		expert="Each historical variable is sourced from a specific dataset (noted below). Values may undergo unit transformation to match the model's internal units. In normalized overlay mode, both simulation and historical data share a combined min/max range so their shapes are directly comparable."
		onclose={close}
	>
		<section>
			<h3>Data Sources</h3>
			{#each sources as src (src.variable)}
				<div class="source-card">
					<div class="source-name">{src.variable}</div>
					<div class="source-detail">{src.source}</div>
					<div class="source-detail">Units: {src.units}</div>
					{#if src.transformation !== 'none'}
						<div class="source-detail">Transform: {src.transformation}</div>
					{/if}
					{#if src.url}
						<a class="source-link" href={src.url} target="_blank" rel="noopener noreferrer">
							View source data &#x2197;
						</a>
					{/if}
					<div class="source-detail source-points">{src.data.length} data points</div>
				</div>
			{/each}
		</section>
	</InfoPanelShell>
{/if}

<style>
	.source-card {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 10px 12px;
		margin-bottom: 8px;
	}
	.source-name {
		font-weight: 600;
		font-size: 13px;
		color: var(--text);
		text-transform: capitalize;
		margin-bottom: 4px;
	}
	.source-detail {
		font-size: 11px;
		color: var(--text-secondary);
		margin-top: 2px;
	}
	.source-link {
		display: inline-block;
		font-size: 11px;
		color: var(--accent);
		margin-top: 4px;
		text-decoration: none;
	}
	.source-link:hover {
		text-decoration: underline;
	}
	.source-points {
		margin-top: 4px;
		opacity: 0.7;
	}
</style>
