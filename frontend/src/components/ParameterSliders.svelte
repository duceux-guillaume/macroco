<script lang="ts">
	import { schemaBySector } from '$lib/stores/schema';
	import { focusedScenarioId, scenarioParamsCache } from '$lib/stores/scenarios';
	import { send } from '$lib/ws';
	import type { ScenarioParams } from '$lib/types';
	import ParameterSlider from './ParameterSlider.svelte';

	const sectorLabels: Record<string, string> = {
		population: 'Population',
		capital: 'Capital & Technology',
		agriculture: 'Agriculture',
		resources: 'Resources',
		pollution: 'Pollution'
	};

	let collapsedSectors = $state<Set<string>>(new Set());
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	function toggleSector(sector: string) {
		const next = new Set(collapsedSectors);
		if (next.has(sector)) {
			next.delete(sector);
		} else {
			next.add(sector);
		}
		collapsedSectors = next;
	}

	function getCurrentParams(): ScenarioParams | null {
		const id = $focusedScenarioId;
		if (!id) return null;
		return $scenarioParamsCache.get(id) ?? null;
	}

	function handleParamChange(field: string, value: number) {
		const id = $focusedScenarioId;
		if (!id) return;

		const params = getCurrentParams();
		if (!params) return;

		// Update local cache immediately
		const updated = { ...params, [field]: value };
		scenarioParamsCache.update((cache) => {
			const next = new Map(cache);
			next.set(id, updated);
			return next;
		});

		// Debounce WS update
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			send({ type: 'update_params', scenario_id: id, params: updated });
		}, 200);
	}

	function getParamValue(params: ScenarioParams, field: string): number {
		return (params as unknown as Record<string, number>)[field] ?? 0;
	}
</script>

{#if $focusedScenarioId && getCurrentParams()}
	{@const params = getCurrentParams()!}
	<div class="sliders">
		{#each [...$schemaBySector] as [sector, descriptors]}
			<div class="sector-group">
				<button class="sector-header" onclick={() => toggleSector(sector)}>
					<span class="sector-arrow" class:collapsed={collapsedSectors.has(sector)}>&#x25BE;</span>
					{sectorLabels[sector] ?? sector}
				</button>
				{#if !collapsedSectors.has(sector)}
					<div class="sector-sliders">
						{#each descriptors as desc}
							<ParameterSlider
								descriptor={desc}
								value={getParamValue(params, desc.field)}
								onchange={handleParamChange}
							/>
						{/each}
					</div>
				{/if}
			</div>
		{/each}
	</div>
{:else}
	<p class="no-selection">Select a scenario to adjust parameters.</p>
{/if}

<style>
	.sliders {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.sector-group {
		border-bottom: 1px solid var(--border);
		padding-bottom: 4px;
	}
	.sector-header {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		background: none;
		border: none;
		cursor: pointer;
		padding: 6px 0;
		font-size: 13px;
		font-weight: 600;
		color: var(--text);
		text-align: left;
	}
	.sector-header:hover {
		color: var(--accent);
	}
	.sector-arrow {
		transition: transform 0.15s;
		font-size: 12px;
	}
	.sector-arrow.collapsed {
		transform: rotate(-90deg);
	}
	.sector-sliders {
		padding-left: 4px;
	}
	.no-selection {
		font-size: 13px;
		color: var(--text-secondary);
		padding: 12px 0;
	}
</style>
