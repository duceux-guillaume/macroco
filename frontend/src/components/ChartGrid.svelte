<script lang="ts">
	import UnifiedChart from '$lib/charts/UnifiedChart.svelte';
	import { activeSimData } from '$lib/stores/simulation';
	import { scenarioColors, focusedScenarioId } from '$lib/stores/scenarios';
</script>

<div class="chart-layout">
	<div class="overview-section">
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
	.overview-section {
		flex: 1;
		min-height: 200px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 8px;
	}
</style>
