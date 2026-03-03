<script lang="ts">
	import { focusedScenarioId, scenarioParamsCache } from '$lib/stores/scenarios';
	import { send } from '$lib/ws';
	import type { ScenarioParams } from '$lib/types';

	const startYearOptions = [1900, 1950, 1970, 2000];
	const endYearOptions = [2050, 2100, 2150, 2200];
	const timeStepOptions = [0.25, 0.5, 1.0, 2.0];

	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	function getParams(): ScenarioParams | null {
		const id = $focusedScenarioId;
		if (!id) return null;
		return $scenarioParamsCache.get(id) ?? null;
	}

	function updateField(field: string, value: number) {
		const id = $focusedScenarioId;
		if (!id) return;
		const params = getParams();
		if (!params) return;

		const updated = { ...params, [field]: value };
		scenarioParamsCache.update((cache) => {
			const next = new Map(cache);
			next.set(id, updated);
			return next;
		});

		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			send({ type: 'update_params', scenario_id: id, params: updated });
		}, 200);
	}
</script>

{#if $focusedScenarioId && getParams()}
	{@const params = getParams()!}
	<div class="sim-controls">
		<h3>Simulation</h3>
		<div class="controls-row">
			<label class="control">
				<span class="control-label">Start</span>
				<select
					value={params.start_year}
					onchange={(e) => updateField('start_year', Number(e.currentTarget.value))}
				>
					{#each startYearOptions as yr}
						<option value={yr}>{yr}</option>
					{/each}
				</select>
			</label>

			<label class="control">
				<span class="control-label">End</span>
				<select
					value={params.end_year}
					onchange={(e) => updateField('end_year', Number(e.currentTarget.value))}
				>
					{#each endYearOptions as yr}
						<option value={yr}>{yr}</option>
					{/each}
				</select>
			</label>

			<label class="control">
				<span class="control-label">Step</span>
				<select
					value={params.time_step}
					onchange={(e) => updateField('time_step', Number(e.currentTarget.value))}
				>
					{#each timeStepOptions as dt}
						<option value={dt}>{dt}yr</option>
					{/each}
				</select>
			</label>
		</div>
	</div>
{/if}

<style>
	.sim-controls {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	h3 {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-secondary);
		margin: 0;
	}
	.controls-row {
		display: flex;
		gap: 8px;
	}
	.control {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
	}
	.control-label {
		font-size: 11px;
		color: var(--text-secondary);
	}
	select {
		background: var(--surface-hover);
		border: 1px solid var(--border);
		border-radius: 4px;
		color: var(--text);
		font-size: 12px;
		padding: 4px 6px;
		cursor: pointer;
		appearance: auto;
	}
	select:focus {
		outline: 1px solid var(--accent);
		outline-offset: -1px;
	}
</style>
