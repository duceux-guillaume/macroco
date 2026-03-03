<script lang="ts">
	import UnifiedChart from '$lib/charts/UnifiedChart.svelte';
	import SparklineChart from '$lib/charts/SparklineChart.svelte';
	import { unifiedVariables } from '$lib/charts/unified-config';
	import { activeSimData } from '$lib/stores/simulation';
	import { scenarioColors, focusedScenarioId } from '$lib/stores/scenarios';
</script>

<div class="chart-layout">
	<div class="overview-section">
		<UnifiedChart data={$activeSimData} colors={$scenarioColors} focusedScenarioId={$focusedScenarioId} />
	</div>
	<div class="sparklines-section">
		{#each unifiedVariables as varConfig (varConfig.id)}
			<div class="sparkline-cell">
				<SparklineChart config={varConfig} data={$activeSimData} focusedScenarioId={$focusedScenarioId} />
			</div>
		{/each}
	</div>
</div>

<style>
	.chart-layout {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		gap: 8px;
	}
	.overview-section {
		flex: 3;
		min-height: 200px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 8px;
	}
	.sparklines-section {
		flex: 2;
		display: grid;
		grid-template-columns: repeat(6, 1fr);
		gap: 4px;
		min-height: 120px;
	}
	.sparkline-cell {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 4px 4px 4px 0;
		min-height: 80px;
	}
	@media (max-width: 1200px) {
		.sparklines-section {
			grid-template-columns: repeat(3, 1fr);
		}
	}
	@media (max-width: 900px) {
		.sparklines-section {
			grid-template-columns: 1fr;
		}
	}
</style>
