<script lang="ts">
	import UnifiedChart from '$lib/charts/UnifiedChart.svelte';
	import { activeSimData } from '$lib/stores/simulation';
	import { scenarioColors, focusedScenarioId } from '$lib/stores/scenarios';
</script>

<div class="chart-layout">
	<div class="chart-section">
		<UnifiedChart data={$activeSimData} colors={$scenarioColors} focusedScenarioId={$focusedScenarioId} />
	</div>
</div>

<style>
	.chart-layout {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
	}
	.chart-section {
		flex: 1;
		min-height: 300px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 8px;
	}

	@media (max-width: 767px) {
		.chart-section {
			min-height: 250px;
		}
	}
</style>
