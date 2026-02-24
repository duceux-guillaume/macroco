<script lang="ts">
	import { chartConfigs } from '$lib/charts/chart-config';
	import TimeSeriesChart from '$lib/charts/TimeSeriesChart.svelte';
	import { activeSimData } from '$lib/stores/simulation';
	import { scenarioColors } from '$lib/stores/scenarios';
</script>

<div class="chart-grid">
	{#each chartConfigs as config (config.id)}
		<div class="chart-cell">
			<TimeSeriesChart {config} data={$activeSimData} colors={$scenarioColors} />
		</div>
	{/each}
</div>

<style>
	.chart-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 12px;
		flex: 1;
	}
	.chart-cell {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 8px;
		min-height: 240px;
	}
	@media (max-width: 900px) {
		.chart-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
