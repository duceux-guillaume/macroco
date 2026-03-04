<script lang="ts">
	import { focusedScenarioId } from '$lib/stores/scenarios';
	import {
		startYearOptions,
		endYearOptions,
		timeStepOptions,
		getSimParams,
		updateSimField
	} from '$lib/stores/simulation-controls';
</script>

{#if $focusedScenarioId}
	{@const params = getSimParams()}
	{#if params}
	<div class="sim-controls">
		<h3>Simulation</h3>
		<div class="controls-row">
			<label class="control">
				<span class="control-label">Start</span>
				<select
					value={params.start_year}
					onchange={(e) => updateSimField('start_year', Number(e.currentTarget.value))}
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
					onchange={(e) => updateSimField('end_year', Number(e.currentTarget.value))}
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
					onchange={(e) => updateSimField('time_step', Number(e.currentTarget.value))}
				>
					{#each timeStepOptions as dt}
						<option value={dt}>{dt}yr</option>
					{/each}
				</select>
			</label>
		</div>
	</div>
	{/if}
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
