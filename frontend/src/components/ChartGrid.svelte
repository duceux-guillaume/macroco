<script lang="ts">
	import UnifiedChart from '$lib/charts/UnifiedChart.svelte';
	import { unifiedVariables } from '$lib/charts/unified-config';
	import { activeSimData } from '$lib/stores/simulation';
	import { scenarioColors, focusedScenarioId } from '$lib/stores/scenarios';
	import { compareMode, compareVariable } from '$lib/stores/chart-ui';

	function toggleCompare() {
		compareMode.update((v) => !v);
	}

	function handleVariableChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		compareVariable.set(target.value);
	}
</script>

<div class="chart-layout">
	<div class="chart-toolbar">
		<button
			class="compare-toggle"
			class:active={$compareMode}
			onclick={toggleCompare}
		>
			{$compareMode ? 'Show all variables' : 'Compare scenarios'}
		</button>
		{#if $compareMode}
			<select class="variable-select" value={$compareVariable} onchange={handleVariableChange}>
				{#each unifiedVariables as v (v.id)}
					<option value={v.fieldPath}>{v.label}</option>
				{/each}
			</select>
		{/if}
	</div>
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
		gap: 8px;
	}
	.chart-toolbar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 0 4px;
		flex-shrink: 0;
	}
	.compare-toggle {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 4px;
		color: var(--text-secondary);
		font-size: 12px;
		padding: 4px 10px;
		cursor: pointer;
		transition: all 0.15s;
	}
	.compare-toggle:hover {
		border-color: var(--accent);
		color: var(--text);
	}
	.compare-toggle.active {
		background: var(--accent);
		border-color: var(--accent);
		color: var(--bg);
	}
	.variable-select {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 4px;
		color: var(--text);
		font-size: 12px;
		padding: 4px 8px;
		cursor: pointer;
	}
	.variable-select:focus {
		outline: 1px solid var(--accent);
		border-color: var(--accent);
	}
	.chart-section {
		flex: 1;
		min-height: 300px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 8px;
	}
</style>
